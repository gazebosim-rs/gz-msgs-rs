#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
import importlib.util, pathlib
spec = importlib.util.spec_from_file_location('gate', pathlib.Path(__file__).with_name('check-coverage.py'))
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)
root = pathlib.Path(__file__).resolve().parents[1]
for count, covered, present, passes in [(100,95,True,False),(100,94,True,False),(100,96,True,True),(0,0,True,False),(100,100,False,False)]:
    report = {'data':[{'files':[{'filename':str(p.resolve()),'summary':{m:{'count':count,'covered':covered} for m in ['lines','regions']}} for p in [*(root/'src').rglob('*.rs'), root/'build.rs']] if present else []}]}
    try: gate.check(report,root); result=True
    except ValueError: result=False
    assert result == passes
print('Coverage gates: exactly 95, below 95, empty and missing rejected; 96 accepted')

# Owned dependency changes get the same strict metrics, including missing profiles.
import hashlib, json, tempfile
with tempfile.TemporaryDirectory() as directory:
    fixture = pathlib.Path(directory)
    (fixture/'vendor').mkdir()
    source = fixture/'vendor/lib.rs'
    source.write_text('value\n'*100)
    (fixture/'vendor/manifest.json').write_text(json.dumps({'packages':[{'name':'fixture','files':[{'path':'vendor/lib.rs','sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'owned_lines':list(range(1,101))}]}]}))
    for lines,regions,present,expected in [(95,100,True,False),(100,95,True,False),(96,96,True,True),(100,100,False,False)]:
        lcov='SF:'+str(source)+'\n'+''.join(f'DA:{n},{int(n<=lines)}\n' for n in range(1,101)) if present else ''
        report={'data':[{'functions':[{'filenames':[str(source)],'regions':[[n,1,n,2,int(n<=regions),0,0,0] for n in range(1,101)]}]}]}
        try:gate.check_patches(report,fixture,lcov);actual=True
        except ValueError:actual=False
        assert actual==expected

    # Compiler-omitted syntax is exact and cannot hide a mapped executable line.
    source.write_text('value\n' * 99 + '{\n')
    manifest = {'packages': [{'name': 'fixture', 'files': [{
        'path': 'vendor/lib.rs', 'sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
        'patch': 'owned.patch', 'changed_lines': list(range(1, 101)), 'owned_lines': list(range(1, 101)),
        'unmapped_syntax': {'100': '{'}}]}]}
    (fixture/'owned.patch').write_text('--- lib.rs\n+++ lib.rs\n@@ -0,0 +1,100 @@\n' + ''.join('+' + line for line in source.read_text().splitlines(True)))
    for mapped, text, regions, expected in [(False, '{', 100, True),
                                          (True, '{', 100, False),
                                          (False, 'value', 100, False),
                                          (False, '}', 100, False),
                                          (False, '{', 95, False)]:
        manifest['packages'][0]['files'][0]['unmapped_syntax']['100'] = text
        (fixture/'vendor/manifest.json').write_text(json.dumps(manifest))
        lcov = 'SF:' + str(source) + '\n' + ''.join(f'DA:{n},1\n' for n in range(1, 101 if mapped else 100))
        report = {'data': [{'functions': [{'filenames': [str(source)],
            'regions': [[n, 1, n, 2, int(n <= regions), 0, 0, 0] for n in range(1, 101)]}]}]}
        try: gate.check_patches(report, fixture, lcov); actual = True
        except ValueError: actual = False
        assert actual == expected
    # A stale report cannot qualify changed bytes or silently omitted additions.
    item = manifest['packages'][0]['files'][0]
    item['unmapped_syntax']['100'] = '{'
    source.write_text('panic!();\n' * 96 + 'value\n' * 3 + '{\n')
    (fixture/'vendor/manifest.json').write_text(json.dumps(manifest))
    try: gate.check_patches(report, fixture, lcov); rejected = False
    except ValueError: rejected = True
    assert rejected
    source.write_text('value\n' * 99 + '{\n')
    for field in ['owned_lines', 'changed_lines']:
        original = item[field]
        item[field] = [number for number in original if number != 96]
        (fixture/'vendor/manifest.json').write_text(json.dumps(manifest))
        try: gate.check_patches(report, fixture, lcov); rejected = False
        except ValueError: rejected = True
        assert rejected
        item[field] = original
    # Regions starting in unchanged context still cover modified source.
    source.write_text('context\ncall();\n')
    (fixture/'owned.patch').write_text('--- lib.rs\n+++ lib.rs\n@@ -1 +1,2 @@\n context\n+call();\n')
    item.update(sha256=hashlib.sha256(source.read_bytes()).hexdigest(),
                changed_lines=[2], owned_lines=[2], unmapped_syntax={})
    (fixture/'vendor/manifest.json').write_text(json.dumps(manifest))
    lcov = 'SF:' + str(source) + '\nDA:2,1\n'
    for covered, expected in [(0, False), (1, True)]:
        report = {'data': [{'functions': [{'filenames': [str(source)],
            'regions': [[1, 1, 2, 8, covered, 0, 0, 0]]}]}]}
        try: gate.check_patches(report, fixture, lcov); actual = True
        except ValueError: actual = False
        assert actual == expected
    report['data'][0]['functions'][0]['regions'] = [
        [1, 1, 2, 1, 0, 0, 0, 0], [2, 1, 2, 8, 1, 0, 0, 0]]
    assert gate.check_patches(report, fixture, lcov)['fixture']['regions'] == [1, 1]
print('Owned dependency gates: exactly 95 line/region and missing instrumentation rejected')

# Global wrapper selection includes both patched dependencies, but not registries.
import os, subprocess
wrapper=root/'scripts/rustc-coverage.py'
for manifest,expected in [(str(root),True),(str(root/'vendor/prost-derive-0.14.4'),True),(str(root/'vendor/prost-reflect-0.16.5'),True),('/tmp/unowned-protobuf',False),(None,False)]:
    env=os.environ.copy();env['GZ_MSGS_COVERAGE_ROOT']=str(root)
    if manifest is None:env.pop('CARGO_MANIFEST_DIR',None)
    else:env['CARGO_MANIFEST_DIR']=manifest
    output=subprocess.check_output([str(wrapper),'/bin/echo','--crate-name','fixture'],env=env,text=True).strip()
    assert output=='--crate-name fixture'+(' -C instrument-coverage' if expected else '')
print('Coverage selection: owned root/two patched vendors included; unowned/query invocations excluded')

# Hunk data must not become a filename or redirect a later hunk.
with tempfile.TemporaryDirectory() as directory:
    patch = pathlib.Path(directory) / 'source.patch'
    patch.write_text('--- lib.rs\n+++ lib.rs\n@@ -1 +1 @@\n-old\n+++ other.rs\n@@ -3 +3 @@\n-old\n+call();\n')
    assert gate.added_patch_lines(patch, 'vendor/lib.rs') == [1, 3]
    assert gate.added_patch_lines(patch, 'vendor/other.rs') == []
    patch.write_text('--- a.rs\n+++ a.rs\n@@ -0,0 +1 @@\n+first();\n--- b.rs\n+++ b.rs\n@@ -1 +1,2 @@\n context\n+second();\n')
    assert gate.added_patch_lines(patch, 'vendor/a.rs') == [1]
    assert gate.added_patch_lines(patch, 'vendor/b.rs') == [2]
    for invalid in ['--- lib.rs\n+++ lib.rs\n@@ -0,0 +1,2 @@\n+one\n',
                    '--- lib.rs\n+++ lib.rs\n@@ -0,0 +1 @@\n+one\n+extra\n',
                    '--- lib.rs\n+++ lib.rs\n@@ malformed\n+one\n']:
        patch.write_text(invalid)
        try: gate.added_patch_lines(patch, 'vendor/lib.rs'); rejected = False
        except ValueError: rejected = True
        assert rejected
print('Patch selector: header-like source stays in its file; declared hunk lengths and file boundaries enforced')

# Exercise the selector against a real Git-generated raw-string replacement.
with tempfile.TemporaryDirectory() as directory:
    fixture = pathlib.Path(directory)
    def git(*args):
        return subprocess.check_output(['git', *args], cwd=fixture, text=True)
    git('init', '-q')
    source = fixture / 'lib.rs'
    original = ['const TEXT: &str = r#"', 'before', '"#;'] + ['// gap'] * 12 + ['fn f() { old(); }']
    source.write_text('\n'.join(original) + '\n')
    git('add', 'lib.rs')
    modified = original.copy(); modified[1] = '++ other.rs'; modified[-1] = 'fn f() { changed(); }'
    source.write_text('\n'.join(modified) + '\n')
    patch = fixture / 'real.patch'
    patch.write_text(git('diff', '--no-ext-diff', '--unified=0'))
    assert gate.added_patch_lines(patch, 'b/lib.rs') == [2, 16]
    assert gate.added_patch_lines(patch, 'other.rs') == []
print('Patch selector: real Git raw-string replacement preserves the later hunk owner')

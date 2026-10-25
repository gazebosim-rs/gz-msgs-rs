#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
import hashlib, json, os, pathlib, subprocess, sys
root = pathlib.Path(__file__).resolve().parents[1]
if '--staged' in sys.argv:
    from snapshot import verify_staged
    verify_staged(root)
    raise SystemExit()
os.chdir(root)
manifest = json.loads(pathlib.Path('schemas/manifest.json').read_text())
assert {p['path'] for p in manifest['schemas']} == {str(p) for p in pathlib.Path('schemas').rglob('*.proto')}
for item in manifest['schemas']:
    assert hashlib.sha256(pathlib.Path(item['path']).read_bytes()).hexdigest() == item['sha256'], item['path']
wire = json.loads(pathlib.Path('tests/fixtures/wire-compatibility.json').read_text())
assert wire['schema_manifest_sha256'] == hashlib.sha256(pathlib.Path('schemas/manifest.json').read_bytes()).hexdigest()
assert wire['probe_source_sha256'] == hashlib.sha256(pathlib.Path('tests/wire_peer.cc').read_bytes()).hexdigest()
options = json.loads(pathlib.Path('tests/fixtures/cli-options.json').read_text())
assert options['upstream_commit'] == manifest['commit']
assert options['schema_manifest_sha256'] == hashlib.sha256(pathlib.Path('schemas/manifest.json').read_bytes()).hexdigest()
info = json.loads(pathlib.Path('schemas/cli-info.manifest.json').read_text())
assert hashlib.sha256(pathlib.Path('schemas/cli-info.json').read_bytes()).hexdigest() == info['corpus_sha256']
assert hashlib.sha256(pathlib.Path('schemas/manifest.json').read_bytes()).hexdigest() == info['schema_manifest_sha256']
for item in json.loads(pathlib.Path('tests/reference/manifest.json').read_text())['files']:
    assert hashlib.sha256(pathlib.Path(item['path']).read_bytes()).hexdigest() == item['sha256'], item['path']
vendor = json.loads(pathlib.Path('vendor/manifest.json').read_text())
for package in vendor['packages']:
    paths = {item['path'] for item in package['files']}
    base = pathlib.Path('vendor') / (package['name'] + '-' + package['version'])
    assert paths == {str(p) for p in base.rglob('*') if p.is_file()}, package['name']
    for item in package['files']:
        assert hashlib.sha256(pathlib.Path(item['path']).read_bytes()).hexdigest() == item['sha256'], item['path']
        assert (item['upstream_sha256'] == item['sha256']) == ('owned_lines' not in item), item['path']
for command in [[os.environ.get('CARGO', 'cargo'), 'fmt', '--all', '--', '--check'], [os.environ.get('CARGO', 'cargo'), 'clippy', '--locked', '--all-targets', '--', '-D', 'warnings'], ['python3', 'scripts/test-gates.py'], ['python3', 'scripts/test-snapshot.py'], ['python3', 'scripts/test-source-package.py'], ['python3', 'scripts/test-downstream.py'], ['python3', 'scripts/coverage.py']]:
    subprocess.run(command, check=True)

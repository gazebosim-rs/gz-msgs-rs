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
flags = json.loads(pathlib.Path('tests/fixtures/cli-flag-values.json').read_text())
for key in ['upstream_commit', 'native_binary_sha256', 'native_source_sha256', 'schema_manifest_sha256', 'native_argv0']:
    assert flags[key] == options[key], key
vectors = json.loads(pathlib.Path('tests/fixtures/cli-info-vectors.json').read_text())
for key in ['upstream_commit', 'native_binary_sha256', 'native_source_sha256', 'schema_manifest_sha256', 'native_argv0']:
    assert vectors[key] == options[key], key
external = json.loads(pathlib.Path('tests/fixtures/cli-external-list.json').read_text())
for key in ['upstream_commit', 'native_binary_sha256', 'native_source_sha256', 'schema_manifest_sha256', 'native_argv0']:
    assert external[key] == options[key], key
for item in external['inputs'].values():
    assert hashlib.sha256(pathlib.Path(item['path']).read_bytes()).hexdigest() == item['sha256'], item['path']
definitions = json.loads(pathlib.Path('tests/fixtures/definitions/cli.json').read_text())
for key in ['upstream_commit', 'native_binary_sha256', 'native_source_sha256', 'schema_manifest_sha256', 'native_argv0']:
    assert definitions[key] == options[key], key
assert definitions['peer_source_sha256'] == hashlib.sha256(pathlib.Path('tests/definition_peer.cc').read_bytes()).hexdigest()
defaults = json.loads(pathlib.Path('tests/fixtures/byte-defaults/native.json').read_text())
assert defaults['default_probe_source_sha256'] == hashlib.sha256(pathlib.Path('tests/byte_defaults_peer.cc').read_bytes()).hexdigest()
for row in defaults['cases']:
    assert hashlib.sha256(bytes.fromhex(row['descriptor_hex'])).hexdigest() == row['descriptor_sha256']
integers = json.loads(pathlib.Path('tests/fixtures/integer-defaults/native.json').read_text())
assert integers['peer_source_sha256'] == hashlib.sha256(pathlib.Path('tests/integer_defaults_peer.cc').read_bytes()).hexdigest()
for key, name in [('study_sha256', 'study-190.json'), ('legacy_study_sha256', 'legacy-190.json'), ('shape_corpus_sha256', 'shapes.json'), ('lineage_sha256', 'lineage.json')]:
    assert integers[key] == hashlib.sha256(pathlib.Path('tests/fixtures/integer-defaults', name).read_bytes()).hexdigest()
lineage = json.loads(pathlib.Path('tests/fixtures/integer-defaults/lineage.json').read_text())
assert lineage['reference_script_sha256'] == hashlib.sha256(pathlib.Path('scripts/integer-defaults-reference.py').read_bytes()).hexdigest()
for row in integers['cases']:
    assert hashlib.sha256(bytes.fromhex(row['descriptor_hex'])).hexdigest() == row['descriptor_sha256']
for row in json.loads(pathlib.Path('tests/fixtures/integer-defaults/shapes.json').read_text()):
    assert hashlib.sha256(bytes.fromhex(row['descriptor_hex'])).hexdigest() == row['sha256']
for path, expected in definitions['inputs'].items():
    assert hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest() == expected, path
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

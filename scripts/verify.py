#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
import argparse, hashlib, json, os, pathlib, subprocess, sys
from changed import make_manifest, run_checks, validate_manifest
root = pathlib.Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('--staged', action='store_true')
parser.add_argument('--changed-lines', type=pathlib.Path)
parser.add_argument('--base')
args = parser.parse_args()
if args.staged:
    if args.changed_lines or args.base:
        parser.error('--staged chooses its own exact index and parent')
    from snapshot import verify_staged
    verify_staged(root)
    raise SystemExit()
if args.changed_lines:
    changed = json.loads(args.changed_lines.read_text())
else:
    base = args.base
    if base and set(base) == {'0'}:
        base = None  # A new branch has no earlier commit: measure all production lines.
    elif base is None:
        base = 'HEAD^'
    changed = make_manifest(root, root, base)
validate_manifest(root, changed)
os.chdir(root)
manifest = json.loads(pathlib.Path('schemas/manifest.json').read_text())
assert {p['path'] for p in manifest['schemas']} == {str(p) for p in pathlib.Path('schemas').rglob('*.proto')}
for item in manifest['schemas']:
    assert hashlib.sha256(pathlib.Path(item['path']).read_bytes()).hexdigest() == item['sha256'], item['path']
wire = json.loads(pathlib.Path('tests/fixtures/wire-compatibility.json').read_text())
assert wire['schema_manifest_sha256'] == hashlib.sha256(pathlib.Path('schemas/manifest.json').read_bytes()).hexdigest()
assert wire['probe_source_sha256'] == hashlib.sha256(pathlib.Path('tests/wire_peer.cc').read_bytes()).hexdigest()
packed = json.loads(pathlib.Path('tests/fixtures/packed-wire.json').read_text())
assert packed['upstream_commit'] == manifest['commit']
assert packed['schema_manifest_sha256'] == hashlib.sha256(pathlib.Path('schemas/manifest.json').read_bytes()).hexdigest()
assert packed['probe_source_sha256'] == hashlib.sha256(pathlib.Path('tests/wire_peer.cc').read_bytes()).hexdigest()
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
field_options = json.loads(pathlib.Path('tests/fixtures/field-options.json').read_text())
for key in ['upstream_commit', 'native_binary_sha256', 'native_source_sha256', 'native_libraries_sha256', 'schema_manifest_sha256', 'peer_source_sha256']:
    assert field_options[key] == definitions[key], key
assert field_options['reference_script_sha256'] == hashlib.sha256(pathlib.Path('scripts/field-options-reference.py').read_bytes()).hexdigest()
enum_options = json.loads(pathlib.Path('tests/fixtures/enum-field-options.json').read_text())
for key in ['upstream_commit', 'native_binary_sha256', 'native_source_sha256', 'native_libraries_sha256', 'schema_manifest_sha256', 'peer_source_sha256']:
    assert enum_options[key] == definitions[key], key
assert enum_options['reference_script_sha256'] == hashlib.sha256(pathlib.Path('scripts/enum-field-options-reference.py').read_bytes()).hexdigest()
message_options = json.loads(pathlib.Path('tests/fixtures/message-options.json').read_text())
for key in ['upstream_commit', 'native_binary_sha256', 'native_source_sha256', 'native_libraries_sha256', 'schema_manifest_sha256', 'peer_source_sha256']:
    assert message_options[key] == definitions[key], key
assert message_options['reference_script_sha256'] == hashlib.sha256(pathlib.Path('scripts/message-options-reference.py').read_bytes()).hexdigest()
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
commands = [[os.environ.get('CARGO', 'cargo'), 'fmt', '--all', '--', '--check'], [os.environ.get('CARGO', 'cargo'), 'clippy', '--locked', '--all-targets', '--', '-D', 'warnings'], ['python3', 'scripts/test-gates.py'], ['python3', 'scripts/test-changed.py'], ['python3', 'scripts/test-snapshot.py'], ['python3', 'scripts/test-source-package.py'], ['python3', 'scripts/test-downstream.py'], ['python3', 'scripts/json-reference.py']]
commands += [
    ['python3', 'tools/test_codec_benchmark.py'],
    ['cargo', 'fmt', '--manifest-path', 'tools/codec-benchmark/Cargo.toml', '--', '--check'],
    ['cargo', 'clippy', '--locked', '--release', '--manifest-path', 'tools/codec-benchmark/Cargo.toml', '--', '-D', 'warnings'],
    ['python3', 'tools/codec_benchmark.py', '--verify-only'],
]
run_checks(root, changed, commands)

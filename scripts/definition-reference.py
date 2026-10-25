#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay complete native external definitions with pinned descriptor inputs."""
import hashlib, json, os, pathlib, subprocess, tempfile

root = pathlib.Path(__file__).resolve().parents[1]
fixture_path = root / 'tests/fixtures/definitions/cli.json'
fixture = json.loads(fixture_path.read_text())
reference = root.parent / 'gz-msgs'
binary = reference / 'build/bin/gz-msgs'
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=reference, text=True).strip()
assert revision == fixture['upstream_commit']
source = subprocess.check_output(['git', 'show', revision + ':core/cmd/msgs_main.cc'], cwd=reference)
assert hashlib.sha256(source).hexdigest() == fixture['native_source_sha256'] == sha(reference / 'core/cmd/msgs_main.cc')
assert sha(binary) == fixture['native_binary_sha256']
assert sha(root / 'schemas/manifest.json') == fixture['schema_manifest_sha256']
assert sha(root / 'tests/definition_peer.cc') == fixture['peer_source_sha256']
libraries = {}
for line in subprocess.check_output(['ldd', str(binary)], text=True).splitlines():
    fields = line.split()
    if len(fields) > 2 and fields[1] == '=>' and fields[0].startswith(('libgz-', 'libprotobuf')):
        libraries[fields[0]] = sha(pathlib.Path(fields[2]))
assert libraries == fixture['native_libraries_sha256']
for path, expected in fixture['inputs'].items():
    assert sha(root / path) == expected, path
with tempfile.TemporaryDirectory(prefix='gz-msgs-definition-') as temporary:
    temporary = pathlib.Path(temporary)
    peer = temporary / 'peer'
    subprocess.run(['c++', '-std=c++17', '-Wall', '-Wextra', '-Werror', str(root / 'tests/definition_peer.cc'), '-lprotobuf', '-o', str(peer)], check=True)
    for name in ['supported', 'unsupported', 'legacy', 'reserved', 'ranges']:
        output = temporary / (name + '.desc')
        subprocess.run(['protoc', '--proto_path=' + str(root / 'tests/fixtures/definitions'), '--proto_path=' + str(root / 'schemas'), '--include_imports', '--descriptor_set_out=' + str(output), name + '.proto'], check=True)
        assert output.read_bytes() == (root / f'tests/fixtures/definitions/{name}.desc').read_bytes()
    unknown = temporary / 'unknown.desc'
    subprocess.run([str(peer), str(root / 'tests/fixtures/definitions/supported.desc'), str(unknown)], check=True)
    assert unknown.read_bytes() == (root / 'tests/fixtures/definitions/unknown-options.desc').read_bytes()
    for row in fixture['cases']:
        env = dict(os.environ, GZ_DESCRIPTOR_PATH=str(root / row['descriptor']))
        result = subprocess.run([fixture['native_argv0'], *row['args']], executable=binary, env=env, capture_output=True, text=True)
        assert (result.returncode, result.stdout, result.stderr) == (row['status'], row['stdout'], row['stderr']), row['args']
proof = dict(upstream_commit=revision, native_binary_sha256=sha(binary), native_source_sha256=fixture['native_source_sha256'], native_libraries_sha256=libraries, schema_manifest_sha256=fixture['schema_manifest_sha256'], peer_source_sha256=fixture['peer_source_sha256'], inputs=fixture['inputs'], fixture_sha256=sha(fixture_path), cases=len(fixture['cases']), scope='Complete native CLI output/status, including cases explicitly Unsupported by the bounded Rust formatter; fixture regeneration and dynamic-descriptor differential tests are separate.')
output = root / 'artifacts/definition-reference.json'
output.parent.mkdir(exist_ok=True)
output.write_text(json.dumps(proof, indent=2) + '\n')
print(f"{len(fixture['cases'])} complete native external definition observations reproduced")

#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay the generated native parser against the bounded wire corpus."""
import argparse, hashlib, json, os, pathlib, shlex, subprocess
root = pathlib.Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('--output', type=pathlib.Path, default=root / 'artifacts/wire-reference')
args = parser.parse_args()
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=True)
fixture = json.loads((root / 'tests/fixtures/wire-compatibility.json').read_text())
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
reference = root.parent / 'gz-msgs'
assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=reference, text=True).strip() == fixture['upstream_commit']
manifest = json.loads((root / 'schemas/manifest.json').read_text())
for item in manifest['schemas']:
    relative = item['path'].removeprefix('schemas/')
    source = reference / 'proto' / relative
    original = subprocess.check_output(['git', 'show', fixture['upstream_commit'] + ':proto/' + relative], cwd=reference)
    assert hashlib.sha256(original).hexdigest() == item['sha256'] == sha(source)
assert fixture['probe_source_sha256'] == sha(root / 'tests/wire_peer.cc')
flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'gz-msgs'], text=True))
libdir = pathlib.Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'gz-msgs'], text=True).strip())
version = subprocess.check_output(['pkg-config', '--modversion', 'protobuf'], text=True).strip()
assert version == fixture['native_protobuf']
binary = out / 'peer'
subprocess.run(['c++', '-O2', str(root / 'tests/wire_peer.cc'), '-o', str(binary), *flags], check=True)
env = dict(os.environ, LD_LIBRARY_PATH=str(libdir) + ':' + os.environ.get('LD_LIBRARY_PATH', ''))
for row in fixture['rows']:
    for mode, key in [([], 'cpp_generated'), (['known'], 'cpp_known')]:
        result = subprocess.run([str(binary), row['type'], *mode], input=bytes.fromhex(row['wire_hex']), capture_output=True, env=env)
        assert result.returncode == row[key]['exit'] and result.stdout.hex() == row[key]['output_hex'], row['id']
proof = dict(upstream_commit=fixture['upstream_commit'], fixture_sha256=sha(root / 'tests/fixtures/wire-compatibility.json'),
             schema_manifest_sha256=sha(root / 'schemas/manifest.json'), probe_source_sha256=sha(root / 'tests/wire_peer.cc'),
             probe_binary_sha256=sha(binary), native_library_sha256=sha(libdir / 'libgz-msgs.so'),
             protobuf_version=version, cases=len(fixture['rows']), outputs_and_statuses_match=True)
(out / 'proof.json').write_text(json.dumps(proof, indent=2) + '\n')
print(f"{len(fixture['rows'])} generated native complete outputs/statuses match")

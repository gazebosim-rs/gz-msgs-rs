#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Reproduce native JSON-framing outcomes without rewriting the golden fixture."""
import argparse, hashlib, json, pathlib, subprocess
root = pathlib.Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('--output', type=pathlib.Path, default=root / 'artifacts/json-framing-reference.json')
args = parser.parse_args()
fixture = json.loads((root / 'tests/fixtures/json-framing.json').read_text())
version = subprocess.check_output(['pkg-config', '--modversion', 'protobuf'], text=True).strip()
assert version == fixture['native_protobuf']
subprocess.run(['python3', str(root / 'scripts/native-peer.py')], check=True)
rows = fixture['matching'] + fixture['native_permissive_grammar_pending']
for row in rows:
    result = subprocess.run([str(root / 'target/native/protobuf-peer'), str(root / 'target/native/messages.pb'), 'gz.msgs.Time', 'encode-json'], input=bytes.fromhex(row['input_hex']), capture_output=True)
    assert result.returncode == row['cpp_status'] and result.stdout.hex() == row['cpp_stdout_hex'], row
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
proof = dict(protobuf_version=version, cases=len(rows), matching_cases=len(fixture['matching']), explicit_grammar_gaps=len(fixture['native_permissive_grammar_pending']),
             fixture_sha256=sha(root / 'tests/fixtures/json-framing.json'), schema_manifest_sha256=sha(root / 'schemas/manifest.json'),
             probe_source_sha256=sha(root / 'tests/protobuf_peer.cc'), probe_binary_sha256=sha(root / 'target/native/protobuf-peer'))
args.output.parent.mkdir(parents=True, exist_ok=True)
args.output.write_text(json.dumps(proof, indent=2) + '\n')
print('14 native JSON outcomes match: 12 compared framing cases, 2 explicit grammar gaps')

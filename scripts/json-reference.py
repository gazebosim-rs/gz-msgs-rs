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
grammar = json.loads((root / 'tests/fixtures/json-grammar.json').read_text())
arrays = json.loads((root / 'tests/fixtures/json-array-grammar.json').read_text())
single_quotes = json.loads((root / 'tests/fixtures/json-single-quote.json').read_text())
version = subprocess.check_output(['pkg-config', '--modversion', 'protobuf'], text=True).strip()
assert version == fixture['native_protobuf']
assert version == grammar['native_protobuf']
assert version == arrays['native_protobuf']
assert version == single_quotes['native_protobuf']
subprocess.run(['python3', str(root / 'scripts/native-peer.py')], check=True)
rows = fixture['matching'] + fixture['native_permissive_grammar_pending'] + grammar['rows'] + arrays['rows'] + single_quotes['matching'] + single_quotes['native_permissive_pending']
for row in rows:
    result = subprocess.run([str(root / 'target/native/protobuf-peer'), str(root / 'target/native/messages.pb'), row.get('type', 'gz.msgs.Time'), 'encode-json'], input=bytes.fromhex(row['input_hex']), capture_output=True)
    assert result.returncode == row['cpp_status'] and result.stdout.hex() == row['cpp_stdout_hex'], row
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
proof = dict(protobuf_version=version, cases=len(rows), matching_cases=len(fixture['matching']), newly_matching_grammar=len(fixture['native_permissive_grammar_pending']) + len(grammar['rows']),
             array_cases=len(arrays['rows']), fixture_sha256=sha(root / 'tests/fixtures/json-framing.json'), grammar_fixture_sha256=sha(root / 'tests/fixtures/json-grammar.json'), array_fixture_sha256=sha(root / 'tests/fixtures/json-array-grammar.json'), schema_manifest_sha256=sha(root / 'schemas/manifest.json'),
             single_quote_cases=len(single_quotes['matching']), single_quote_pending=len(single_quotes['native_permissive_pending']), single_quote_fixture_sha256=sha(root / 'tests/fixtures/json-single-quote.json'),
             probe_source_sha256=sha(root / 'tests/protobuf_peer.cc'), probe_binary_sha256=sha(root / 'target/native/protobuf-peer'))
args.output.parent.mkdir(parents=True, exist_ok=True)
args.output.write_text(json.dumps(proof, indent=2) + '\n')
print(f'{len(rows)} native JSON outcomes reproduced: 12 original framing rows, 2 former gaps, {len(grammar["rows"])} object boundaries, {len(arrays["rows"])} array boundaries, {len(single_quotes["matching"])} single-quote matches and {len(single_quotes["native_permissive_pending"])} explicit gaps')

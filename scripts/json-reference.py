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
raw_controls = json.loads((root / 'tests/fixtures/json-raw-controls.json').read_text())
escaped_bytes = json.loads((root / 'tests/fixtures/json-escaped-bytes.json').read_text())
escaped_utf8 = json.loads((root / 'tests/fixtures/json-escaped-utf8.json').read_text())
trailing_dot = json.loads((root / 'tests/fixtures/json-trailing-dot.json').read_text())
negative_zero = json.loads((root / 'tests/fixtures/json-negative-zero.json').read_text())
vtff = json.loads((root / 'tests/fixtures/json-vtff-whitespace.json').read_text())
quoted_hex = json.loads((root / 'tests/fixtures/json-quoted-hex.json').read_text())
repeated_data = json.loads((root / 'tests/fixtures/json-repeated-data.json').read_text())
repeated_integer = json.loads((root / 'tests/fixtures/json-repeated-integer-data.json').read_text())
repeated_string = json.loads((root / 'tests/fixtures/json-repeated-string-data.json').read_text())
single_scalar = json.loads((root / 'tests/fixtures/json-single-scalar-data.json').read_text())
uint64_range = json.loads((root / 'tests/fixtures/json-uint64-range.json').read_text())
uint64_occurrences = json.loads((root / 'tests/fixtures/json-uint64-occurrences.json').read_text())
uint64_hex = json.loads((root / 'tests/fixtures/json-uint64-unquoted-hex.json').read_text())
version = subprocess.check_output(['pkg-config', '--modversion', 'protobuf'], text=True).strip()
assert version == fixture['native_protobuf']
assert version == grammar['native_protobuf']
assert version == arrays['native_protobuf']
assert version == single_quotes['native_protobuf']
assert version == raw_controls['native_protobuf']
assert version == escaped_bytes['native_protobuf']
assert version == escaped_utf8['native_protobuf']
assert version == trailing_dot['native_protobuf']
assert version == negative_zero['native_protobuf']
assert version == vtff['native_protobuf']
assert version == quoted_hex['native_protobuf']
assert version == repeated_data['native_protobuf']
assert version == repeated_integer['native_protobuf']
assert version == repeated_string['native_protobuf']
assert version == single_scalar['native_protobuf']
assert version == uint64_range['native_protobuf']
assert version == uint64_occurrences['native_protobuf']
assert version == uint64_hex['native_protobuf']
subprocess.run(['python3', str(root / 'scripts/native-peer.py')], check=True)
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == escaped_bytes['probe_source_sha256']
assert hashlib.sha256((root / 'target/native/protobuf-peer').read_bytes()).hexdigest() == escaped_bytes['probe_binary_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == escaped_utf8['probe_source_sha256']
assert hashlib.sha256((root / 'target/native/protobuf-peer').read_bytes()).hexdigest() == escaped_utf8['probe_binary_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == trailing_dot['probe_source_sha256']
assert hashlib.sha256((root / 'target/native/protobuf-peer').read_bytes()).hexdigest() == trailing_dot['probe_binary_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == negative_zero['probe_source_sha256']
assert hashlib.sha256((root / 'target/native/protobuf-peer').read_bytes()).hexdigest() == negative_zero['probe_binary_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == vtff['probe_source_sha256']
assert hashlib.sha256((root / 'target/native/protobuf-peer').read_bytes()).hexdigest() == vtff['probe_binary_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == quoted_hex['probe_source_sha256']
assert hashlib.sha256((root / 'target/native/protobuf-peer').read_bytes()).hexdigest() == quoted_hex['probe_binary_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == repeated_data['probe_source_sha256']
assert hashlib.sha256((root / 'target/native/protobuf-peer').read_bytes()).hexdigest() == repeated_data['probe_binary_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == repeated_integer['probe_source_sha256']
assert hashlib.sha256((root / 'target/native/protobuf-peer').read_bytes()).hexdigest() == repeated_integer['probe_binary_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == repeated_string['probe_source_sha256']
assert hashlib.sha256((root / 'target/native/protobuf-peer').read_bytes()).hexdigest() == repeated_string['probe_binary_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == single_scalar['probe_source_sha256']
assert hashlib.sha256((root / 'target/native/protobuf-peer').read_bytes()).hexdigest() == single_scalar['probe_binary_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == uint64_range['probe_source_sha256']
assert hashlib.sha256((root / 'target/native/protobuf-peer').read_bytes()).hexdigest() == uint64_range['probe_binary_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == uint64_occurrences['probe_source_sha256']
assert hashlib.sha256((root / 'target/native/protobuf-peer').read_bytes()).hexdigest() == uint64_occurrences['probe_binary_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == uint64_hex['probe_source_sha256']
assert hashlib.sha256((root / 'target/native/protobuf-peer').read_bytes()).hexdigest() == uint64_hex['probe_binary_sha256']
for name, expected in uint64_hex['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
assert len(uint64_hex['rows']) == 220
assert sum(row['cpp_status'] == 0 for row in uint64_hex['rows']) == 116
assert len(uint64_hex['native_only_pending']) == 2
assert all(row['cpp_status'] == 0 and row['parent_rust_status'] != 0 for row in uint64_hex['native_only_pending'])
for name, expected in quoted_hex['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / 'tests/fixtures' / f'{name}.json').read_bytes()).hexdigest() == expected, name
assert len(quoted_hex['rows']) == quoted_hex['baseline_cases'] + quoted_hex['boundary_cases'] == 216
assert len(quoted_hex['repeated_data_pending']) == 2
for name, expected in repeated_data['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
assert len(repeated_data['rows']) == 66
assert len(repeated_integer['rows']) == 117
assert repeated_integer['types'] == ['Int32_V', 'UInt32_V', 'Int64_V']
for name, expected in repeated_integer['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
assert len(repeated_string['rows']) == 46
assert sum(row['cpp_status'] == 0 for row in repeated_string['rows']) == 34
for name, expected in repeated_string['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
assert single_scalar['types'] == ['Double_V', 'Float_V', 'Int32_V', 'UInt32_V', 'Int64_V', 'StringMsg_V']
assert len(single_scalar['rows']) == 95
assert sum(row['cpp_status'] == 0 for row in single_scalar['rows']) == 64
assert len(single_scalar['unquoted_hex_pending']) == 2
for name, expected in single_scalar['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
assert len(uint64_range['matching']) == 107
assert len(uint64_range['unqualified']) == 33
for name, expected in uint64_range['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
assert len(uint64_occurrences['matching']) == 185
assert len(uint64_occurrences['unqualified']) == 8
assert all(row['cpp_status'] == 0 and row['rust_status'] != 0 for row in uint64_occurrences['unqualified'])
for name, expected in uint64_occurrences['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
rows = fixture['matching'] + fixture['native_permissive_grammar_pending'] + grammar['rows'] + arrays['rows'] + single_quotes['matching'] + single_quotes['native_permissive_pending'] + raw_controls['matching'] + raw_controls['rejected'] + escaped_bytes['matching'] + escaped_bytes['rejected'] + escaped_utf8['matching'] + escaped_utf8['rejected'] + trailing_dot['scalar_rows'] + trailing_dot['composite_rows'] + trailing_dot['fraction_or_exponent_rejected'] + trailing_dot['native_permissive_pending'] + negative_zero['scalar_rows'] + negative_zero['composite_rows'] + negative_zero['native_permissive_pending'] + vtff['rows'] + quoted_hex['rows'] + quoted_hex['repeated_data_pending'] + repeated_data['rows'] + repeated_integer['rows'] + repeated_string['rows'] + single_scalar['rows'] + single_scalar['unquoted_hex_pending'] + uint64_range['matching'] + uint64_range['unqualified'] + uint64_occurrences['matching'] + uint64_occurrences['unqualified'] + uint64_hex['rows'] + uint64_hex['native_only_pending']
for row in rows:
    result = subprocess.run([str(root / 'target/native/protobuf-peer'), str(root / 'target/native/messages.pb'), row.get('type', 'gz.msgs.Time'), 'encode-json'], input=bytes.fromhex(row['input_hex']), capture_output=True)
    assert result.returncode == row['cpp_status'] and result.stdout.hex() == row['cpp_stdout_hex'], row
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
proof = dict(protobuf_version=version, cases=len(rows), matching_cases=len(fixture['matching']), newly_matching_grammar=len(fixture['native_permissive_grammar_pending']) + len(grammar['rows']),
             array_cases=len(arrays['rows']), fixture_sha256=sha(root / 'tests/fixtures/json-framing.json'), grammar_fixture_sha256=sha(root / 'tests/fixtures/json-grammar.json'), array_fixture_sha256=sha(root / 'tests/fixtures/json-array-grammar.json'), schema_manifest_sha256=sha(root / 'schemas/manifest.json'),
             single_quote_cases=len(single_quotes['matching']), single_quote_pending=len(single_quotes['native_permissive_pending']), single_quote_fixture_sha256=sha(root / 'tests/fixtures/json-single-quote.json'),
             raw_control_cases=len(raw_controls['matching']), raw_control_rejected=len(raw_controls['rejected']), raw_control_fixture_sha256=sha(root / 'tests/fixtures/json-raw-controls.json'),
             escaped_byte_cases=len(escaped_bytes['matching']), escaped_byte_rejected=len(escaped_bytes['rejected']), escaped_byte_fixture_sha256=sha(root / 'tests/fixtures/json-escaped-bytes.json'),
             escaped_utf8_cases=len(escaped_utf8['matching']), escaped_utf8_rejected=len(escaped_utf8['rejected']), escaped_utf8_fixture_sha256=sha(root / 'tests/fixtures/json-escaped-utf8.json'),
             trailing_dot_scalar_cases=len(trailing_dot['scalar_rows']), trailing_dot_composite_cases=len(trailing_dot['composite_rows']), trailing_dot_second_dot_rejected=len(trailing_dot['fraction_or_exponent_rejected']), trailing_dot_pending=len(trailing_dot['native_permissive_pending']), trailing_dot_fixture_sha256=sha(root / 'tests/fixtures/json-trailing-dot.json'),
             negative_zero_scalar_cases=len(negative_zero['scalar_rows']), negative_zero_composite_cases=len(negative_zero['composite_rows']), negative_zero_pending=len(negative_zero['native_permissive_pending']), negative_zero_fixture_sha256=sha(root / 'tests/fixtures/json-negative-zero.json'),
             vtff_cases=len(vtff['rows']), vtff_fixture_sha256=sha(root / 'tests/fixtures/json-vtff-whitespace.json'),
             quoted_hex_cases=len(quoted_hex['rows']), quoted_hex_repeated_data_pending=len(quoted_hex['repeated_data_pending']), quoted_hex_fixture_sha256=sha(root / 'tests/fixtures/json-quoted-hex.json'),
             repeated_data_cases=len(repeated_data['rows']), repeated_data_fixture_sha256=sha(root / 'tests/fixtures/json-repeated-data.json'),
             repeated_integer_cases=len(repeated_integer['rows']), repeated_integer_fixture_sha256=sha(root / 'tests/fixtures/json-repeated-integer-data.json'),
             repeated_string_cases=len(repeated_string['rows']), repeated_string_fixture_sha256=sha(root / 'tests/fixtures/json-repeated-string-data.json'),
             single_scalar_cases=len(single_scalar['rows']), single_scalar_unquoted_hex_pending=len(single_scalar['unquoted_hex_pending']), single_scalar_fixture_sha256=sha(root / 'tests/fixtures/json-single-scalar-data.json'),
             uint64_range_matching=len(uint64_range['matching']), uint64_range_unqualified=len(uint64_range['unqualified']), uint64_range_fixture_sha256=sha(root / 'tests/fixtures/json-uint64-range.json'),
             uint64_occurrence_cases=len(uint64_occurrences['matching']), uint64_occurrence_unquoted_hex_gaps=len(uint64_occurrences['unqualified']), uint64_occurrence_fixture_sha256=sha(root / 'tests/fixtures/json-uint64-occurrences.json'),
             uint64_unquoted_hex_cases=len(uint64_hex['rows']), uint64_unquoted_hex_pending=len(uint64_hex['native_only_pending']), uint64_unquoted_hex_fixture_sha256=sha(root / 'tests/fixtures/json-uint64-unquoted-hex.json'),
             probe_source_sha256=sha(root / 'tests/protobuf_peer.cc'), probe_binary_sha256=sha(root / 'target/native/protobuf-peer'))
args.output.parent.mkdir(parents=True, exist_ok=True)
args.output.write_text(json.dumps(proof, indent=2) + '\n')
print(f'{len(rows)} native JSON outcomes reproduced: 1468 retained rows, {len(uint64_hex["rows"])} bounded UInt64 hex rows and {len(uint64_hex["native_only_pending"])} native-only gaps')

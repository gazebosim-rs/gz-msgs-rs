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
uint64_undotted = json.loads((root / 'tests/fixtures/json-uint64-undotted-hex.json').read_text())
float_unquoted = json.loads((root / 'tests/fixtures/json-float-unquoted-hex.json').read_text())
leading_zero = json.loads((root / 'tests/fixtures/json-leading-zero-float.json').read_text())
time_float = json.loads((root / 'tests/fixtures/json-time-leading-zero.json').read_text())
quoted_float = json.loads((root / 'tests/fixtures/json-quoted-float-admission.json').read_text())
float_max = json.loads((root / 'tests/fixtures/json-float-max-range.json').read_text())
double_round = json.loads((root / 'tests/fixtures/json-double-decimal-rounding.json').read_text())
quoted_space = json.loads((root / 'tests/fixtures/json-quoted-float-whitespace.json').read_text())
quoted_hex_space = json.loads((root / 'tests/fixtures/json-quoted-hex-whitespace.json').read_text())
vector3d_quoted = json.loads((root / 'tests/fixtures/json-vector3d-quoted-numeric.json').read_text())
quaternion_quoted = json.loads((root / 'tests/fixtures/json-quaternion-quoted-numeric.json').read_text())
pose_quoted = json.loads((root / 'tests/fixtures/json-pose-quoted-numeric.json').read_text())
pose_merge = json.loads((root / 'tests/fixtures/json-pose-parent-merge.json').read_text())
pose_header = json.loads((root / 'tests/fixtures/json-pose-header-stamp-merge.json').read_text())
pose_data = json.loads((root / 'tests/fixtures/json-pose-header-data-append.json').read_text())
version = subprocess.check_output(['pkg-config', '--modversion', 'protobuf'], text=True).strip()
assert version == fixture['native_protobuf']
assert version == time_float['native_protobuf']
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
assert version == uint64_undotted['native_protobuf']
assert version == float_unquoted['native_protobuf']
assert version == leading_zero['native_protobuf']
assert version == quoted_float['native_protobuf']
assert version == float_max['native_protobuf']
assert version == double_round['native_protobuf']
assert version == quoted_space['native_protobuf']
assert version == quoted_hex_space['native_protobuf']
assert version == vector3d_quoted['native_protobuf']
assert version == quaternion_quoted['native_protobuf']
assert version == pose_quoted['native_protobuf']
assert version == pose_merge['native_protobuf']
subprocess.run(['python3', str(root / 'scripts/native-peer.py')], check=True)
# Fixture hashes describe the original reference build; ELF bytes vary across
# compilers and hosts. Pin their shared provenance, then check fresh behavior.
historical_peer_sha256 = '16744aebb5531163dc87224e6d6cec56bba40d4d1a62035ae465342649ac540b'
for prior in (
    escaped_bytes, escaped_utf8, trailing_dot, negative_zero, vtff, quoted_hex,
    quoted_hex_space, repeated_data, repeated_integer, repeated_string, single_scalar,
    uint64_range, uint64_occurrences, uint64_hex, uint64_undotted,
    float_unquoted, leading_zero, quoted_float, float_max, double_round, quoted_space,
):
    assert prior.get('probe_binary_sha256', prior.get('native_peer_sha256')) == historical_peer_sha256
current_peer_sha256 = hashlib.sha256((root / 'target/native/protobuf-peer').read_bytes()).hexdigest()
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == escaped_bytes['probe_source_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == escaped_utf8['probe_source_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == trailing_dot['probe_source_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == negative_zero['probe_source_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == vtff['probe_source_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == quoted_hex['probe_source_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == repeated_data['probe_source_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == repeated_integer['probe_source_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == repeated_string['probe_source_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == single_scalar['probe_source_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == uint64_range['probe_source_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == uint64_occurrences['probe_source_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == uint64_hex['probe_source_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == uint64_undotted['probe_source_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == float_unquoted['probe_source_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == leading_zero['probe_source_sha256']
assert hashlib.sha256((root / 'target/native/messages.pb').read_bytes()).hexdigest() == quoted_float['descriptor_sha256']
assert hashlib.sha256((root / 'target/native/messages.pb').read_bytes()).hexdigest() == float_max['descriptor_sha256']
assert hashlib.sha256((root / 'target/native/messages.pb').read_bytes()).hexdigest() == double_round['descriptor_sha256']
assert hashlib.sha256((root / 'target/native/messages.pb').read_bytes()).hexdigest() == quoted_space['descriptor_sha256']
assert hashlib.sha256((root / 'target/native/messages.pb').read_bytes()).hexdigest() == quoted_hex_space['descriptor_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == quoted_hex_space['probe_source_sha256']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == vector3d_quoted['probe_source_sha256']
assert hashlib.sha256((root / 'target/native/messages.pb').read_bytes()).hexdigest() == vector3d_quoted['descriptor_sha256']
assert vector3d_quoted['native_peer_sha256'] == historical_peer_sha256
assert len(vector3d_quoted['prior_fixture_sha256']) == 27
for name, expected in vector3d_quoted['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
assert len(vector3d_quoted['rows']) == 319
assert sum(row['cpp_status'] == 0 for row in vector3d_quoted['rows']) == 126
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == quaternion_quoted['probe_source_sha256']
assert hashlib.sha256((root / 'target/native/messages.pb').read_bytes()).hexdigest() == quaternion_quoted['descriptor_sha256']
assert quaternion_quoted['native_peer_sha256'] == historical_peer_sha256
assert len(quaternion_quoted['prior_fixture_sha256']) == 28
for name, expected in quaternion_quoted['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
assert len(quaternion_quoted['rows']) == 408
assert quaternion_quoted['native_positive'] == 155
assert sum(row['cpp_status'] == 0 for row in quaternion_quoted['rows']) == 155
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == pose_quoted['probe_source_sha256']
assert hashlib.sha256((root / 'target/native/messages.pb').read_bytes()).hexdigest() == pose_quoted['descriptor_sha256']
assert pose_quoted['native_peer_sha256'] == historical_peer_sha256
assert len(pose_quoted['prior_fixture_sha256']) == 29
for name, expected in pose_quoted['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
assert len(pose_quoted['rows']) == 694
assert sum(row['cpp_status'] == 0 for row in pose_quoted['rows']) == 257
assert len(pose_quoted['repeated_parent_pending']) == 4
assert all(row['cpp_status'] == 0 for row in pose_quoted['repeated_parent_pending'])
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == pose_merge['probe_source_sha256']
assert hashlib.sha256((root / 'target/native/messages.pb').read_bytes()).hexdigest() == pose_merge['descriptor_sha256']
assert pose_merge['native_peer_sha256'] == historical_peer_sha256
assert len(pose_merge['prior_fixture_sha256']) == 30
for name, expected in pose_merge['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
assert len(pose_merge['rows']) == 54
assert len({row['input_hex'] for row in pose_merge['rows']}) == 54
assert sum(row['cpp_status'] == 0 for row in pose_merge['rows']) == 47
assert version == pose_header['native_protobuf']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == pose_header['probe_source_sha256']
assert hashlib.sha256((root / 'target/native/messages.pb').read_bytes()).hexdigest() == pose_header['descriptor_sha256']
assert pose_header['native_peer_sha256'] == historical_peer_sha256
assert len(pose_header['prior_fixture_sha256']) == 31
for name, expected in pose_header['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
assert len(pose_header['rows']) == 73
assert len({row['input_hex'] for row in pose_header['rows']}) == 73
assert sum(row['cpp_status'] == 0 for row in pose_header['rows']) == 57
assert len(pose_header['repeated_header_data_pending']) == 2
assert all(row['cpp_status'] == 0 for row in pose_header['repeated_header_data_pending'])
assert version == pose_data['native_protobuf']
assert hashlib.sha256((root / 'tests/protobuf_peer.cc').read_bytes()).hexdigest() == pose_data['probe_source_sha256']
assert hashlib.sha256((root / 'target/native/messages.pb').read_bytes()).hexdigest() == pose_data['descriptor_sha256']
assert pose_data['native_peer_sha256'] == historical_peer_sha256
assert len(pose_data['prior_fixture_sha256']) == 32
for name, expected in pose_data['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
assert len(pose_data['rows']) == 43
assert sum(row['cpp_status'] == 0 for row in pose_data['rows']) == 35
assert len(pose_data['pending_grammar']) == 12
assert all(row['cpp_status'] == 0 for row in pose_data['pending_grammar'])
assert len({row['input_hex'] for row in pose_data['rows'] + pose_data['pending_grammar']}) == 55
assert len(quoted_hex_space['prior_fixture_sha256']) == 26
for name, expected in quoted_hex_space['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
assert len(quoted_hex_space['rows']) == 768
assert sum(row['cpp_status'] == 0 for row in quoted_hex_space['rows']) == 360
for name, expected in uint64_hex['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
assert len(uint64_hex['rows']) == 220
assert sum(row['cpp_status'] == 0 for row in uint64_hex['rows']) == 116
assert len(uint64_undotted['rows']) == 152
assert sum(row['cpp_status'] == 0 for row in uint64_undotted['rows']) == 76
assert len(float_unquoted['rows']) == 318
assert len(float_unquoted['leading_zero_pending']) == 6
assert sum(row['cpp_status'] == 0 for row in float_unquoted['rows']) == 201
for name, expected in float_unquoted['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
assert len(leading_zero['rows']) == 252
assert sum(row['cpp_status'] == 0 for row in leading_zero['rows']) == 180
assert len(leading_zero['time_pending']) == 6
assert all(row['cpp_status'] == 0 and row['parent_rust_status'] != 0 for row in leading_zero['time_pending'])
for name, expected in leading_zero['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
assert len(time_float['rows']) == 139
assert sum(row['cpp_status'] == 0 for row in time_float['rows']) == 71
assert len(time_float['prior_fixture_sha256']) == 21
for name, expected in time_float['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
assert len(quoted_float['rows']) == 144
assert sum(row['cpp_status'] == 0 for row in quoted_float['rows']) == 65
assert len(quoted_float['native_only_pending']) == 60
assert all(row['cpp_status'] == 0 and row['parent_rust_status'] != 0 for row in quoted_float['native_only_pending'])
assert len(quoted_float['whitespace_rejected']) == 48
assert all(row['cpp_status'] == 6 and row['parent_rust_status'] != 0 for row in quoted_float['whitespace_rejected'])
for name, expected in quoted_float['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
assert len(float_max['matching']) == 204
assert len(float_max['double_wire_pending']) == 18
assert all(row['type'] == 'gz.msgs.Double_V' and row['cpp_status'] == 0 for row in float_max['double_wire_pending'])
for name, expected in float_max['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
assert len(double_round['rows']) == 135
assert {category: sum(row['category'] == category for row in double_round['rows']) for category in ('corrected', 'matched', 'rejected', 'descriptor_control')} == {'corrected': 90, 'matched': 18, 'rejected': 18, 'descriptor_control': 9}
for name, expected in double_round['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
assert len(quoted_space['rows']) == 1104
assert sum(row['cpp_status'] == 0 for row in quoted_space['rows']) == 720
for name, expected in quoted_space['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
for name, expected in uint64_undotted['prior_fixture_sha256'].items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
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
rows = fixture['matching'] + fixture['native_permissive_grammar_pending'] + grammar['rows'] + arrays['rows'] + single_quotes['matching'] + single_quotes['native_permissive_pending'] + raw_controls['matching'] + raw_controls['rejected'] + escaped_bytes['matching'] + escaped_bytes['rejected'] + escaped_utf8['matching'] + escaped_utf8['rejected'] + trailing_dot['scalar_rows'] + trailing_dot['composite_rows'] + trailing_dot['fraction_or_exponent_rejected'] + trailing_dot['native_permissive_pending'] + negative_zero['scalar_rows'] + negative_zero['composite_rows'] + negative_zero['native_permissive_pending'] + vtff['rows'] + quoted_hex['rows'] + quoted_hex['repeated_data_pending'] + repeated_data['rows'] + repeated_integer['rows'] + repeated_string['rows'] + single_scalar['rows'] + single_scalar['unquoted_hex_pending'] + uint64_range['matching'] + uint64_range['unqualified'] + uint64_occurrences['matching'] + uint64_occurrences['unqualified'] + uint64_hex['rows'] + uint64_hex['native_only_pending'] + uint64_undotted['rows'] + float_unquoted['rows'] + float_unquoted['leading_zero_pending'] + leading_zero['rows'] + leading_zero['time_pending'] + time_float['rows'] + quoted_float['rows'] + quoted_float['native_only_pending'] + quoted_float['whitespace_rejected'] + float_max['matching'] + float_max['double_wire_pending'] + double_round['rows'] + quoted_space['rows'] + quoted_hex_space['rows']
rows += vector3d_quoted['rows']
rows += quaternion_quoted['rows']
rows += pose_quoted['rows'] + pose_quoted['repeated_parent_pending']
rows += pose_merge['rows']
rows += pose_header['rows'] + pose_header['repeated_header_data_pending']
rows += pose_data['rows'] + pose_data['pending_grammar']
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
             uint64_undotted_hex_cases=len(uint64_undotted['rows']), uint64_undotted_hex_fixture_sha256=sha(root / 'tests/fixtures/json-uint64-undotted-hex.json'),
             float_unquoted_hex_cases=len(float_unquoted['rows']), float_unquoted_leading_zero_pending=len(float_unquoted['leading_zero_pending']), float_unquoted_hex_fixture_sha256=sha(root / 'tests/fixtures/json-float-unquoted-hex.json'),
             leading_zero_float_cases=len(leading_zero['rows']), leading_zero_historical_time_cases=len(leading_zero['time_pending']), leading_zero_float_fixture_sha256=sha(root / 'tests/fixtures/json-leading-zero-float.json'),
             time_float_cases=len(time_float['rows']), time_float_native_positive=sum(row['cpp_status'] == 0 for row in time_float['rows']), time_float_fixture_sha256=sha(root / 'tests/fixtures/json-time-leading-zero.json'),
             quoted_float_cases=len(quoted_float['rows']), quoted_float_native_positive=sum(row['cpp_status'] == 0 for row in quoted_float['rows']), quoted_float_native_only_pending=len(quoted_float['native_only_pending']), quoted_float_whitespace_rejected=len(quoted_float['whitespace_rejected']), quoted_float_fixture_sha256=sha(root / 'tests/fixtures/json-quoted-float-admission.json'),
             float_max_matching=len(float_max['matching']), float_max_double_wire_pending=len(float_max['double_wire_pending']), float_max_fixture_sha256=sha(root / 'tests/fixtures/json-float-max-range.json'),
             double_round_cases=len(double_round['rows']), double_round_fixture_sha256=sha(root / 'tests/fixtures/json-double-decimal-rounding.json'),
             quoted_float_whitespace_cases=len(quoted_space['rows']), quoted_float_whitespace_native_positive=sum(row['cpp_status'] == 0 for row in quoted_space['rows']), quoted_float_whitespace_fixture_sha256=sha(root / 'tests/fixtures/json-quoted-float-whitespace.json'),
             quoted_hex_whitespace_cases=len(quoted_hex_space['rows']), quoted_hex_whitespace_native_positive=sum(row['cpp_status'] == 0 for row in quoted_hex_space['rows']), quoted_hex_whitespace_fixture_sha256=sha(root / 'tests/fixtures/json-quoted-hex-whitespace.json'),
             vector3d_quoted_cases=len(vector3d_quoted['rows']), vector3d_quoted_native_positive=sum(row['cpp_status'] == 0 for row in vector3d_quoted['rows']), vector3d_quoted_fixture_sha256=sha(root / 'tests/fixtures/json-vector3d-quoted-numeric.json'),
             quaternion_quoted_cases=len(quaternion_quoted['rows']), quaternion_quoted_native_positive=quaternion_quoted['native_positive'], quaternion_quoted_fixture_sha256=sha(root / 'tests/fixtures/json-quaternion-quoted-numeric.json'),
             pose_quoted_cases=len(pose_quoted['rows']), pose_quoted_native_positive=sum(row['cpp_status'] == 0 for row in pose_quoted['rows']), pose_repeated_parent_prior_pending=len(pose_quoted['repeated_parent_pending']), pose_quoted_fixture_sha256=sha(root / 'tests/fixtures/json-pose-quoted-numeric.json'),
             pose_merge_cases=len(pose_merge['rows']), pose_merge_native_positive=sum(row['cpp_status'] == 0 for row in pose_merge['rows']), pose_merge_fixture_sha256=sha(root / 'tests/fixtures/json-pose-parent-merge.json'),
             pose_header_cases=len(pose_header['rows']), pose_header_native_positive=sum(row['cpp_status'] == 0 for row in pose_header['rows']), pose_header_data_pending=len(pose_header['repeated_header_data_pending']), pose_header_fixture_sha256=sha(root / 'tests/fixtures/json-pose-header-stamp-merge.json'),
             pose_data_cases=len(pose_data['rows']), pose_data_native_positive=sum(row['cpp_status'] == 0 for row in pose_data['rows']), pose_data_pending=len(pose_data['pending_grammar']), pose_data_fixture_sha256=sha(root / 'tests/fixtures/json-pose-header-data-append.json'),
             probe_source_sha256=sha(root / 'tests/protobuf_peer.cc'), probe_binary_sha256=current_peer_sha256, historical_peer_binary_sha256=historical_peer_sha256)
args.output.parent.mkdir(parents=True, exist_ok=True)
args.output.write_text(json.dumps(proof, indent=2) + '\n')
print(f'{len(rows)} native JSON outcomes reproduced')

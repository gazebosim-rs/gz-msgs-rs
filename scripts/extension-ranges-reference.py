#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay ordinary proto2 extension-range declarations and complete native CLI observations."""
import argparse
import hashlib
import itertools
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]

def inputs():
    from google.protobuf import descriptor_pb2 as d
    rows = []
    spans = [[], [(1, 2)], [(10, 20)], [(100, 536870912)],
             [(536870911, 536870912)], [(20, 30), (1, 10)],
             [(10, 20), (20, 30)], [(10, 20), (15, 25)],
             [(0, 2)], [(-1, 2)], [(2, 2)], [(3, 2)],
             [(1, 536870913)], [(19000, 20000)],
             [(1, 2147483647)], [(2147483647, -2147483648)]]
    def add(syntax, ranges, layout, mode='ordinary'):
        files = d.FileDescriptorSet()
        file = files.file.add(name='ranges.proto', package='ranges', syntax=syntax)
        message = file.message_type.add(name='Root')
        target = message.nested_type.add(name='Child') if layout == 'nested' else message
        if layout == 'field':
            message.field.add(name='value', number=5, label=1, type=5, json_name='value')
        for start, end in ranges:
            target.extension_range.add(start=start, end=end)
        if mode.startswith('missing'):
            for field in mode.split('-')[1:]:
                target.extension_range[0].ClearField(field)
        elif mode.startswith('reserved'):
            start, end = {'reserved-overlap': (15, 25), 'reserved-touch': (20, 25),
                          'reserved-before': (1, 10)}[mode]
            target.reserved_range.add(start=start, end=end)
        elif mode == 'unknown':
            target.extension_range[0].options.MergeFromString(bytes([0x80, 0x7d, 7]))
        elif mode == 'empty-options':
            target.extension_range[0].options.SetInParent()
        elif mode == 'uninterpreted':
            option = target.extension_range[0].options.uninterpreted_option.add()
            option.name.add(name_part='unresolved', is_extension=True)
            option.identifier_value = 'true'
        rows.append(dict(id=len(rows), syntax=syntax, ranges=ranges, layout=layout,
                         mode=mode, descriptor_hex=files.SerializeToString().hex()))
    for syntax, ranges, layout in itertools.product(('proto2', 'proto3'), spans,
                                                   ('empty', 'field', 'nested')):
        add(syntax, ranges, layout)
    for mode, layout in itertools.product(('missing-start', 'missing-end',
            'missing-start-end', 'reserved-overlap', 'reserved-touch',
            'reserved-before', 'unknown', 'empty-options', 'uninterpreted'),
            ('empty', 'field', 'nested')):
        add('proto2', [(10, 20)], layout, mode)
    return rows

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--upstream', required=True, type=Path)
    parser.add_argument('--write-fixture', action='store_true')
    parser.add_argument('--output', type=Path, default=ROOT / 'artifacts/extension-ranges-reference')
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    path = ROOT / 'tests/fixtures/extension-ranges.json'
    baseline = json.loads((ROOT / 'tests/fixtures/definitions/cli.json').read_text())
    sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
    upstream = args.upstream.resolve()
    binary = upstream / 'build/bin/gz-msgs'
    assert subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], text=True).strip() == baseline['upstream_commit']
    assert sha(binary) == baseline['native_binary_sha256']
    assert sha(upstream / 'core/cmd/msgs_main.cc') == baseline['native_source_sha256']
    assert subprocess.check_output(['pkg-config', '--modversion', 'protobuf'], text=True).strip() == '3.21.12'
    libraries = {}
    for line in subprocess.check_output(['ldd', str(binary)], text=True).splitlines():
        words = line.split()
        if len(words) > 2 and words[1] == '=>' and words[0].startswith(('libgz-', 'libprotobuf')):
            libraries[words[0]] = sha(Path(words[2]))
    assert libraries == baseline['native_libraries_sha256']
    peer = out / 'peer'
    build = ['c++', '-std=c++17', '-Wall', '-Wextra', '-Werror',
             str(ROOT / 'tests/definition_peer.cc'), '-lprotobuf', '-o', str(peer)]
    subprocess.run(build, check=True)
    for line in subprocess.check_output(['ldd', str(peer)], text=True).splitlines():
        words = line.split()
        if len(words) > 2 and words[1] == '=>' and words[0].startswith('libprotobuf'):
            assert sha(Path(words[2])) == libraries[words[0]]
    old = None if args.write_fixture else json.loads(path.read_text())
    rows = inputs() if args.write_fixture else old['rows']
    for row in rows:
        input_path = out / 'input.desc'
        input_path.write_bytes(bytes.fromhex(row['descriptor_hex']))
        native = subprocess.run([str(peer), str(input_path)], capture_output=True, timeout=10)
        observation = dict(exit=native.returncode, stdout=native.stdout.decode(), stderr=native.stderr.decode())
        cli = subprocess.run(['gz-msgs', '--info', 'ranges.Root'], executable=binary,
            env=dict(os.environ, GZ_DESCRIPTOR_PATH=str(input_path)), capture_output=True, timeout=10)
        cli_observation = dict(exit=cli.returncode, stdout=cli.stdout.decode(),
            stderr=cli.stderr.decode().replace(str(input_path), '@descriptor@'))
        if args.write_fixture:
            row.update(native=observation, native_cli=cli_observation)
            row['expected'] = ('registry' if row['mode'] == 'uninterpreted' else
                'unknown options' if row['mode'] == 'unknown' else
                'native' if native.returncode == 0 else 'invalid extension range')
        else:
            assert observation == row['native'], row['id']
            assert cli_observation == row['native_cli'], row['id']
    fixture = {key: baseline[key] for key in ['upstream_commit', 'native_binary_sha256',
        'native_source_sha256', 'native_libraries_sha256', 'schema_manifest_sha256']}
    fixture.update(native_protobuf='3.21.12', peer_source_sha256=sha(ROOT / 'tests/definition_peer.cc'),
                   reference_script_sha256=sha(Path(__file__)), diagnostic_path_token='@descriptor@', rows=rows)
    if args.write_fixture:
        path.write_text(json.dumps(fixture, indent=2) + '\n')
    else:
        assert fixture == old
    proof = dict(cases=len(rows), supported=sum(r['expected'] == 'native' for r in rows),
        fixture_sha256=sha(path), command=build, libraries=libraries,
        native_outputs_and_statuses_match=True)
    (out / 'proof.json').write_text(json.dumps(proof, indent=2) + '\n')
    print(f'{len(rows)} native extension-range cases; {proof["supported"]} declarations in the supported subset')

if __name__ == '__main__':
    main()

#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay standard message-option declarations and complete native CLI observations."""
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
    def add(syntax, layout, flag, deprecated, message_set=None):
        files = d.FileDescriptorSet()
        file = files.file.add(name='message_options.proto', package='opts', syntax=syntax)
        message = file.message_type.add(name='Root')
        def options(value):
            if flag is not None:
                value.no_standard_descriptor_accessor = flag
            if deprecated is not None:
                value.deprecated = deprecated
            if message_set is not None:
                value.message_set_wire_format = message_set
        options(message.options)
        if layout in ('field', 'oneof'):
            field = message.field.add(name='value', number=1, label=1, type=5, json_name='value')
            if layout == 'oneof':
                message.oneof_decl.add(name='choice')
                field.oneof_index = 0
        elif layout == 'nested':
            child = message.nested_type.add(name='Child')
            options(child.options)
            child.field.add(name='value', number=1, label=1, type=5, json_name='value')
            message.field.add(name='child', number=1, label=1, type=11,
                              type_name='.opts.Root.Child', json_name='child')
        elif layout == 'map':
            child = message.nested_type.add(name='EntriesEntry')
            child.options.map_entry = True
            options(child.options)
            child.field.add(name='key', number=1, label=1, type=9)
            child.field.add(name='value', number=2, label=1, type=5)
            message.field.add(name='entries', number=1, label=3, type=11,
                              type_name='.opts.Root.EntriesEntry', json_name='entries')
        wire = files.SerializeToString(deterministic=True)
        rows.append(dict(id=f'{syntax}_{layout}_flag{flag}_deprecated{deprecated}_messageset{message_set}',
            syntax=syntax, layout=layout, flag=flag, deprecated=deprecated,
            message_set=message_set, descriptor_hex=wire.hex()))
    for syntax, layout, flag, deprecated in itertools.product(
            ('proto2', 'proto3'), ('empty', 'field', 'oneof', 'nested', 'map'),
            (None, False, True), (None, False, True)):
        add(syntax, layout, flag, deprecated)
    for syntax, layout, value in itertools.product(('proto2', 'proto3'), ('empty', 'field'), (False, True)):
        add(syntax, layout, True, False, value)
    return rows

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--upstream', required=True, type=Path)
    parser.add_argument('--write-fixture', action='store_true')
    parser.add_argument('--output', type=Path, default=ROOT / 'artifacts/message-options-reference')
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    path = ROOT / 'tests/fixtures/message-options.json'
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
        cli = subprocess.run(['gz-msgs', '--info', 'opts.Root'], executable=binary,
            env=dict(os.environ, GZ_DESCRIPTOR_PATH=str(input_path)), capture_output=True, timeout=10)
        cli_observation = dict(exit=cli.returncode, stdout=cli.stdout.decode(),
            stderr=cli.stderr.decode().replace(str(input_path), '@descriptor@'))
        if args.write_fixture:
            row.update(native=observation, native_cli=cli_observation)
            row['expected'] = 'native' if row['message_set'] is None else 'option'
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
    print(f'{len(rows)} native message-option cases; {proof["supported"]} declarations in the supported subset')

if __name__ == '__main__':
    main()

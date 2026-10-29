#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay enum-valued field-option declarations and complete native CLI observations."""
import argparse
import hashlib
import itertools
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]

def inputs():
    from google.protobuf import descriptor_pb2 as d, descriptor_pool, message_factory
    kinds = {
        'int32': 5, 'int64': 3, 'uint64': 4, 'sint64': 18, 'fixed64': 6,
        'sfixed64': 16, 'uint32': 13, 'sint32': 17, 'fixed32': 7, 'sfixed32': 15,
        'bool': 8, 'float': 2, 'double': 1, 'string': 9, 'bytes': 12,
        'message': 11, 'enum': 14,
    }
    # Generated proto2 options reject unknown enum assignments. Let protobuf's
    # integer serializer produce their wire values without a custom encoder.
    surrogate = d.FileDescriptorProto(
        name='options_surrogate.proto', package='study', syntax='proto2')
    message = surrogate.message_type.add(name='EnumOptions')
    for name in ('ctype', 'jstype'):
        message.field.add(name=name,
            number=d.FieldOptions.DESCRIPTOR.fields_by_name[name].number,
            label=1, type=5)
    pool = descriptor_pool.DescriptorPool()
    pool.Add(surrogate)
    option_ints = message_factory.MessageFactory(pool).GetPrototype(
        pool.FindMessageTypeByName('study.EnumOptions'))
    rows = []

    def add(syntax, label, kind, options, layout='ordinary'):
        files = d.FileDescriptorSet()
        file = files.file.add(name='enum_options.proto', package='opts', syntax=syntax)
        file.message_type.add(name='Leaf')
        enum = file.enum_type.add(name='Choice')
        enum.value.add(name='ZERO', number=0)
        enum.value.add(name='ONE', number=1)
        message = file.message_type.add(name='Root')
        field = message.field.add(name='value', number=1,
            label={'optional': 1, 'required': 2, 'repeated': 3}[label],
            type=kinds[kind])
        if kind == 'message':
            field.type_name = '.opts.Leaf'
        if kind == 'enum':
            field.type_name = '.opts.Choice'
        if layout in ('oneof', 'synthetic_optional'):
            message.oneof_decl.add(name='choice' if layout == 'oneof' else '_value')
            field.oneof_index = 0
            if layout == 'synthetic_optional':
                field.proto3_optional = True
        if layout == 'map':
            entry = message.nested_type.add(name='EntriesEntry')
            entry.options.map_entry = True
            entry.field.add(name='key', number=1, label=1, type=9)
            entry.field.add(name='value', number=2, label=1, type=3)
            field.name = 'entries'
            field.label = 3
            field.type = 11
            field.type_name = '.opts.Root.EntriesEntry'
        field.json_name = field.name
        for name, value in options.items():
            if name in ('ctype', 'jstype') and value not in (0, 1, 2):
                field.options.MergeFromString(
                    option_ints(**{name: value}).SerializeToString(deterministic=True))
            else:
                setattr(field.options, name, value)
        suffix = '_'.join(f'{name}{value}' for name, value in options.items())
        rows.append(dict(id=f'{syntax}_{label}_{kind}_{layout}_{suffix}',
            syntax=syntax, label=label, kind=kind, layout=layout, options=options,
            descriptor_hex=files.SerializeToString(deterministic=True).hex()))

    for syntax, label, kind, name, value in itertools.product(
            ('proto2', 'proto3'), ('optional', 'required', 'repeated'),
            kinds, ('ctype', 'jstype'), range(3)):
        add(syntax, label, kind, {name: value})
    for syntax, layout, kind, ctype, jstype in itertools.product(
            ('proto2', 'proto3'), ('ordinary', 'oneof', 'map', 'synthetic_optional'),
            ('string', 'int64'), range(3), range(3)):
        if syntax == 'proto2' and layout == 'synthetic_optional':
            continue
        if layout == 'map' and kind == 'int64':
            continue
        add(syntax, 'optional', kind,
            dict(ctype=ctype, jstype=jstype, lazy=False, unverified_lazy=False,
                 weak=True, deprecated=False), layout)
    for name, value, kind in itertools.product(
            ('ctype', 'jstype'), (-1, 3, 2147483647, -2147483648), ('string', 'int64')):
        add('proto2', 'optional', kind, {name: value})
    return rows

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--upstream', required=True, type=Path)
    parser.add_argument('--write-fixture', action='store_true')
    parser.add_argument('--output', type=Path, default=ROOT / 'artifacts/enum-field-options-reference')
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    path = ROOT / 'tests/fixtures/enum-field-options.json'
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
            if any(key in ('ctype', 'jstype') and value not in (0, 1, 2)
                   for key, value in row['options'].items()):
                row['expected'] = 'option'
            elif row['syntax'] == 'proto3' and row['label'] == 'required':
                row['expected'] = 'required'
            elif native.returncode == 0:
                row['expected'] = 'native'
            else:
                row['expected'] = 'jstype'
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
    print(f'{len(rows)} native field-option cases; {proof["supported"]} declarations in the supported subset')

if __name__ == '__main__':
    main()

#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay standard field-option declarations and complete native CLI observations."""
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
    def add(name, syntax, label, kind, options, layout='field', explicit=False, scope='boolean'):
        files = d.FileDescriptorSet()
        file = files.file.add(name='options.proto', package='opts', syntax=syntax)
        file.message_type.add(name='Leaf')
        message = file.message_type.add(name='Root')
        field = message.field.add(name='value', number=1,
            label={'optional': 1, 'required': 2, 'repeated': 3}[label],
            type={'int32': 5, 'string': 9, 'message': 11}[kind])
        if kind == 'message':
            field.type_name = '.opts.Leaf'
        if layout in ('oneof', 'proto3_optional'):
            message.oneof_decl.add(name='_value' if layout == 'proto3_optional' else 'choice')
            field.oneof_index = 0
            if layout == 'proto3_optional':
                field.proto3_optional = True
        if layout == 'map':
            entry = message.nested_type.add(name='EntriesEntry')
            entry.options.map_entry = True
            entry.field.add(name='key', number=1, label=1, type=9)
            entry.field.add(name='value', number=2, label=1, type=11, type_name='.opts.Leaf')
            field.name = 'entries'
            field.type_name = '.opts.Root.EntriesEntry'
        if explicit:
            field.json_name = field.name
        for key, value in options.items():
            setattr(field.options, key, value)
        rows.append(dict(id=name + ('_json_explicit' if explicit else '_json_absent'),
            family=name, syntax=syntax, label=label, kind=kind, layout=layout,
            json_explicit=explicit, scope=scope, options=options,
            descriptor_hex=files.SerializeToString(deterministic=True).hex()))
    for explicit in (False, True):
        for syntax, label, kind, option, enabled in itertools.product(
                ('proto2', 'proto3'), ('optional', 'required', 'repeated'),
                ('int32', 'string', 'message'), ('lazy', 'unverified_lazy', 'weak'), (False, True)):
            add(f'{syntax}_{label}_{kind}_{option}_{enabled}', syntax, label, kind,
                {option: enabled}, explicit=explicit)
        for syntax, oneof, lazy, unverified, weak in itertools.product(
                ('proto2', 'proto3'), (False, True), (False, True), (False, True), (None, False, True)):
            options = dict(lazy=lazy, unverified_lazy=unverified)
            if weak is not None:
                options['weak'] = weak
            add(f'{syntax}_oneof{oneof}_lazy{lazy}_unverified{unverified}_weak{weak}',
                syntax, 'optional', 'message', options,
                layout='oneof' if oneof else 'field', explicit=explicit)
        for syntax, option, enabled in itertools.product(
                ('proto2', 'proto3'), ('lazy', 'unverified_lazy', 'weak'), (False, True)):
            add(f'{syntax}_map_{option}_{enabled}', syntax, 'repeated', 'message',
                {option: enabled}, layout='map', explicit=explicit)
        for option, enabled in itertools.product(('lazy', 'unverified_lazy', 'weak'), (False, True)):
            add(f'proto3_optional_{option}_{enabled}', 'proto3', 'optional', 'message',
                {option: enabled}, layout='proto3_optional', explicit=explicit)
    for ctype, lazy in itertools.product(range(3), (False, True)):
        add(f'unsupported_ctype{ctype}_lazy{lazy}', 'proto2', 'optional', 'string',
            dict(ctype=ctype, lazy=lazy), explicit=True, scope='unsupported_ctype')
    return rows

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--upstream', required=True, type=Path)
    parser.add_argument('--write-fixture', action='store_true')
    parser.add_argument('--output', type=Path, default=ROOT / 'artifacts/field-options-reference')
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    path = ROOT / 'tests/fixtures/field-options.json'
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
            if row['scope'] == 'unsupported_ctype':
                row['expected'] = 'option'
            elif native.returncode == 0:
                row['expected'] = 'native'
            elif row['syntax'] == 'proto3' and row['label'] == 'required':
                row['expected'] = 'required'
            else:
                row['expected'] = 'lazy'
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

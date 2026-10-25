#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Capture exact native ordinary proto2 integer defaults; never normalize input."""
from pathlib import Path
import hashlib
import json
import os
import re
import subprocess
from google.protobuf.descriptor_pb2 import FileDescriptorSet

root = Path(__file__).resolve().parents[1]
out = root / 'artifacts/integer-defaults'
out.mkdir(parents=True, exist_ok=True)
peer = out / 'peer'
subprocess.run(['c++', '-std=c++17', '-Wall', '-Wextra', '-Werror',
                str(root / 'tests/integer_defaults_peer.cc'), '-lprotobuf', '-o', str(peer)], check=True)
study = json.loads((root / 'tests/fixtures/integer-defaults/study-190.json').read_text())
cases = [(row['kind'], row['input']) for row in study]
extra = [
    '2147483647', '-2147483648', '-2147483649', '4294967295', '-4294967296',
    '9223372036854775807', '9223372036854775808', '-9223372036854775808',
    '-9223372036854775809', '18446744073709551615', '-18446744073709551615',
    '-18446744073709551616', '999999999999999999999999999999999999',
    '-999999999999999999999999999999999999',
    '0x7fffffffffffffff', '0x8000000000000000', '-0x8000000000000000',
    '0xffffffffffffffff', '-0xffffffffffffffff', '0x10000000000000000',
    '-0x10000000000000000', '01777777777777777777777', '02000000000000000000000',
    '+', '-', '--1', '++1', '-+1', '+ 1', '0xg', '-0X', '0009', '1_000',
    '42\n', '42\t', '42 ', ' ', '\t', '\n', '\r', '\v', '\f',
    '\t010', '\n010', '\r010', '\v010', '\f010', '\u00a010', '\u22121', '\u0661',
    '\0', '\0junk', ' \0', '+\0', '42\0junk', '010\0junk', '0x2a\0junk',
    '-0\0junk', '18446744073709551616\0junk', '0x\0junk', '9é',
    '+0', '-0', '000', '-00', '+0Xf', '00\0x',
    '0b0', '0B101', '-0b1', '+0B111', '0b', '0B', '0b2',
    '0b1111111111111111111111111111111111111111111111111111111111111111',
    '0b10000000000000000000000000000000000000000000000000000000000000000',
    '-0b1000000000000000000000000000000000000000000000000000000000000000',
    '-0b10000000000000000000000000000000000000000000000000000000000000000',
    '0b10\0junk', '999999999999999999999999999999999999junk',
]
for kind in [5, 3, 13, 4, 17, 18, 15, 16, 7, 6]:
    cases.extend((kind, value) for value in extra if (kind, value) not in cases)
rows = []
for index, (kind, value) in enumerate(cases):
    schema = FileDescriptorSet()
    file = schema.file.add(name='defaults.proto', syntax='proto2')
    field = file.message_type.add(name='Default').field.add(name='value', number=1,
        type=kind, label=1, default_value=value, json_name='value')
    raw = schema.SerializeToString()
    digest = hashlib.sha256(raw).hexdigest()
    if index < len(study):
        assert digest == study[index]['descriptor_sha256']
    path = out / 'input.desc'
    path.write_bytes(raw)
    run = subprocess.run([str(peer), str(path)], capture_output=True, timeout=10, env=dict(os.environ, LC_ALL='C'))
    assert run.returncode in [0, 5], (kind, value, run.returncode, run.stderr)
    assert index >= len(study) or run.returncode == study[index]['status']
    if index < len(study) and run.returncode == 0:
        expected = re.search(r'default = (-?\d+)', study[index]['body']).group(1)
        assert run.stdout.decode().splitlines()[0] == expected
    rows.append(dict(id=index, kind=kind, input=value, descriptor_hex=raw.hex(),
        descriptor_sha256=digest, status=run.returncode, stdout_hex=run.stdout.hex(),
        stderr_hex=run.stderr.hex()))
shapes = []
for shape in ['repeated', 'oneof', 'required-oneof', 'required']:
    for literal in ['010', '0x2a']:
        schema = FileDescriptorSet()
        file = schema.file.add(name='shapes.proto', syntax='proto2')
        message = file.message_type.add(name='Default')
        field = message.field.add(name='value', number=1, type=5,
            label=3 if shape == 'repeated' else 2 if shape.startswith('required') else 1,
            default_value=literal, json_name='value')
        if 'oneof' in shape:
            message.oneof_decl.add(name='choice')
            field.oneof_index = 0
        raw = schema.SerializeToString()
        path.write_bytes(raw)
        run = subprocess.run([str(peer), str(path)], capture_output=True, timeout=10,
                             env=dict(os.environ, LC_ALL='C'))
        assert run.returncode == (0 if shape in ['oneof', 'required'] else 5)
        shapes.append(dict(shape=shape, literal=literal, descriptor_hex=raw.hex(),
            sha256=hashlib.sha256(raw).hexdigest(), status=run.returncode,
            stdout_hex=run.stdout.hex(), stderr_hex=run.stderr.hex()))
shape_path = root / 'tests/fixtures/integer-defaults/shapes.json'
shape_path.write_text(json.dumps(shapes, indent=2) + '\n')
linkage = subprocess.check_output(['ldd', str(peer)], text=True)
libraries = {name: hashlib.sha256(Path(path).read_bytes()).hexdigest()
             for name, path in re.findall(r'\s+(lib(?:protobuf|stdc\+\+|c)\.[^ ]+)\s+=>\s+(\S+)', linkage)}
protobuf = next(path for name, path in re.findall(r'\s+(libprotobuf\.[^ ]+)\s+=>\s+(\S+)', linkage))
imports = subprocess.check_output(['readelf', '--dyn-syms', '--wide', protobuf], text=True)
context = dict(protobuf_commit='f0dc78d7e6e331b8c6bb2d5283e06aa26883ca7c',
    descriptor_source=dict(url='https://raw.githubusercontent.com/protocolbuffers/protobuf/f0dc78d7e6e331b8c6bb2d5283e06aa26883ca7c/src/google/protobuf/descriptor.cc',
        sha256='05ee5c034613801857288a77f6eb758037421d8f59b01e4e50ba573d74013d7f'),
    libc_version=subprocess.check_output(['ldd', '--version'], text=True).splitlines()[0],
    native_C23_imports=[line.split(' UND ', 1)[1].strip() for line in imports.splitlines() if '__isoc23_strto' in line],
    libraries_sha256=libraries,
    reference_script_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    qualification='Installed native binary/library provenance, not a fresh protobuf rebuild. Pinned primary source bytes verified by the author; independent semantic review remains required. All 990 scalar and 8 shape inputs replay against freshly compiled peer.')
lineage_path = root / 'tests/fixtures/integer-defaults/lineage.json'
lineage_path.write_text(json.dumps(context, indent=2) + '\n')
fixture = dict(scope='Native Linux LP64 ordinary proto2 integer defaults and explicit-presence wire; not formatter/extensions/full proto2 compatibility.',
    protobuf_commit='f0dc78d7e6e331b8c6bb2d5283e06aa26883ca7c',
    platform=subprocess.check_output([str(peer), '--platform'], text=True).strip(),
    peer_source_sha256=hashlib.sha256((root / 'tests/integer_defaults_peer.cc').read_bytes()).hexdigest(),
    native_binary_sha256=hashlib.sha256(peer.read_bytes()).hexdigest(),
    study_sha256=hashlib.sha256((root / 'tests/fixtures/integer-defaults/study-190.json').read_bytes()).hexdigest(),
    legacy_study_sha256=hashlib.sha256((root / 'tests/fixtures/integer-defaults/legacy-190.json').read_bytes()).hexdigest(),
    shape_corpus_sha256=hashlib.sha256(shape_path.read_bytes()).hexdigest(),
    lineage_sha256=hashlib.sha256(lineage_path.read_bytes()).hexdigest(),
    cases=rows)
(root / 'tests/fixtures/integer-defaults/native.json').write_text(json.dumps(fixture, indent=2) + '\n')
print(f'{len(rows)} exact native observations; all190 original bytes/statuses preserved')

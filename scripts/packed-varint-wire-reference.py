#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay a bounded packed-varint corpus with canonical and installed C++ parsers."""
import argparse
import hashlib
import json
from pathlib import Path
import shlex
import subprocess

ROOT = Path(__file__).resolve().parents[1]

def varint(value):
    result = bytearray()
    while value > 127:
        result.append((value & 127) | 128)
        value >>= 7
    return bytes(result) + bytes([value])

def corpus():
    rows = []
    edges = [0, 1, 127, 128, 16383, 16384, (1 << 31) - 1, 1 << 31,
             (1 << 32) - 1, 1 << 32, (1 << 63) - 1, 1 << 63, (1 << 64) - 1]
    for kind in ['Int32_V', 'UInt32_V', 'Int64_V', 'UInt64_V']:
        width = 32 if '32' in kind else 64
        def value(n):
            n &= (1 << width) - 1
            return n - (1 << width) if kind.startswith('Int') and n >= 1 << (width - 1) else n
        def add(name, wire, values=None, strict=None, compatible=None, header=None):
            accepted = values is not None
            rows.append(dict(id=kind + '_' + name, message='gz.msgs.' + kind,
                wire_hex=wire.hex(), expected_values=None if values is None else [value(n) for n in values],
                accepted=accepted, strict=accepted if strict is None else strict,
                compatible=accepted if compatible is None else compatible, header=header))
        def packed(values):
            payload = b''.join(varint(n) for n in values)
            return b'\x12' + varint(len(payload)) + payload
        add('absent', b'', [])
        add('empty', packed([]), [])
        for n in edges:
            add('value_' + str(n), packed([n]), [n])
        add('all_packed', packed(edges), edges)
        add('all_unpacked', b''.join(b'\x10' + varint(n) for n in edges), edges)
        add('split_fields', packed(edges[:5]) + packed(edges[5:]), edges)
        add('mixed', packed(edges[:5]) + b'\x10' + varint(edges[5]) + packed(edges[6:]), edges)
        for count in [127, 128, 129]:
            add('length_' + str(count), packed([0] * count), [0] * count)
        add('nonminimal_zero', b'\x12\x02\x80\x00', [0])
        add('nonminimal_ten_zero', b'\x12\x0a' + b'\x80' * 9 + b'\x00', [0])
        for last in [2, 127]:
            payload = b'\x81' + b'\x80' * 8 + bytes([last])
            values = [1 | ((last & 1) << 63)]
            add('overflow_' + str(last) + '_packed', b'\x12\x0a' + payload, values, False, False)
            add('overflow_' + str(last) + '_unpacked', b'\x10' + payload, values, False, False)
        add('known_then_unknown', packed(edges) + b'\x18\x01', edges)
        add('wrong_known_before', b'\x15\x01\x00\x00\x00' + packed(edges), edges, False)
        add('wrong_known_after', packed(edges) + b'\x15\x01\x00\x00\x00', edges, False)
        add('empty_header', b'\x0a\x00' + packed(edges[:3]), edges[:3], header='present')
        add('stamp_header', b'\x0a\x04\x0a\x02\x08\x07' + packed(edges[:3]), edges[:3], header='stamp7')
        add('unknown_header', b'\x0a\x03\xa0\x06\x7b' + packed(edges[:3]), edges[:3], header='unknown')
        add('wrong_header', b'\x08\x07' + packed(edges[:3]), edges[:3], False)
        add('malformed_header', b'\x0a\x01\x00' + packed(edges[:3]))
        for removed in range(1, 6):
            add('truncated_' + str(removed), packed(edges)[:-removed])
        for count in [1, 2, 9, 10, 11]:
            add('unterminated_value_' + str(count), b'\x12' + varint(count) + b'\x80' * count)
        add('eleven_byte_value', b'\x12\x0b' + b'\x80' * 10 + b'\x00')
        add('missing_length', b'\x12')
        for count in [1, 9, 10]:
            add('unterminated_length_' + str(count), b'\x12' + b'\xff' * count)
        add('maximum_length', b'\x12' + varint((1 << 64) - 1))
    return rows

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--upstream', required=True, type=Path)
    parser.add_argument('--write-fixture', action='store_true')
    parser.add_argument('--output', type=Path, default=ROOT / 'artifacts/packed-varint-wire-reference')
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    manifest = json.loads((ROOT / 'schemas/manifest.json').read_text())
    revision = subprocess.check_output(['git', '-C', str(args.upstream), 'rev-parse', 'HEAD'], text=True).strip()
    assert revision == manifest['commit']
    for item in manifest['schemas']:
        assert sha(args.upstream / 'proto' / item['path'].removeprefix('schemas/')) == item['sha256']
    version = subprocess.check_output(['pkg-config', '--modversion', 'protobuf'], text=True).strip()
    assert version == '3.21.12'
    subprocess.run(['python3', 'scripts/native-varint-wire-peer.py'], cwd=ROOT, check=True)
    libdir = Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'gz-msgs'], text=True).strip())
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'gz-msgs'], text=True))
    installed = out / 'installed-peer'
    command = ['c++', '-std=c++17', '-O2', str(ROOT / 'tests/wire_peer.cc'), *flags,
               '-Wl,--disable-new-dtags', f'-Wl,-rpath,{libdir}', '-o', str(installed)]
    subprocess.run(command, check=True)
    libraries = {}
    for line in subprocess.check_output(['ldd', str(installed)], text=True).splitlines():
        words = line.split()
        if len(words) > 2 and words[1] == '=>' and Path(words[2]).is_file():
            path = Path(words[2]).resolve()
            libraries[str(path)] = sha(path)
    rows = corpus()
    for row in rows:
        for mode, key in [([], 'cpp_generated'), (['known'], 'cpp_known')]:
            observed = []
            for binary in [installed, ROOT / 'target/native-varint-wire/peer']:
                result = subprocess.run([str(binary), row['message'], *mode],
                    input=bytes.fromhex(row['wire_hex']), capture_output=True, timeout=10)
                observed.append(dict(exit=result.returncode, output_hex=result.stdout.hex(), stderr=result.stderr.decode()))
            assert observed[0] == observed[1], row['id']
            assert observed[0]['exit'] == (0 if row['accepted'] else 6), row['id']
            row[key] = observed[0]
    fixture = dict(upstream_commit=revision, native_protobuf=version,
        schema_manifest_sha256=sha(ROOT / 'schemas/manifest.json'),
        probe_source_sha256=sha(ROOT / 'tests/wire_peer.cc'),
        canonical_builder_sha256=sha(ROOT / 'scripts/native-varint-wire-peer.py'),
        reference_script_sha256=sha(Path(__file__)), rows=rows)
    path = ROOT / 'tests/fixtures/packed-varint-wire.json'
    if args.write_fixture:
        path.write_text(json.dumps(fixture, indent=2) + '\n')
    else:
        assert fixture == json.loads(path.read_text())
    proof = dict(cases=len(rows), fixture_sha256=sha(path), command=command, libraries=libraries,
                 source_revision=revision, canonical_and_installed_agree=True)
    (out / 'proof.json').write_text(json.dumps(proof, indent=2) + '\n')
    print(f'{len(rows)} packed-varint cases: installed and fresh canonical native outputs/statuses agree')

if __name__ == '__main__':
    main()

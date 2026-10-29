#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay a bounded packed-float corpus with canonical and installed C++ parsers."""
import argparse
import hashlib
import itertools
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
    patterns = [0, 1 << 31, 0x3fa00000, 1, (1 << 31) | 1,
                0x00800000, 0x7f7fffff, 0xff7fffff,
                0x7f800000, 0xff800000, 0x7fc01234,
                0x7f801234, 0xffc04321]
    rows = []
    def add(name, wire, bits=None, strict=None, header=None):
        rows.append(dict(id=name, wire_hex=wire.hex(), expected_bits=bits,
                         accepted=bits is not None,
                         strict=bits is not None if strict is None else strict,
                         header=header))
    def payload(bits):
        return b''.join(value.to_bytes(4, 'little') for value in bits)
    def packed(bits):
        data = payload(bits)
        return b'\x12' + varint(len(data)) + data
    add('absent', b'', [])
    add('empty', packed([]), [])
    for index, bits in enumerate(patterns):
        add(f'bits_{index}', packed([bits]), [bits])
    for count in [31, 32, 33, 65]:
        bits = list(itertools.islice(itertools.cycle(patterns), count))
        add(f'length_boundary_{count}', packed(bits), bits)
    add('all_unpacked', b''.join(b'\x15' + payload([n]) for n in patterns), patterns)
    add('split_packed_fields', packed(patterns[:5]) + packed(patterns[5:]), patterns)
    add('mixed_packed_unpacked', packed(patterns[:5]) + b'\x15' + payload([patterns[5]]) + packed(patterns[6:]), patterns)
    add('known_then_unknown', packed(patterns) + b'\x18\x01', patterns)
    add('wrong_known_then_packed', b'\x10\x07' + packed(patterns), patterns, False)
    add('packed_then_wrong_known', packed(patterns) + b'\x10\x07', patterns, False)
    add('nonminimal_length', b'\x12\x84\x00' + payload([patterns[2]]), [patterns[2]])
    add('nonminimal_empty_length', b'\x12\x80\x00', [])
    for count in range(1, 10):
        if count % 4:
            add(f'misaligned_{count}', b'\x12' + varint(count) + bytes(count) + b'\x18\x01')
    full = packed(list(itertools.islice(itertools.cycle(patterns), 32)))
    for removed in range(1, 10):
        add(f'truncated_{removed}', full[:-removed])
    add('missing_length', b'\x12')
    for count in range(1, 12):
        add(f'unterminated_length_{count}', b'\x12' + bytes([255]) * count)
    add('declared_u64_max', b'\x12' + varint((1 << 64) - 1))
    # Float_V has a distinct message-valued Header at tag 1; data is tag 2.
    add('empty_header_and_data', b'\x0a\x00' + packed(patterns[:3]), patterns[:3], header='present')
    add('stamp_header_and_data', b'\x0a\x04\x0a\x02\x08\x07' + packed(patterns[:3]), patterns[:3], header='stamp7')
    add('unknown_inside_header', b'\x0a\x03\xa0\x06\x7b' + packed(patterns[:3]), patterns[:3], header='unknown')
    add('wrong_header_wire_then_data', b'\x08\x07' + packed(patterns[:3]), patterns[:3], False)
    add('malformed_known_header', b'\x0a\x01\x00' + packed(patterns[:3]))
    return rows

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--upstream', required=True, type=Path)
    parser.add_argument('--write-fixture', action='store_true')
    parser.add_argument('--output', type=Path, default=ROOT / 'artifacts/packed-float-wire-reference')
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
    subprocess.run(['python3', 'scripts/native-wire-peer.py'], cwd=ROOT, check=True)
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
            for binary in [installed, ROOT / 'target/native-wire/peer']:
                result = subprocess.run([str(binary), 'gz.msgs.Float_V', *mode],
                    input=bytes.fromhex(row['wire_hex']), capture_output=True, timeout=10)
                observed.append(dict(exit=result.returncode, output_hex=result.stdout.hex(), stderr=result.stderr.decode()))
            assert observed[0] == observed[1], row['id']
            assert observed[0]['exit'] == (0 if row['accepted'] else 6), row['id']
            row[key] = observed[0]
    fixture = dict(upstream_commit=revision, native_protobuf=version,
        schema_manifest_sha256=sha(ROOT / 'schemas/manifest.json'),
        probe_source_sha256=sha(ROOT / 'tests/wire_peer.cc'),
        canonical_builder_sha256=sha(ROOT / 'scripts/native-wire-peer.py'),
        reference_script_sha256=sha(Path(__file__)), rows=rows)
    path = ROOT / 'tests/fixtures/packed-float-wire.json'
    if args.write_fixture:
        path.write_text(json.dumps(fixture, indent=2) + '\n')
    else:
        assert fixture == json.loads(path.read_text())
    proof = dict(cases=len(rows), fixture_sha256=sha(path), command=command, libraries=libraries,
                 source_revision=revision, canonical_and_installed_agree=True)
    (out / 'proof.json').write_text(json.dumps(proof, indent=2) + '\n')
    print(f'{len(rows)} packed-float cases: installed and fresh canonical native outputs/statuses agree')

if __name__ == '__main__':
    main()

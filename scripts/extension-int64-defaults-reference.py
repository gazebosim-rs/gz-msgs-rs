#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Observe bounded canonical INT64 extension declarations through the pinned native CLI."""
import argparse
import hashlib
import importlib.util
import itertools
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
BUILDER = ROOT / 'scripts/nested-extension-declarations-reference.py'


def inputs():
    from google.protobuf import descriptor_pb2 as d
    spec = importlib.util.spec_from_file_location('extension_inputs', BUILDER)
    builder = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(builder)
    bases = builder.inputs()
    rows = []

    def add(name, default=None, layout='ordinary', deprecated=None, label=1,
            kind=3, mode='ordinary', policy='supported', reason=None, profile=None):
        template = next(row for row in bases if row['kind'] == 3
                        and row['label'] == 1 and row['layout'] == layout
                        and row['mode'] == 'ordinary')
        if mode in ['unknown-option', 'custom-option', 'proto3', 'explicit-json']:
            template = next(row for row in bases if row['mode'] == mode)
        files = d.FileDescriptorSet.FromString(bytes.fromhex(template['descriptor_hex']))
        owner = files.file[-1].message_type[-1]
        if layout == 'nested':
            owner = owner.nested_type[0]
        for index, value in enumerate(owner.extension):
            value.type = kind
            value.label = label
            chosen = default[index] if isinstance(default, list) else default
            if mode == 'late-unsupported' and index == 1:
                value.type = 1
                chosen = '1.25'
            if chosen is not None:
                value.default_value = chosen
            if deprecated is not None:
                value.options.deprecated = deprecated
            if profile is not None:
                for option, declared in profile.items():
                    setattr(value.options, option, declared)
        rows.append(dict(id=name, layout=layout, default=default, kind=kind,
                         label=label, policy=policy, reason=reason,
                         descriptor_hex=files.SerializeToString().hex()))

    canonical = ['0', '3', '-3', '2147483648', '4294967296', '-9223372036854775808', '9223372036854775807']
    for layout, default, deprecated in itertools.product(
            ['ordinary', 'nested', 'sections'], canonical, [None, False, True]):
        add(f'{layout}-{default}-{deprecated}', default, layout, deprecated)
    for layout in ['ordinary', 'nested', 'sections']:
        add(f'{layout}-omitted', layout=layout)
        add(f'{layout}-mixed-presence', ['-3', '0', None], layout)
    for default in ['+3', '+0', '-0', '00', '003', '007', '010', '09', '-010', '+010']:
        add('noncanonical-' + repr(default), default, policy='unsupported',
            reason='extension declarations')
    for default in ['0x2a', '0b10', ' 3', '3 ', '\t3', '', '+', '-',
                    '9223372036854775808', '-9223372036854775809', '18446744073709551616',
                    '3\x00junk', '\x00', '1_0', '１２', '--1']:
        add('loader-' + repr(default), default, policy='loader-reject')
    for ctype, jstype in itertools.product(range(3), range(3)):
        add(f'options-{ctype}-{jstype}', '3', deprecated=False,
            profile=dict(ctype=ctype, jstype=jstype, packed=False))
    add('packed-true', '3', profile=dict(packed=True), policy='unsupported',
        reason='packed extension shape')
    for label, default in itertools.product([2, 3], ['3', '-3']):
        add(f'label-{label}-{default}', default, label=label,
            policy='unsupported', reason='extension declarations')
    for mode, reason in [('explicit-json', 'extension declarations'),
                         ('proto3', 'extension declarations'),
                         ('unknown-option', 'unknown options'), ('custom-option', 'custom options')]:
        add(mode, '3', mode=mode, policy='unsupported', reason=reason)
    for kind in [4, 13, 17, 18]:
        add(f'other-kind-{kind}', '3', kind=kind, policy='unsupported',
            reason='extension declarations')
    add('late-unsupported', '3', mode='late-unsupported', policy='unsupported',
        reason='extension declarations')
    for filename, identity in [('extension-int32-defaults.json', 'other-kind-3')]:
        old = next(row for row in json.loads((ROOT / 'tests/fixtures' / filename).read_text())['rows']
                   if row['id'] == identity)
        rows.append(dict(id=filename + ':' + str(identity), layout=old['layout'],
                         policy='supported', reason=None, descriptor_hex=old['descriptor_hex'],
                         previous_fixture=filename, previous_id=identity,
                         previous_fixture_sha256=hashlib.sha256((ROOT / 'tests/fixtures' / filename).read_bytes()).hexdigest()))
    historical = ROOT / 'tests/fixtures/extension-int32-defaults.json'
    for old in json.loads(historical.read_text())['rows']:
        if old['policy'] == 'loader-reject' and old['native']['exit'] == 0:
            rows.append(dict(id='retained-int32:' + old['id'], layout=old['layout'],
                             default=old['default'], kind=old['kind'], label=old['label'],
                             policy='loader-reject', reason=None,
                             descriptor_hex=old['descriptor_hex'],
                             previous_fixture=historical.name, previous_id=old['id'],
                             previous_fixture_sha256=hashlib.sha256(historical.read_bytes()).hexdigest()))
    assert len(rows) == 129
    return rows


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--upstream', type=Path, required=True)
    parser.add_argument('--write-fixture', action='store_true')
    args = parser.parse_args()
    upstream = args.upstream.resolve()
    baseline = json.loads((ROOT / 'tests/fixtures/definitions/cli.json').read_text())
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    binary = upstream / 'build/bin/gz-msgs'
    git_env = dict(os.environ)
    for name in subprocess.check_output(['git', 'rev-parse', '--local-env-vars'], text=True).splitlines():
        git_env.pop(name, None)
    assert subprocess.check_output(['git', '-C', str(upstream), 'rev-parse', 'HEAD'], env=git_env, text=True).strip() == baseline['upstream_commit']
    subprocess.run(['git', '-C', str(upstream), 'diff', '--quiet', 'HEAD', '--'], env=git_env, check=True)
    assert sha(binary) == baseline['native_binary_sha256']
    assert sha(upstream / 'core/cmd/msgs_main.cc') == baseline['native_source_sha256']
    assert sha(ROOT / 'tests/definition_peer.cc') == baseline['peer_source_sha256']
    standard = ROOT / 'tests/fixtures/definitions/unsupported.desc'
    assert sha(standard) == baseline['inputs']['tests/fixtures/definitions/unsupported.desc']
    assert subprocess.check_output(['pkg-config', '--modversion', 'protobuf'], text=True).strip() == '3.21.12'
    libraries = {}
    for line in subprocess.check_output(['ldd', str(binary)], text=True).splitlines():
        words = line.split()
        if len(words) > 2 and words[1] == '=>' and words[0].startswith(('libgz-', 'libprotobuf')):
            libraries[words[0]] = sha(Path(words[2]))
    assert libraries == baseline['native_libraries_sha256']
    out = ROOT / 'artifacts/extension-int64-defaults-reference'
    out.mkdir(parents=True, exist_ok=True)
    peer = out / 'peer'
    subprocess.run(['c++', '-std=c++17', '-Wall', '-Wextra', '-Werror',
                    str(ROOT / 'tests/definition_peer.cc'), '-lprotobuf', '-o', str(peer)], check=True)
    for line in subprocess.check_output(['ldd', str(peer)], text=True).splitlines():
        words = line.split()
        if len(words) > 2 and words[1] == '=>' and words[0].startswith('libprotobuf'):
            assert sha(Path(words[2])) == libraries[words[0]]
    rows = inputs()
    for row in rows:
        path = out / 'input.desc'
        path.write_bytes(bytes.fromhex(row['descriptor_hex']))
        commands = {'native': [str(peer), str(path)],
                    'native_cli': ['gz-msgs', '--info', 'probe.Root']}
        for name, command in commands.items():
            result = subprocess.run(command, executable=binary if name == 'native_cli' else None,
                                    cwd=ROOT, env=dict(os.environ, GZ_DESCRIPTOR_PATH=str(path.relative_to(ROOT))),
                                    capture_output=True, timeout=10)
            row[name] = dict(exit=result.returncode, stdout=result.stdout.decode(), stderr=result.stderr.decode())
        if 'previous_fixture' in row:
            old = next(value for value in json.loads((ROOT / 'tests/fixtures' / row['previous_fixture']).read_text())['rows']
                       if value['id'] == row['previous_id'])
            assert row['descriptor_hex'] == old['descriptor_hex']
            assert row['native'] == old['native']
            assert row['native_cli'] == old['native_cli']
    result = dict(reference_script_sha256=sha(Path(__file__)), input_builder_sha256=sha(BUILDER),
                  standard_descriptor_fixture_sha256=sha(standard),
                  upstream_commit=baseline['upstream_commit'], protobuf='3.21.12',
                  native_binary_sha256=sha(binary), native_source_sha256=baseline['native_source_sha256'],
                  native_libraries_sha256=libraries, peer_source_sha256=sha(ROOT / 'tests/definition_peer.cc'),
                  schema_manifest_sha256=sha(ROOT / 'schemas/manifest.json'), rows=rows)
    fixture = ROOT / 'tests/fixtures/extension-int64-defaults.json'
    if args.write_fixture:
        fixture.write_text(json.dumps(result, indent=2) + '\n')
    else:
        assert result == json.loads(fixture.read_text())
    print(f'{len(rows)} complete native canonical-INT64-extension declaration/CLI observations passed')


if __name__ == '__main__':
    main()

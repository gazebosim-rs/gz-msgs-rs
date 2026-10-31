#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Observe bounded boolean extension declarations through the pinned native CLI."""
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
    spec = importlib.util.spec_from_file_location('nested_extension_inputs', BUILDER)
    builder = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(builder)
    bases = builder.inputs()
    rows = []

    def add(name, default=None, layout='ordinary', label=1, kind=8,
            deprecated=None, packed=None, explicit_json=False, mode='ordinary',
            policy='supported', reason=None):
        template = next(row for row in bases if row['kind'] == 8
                        and row['label'] == 1 and row['layout'] == layout
                        and row['mode'] == 'ordinary')
        if mode in ['unknown-option', 'custom-option']:
            template = next(row for row in bases if row['mode'] == mode)
        files = d.FileDescriptorSet.FromString(bytes.fromhex(template['descriptor_hex']))
        file = files.file[-1]
        root = file.message_type[-1]
        owner = root.nested_type[0] if layout == 'nested' else root
        for index, value in enumerate(owner.extension):
            value.type = kind
            value.label = label
            if kind == 14:
                value.type_name = '.probe.Choice'
            chosen = default[index] if isinstance(default, list) else default
            if mode == 'mixed-default-kind' and index == 1:
                value.type = 5
                chosen = '3'
            if chosen is not None:
                value.default_value = chosen
            if deprecated is not None:
                value.options.deprecated = deprecated
            if packed is not None:
                value.options.packed = packed
            if explicit_json:
                value.json_name = value.name
        rows.append(dict(id=name, layout=layout, default=default, kind=kind,
                         label=label, policy=policy, reason=reason,
                         descriptor_hex=files.SerializeToString().hex()))

    for layout, default, deprecated in itertools.product(
            ['ordinary', 'nested', 'sections'], [None, 'true', 'false'], [None, False, True]):
        add(f'{layout}-{default}-{deprecated}', default, layout, deprecated=deprecated)
    for layout in ['ordinary', 'nested', 'sections']:
        add(f'{layout}-mixed-presence', ['true', 'false', None], layout)
    for default in ['true', 'false']:
        add(f'packed-false-{default}', default, packed=False)
        for label in [2, 3]:
            add(f'label-{label}-{default}', default, label=label,
                policy='unsupported', reason='extension declarations')
        add(f'explicit-json-{default}', default, explicit_json=True,
            policy='unsupported', reason='extension declarations')
    for kind, default in [(5, '3'), (9, 'foo'), (12, 'x'), (14, 'ZERO'), (1, '1.25')]:
        add(f'other-default-kind-{kind}', default, kind=kind,
            policy='unsupported', reason='extension declarations')
    for default in ['True', 'False', 'TRUE', 'FALSE', '1', '0', '', 'true ', ' false', 'yes']:
        add('invalid-' + repr(default), default, policy='loader-reject')
    for mode, reason in [('unknown-option', 'unknown options'), ('custom-option', 'custom options')]:
        add(mode, 'true', mode=mode, policy='unsupported', reason=reason)
    add('mixed-default-kind', 'true', mode='mixed-default-kind',
        policy='unsupported', reason='extension declarations')
    assert len(rows) == 56
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
    out = ROOT / 'artifacts/extension-boolean-defaults-reference'
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
    result = dict(reference_script_sha256=sha(Path(__file__)), input_builder_sha256=sha(BUILDER),
                  standard_descriptor_fixture_sha256=sha(standard),
                  upstream_commit=baseline['upstream_commit'], protobuf='3.21.12',
                  native_binary_sha256=sha(binary), native_source_sha256=baseline['native_source_sha256'],
                  native_libraries_sha256=libraries, peer_source_sha256=sha(ROOT / 'tests/definition_peer.cc'),
                  schema_manifest_sha256=sha(ROOT / 'schemas/manifest.json'), rows=rows)
    fixture = ROOT / 'tests/fixtures/extension-boolean-defaults.json'
    if args.write_fixture:
        fixture.write_text(json.dumps(result, indent=2) + '\n')
    else:
        assert result == json.loads(fixture.read_text())
    print(f'{len(rows)} complete native boolean-extension declaration/CLI observations passed')


if __name__ == '__main__':
    main()

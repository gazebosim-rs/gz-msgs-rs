#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay explicit runtime-prefix workflows; preserve native output bytes."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import resource
import shlex
import subprocess
import tempfile
from google.protobuf import descriptor_pb2 as d
ROOT = Path(__file__).resolve().parents[1]
CASES = ['normal', 'empty', 'raw', 'colon', 'relative', 'missing', 'missing-colon',
         'loop', 'env-first', 'explicit-after', 'late', 'freeze', 'generated',
         'registered', 'bad-share', 'errors']
def inputs():
    result = {}
    for name, filename, message in [('A', 'a.proto', 'A'), ('B_conflict', 'a.proto', 'B'), ('B_other', 'b.proto', 'B')]:
        value = d.FileDescriptorSet()
        value.file.add(name=filename, package='audit', syntax='proto2').message_type.add(name=message)
        result[name] = value.SerializeToString()
    return result

def bind(data, directory):
    result = []
    for i, part in enumerate(data.split(os.fsencode(directory))):
        if i: result.append({'case_directory': True})
        result.append({'hex': part.hex()})
    return result

def expand(parts, directory):
    return b''.join(os.fsencode(directory) if part.get('case_directory') else bytes.fromhex(part['hex']) for part in parts)

def libraries(binary, env):
    result = {}
    for line in subprocess.check_output(['ldd', str(binary)], env=env, text=True).split('\n'):
        words = line.split()
        if len(words) > 2 and words[1] == '=>' and Path(words[2]).is_file():
            result[words[0]] = hashlib.sha256(Path(words[2]).read_bytes()).hexdigest()
    return result

def no_core():
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--upstream', type=Path, required=True)
    parser.add_argument('--write-fixture', action='store_true')
    args = parser.parse_args()
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    up = args.upstream.resolve()
    baseline = json.loads((ROOT / 'tests/fixtures/definitions/cli.json').read_text())
    assert subprocess.check_output(['git', '-C', str(up), 'rev-parse', 'HEAD'], text=True).strip() == baseline['upstream_commit']
    subprocess.run(['git', '-C', str(up), 'diff', '--quiet'], check=True)
    sources = ['core/src/DynamicFactory.cc', 'core/src/MessageFactory.cc', 'core/src/Factory.cc',
               'core/include/gz/msgs/MessageFactory.hh', 'core/include/gz/msgs/Factory.hh',
               'core/include/gz/msgs/InstallationDirectories.hh', 'core/CMakeLists.txt',
               'build/core/gz-msgs_get_install_prefix_impl.cc']
    source_hashes = {name: sha(up / name) for name in sources}
    include = Path(subprocess.check_output(['pkg-config', '--variable=includedir', 'gz-msgs'], text=True).strip())
    for name in ['MessageFactory.hh', 'Factory.hh', 'InstallationDirectories.hh']:
        assert sha(include / 'gz/msgs' / name) == source_hashes['core/include/gz/msgs/' + name]
    lib = Path(subprocess.check_output(['pkg-config', '--variable=libdir', 'gz-msgs'], text=True).strip()).resolve()
    flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', '--libs', 'gz-msgs'], text=True))
    output = ROOT / 'artifacts/factory-install-share-reference'
    output.mkdir(parents=True, exist_ok=True)
    peer = output / 'peer'
    subprocess.run(['c++', '-std=c++17', '-I' + str(up / 'core/include'), str(ROOT / 'tests/factory_install_share_reference.cc'), *flags,
                    '-Wl,--disable-new-dtags', '-Wl,-rpath,' + str(lib), '-o', str(peer)], check=True)
    values = inputs()
    input_dir = output / 'inputs'
    input_dir.mkdir(exist_ok=True)
    for name, data in values.items(): (input_dir / (name + '.desc')).write_bytes(data)
    fixture = ROOT / 'tests/fixtures/factory-install-share.json'
    old = None if args.write_fixture else json.loads(fixture.read_text())
    rows, raw, linked = [], [], None
    for index, mode in enumerate(CASES):
        with tempfile.TemporaryDirectory(prefix='case-', dir=output) as directory:
            work = Path(directory)
            prefix = {'empty': Path(''), 'raw': Path(os.fsdecode(b'raw-\xff')), 'colon': Path('pre:fix'),
                      'relative': Path('prefix'), 'missing-colon': work / 'missing.desc:gone', 'loop': work / 'loop'}.get(mode, work / 'prefix')
            share = prefix / 'share/gz/protos'
            actual_share = work / share if not share.is_absolute() else share
            if mode == 'loop': (work / 'loop').symlink_to('loop')
            elif mode not in ['missing', 'missing-colon']:
                actual_share.mkdir(parents=True)
                (actual_share / 'a.desc').write_bytes(values['A'])
            other = work / 'other'
            (other / 'share/gz/protos').mkdir(parents=True)
            (other / 'share/gz/protos/b.desc').write_bytes(values['B_other'])
            explicit = work / 'explicit.desc'
            explicit.write_bytes(values['B_conflict'])
            env = dict(os.environ, LD_LIBRARY_PATH=str(lib), GZ_MSGS_INSTALL_PREFIX='' if mode == 'empty' else str(prefix))
            env.pop('GZ_DESCRIPTOR_PATH', None)
            if mode == 'env-first': env['GZ_DESCRIPTOR_PATH'] = str(explicit)
            if mode in ['bad-share', 'errors']: (actual_share / 'a.desc').write_bytes(b'\xff')
            if mode == 'errors':
                bad_env = work / 'bad-env.desc'; bad_env.write_bytes(b'\xff')
                explicit = work / 'bad-explicit.desc'; explicit.write_bytes(b'\xff')
                env['GZ_DESCRIPTOR_PATH'] = str(bad_env)
            actual_libraries = libraries(peer, env)
            assert {n: h for n, h in actual_libraries.items() if n.startswith(('libgz-', 'libprotobuf'))} == baseline['native_libraries_sha256']
            if linked is None: linked = actual_libraries
            assert linked == actual_libraries
            value = subprocess.run([str(peer), mode, str(other), str(explicit), str(input_dir)], env=env, cwd=work,
                                   capture_output=True, timeout=10, preexec_fn=no_core)
            row = {'mode': mode, 'native': {'exit': value.returncode, 'stdout': bind(value.stdout, directory), 'stderr': bind(value.stderr, directory)},
                   'states': [line.decode().split()[2:] for line in value.stdout.split(b'\n') if line.startswith(b'STATE ')],
                   'messages': [line.decode().split()[1:] for line in value.stdout.split(b'\n') if line.startswith(b'MESSAGE ')]}
            if old:
                expected = old['rows'][index]
                assert mode == expected['mode'] and value.returncode == expected['native']['exit']
                assert value.stdout == expand(expected['native']['stdout'], directory), mode
                assert value.stderr == expand(expected['native']['stderr'], directory), mode
            raw.append({'mode': mode, 'case_directory': directory, 'exit': value.returncode, 'stdout_hex': value.stdout.hex(), 'stderr_hex': value.stderr.hex()})
            rows.append(row)
    cli = up / 'build/bin/gz-msgs'
    assert sha(cli) == baseline['native_binary_sha256']
    with tempfile.TemporaryDirectory(prefix='cli-', dir=output) as directory:
        prefix = Path(directory) / 'prefix'
        share = prefix / 'share/gz/protos'
        share.mkdir(parents=True)
        (share / 'a.desc').write_bytes(values['A'])
        cli_env = dict(os.environ, LD_LIBRARY_PATH=str(lib), GZ_MSGS_INSTALL_PREFIX=str(prefix))
        cli_env.pop('GZ_DESCRIPTOR_PATH', None)
        cli_libraries = libraries(cli, cli_env)
        actual = subprocess.run(['gz-msgs', '--info', 'audit.A'], executable=str(cli), env=cli_env, cwd=ROOT, capture_output=True, timeout=10)
        cli_positive = {'exit': actual.returncode, 'stdout_hex': actual.stdout.hex(), 'stderr_hex': actual.stderr.hex()}
        assert actual.returncode == 0 and actual.stdout and not actual.stderr
    result = {'upstream_commit': baseline['upstream_commit'], 'native_sources_sha256': source_hashes,
              'schema_manifest_sha256': sha(ROOT / 'schemas/manifest.json'), 'native_libraries_sha256': linked,
              'peer_source_sha256': sha(ROOT / 'tests/factory_install_share_reference.cc'), 'reference_script_sha256': sha(Path(__file__)),
              'inputs': {name: data.hex() for name, data in values.items()}, 'rows': rows,
              'native_cli_sha256': sha(cli), 'native_cli_libraries_sha256': cli_libraries, 'cli_positive': cli_positive}
    if args.write_fixture: fixture.write_text(json.dumps(result, indent=2) + '\n')
    else: assert result == old
    (output / 'actual-observations.json').write_text(json.dumps({'peer_sha256': sha(peer), 'compiler': subprocess.check_output(['c++', '--version'], text=True), 'rows': raw}, indent=2) + '\n')
    print('16 explicit-prefix native workflows:', sum(row['native']['exit'] == 0 for row in rows), 'normal completions;', sum(row['native']['exit'] < 0 for row in rows), 'explicit native abort')
if __name__ == '__main__': main()

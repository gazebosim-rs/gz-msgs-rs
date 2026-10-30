#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay complete native CLI descriptor-load diagnostics."""
import argparse
import tempfile
import xml.etree.ElementTree as ET
import hashlib
import json
import os
from pathlib import Path
import subprocess
from google.protobuf import descriptor_pb2 as d

REPO = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('--upstream', type=Path, required=True)
parser.add_argument('--write-fixture', action='store_true')
options = parser.parse_args()
UP = options.upstream.resolve()
workspace = tempfile.TemporaryDirectory(prefix='gz-cli-load-diagnostics-')
ROOT = Path(workspace.name)
BINARY = UP / 'build/bin/gz-msgs'
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
baseline = json.loads((REPO / 'tests/fixtures/definitions/cli.json').read_text())
assert sha(BINARY) == baseline['native_binary_sha256']
assert subprocess.check_output(['git', '-C', str(UP), 'rev-parse', 'HEAD'], text=True).strip() == baseline['upstream_commit']
assert sha(UP / 'core/cmd/msgs_main.cc') == baseline['native_source_sha256']
for selected in ['core/cmd/msgs_main.cc', 'core/src/DynamicFactory.cc', 'test/integration/gz_TEST.cc']:
    assert (UP / selected).read_bytes() == subprocess.check_output(['git', '-C', str(UP), 'show', baseline['upstream_commit'] + ':' + selected])
env = dict(os.environb)
env.pop(b'GZ_DESCRIPTOR_PATH', None)
env.pop(b'GZ_MSGS_INSTALL_PREFIX', None)
libs = {}
for line in subprocess.check_output(['ldd', str(BINARY)], env=env, text=True).split('\n'):
    words = line.split()
    if len(words) > 2 and words[1] == '=>' and words[0].startswith(('libgz-', 'libprotobuf')):
        libs[words[0]] = sha(Path(words[2]))
assert libs == baseline['native_libraries_sha256']

def descriptor(name='valid.proto', message='Valid', dependency=None):
    value = d.FileDescriptorSet()
    file = value.file.add(name=name, package='errors', syntax='proto2')
    file.message_type.add(name=message)
    if dependency:
        file.dependency.append(dependency)
    return value.SerializeToString()

inputs = {
    'empty.desc': b'',
    'garbage.desc': b'not a descriptor',
    'truncated.desc': b'\x0a\x05x',
    'valid.desc': descriptor(),
    'invalid-name.desc': descriptor(message='bad-name'),
    'invalid-field.desc': d.FileDescriptorSet(file=[d.FileDescriptorProto(name='invalid-field.proto', package='errors', syntax='proto2', message_type=[d.DescriptorProto(name='Invalid', field=[d.FieldDescriptorProto(name='value', number=0, label=1, type=5)])])]).SerializeToString(),
    'missing-import.desc': descriptor(dependency='absent.proto'),
    'ignored.txt': b'bad',
}
for name, data in inputs.items():
    (ROOT / name).write_bytes(data)
(ROOT / 'directory.desc').mkdir(exist_ok=True)
assert Path('/proc/self/mem').exists()
(ROOT / 'read-error.desc').symlink_to('/proc/self/mem')
raw_path = b'missing-\xff.desc'
cases = [
    ('missing', b'missing.desc'), ('empty', b'empty.desc'),
    ('garbage', b'garbage.desc'), ('truncated', b'truncated.desc'),
    ('invalid-name', b'invalid-name.desc'), ('invalid-field', b'invalid-field.desc'), ('missing-import', b'missing-import.desc'),
    ('ignored', b'ignored.txt'), ('directory', b'directory.desc'),
    ('raw-missing', raw_path), ('read-error', b'read-error.desc'), ('missing-valid', b'missing.desc:valid.desc'),
    ('garbage-missing', b'garbage.desc:missing.desc'),
]
canonical = json.loads((REPO / 'tests/fixtures/cli-external-list.json').read_text())['canonical_names']
rows = []
for case, path in cases:
    for label, args in [('list', [b'--list']), ('missing-info', [b'--info', b'errors.Missing']), ('generated-info', [b'--info', b'gz.msgs.Int32']), ('help', [b'--help']), ('bad-option', [b'--not-an-option'])]:
        child = dict(env)
        child[b'GZ_DESCRIPTOR_PATH'] = path
        result = subprocess.run([b'gz-msgs', *args], executable=os.fsencode(BINARY), cwd=ROOT, env=child, capture_output=True, timeout=10)
        if case == 'missing' and label == 'list':
            assert set(result.stdout.decode().splitlines()) == set(canonical)
        rows.append(dict(case=case, command=label, descriptor_path_hex=path.hex(), args_hex=[arg.hex() for arg in args], exit=result.returncode, stdout_hex=result.stdout.hex(), stderr_hex=result.stderr.hex()))
result = dict(schema_manifest_sha256=sha(REPO / 'schemas/manifest.json'), canonical_listing_sha256=sha(REPO / 'tests/fixtures/cli-external-list.json'), canonical_names=canonical, native_install_prefix='unset; pinned SDK baseline canonical names verified', upstream_commit=baseline['upstream_commit'], native_binary_sha256=sha(BINARY), native_source_sha256=baseline['native_source_sha256'], dynamic_factory_source_sha256=sha(UP / 'core/src/DynamicFactory.cc'), native_libraries_sha256=libs, probe_sha256=sha(Path(__file__)), inputs_hex={k:v.hex() for k,v in inputs.items()}, rows=rows)
original = UP / 'build/bin/INTEGRATION_gz_TEST'
report = ROOT / 'original.xml'
process = subprocess.run([str(original), '--gtest_filter=CmdLine.*', '--gtest_output=xml:' + str(report)], env=env, cwd=ROOT, capture_output=True, timeout=30)
summary = ET.parse(report).getroot()
assert process.returncode == 0 and summary.attrib['tests'] == '5' and summary.attrib['failures'] == '0' and summary.attrib['errors'] == '0'
assert not process.stderr
original_libraries = {}
for line in subprocess.check_output(['ldd', str(original)], env=env, text=True).split('\n'):
    words = line.split()
    if len(words) > 2 and words[1] == '=>' and Path(words[2]).is_file():
        original_libraries[words[0]] = sha(Path(words[2]))
assert os.fsencode(BINARY) + b'\0' in original.read_bytes()
result['original_cli'] = dict(binary_sha256=sha(original), source_sha256=sha(UP / 'test/integration/gz_TEST.cc'), libraries_sha256=original_libraries, tests=5)
output = REPO / 'artifacts/cli-load-diagnostics-reference'
output.mkdir(parents=True, exist_ok=True)
(output / 'original-stdout.bin').write_bytes(process.stdout)
(output / 'original-stderr.bin').write_bytes(process.stderr)
fixture = REPO / 'tests/fixtures/cli-load-diagnostics.json'
if options.write_fixture:
    fixture.write_text(json.dumps(result, indent=2) + '\n')
else:
    assert json.loads(fixture.read_text()) == result
workspace.cleanup()
print('Complete native CLI descriptor-load observations', len(rows), 'and five unchanged originals')

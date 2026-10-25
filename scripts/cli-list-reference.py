#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay complete native CLI type sets with external descriptor search paths."""
import hashlib, json, os, pathlib, shutil, subprocess, tempfile

root = pathlib.Path(__file__).resolve().parents[1]
fixture_path = root / 'tests/fixtures/cli-external-list.json'
fixture = json.loads(fixture_path.read_text())
reference = root.parent / 'gz-msgs'
binary = reference / 'build/bin/gz-msgs'
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=reference, text=True).strip()
assert revision == fixture['upstream_commit']
source = subprocess.check_output(['git', 'show', revision + ':core/cmd/msgs_main.cc'], cwd=reference)
assert hashlib.sha256(source).hexdigest() == fixture['native_source_sha256'] == sha(reference / 'core/cmd/msgs_main.cc')
assert sha(binary) == fixture['native_binary_sha256']
assert sha(root / 'schemas/manifest.json') == fixture['schema_manifest_sha256']
libraries = {}
for line in subprocess.check_output(['ldd', str(binary)], text=True).splitlines():
    fields = line.split()
    if len(fields) > 2 and fields[1] == '=>' and fields[0].startswith(('libgz-', 'libprotobuf')):
        libraries[fields[0]] = sha(pathlib.Path(fields[2]))
assert libraries == fixture['native_libraries_sha256']
for item in fixture['inputs'].values():
    assert sha(root / item['path']) == item['sha256'], item['path']
with tempfile.TemporaryDirectory(prefix='gz-msgs-cli-list-') as temporary:
    temporary = pathlib.Path(temporary)
    compiled = temporary / 'generated.desc'
    subprocess.run(['protoc', '--proto_path=' + str(root / 'tests/fixtures/cli-external-list'),
                    '--proto_path=' + str(root / 'schemas'), '--include_imports',
                    '--descriptor_set_out=' + str(compiled), 'external.proto'], check=True)
    paths = {key: root / item['path'] for key, item in fixture['inputs'].items() if key != 'source'}
    assert compiled.read_bytes() == paths['external'].read_bytes()
    for kind in ['directory', 'priority']:
        directory = temporary / kind
        directory.mkdir()
        for key in ['testing', 'alias', 'external']:
            filename = 'gz-msgs.gz_desc' if kind == 'priority' and key == 'external' else key + '.desc'
            shutil.copyfile(paths[key], directory / filename)
        paths[kind] = directory
    paths['ignored'] = temporary / 'ignored.bin'
    paths['ignored'].write_text('not a descriptor')
    shutil.copyfile(paths['ignored'], paths['directory'] / 'ignored.bin')
    for row in fixture['cases']:
        env = dict(os.environ)
        env.pop('GZ_DESCRIPTOR_PATH', None)
        if row['environment_present']:
            env['GZ_DESCRIPTOR_PATH'] = os.pathsep.join(str(paths[key]) for key in row['paths'])
        result = subprocess.run([fixture['native_argv0'], '--list'], executable=binary,
                                env=env, capture_output=True, text=True)
        assert result.returncode == row['status'] and result.stderr == row['stderr'], row['label']
        expected = sorted(fixture['canonical_names'] + row['additional_names'])
        assert sorted(result.stdout.splitlines()) == expected, row['label']
proof = dict(upstream_commit=revision, native_binary_sha256=sha(binary),
             native_libraries_sha256=libraries, native_source_sha256=fixture['native_source_sha256'],
             schema_manifest_sha256=fixture['schema_manifest_sha256'], inputs=fixture['inputs'],
             fixture_sha256=sha(fixture_path), cases=len(fixture['cases']),
             scope='Complete sorted name sets, statuses and stderr; valid descriptors only. No external info formatter or malformed-input diagnostic claim.')
output = root / 'artifacts/cli-external-list-reference.json'
output.parent.mkdir(exist_ok=True)
output.write_text(json.dumps(proof, indent=2) + '\n')
print(f"{len(fixture['cases'])} complete native descriptor-aware CLI lists reproduced")

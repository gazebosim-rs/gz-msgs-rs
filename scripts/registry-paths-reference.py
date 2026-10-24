#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Capture the original native explicit-path/directory Factory outcomes."""
import hashlib,json,pathlib,shlex,subprocess
root=pathlib.Path(__file__).resolve().parents[1];reference=root.parent/'gz-msgs';out=root/'target/reference';out.mkdir(parents=True,exist_ok=True)
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=reference,text=True).strip()=='89c0c3d1474f545cb889335daf930bfdab6ab852'
for item in json.loads((root/'tests/reference/manifest.json').read_text())['files']:
 assert hashlib.sha256((root/item['path']).read_bytes()).hexdigest()==item['sha256']
flags=shlex.split(subprocess.check_output(['pkg-config','--cflags','--libs','gz-msgs'],text=True));libdir=subprocess.check_output(['pkg-config','--variable=libdir','gz-msgs'],text=True).strip()
subprocess.run(['c++','-std=c++17','-O2','-I'+str(reference/'core/include'),str(root/'tests/registry_paths_reference.cc'),*flags,'-Wl,-rpath,'+libdir,'-o',str(out/'registry-paths-reference')],check=True)
result=subprocess.run([str(out/'registry-paths-reference'),'tests/reference/descriptors','target/reference/paths-execute-only'],cwd=root,check=True,capture_output=True,text=True)
assert 'already exists in database' not in result.stderr
fixture=json.loads(result.stdout)
fixture.update(upstream_commit='89c0c3d1474f545cb889335daf930bfdab6ab852',source_sha256=hashlib.sha256((reference/'test/integration/descriptors.cc').read_bytes()).hexdigest(),native_binary_sha256=hashlib.sha256((out/'registry-paths-reference').read_bytes()).hexdigest(),stderr=result.stderr)
(root/'tests/fixtures/registry-paths-reference.json').write_text(json.dumps(fixture,indent=2)+'\n');print('native explicit-path/directory and duplicate descriptor outcomes captured')

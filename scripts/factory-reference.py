#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Capture the canonical native Factory's types and original text construction."""
import hashlib,json,pathlib,shlex,subprocess
root=pathlib.Path(__file__).resolve().parents[1];reference=root.parent/'gz-msgs';out=root/'target/reference';out.mkdir(parents=True,exist_ok=True)
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=reference,text=True).strip()=='89c0c3d1474f545cb889335daf930bfdab6ab852'
flags=shlex.split(subprocess.check_output(['pkg-config','--cflags','--libs','gz-msgs'],text=True));libdir=subprocess.check_output(['pkg-config','--variable=libdir','gz-msgs'],text=True).strip()
subprocess.run(['c++','-std=c++17','-O2','-I'+str(reference/'core/include'),str(root/'tests/factory_reference.cc'),*flags,'-Wl,-rpath,'+libdir,'-o',str(out/'factory-reference')],check=True)
result=subprocess.run([str(out/'factory-reference')],check=True,capture_output=True,text=True)
fixture=json.loads(result.stdout)
fixture.update(upstream_commit='89c0c3d1474f545cb889335daf930bfdab6ab852',header_sha256=hashlib.sha256((reference/'core/include/gz/msgs/Factory.hh').read_bytes()).hexdigest(),native_binary_sha256=hashlib.sha256((out/'factory-reference').read_bytes()).hexdigest(),malformed_stderr=result.stderr)
(root/'tests/fixtures/factory-reference.json').write_text(json.dumps(fixture,indent=2)+'\n');print(len(fixture['types']),'native types constructed; valid and malformed text recorded')

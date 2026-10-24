#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Capture safe aligned native iterator writes for every scalar and color order."""
import hashlib,json,pathlib,shlex,subprocess,sys
root=pathlib.Path(__file__).resolve().parents[1];reference=root.parent/'gz-msgs';out=root/'target/reference';out.mkdir(parents=True,exist_ok=True)
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=reference,text=True).strip()=='89c0c3d1474f545cb889335daf930bfdab6ab852'
flags=shlex.split(subprocess.check_output(['pkg-config','--cflags','--libs','gz-msgs','gz-math'],text=True));libdir=subprocess.check_output(['pkg-config','--variable=libdir','gz-msgs'],text=True).strip()
subprocess.run(['c++','-std=c++17','-O2','-I'+str(reference/'core/include'),str(root/'tests/cloud_scalar_reference.cc'),*flags,'-Wl,-rpath,'+libdir,'-o',str(out/'cloud-scalar-reference')],check=True)
rows=[]
for mode,kinds in [('scalar',range(8)),('color',range(2))]:
 for kind in kinds:
  result=subprocess.run([str(out/'cloud-scalar-reference'),mode,str(kind)],check=True,capture_output=True,text=True)
  assert not result.stderr
  rows.append({'mode':mode,'kind':kind,'wire_hex':result.stdout})
fixture={'upstream_commit':'89c0c3d1474f545cb889335daf930bfdab6ab852','header_sha256':hashlib.sha256((reference/'core/include/gz/msgs/detail/PointCloudPackedUtils.hh').read_bytes()).hexdigest(),'native_binary_sha256':hashlib.sha256((out/'cloud-scalar-reference').read_bytes()).hexdigest(),'byte_order':sys.byteorder,'rows':rows}
(root/'tests/fixtures/cloud-scalar-reference.json').write_text(json.dumps(fixture,indent=2)+'\n');print(len(rows),'native iterator write layouts captured')

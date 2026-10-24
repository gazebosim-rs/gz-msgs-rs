#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Measure packed-cloud metadata layout from the pinned native helper."""
import hashlib,json,pathlib,shlex,subprocess
root=pathlib.Path(__file__).resolve().parents[1];reference=root.parent/'gz-msgs';out=root/'target/reference';out.mkdir(parents=True,exist_ok=True)
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=reference,text=True).strip()=='89c0c3d1474f545cb889335daf930bfdab6ab852'
flags=shlex.split(subprocess.check_output(['pkg-config','--cflags','--libs','gz-msgs','gz-math'],text=True));libdir=subprocess.check_output(['pkg-config','--variable=libdir','gz-msgs'],text=True).strip()
subprocess.run(['c++','-std=c++17','-O2','-I'+str(reference/'core/include'),str(root/'tests/cloud_reference.cc'),*flags,'-Wl,-rpath,'+libdir,'-o',str(out/'cloud-reference')],check=True)
def run(*args):return subprocess.run([str(out/'cloud-reference'),*map(str,args)],check=True,capture_output=True,text=True)
layouts=[]
fields=[[],[['xyz',6],['rgb',6]],[['xyz',6],['rgba',6]],[['x',0],['y',1],['z',2],['r',3],['s',4],['t',5],['u',7]],[['bad',100]],[['xyz',100]]]+[[['value',dtype]] for dtype in range(8)]
for align in [False,True]:
 for specification in fields:
  result=run('init',int(align),'frame',*[part for field in specification for part in field]);layouts.append({'aligned':align,'frame':'frame','fields':specification,'wire_hex':result.stdout,'stderr':result.stderr})
for frame in ['', 'frame with spaces']:
 result=run('init',0,frame,'xyz',6);layouts.append({'aligned':False,'frame':frame,'fields':[['xyz',6]],'wire_hex':result.stdout,'stderr':result.stderr})
sizes=[]
for datatype in list(range(8))+[-1,100]:
 result=run('size',datatype);sizes.append({'datatype':datatype,'size':int(result.stdout),'stderr':result.stderr})
header=reference/'core/include/gz/msgs/PointCloudPackedUtils.hh'
fixture={'upstream_commit':'89c0c3d1474f545cb889335daf930bfdab6ab852','header_sha256':hashlib.sha256(header.read_bytes()).hexdigest(),'native_binary_sha256':hashlib.sha256((out/'cloud-reference').read_bytes()).hexdigest(),'word_bytes':int(run('word').stdout),'layouts':layouts,'sizes':sizes}
(root/'tests/fixtures/cloud-reference.json').write_text(json.dumps(fixture,indent=2)+'\n');print(len(layouts),'native layouts and',len(sizes),'datatype sizes captured')

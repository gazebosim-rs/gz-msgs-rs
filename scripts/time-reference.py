#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Regenerate measured duration fixtures; requires the real pinned native reference."""
import hashlib, json, pathlib, shlex, subprocess
root=pathlib.Path(__file__).resolve().parents[1]
reference=root.parent/'gz-msgs'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=reference,text=True).strip()=='89c0c3d1474f545cb889335daf930bfdab6ab852'
out=root/'target/reference'
out.mkdir(parents=True,exist_ok=True)
flags=shlex.split(subprocess.check_output(['pkg-config','--cflags','--libs','gz-msgs','gz-math'],text=True))
libdir=subprocess.check_output(['pkg-config','--variable=libdir','gz-msgs'],text=True).strip()
subprocess.run(['c++','-std=c++17','-O2','-I'+str(reference/'core/include'),str(root/'tests/time_reference.cc'),*flags,'-Wl,-rpath,'+libdir,'-o',str(out/'time-reference')],check=True)
def run(*args):return subprocess.check_output([str(out/'time-reference'),*map(str,args)],text=True).strip()
encode=[]
for nanos in [-(2**63),-200_000_000_999,-1_000_000_001,-1_000_000_000,-999_999_999,-1,0,1,999_999_999,1_000_000_000,1_000_000_001,200_000_000_999,2**63-1]:
    fields=run('encode',nanos).split()
    encode.append({'nanos':nanos,'sec':int(fields[0]),'nsec':int(fields[1]),'wire_hex':fields[2] if len(fields)>2 else ''})
decode=[]
for sec,nsec in [(0,-1),(1,-1),(-1,1),(0,1_000_000_000),(0,-1_000_000_000),(0,2**31-1),(0,-2**31),(9223372036,854775807),(-9223372036,-854775808)]:
    decode.append({'sec':sec,'nsec':nsec,'nanos':int(run('decode',sec,nsec))})
header=reference/'core/include/gz/msgs/convert/StdTypes.hh'
fixture={'upstream_commit':'89c0c3d1474f545cb889335daf930bfdab6ab852','header_sha256':hashlib.sha256(header.read_bytes()).hexdigest(),'native_binary_sha256':hashlib.sha256((out/'time-reference').read_bytes()).hexdigest(),'math_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root.parent/'gz-math',text=True).strip(),'math_helpers_sha256':hashlib.sha256((root.parent/'gz-math/src/Helpers.cc').read_bytes()).hexdigest(),'native_protobuf':subprocess.check_output(['pkg-config','--modversion','protobuf'],text=True).strip(),'encode':encode,'decode':decode}
(root/'tests/fixtures').mkdir(exist_ok=True)
(root/'tests/fixtures/time-reference.json').write_text(json.dumps(fixture,indent=2)+'\n')
for row in encode:print(row)
for row in decode:print(row)

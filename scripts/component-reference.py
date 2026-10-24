#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Capture native component setters/Euler inputs and nine original cases."""
import hashlib,json,pathlib,shlex,subprocess,xml.etree.ElementTree as ET
root=pathlib.Path(__file__).resolve().parents[1];reference=root.parent/'gz-msgs';out=root/'target/reference';out.mkdir(parents=True,exist_ok=True)
revision='89c0c3d1474f545cb889335daf930bfdab6ab852'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=reference,text=True).strip()==revision
flags=shlex.split(subprocess.check_output(['pkg-config','--cflags','--libs','gz-msgs'],text=True));libdir=subprocess.check_output(['pkg-config','--variable=libdir','gz-msgs'],text=True).strip();binary=out/'component-reference'
subprocess.run(['c++','-std=c++17','-O2','-I'+str(reference/'core/include'),str(root/'tests/component_reference.cc'),*flags,'-Wl,-rpath,'+libdir,'-o',str(binary)],check=True)
result=subprocess.run([str(binary)],check=True,capture_output=True,text=True);assert not result.stderr
fixture=json.loads(result.stdout);native=reference/'build/bin/INTEGRATION_Utility_TEST'
cases=['UtilityTest.ConvertMathQuaterionToMsgs','UtilityTest.ConvertMsgsQuaterionToMath','UtilityTest.ConvertPoseMathToMsgs','UtilityTest.ConvertMsgPoseToMath','UtilityTest.SetVector3','UtilityTest.SetVector2d','UtilityTest.SetQuaternion','UtilityTest.SetPose','MsgsTest.SetColor']
for case in cases:
 report=out/(case+'.xml');run=subprocess.run([str(native),'--gtest_filter='+case,'--gtest_output=xml:'+str(report)],check=True,capture_output=True,text=True);(out/(case+'.log')).write_text(run.stdout+run.stderr)
 tests=[(suite.attrib['name']+'.'+test.attrib['name'],test) for suite in ET.parse(report).getroot() for test in suite.findall('testcase')]
 assert len(tests)==1 and tests[0][0]==case and tests[0][1].attrib.get('status')=='run' and not list(tests[0][1]),case
sources=['core/include/gz/msgs/convert/'+name+'.hh' for name in ['Vector2','Vector3','Color','Quaternion','Pose']]+['test/integration/Utility_TEST.cc']
fixture.update(upstream_commit=revision,source_sha256={p:hashlib.sha256((reference/p).read_bytes()).hexdigest() for p in sources},probe_source_sha256=hashlib.sha256((root/'tests/component_reference.cc').read_bytes()).hexdigest(),probe_binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),original_native_binary_sha256=hashlib.sha256(native.read_bytes()).hexdigest(),original_native_tests=cases)
(root/'tests/fixtures/component-reference.json').write_text(json.dumps(fixture,indent=2)+'\n');print('18 full-wire component setters/native reverse bits and nine original cases pass')

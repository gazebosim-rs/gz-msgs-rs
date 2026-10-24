#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Capture native callback behavior and rerun selected original Factory/header cases."""
import hashlib,json,os,pathlib,shlex,subprocess,xml.etree.ElementTree as ET
root=pathlib.Path(__file__).resolve().parents[1];reference=root.parent/'gz-msgs';out=root/'target/reference';out.mkdir(parents=True,exist_ok=True)
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=reference,text=True).strip()=='89c0c3d1474f545cb889335daf930bfdab6ab852'
env=dict(os.environ);env.pop('GZ_DESCRIPTOR_PATH',None)
flags=shlex.split(subprocess.check_output(['pkg-config','--cflags','--libs','gz-msgs'],text=True));libdir=subprocess.check_output(['pkg-config','--variable=libdir','gz-msgs'],text=True).strip()
subprocess.run(['c++','-std=c++17','-O2','-I'+str(reference/'core/include'),str(root/'tests/factory_state_reference.cc'),*flags,'-Wl,-rpath,'+libdir,'-o',str(out/'factory-state-reference')],check=True)
result=subprocess.run([str(out/'factory-state-reference')],cwd=root,env=env,check=True,capture_output=True,text=True,timeout=30);assert not result.stderr
fixture=json.loads(result.stdout);native=[]
for binary,case in [('INTEGRATION_Factory_TEST','FactoryTest.DynamicMessageWithEnv'),('INTEGRATION_factory_lazy','FactoryLazyTest.DescriptorPathReadAtFirstDynamicUse'),('INTEGRATION_factory_concurrency','FactoryConcurrency.ParallelUse'),('INTEGRATION_Factory_TEST','FactoryTest.MultipleMessagesInAProto'),('INTEGRATION_headers','Messages.NestedHeaders')]:
 report=out/(case+'-bounded.xml');result=subprocess.run([str(reference/'build/bin'/binary),'--gtest_filter='+case,'--gtest_output=xml:'+str(report)],cwd=root,env=env,check=True,capture_output=True,text=True,timeout=30)
 (out/(case+'-bounded.log')).write_text(result.stdout+result.stderr)
 tests=[(suite.attrib['name']+'.'+test.attrib['name'],test) for suite in ET.parse(report).getroot() for test in suite.findall('testcase')]
 assert len(tests)==1 and tests[0][0]==case and tests[0][1].attrib.get('status')=='run' and not list(tests[0][1]),case
 native.append({'case':case,'status':'passed','binary_sha256':hashlib.sha256((reference/'build/bin'/binary).read_bytes()).hexdigest()})
sources=['test/integration/Factory_TEST.cc','test/integration/factory_lazy.cc','test/integration/factory_concurrency.cc','test/integration/headers.cc']
fixture.update(upstream_commit='89c0c3d1474f545cb889335daf930bfdab6ab852',source_sha256={name:hashlib.sha256((reference/name).read_bytes()).hexdigest() for name in sources},native_binary_sha256=hashlib.sha256((out/'factory-state-reference').read_bytes()).hexdigest(),original_native_tests=native)
(root/'tests/fixtures/factory-state-reference.json').write_text(json.dumps(fixture,indent=2)+'\n');print('native callback replacement/text/reentry and five original Factory/header cases pass')

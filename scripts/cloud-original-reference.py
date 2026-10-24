#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Rerun packed-cloud and initialization originals, retaining exact XML IDs."""
import hashlib,json,pathlib,subprocess,xml.etree.ElementTree as ET
root=pathlib.Path(__file__).resolve().parents[1];reference=root.parent/'gz-msgs';out=root/'target/reference';out.mkdir(parents=True,exist_ok=True)
revision='89c0c3d1474f545cb889335daf930bfdab6ab852'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=reference,text=True).strip()==revision
cases=[]
selected=[('INTEGRATION_PointCloudPackedUtils_TEST','PointCloudPackedUtilsTest.'+name) for name in ['BadFields','Operators','MultipleFields','XYZRGBA']]
selected.append(('INTEGRATION_Utility_TEST','UtilityTest.Initialization'))
for filename,case in selected:
 binary=reference/'build/bin'/filename;report=out/(case+'.xml')
 run=subprocess.run([str(binary),'--gtest_filter='+case,'--gtest_output=xml:'+str(report)],cwd=root,check=True,capture_output=True,text=True)
 (out/(case+'.log')).write_text(run.stdout+run.stderr)
 tests=[(suite.attrib['name']+'.'+test.attrib['name'],test) for suite in ET.parse(report).getroot() for test in suite.findall('testcase')]
 assert len(tests)==1 and tests[0][0]==case and tests[0][1].attrib.get('status')=='run' and not list(tests[0][1]),case
 cases.append(dict(case=case,binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest()))
sources=['test/integration/PointCloudPackedUtils_TEST.cc','test/integration/Utility_TEST.cc']
proof=dict(upstream_commit=revision,source_sha256={name:hashlib.sha256((reference/name).read_bytes()).hexdigest() for name in sources},original_native_tests=cases)
(out/'cloud-originals.json').write_text(json.dumps(proof,indent=2)+'\n');print('four original native packed-cloud cases and initialization pass; cursor identity mapping requires independent review')

#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Run an existing pinned native build, retaining all successes and failures."""
import hashlib, json, os, pathlib, subprocess, sys, xml.etree.ElementTree as ET
root = pathlib.Path(__file__).resolve().parents[1]
upstream = pathlib.Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else root.parent / 'gz-msgs'
assert subprocess.check_output(['git','rev-parse','HEAD'], cwd=upstream, text=True).strip() == '89c0c3d1474f545cb889335daf930bfdab6ab852'
out = root / 'target/reference'
out.mkdir(parents=True,exist_ok=True)
results=[]
for binary in sorted((upstream/'build/bin').glob('INTEGRATION_*')):
    env=os.environ.copy();env['GZ_MSG_INSTALL_PREFIX']='/home/nkoenig/local'
    command=[str(binary),'--gtest_output=xml:'+str(out/(binary.name+'.xml'))]
    result=subprocess.run(command,cwd=root,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=240)
    (out/(binary.name+'.log')).write_text(result.stdout)
    cases=[]
    for suite in ET.parse(out/(binary.name+'.xml')).getroot():
        for case in suite.findall('testcase'):
            cases.append({'id':suite.attrib['name']+'.'+case.attrib['name'],'status':'failed' if case.find('failure') is not None else ('skipped' if case.find('skipped') is not None else 'passed')})
    results.append({'binary':binary.name,'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'command':command,'exit_code':result.returncode,'tests':cases})
    print(binary.name, result.returncode, len(cases))
(out/'results.json').write_text(json.dumps(results,indent=2)+'\n')
if any(r['exit_code'] for r in results): raise SystemExit(1)

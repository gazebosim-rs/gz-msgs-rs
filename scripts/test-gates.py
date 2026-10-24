#!/usr/bin/env python3
import importlib.util, pathlib
spec = importlib.util.spec_from_file_location('gate', pathlib.Path(__file__).with_name('check-coverage.py'))
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)
root = pathlib.Path(__file__).resolve().parents[1]
for count, covered, present, passes in [(100,95,True,False),(100,94,True,False),(100,96,True,True),(0,0,True,False),(100,100,False,False)]:
    report = {'data':[{'files':[{'filename':str(p.resolve()),'summary':{m:{'count':count,'covered':covered} for m in ['lines','regions']}} for p in [*(root/'src').rglob('*.rs'), root/'build.rs']] if present else []}]}
    try: gate.check(report,root); result=True
    except ValueError: result=False
    assert result == passes
print('Coverage gates: exactly 95, below 95, empty and missing rejected; 96 accepted')

# Owned dependency changes get the same strict metrics, including missing profiles.
import json, tempfile
with tempfile.TemporaryDirectory() as directory:
    fixture = pathlib.Path(directory)
    (fixture/'vendor').mkdir()
    source = fixture/'vendor/lib.rs'
    source.write_text('value\n'*100)
    (fixture/'vendor/manifest.json').write_text(json.dumps({'packages':[{'name':'fixture','files':[{'path':'vendor/lib.rs','owned_lines':list(range(1,101))}]}]}))
    for lines,regions,present,expected in [(95,100,True,False),(100,95,True,False),(96,96,True,True),(100,100,False,False)]:
        lcov='SF:'+str(source)+'\n'+''.join(f'DA:{n},{int(n<=lines)}\n' for n in range(1,101)) if present else ''
        report={'data':[{'functions':[{'filenames':[str(source)],'regions':[[n,1,n,2,int(n<=regions),0,0,0] for n in range(1,101)]}]}]}
        try:gate.check_patches(report,fixture,lcov);actual=True
        except ValueError:actual=False
        assert actual==expected
print('Owned dependency gates: exactly 95 line/region and missing instrumentation rejected')

# Global wrapper selection includes both patched dependencies, but not registries.
import os, subprocess
wrapper=root/'scripts/rustc-coverage.py'
for manifest,expected in [(str(root),True),(str(root/'vendor/prost-derive-0.14.4'),True),(str(root/'vendor/prost-reflect-0.16.5'),True),('/tmp/unowned-protobuf',False),(None,False)]:
    env=os.environ.copy();env['GZ_MSGS_COVERAGE_ROOT']=str(root)
    if manifest is None:env.pop('CARGO_MANIFEST_DIR',None)
    else:env['CARGO_MANIFEST_DIR']=manifest
    output=subprocess.check_output([str(wrapper),'/bin/echo','--crate-name','fixture'],env=env,text=True).strip()
    assert output=='--crate-name fixture'+(' -C instrument-coverage' if expected else '')
print('Coverage selection: owned root/two patched vendors included; unowned/query invocations excluded')

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

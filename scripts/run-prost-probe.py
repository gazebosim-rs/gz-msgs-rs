#!/usr/bin/env python3
"""Record candidate failures and patched results; do not count expected failures as passes."""
import json, os, pathlib, subprocess
root=pathlib.Path(__file__).resolve().parents[1]
os.chdir(root)
subprocess.run(['python3','scripts/native-peer.py'],check=True)
subprocess.run(['python3','scripts/prepare-prost-probe.py'],check=True)
for variant in ['prost-probe','prost-probe-patched']:
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(root/'target'/variant)
    result=subprocess.run(['cargo','run','--locked','--manifest-path',f'experiments/{variant}/Cargo.toml'],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
    (root/'target'/f'{variant}.log').write_text(result.stdout+result.stderr)
    report=json.loads(result.stdout[result.stdout.index('[\n'):])
    (root/'target'/f'{variant}.json').write_text(json.dumps(report,indent=2)+'\n')
    required_failures={r['probe'] for r in report if r['required'] and not r['passed']}
    if variant=='prost-probe':
        expected={prefix+suffix for prefix in ['generated_','dynamic_','text_','json_'] for suffix in ['signed_zero','f32_signed_zero']}
        assert result.returncode==1 and required_failures==expected,(result.returncode,required_failures)
    else:
        assert result.returncode==0 and not required_failures,(result.returncode,required_failures)
    diagnostic={r['probe'] for r in report if not r['required'] and not r['passed']}
    assert diagnostic=={'generated_unknown_fields'}
    print(variant,len(report),'checks;',len(required_failures),'required failures;',len(diagnostic),'unsupported diagnostics')

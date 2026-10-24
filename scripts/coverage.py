#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
import hashlib, json, os, pathlib, shutil, subprocess
root = pathlib.Path(__file__).resolve().parents[1]
os.chdir(root)
env = os.environ.copy()
cargo = env.get('CARGO', 'cargo')
sysroot = pathlib.Path(subprocess.check_output([env.get('RUSTC', 'rustc'), '--print', 'sysroot'], text=True).strip())
host = next(line.split(': ', 1)[1] for line in subprocess.check_output([env.get('RUSTC', 'rustc'), '-vV'], text=True).splitlines() if line.startswith('host: '))
llvm = sysroot / 'lib/rustlib' / host / 'bin'
out = root / 'target/coverage'
out.mkdir(parents=True, exist_ok=True)
for p in out.glob('*.profraw'): p.unlink()
env['CARGO_TARGET_DIR'] = str(out / 'build')
wrapper_source=root/'scripts/rustc-coverage.py'
wrapper_digest=hashlib.sha256(wrapper_source.read_bytes()).hexdigest()
wrapper=out/f'rustc-coverage-{wrapper_digest}.py'
shutil.copy2(wrapper_source,wrapper)
env['RUSTC_WRAPPER']=str(wrapper)
env.pop('RUSTC_WORKSPACE_WRAPPER',None)
env.pop('CARGO_ENCODED_RUSTFLAGS',None)
env['GZ_MSGS_COVERAGE_ROOT']=str(root)
# Cargo fingerprints this otherwise-unused cfg, preventing stale artifacts when
# wrapper policy changes even for owned path dependencies outside the workspace.
env['RUSTFLAGS']='--cfg=gz_msgs_coverage_'+wrapper_digest
env['LLVM_PROFILE_FILE'] = str(out / '%p-%m.profraw')
subprocess.run([cargo, 'clean', '-p', 'gz-msgs', '-p', 'prost-derive', '-p', 'prost-reflect'], env=env, check=True)
subprocess.run([cargo, 'test', '--locked', '--tests'], env=env, check=True)
subprocess.run([str(llvm / 'llvm-profdata'), 'merge', '-sparse', *map(str, out.glob('*.profraw')), '-o', str(out / 'merged.profdata')], check=True)
metadata = subprocess.check_output([cargo, 'test', '--locked', '--tests', '--no-run', '--message-format=json'], env=env, text=True)
objects = [json.loads(line)['executable'] for line in metadata.splitlines() if line.startswith('{') and json.loads(line).get('executable')]
objects += list(map(str, (out / 'build/debug/build').glob('gz-msgs-*/build-script-build')))
for line in metadata.splitlines():
    if not line.startswith('{'): continue
    artifact=json.loads(line)
    target=artifact.get('target',{})
    if 'proc-macro' in target.get('kind',[]) and pathlib.Path(target['src_path']).is_relative_to(root/'vendor'):
        objects += [path for path in artifact['filenames'] if path.endswith('.so')]
# Coverage's denominator is all handwritten production Rust; only compiler-generated
# OUT_DIR artifacts and nonproduction dependencies/tests/tooling are outside it.
# The handwritten build.rs is an explicitly required measured production source.
args = [str(llvm / 'llvm-cov'), 'export', '--fatal-warnings', '-num-threads=1', '-instr-profile=' + str(out / 'merged.profdata')]
for obj in objects: args += ['-object', obj]
with (out / 'coverage.json').open('w') as stream:
    subprocess.run(args, stdout=stream, check=True)
with (out / 'coverage.lcov').open('w') as stream:
    subprocess.run([*args, '-format=lcov'], stdout=stream, check=True)
subprocess.run(['python3', 'scripts/check-coverage.py', str(out / 'coverage.json')], check=True)

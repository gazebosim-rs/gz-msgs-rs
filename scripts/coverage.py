#!/usr/bin/env python3
import json, os, pathlib, subprocess
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
env['RUSTFLAGS'] = '-C instrument-coverage'
env['LLVM_PROFILE_FILE'] = str(out / '%p-%m.profraw')
subprocess.run([cargo, 'clean', '-p', 'gz-msgs'], env=env, check=True)
subprocess.run([cargo, 'test', '--locked', '--tests'], env=env, check=True)
subprocess.run([str(llvm / 'llvm-profdata'), 'merge', '-sparse', *map(str, out.glob('*.profraw')), '-o', str(out / 'merged.profdata')], check=True)
metadata = subprocess.check_output([cargo, 'test', '--locked', '--tests', '--no-run', '--message-format=json'], env=env, text=True)
objects = [json.loads(line)['executable'] for line in metadata.splitlines() if line.startswith('{') and json.loads(line).get('executable')]
objects += list(map(str, (out / 'build/debug/build').glob('gz-msgs-*/build-script-build')))
# Coverage's denominator is all handwritten production Rust; only compiler-generated
# OUT_DIR artifacts and nonproduction dependencies/tests/tooling are outside it.
# The handwritten build.rs is an explicitly required measured production source.
args = [str(llvm / 'llvm-cov'), 'export', '-instr-profile=' + str(out / 'merged.profdata')]
for obj in objects: args += ['-object', obj]
with (out / 'coverage.json').open('w') as stream:
    subprocess.run(args, stdout=stream, check=True)
subprocess.run(['python3', 'scripts/check-coverage.py', str(out / 'coverage.json')], check=True)

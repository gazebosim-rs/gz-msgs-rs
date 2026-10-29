#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Compare generated codecs at equal output fidelity; no performance gate."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import random
import re
import shlex
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
import tomllib

ROOT = Path(__file__).resolve().parents[1]
TOOLS = ROOT / 'tools/codec-benchmark'
FIXTURES = ('clock', 'imu', 'poses_1', 'poses_100', 'poses_1000', 'scan_360', 'scan_1080')
MODES = ('encode_fresh', 'encode_reuse', 'decode_fresh', 'decode_reuse', 'roundtrip_fresh', 'roundtrip_reuse')
ENVELOPES = ('time_plain', 'time_unknown', 'time_wrong')
MASK = (1 << 64) - 1


def command(argv, **kwargs):
    try:
        return subprocess.run(list(map(str, argv)), cwd=kwargs.pop('cwd', ROOT), check=True, capture_output=True, **kwargs)
    except subprocess.CalledProcessError as error:
        sys.stderr.buffer.write(error.stdout + error.stderr)
        raise


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def rust_invocations(stderr, compiler, host):
    units = []
    for line in stderr.decode().splitlines():
        match = re.match(r'^\s*Running `(.+)`$', line)
        if not match:
            continue
        args = shlex.split(match[1])
        if '--crate-name' not in args:
            continue  # Build-script execution, not compilation.
        assert Path(args[0]).resolve() == compiler, 'Unexpected Rust compiler or wrapper'
        options = {}
        iterator = iter(args[1:])
        for arg in iterator:
            if arg == '-C' or arg.startswith('-C'):
                key, _, value = (next(iterator) if arg == '-C' else arg[2:]).partition('=')
                options[key] = value
        runtime = ('--target' in args and args[args.index('--target') + 1] == host
                   and args[args.index('--crate-name') + 1] != 'build_script_build'
                   and not ('--crate-type' in args and args[args.index('--crate-type') + 1] == 'proc-macro'))
        if runtime:
            assert options.get('opt-level') == '3', 'Runtime compiler is not at opt-level 3'
            for key in ('debug-assertions', 'overflow-checks', 'lto'):
                assert options.get(key, 'off') in ('off', 'false', 'no'), (key, options)
            assert options.get('codegen-units', '16') == '16', options
            assert not set(options) & {'instrument-coverage', 'profile-generate', 'profile-use', 'target-cpu', 'target-feature', 'incremental'}, options
        units.append({'crate': args[args.index('--crate-name') + 1], 'runtime': runtime,
                      'arguments': args, 'codegen': options})
    assert any(unit['runtime'] for unit in units), 'No fresh runtime compiler evidence'
    return units


def rust_build(manifest, output, name):
    channel = tomllib.loads((ROOT / 'rust-toolchain.toml').read_text())['toolchain']['channel']
    compiler = Path(command(['rustup', 'which', '--toolchain', channel, 'rustc']).stdout.decode().strip()).resolve(strict=True)
    cargo_binary = Path(command(['rustup', 'which', '--toolchain', channel, 'cargo']).stdout.decode().strip()).resolve(strict=True)
    version = command([compiler, '-Vv']).stdout.decode()
    host = re.search(r'^host: (.+)$', version, re.M).group(1)
    env = dict(os.environ)
    for key in list(env):
        if key.startswith('CARGO_PROFILE_RELEASE_'):
            del env[key]
    for key in ('RUSTC', 'CARGO_BUILD_RUSTC'):
        env[key] = str(compiler)
    env['CARGO_INCREMENTAL'] = '0'
    for key in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'CARGO_BUILD_RUSTFLAGS', 'RUSTC_WRAPPER',
                'RUSTC_WORKSPACE_WRAPPER', 'CARGO_BUILD_RUSTC_WRAPPER', 'CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER'):
        env[key] = ''
    settings = {'build.rustc': str(compiler), 'build.rustc-wrapper': '', 'build.rustc-workspace-wrapper': '',
                'profile.release.opt-level': 3, 'profile.release.debug': False,
                'profile.release.debug-assertions': False, 'profile.release.overflow-checks': False,
                'profile.release.lto': False, 'profile.release.codegen-units': 16,
                'profile.release.incremental': False, 'profile.release.panic': 'unwind'}
    # Fresh compilation makes the observed compiler argv evidence complete.
    with tempfile.TemporaryDirectory(prefix='rust-build-', dir=output) as target:
        cargo = [cargo_binary, 'build', '--locked', '--release', '--verbose', '--color', 'never', '--message-format=json',
                 '--manifest-path', manifest, '--target', host, '--target-dir', target]
        for key, value in settings.items():
            cargo.extend(['--config', key + '=' + json.dumps(value)])
        built = command(cargo, env=env, cwd=manifest.parent)
        units = rust_invocations(built.stderr, compiler, host)
        assert any(unit['runtime'] and unit['crate'] == name.replace('-', '_') for unit in units)
        profiles = [row for line in built.stdout.splitlines()
                    if (row := json.loads(line)).get('reason') == 'compiler-artifact']
        binary = output / 'rust'
        shutil.copy2(Path(target) / host / 'release' / name, binary)
    return binary, {'command': list(map(str, cargo)), 'cargo_sha256': sha(cargo_binary),
                    'cargo_version': command([cargo_binary, '--version']).stdout.decode().strip(),
                    'compiler': str(compiler), 'compiler_sha256': sha(compiler),
                    'version': version, 'invocations': units,
                    'profiles': [{'package': row['package_id'], 'target': row['target']['name'],
                                  'kind': row['target']['kind'], 'profile': row['profile']} for row in profiles]}


def schema_closure():
    pending = ['gz/msgs/' + name + '.proto' for name in ('clock', 'imu', 'pose_v', 'laserscan')]
    result = set()
    while pending:
        name = pending.pop()
        if name not in result:
            result.add(name)
            pending.extend(re.findall(r'^import "([^"]+)";', (ROOT / 'schemas' / name).read_text(), re.M))
    return sorted(result)


def build(output):
    generated = output / 'generated'
    generated.mkdir(parents=True, exist_ok=True)
    manifest = json.loads((ROOT / 'schemas/manifest.json').read_text())
    for item in manifest['schemas']:
        assert sha(ROOT / item['path']) == item['sha256'], item['path']
    for package in json.loads((ROOT / 'vendor/manifest.json').read_text())['packages']:
        for item in package['files']:
            assert sha(ROOT / item['path']) == item['sha256'], item['path']
    root_lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())
    bench_lock = tomllib.loads((TOOLS / 'Cargo.lock').read_text())
    def packages(lock):
        return {(p['name'], p['version'], p.get('source'), p.get('checksum')) for p in lock['package']}
    assert packages(bench_lock) - {('gz-msgs-codec-benchmark', '0.0.0', None, None)} <= packages(root_lock), 'Benchmark runtime differs from root lock'
    schemas = schema_closure()
    protoc = ['protoc', '-I' + str(ROOT / 'schemas'), '--cpp_out=' + str(generated), *schemas]
    command(protoc)
    native = output / 'native'
    compiler = Path(shutil.which('c++')).resolve(strict=True)
    cxx = [compiler, '-std=c++17', '-O3', '-DNDEBUG', '-Wall', '-Wextra', '-Werror',
           '-I' + str(generated), TOOLS / 'native.cc',
           *[generated / (name[:-6] + '.pb.cc') for name in schemas], '-lprotobuf', '-pthread', '-o', native]
    command(cxx)
    rust, rust_info = rust_build(TOOLS / 'Cargo.toml', output, 'gz-msgs-codec-benchmark')
    binaries = {'cpp': native, 'rust': rust}
    def portable(value):
        if isinstance(value, str):
            return value.replace(str(output), '$OUTPUT').replace(str(ROOT), '$REPOSITORY').replace(str(Path.home()), '$HOME')
        if isinstance(value, list):
            return [portable(item) for item in value]
        if isinstance(value, dict):
            return {key: portable(item) for key, item in value.items()}
        return value
    return binaries, {'protoc': portable(list(map(str, protoc))), 'cxx': portable(list(map(str, cxx))),
                      'cxx_sha256': sha(compiler), 'cxx_version': command([compiler, '--version']).stdout.decode().splitlines()[0],
                      'rust': portable(rust_info),
                      'generated': {name[:-6] + suffix: sha(generated / (name[:-6] + suffix))
                                    for name in schemas for suffix in ('.pb.cc', '.pb.h')}}


def invoke(binary, cpu, fixture, mode, count):
    output = command(['taskset', '-c', cpu, binary, 'run', fixture, mode, count])
    assert not output.stderr, output.stderr.decode(errors='replace')
    row = json.loads(output.stdout)
    assert set(row) == {'elapsed_ns', 'checksum', 'iterations', 'bytes'}
    assert all(type(value) is int for value in row.values())
    assert row['elapsed_ns'] > 0 and row['bytes'] > 0 and row['iterations'] == count
    return row


def validate_sample(row, count, unit, size):
    assert row['iterations'] == count and row['bytes'] == size
    assert row['checksum'] == (unit * count) & MASK, 'unequal or incomplete timed work'


def verify(binaries, cpu):
    fixtures, units = {}, {}
    for fixture in FIXTURES + ENVELOPES:
        wire = {}
        for name, binary in binaries.items():
            result = command(['taskset', '-c', cpu, binary, 'dump', fixture])
            assert not result.stderr
            wire[name] = result.stdout
        assert wire['cpp'] == wire['rust'], fixture
        fixtures[fixture] = {'bytes': len(wire['cpp']), 'sha256': hashlib.sha256(wire['cpp']).hexdigest(),
                             'wire_hex': wire['cpp'].hex()}
        for mode in MODES if fixture in FIXTURES else ('envelope',):
            single = {name: invoke(binary, cpu, fixture, mode, 1) for name, binary in binaries.items()}
            assert single['cpp']['checksum'] == single['rust']['checksum'], (fixture, mode)
            unit = single['cpp']['checksum']
            units[fixture, mode] = unit
            for name, binary in binaries.items():
                validate_sample(single[name], 1, unit, len(wire[name]))
                validate_sample(invoke(binary, cpu, fixture, mode, 3), 3, unit, len(wire[name]))
    return fixtures, units


def summarize(pairs):
    ratios = [pair['cpp']['elapsed_ns'] / pair['rust']['elapsed_ns'] for pair in pairs]
    rng = random.Random(73119)
    medians = sorted(statistics.median(rng.choices(ratios, k=len(ratios))) for _ in range(10000))
    return {'cpp_over_rust_paired_median': statistics.median(ratios),
            'paired_median_bootstrap_95_percent': [medians[249], medians[9749]],
            'ns_per_operation': {name: {'median': statistics.median(p[name]['elapsed_ns'] / p[name]['iterations'] for p in pairs),
                                      'min': min(p[name]['elapsed_ns'] / p[name]['iterations'] for p in pairs),
                                      'max': max(p[name]['elapsed_ns'] / p[name]['iterations'] for p in pairs)}
                                 for name in ('cpp', 'rust')}}


def measure(binaries, cpu, fixtures, units, repeats, sample_ms):
    rng = random.Random(73119)
    cases = list(units)
    counts, calibrations = {}, {}
    for fixture, mode in cases:
        size = fixtures[fixture]['bytes']
        count = max(16, min(10000, 1048576 // size))
        rows = {name: invoke(binary, cpu, fixture, mode, count) for name, binary in binaries.items()}
        for row in rows.values():
            validate_sample(row, count, units[fixture, mode], size)
        fastest = min(row['elapsed_ns'] / count for row in rows.values())
        counts[fixture, mode] = min(1000000000, max(16, math.ceil(sample_ms * 1000000 / fastest)))
        calibrations[fixture, mode] = rows
    raw = {case: [] for case in cases}
    # Every round visits every case; pair order and case order vary independently.
    for repeat in range(repeats):
        rng.shuffle(cases)
        for fixture, mode in cases:
            order = ['cpp', 'rust']
            rng.shuffle(order)
            pair = {'repeat': repeat, 'order': order, 'load_before': list(os.getloadavg())}
            for name in order:
                row = invoke(binaries[name], cpu, fixture, mode, counts[fixture, mode])
                validate_sample(row, counts[fixture, mode], units[fixture, mode], fixtures[fixture]['bytes'])
                pair[name] = row
            pair['load_after'] = list(os.getloadavg())
            raw[fixture, mode].append(pair)
    return [{'fixture': fixture, 'mode': mode, 'iterations': counts[fixture, mode],
             'calibration': calibrations[fixture, mode], 'samples': raw[fixture, mode],
             'summary': summarize(raw[fixture, mode])} for fixture, mode in sorted(raw)]


def libraries(binary):
    result = command(['ldd', binary]).stdout.decode()
    paths = re.findall(r'=> (/\S+)', result) + re.findall(r'^\s*(/\S+)', result, re.M)
    return {Path(path).resolve().name: sha(Path(path).resolve()) for path in paths}


def identity(binaries, cpu, build_info):
    cpuinfo = Path('/proc/cpuinfo').read_text()
    governor = Path(f'/sys/devices/system/cpu/cpu{cpu}/cpufreq/scaling_governor')
    paths = ['tools/codec_benchmark.py', 'tools/test_codec_benchmark.py', 'tools/codec-benchmark/native.cc', 'tools/codec-benchmark/release_checks.hh',
             'tools/codec-benchmark/src/main.rs', 'tools/codec-benchmark/Cargo.toml',
             'tools/codec-benchmark/Cargo.lock', 'Cargo.toml', 'Cargo.lock', 'build.rs',
             'rust-toolchain.toml', 'schemas/manifest.json', 'schemas/cli-info.json', 'vendor/manifest.json']
    paths += [str(p.relative_to(ROOT)) for p in sorted((ROOT / 'src').rglob('*.rs'))]
    return {'schema_commit': json.loads((ROOT / 'schemas/manifest.json').read_text())['commit'],
            'rustc': build_info['rust']['version'],
            'cxx': build_info['cxx_version'],
            'protoc': command(['protoc', '--version']).stdout.decode().strip(),
            'protobuf': command(['pkg-config', '--modversion', 'protobuf']).stdout.decode().strip(),
            'platform': platform.platform(), 'cpu_model': re.search(r'^model name\s*:\s*(.*)', cpuinfo, re.M).group(1),
            'logical_cpus': os.cpu_count(), 'allowed_affinity': sorted(os.sched_getaffinity(0)), 'pinned_cpu': cpu,
            'governor': governor.read_text().strip() if governor.exists() else None,
            'build': build_info, 'source_sha256': {p: sha(ROOT / p) for p in paths},
            'binaries': {name: {'sha256': sha(binary), 'libraries': libraries(binary)} for name, binary in binaries.items()}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT / ('artifacts/codec-benchmark-' + time.strftime('%Y%m%dT%H%M%S', time.gmtime()) + '-' + str(os.getpid())))
    parser.add_argument('--verify-only', action='store_true')
    parser.add_argument('--cpu', type=int, default=min(os.sched_getaffinity(0)))
    parser.add_argument('--samples', type=int, default=10)
    parser.add_argument('--sample-ms', type=float, default=50)
    args = parser.parse_args()
    assert args.cpu in os.sched_getaffinity(0)
    assert args.samples >= 10 and math.isfinite(args.sample_ms) and args.sample_ms >= 10
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    assert not (output / 'results.json').exists(), 'Choose a new output directory to preserve prior measurements'
    binaries, build_info = build(output)
    fixtures, units = verify(binaries, args.cpu)
    report = {'scope': 'Generated canonical known-field codecs; Time owned-byte envelopes are separate. No simulator or broad performance acceptance.',
              'utc_started': time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
              'warmup_iterations_per_process': 64, 'seed': 73119, 'samples': args.samples,
              'target_sample_ms': args.sample_ms, 'identity': identity(binaries, args.cpu, build_info),
              'fixtures': fixtures, 'checks': len(units), 'rows': []}
    if not args.verify_only:
        report['rows'] = measure(binaries, args.cpu, fixtures, units, args.samples, args.sample_ms)
    report['utc_finished'] = time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())
    (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(f'Passed {len(units)} cross-language work controls; measured {len(report["rows"])} cases.')


if __name__ == '__main__':
    main()

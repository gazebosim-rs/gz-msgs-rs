#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Failure and sampling controls; real codec fidelity is replayed separately."""
import io
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import codec_benchmark as bench


class EvidenceControls(unittest.TestCase):
    def test_wrong_wire_stops_before_measurement(self):
        outputs = [SimpleNamespace(stdout=b'A', stderr=b''), SimpleNamespace(stdout=b'B', stderr=b'')]
        with patch.object(bench, 'command', side_effect=outputs), patch.object(bench, 'invoke') as invoke:
            with self.assertRaises(AssertionError):
                bench.verify({'cpp': 'cpp', 'rust': 'rust'}, 0)
            invoke.assert_not_called()

    def test_incomplete_or_unequal_work_is_rejected(self):
        good = {'elapsed_ns': 3, 'iterations': 3, 'checksum': 21, 'bytes': 10}
        bench.validate_sample(good, 3, 7, 10)
        for key, value in [('iterations', 2), ('checksum', 20), ('bytes', 9)]:
            with self.assertRaises(AssertionError):
                bench.validate_sample({**good, key: value}, 3, 7, 10)
        wrapped = {**good, 'checksum': ((1 << 64) - 1) * 3 & bench.MASK}
        bench.validate_sample(wrapped, 3, bench.MASK, 10)

    def test_every_round_keeps_both_runtimes_and_equal_counts(self):
        units = {('clock', 'encode_fresh'): 7, ('imu', 'decode_reuse'): 11}
        fixtures = {'clock': {'bytes': 60}, 'imu': {'bytes': 341}}
        def run(binary, cpu, fixture, mode, count):
            return {'elapsed_ns': count * (20 if binary == 'cpp' else 10),
                    'iterations': count, 'checksum': units[fixture, mode] * count & bench.MASK,
                    'bytes': fixtures[fixture]['bytes']}
        with patch.object(bench, 'invoke', side_effect=run):
            rows = bench.measure({'cpp': 'cpp', 'rust': 'rust'}, 0, fixtures, units, 10, 10)
        self.assertEqual(len(rows), 2)
        for row in rows:
            self.assertEqual([p['repeat'] for p in row['samples']], list(range(10)))
            self.assertEqual({tuple(p['order']) for p in row['samples']}, {('cpp', 'rust'), ('rust', 'cpp')})
            for pair in row['samples']:
                self.assertEqual(pair['cpp']['iterations'], pair['rust']['iterations'])
            self.assertEqual(row['summary']['cpp_over_rust_paired_median'], 2)
            self.assertEqual(row['summary']['paired_median_bootstrap_95_percent'], [2, 2])

    def test_bootstrap_keeps_real_paired_spread(self):
        rows = [{'cpp': {'elapsed_ns': v, 'iterations': 1},
                 'rust': {'elapsed_ns': 10, 'iterations': 1}} for v in [10, 20, 30]]
        result = bench.summarize(rows)
        self.assertEqual(result['cpp_over_rust_paired_median'], 2)
        self.assertEqual(result['paired_median_bootstrap_95_percent'], [1, 3])
        self.assertEqual(result['ns_per_operation']['cpp'], {'median': 20, 'min': 10, 'max': 30})

    def test_failed_command_retains_output_and_original_exit(self):
        captured = io.BytesIO()
        with patch.object(sys, 'stderr', SimpleNamespace(buffer=captured)):
            with self.assertRaises(subprocess.CalledProcessError) as error:
                bench.command([sys.executable, '-c', 'import sys; print("out"); print("err", file=sys.stderr); sys.exit(37)'])
        self.assertEqual(error.exception.returncode, 37)
        self.assertEqual(captured.getvalue(), b'out\nerr\n')

    def package(self, root):
        (root / 'Cargo.toml').write_text('[package]\nname="codec-build-control"\nversion="0.0.0"\nedition="2021"\n')
        (root / 'Cargo.lock').write_text('version=4\n[[package]]\nname="codec-build-control"\nversion="0.0.0"\n')
        (root / 'src').mkdir()
        (root / 'src/main.rs').write_text('fn main() { println!("{}", cfg!(debug_assertions)); }')
        (root / '.cargo').mkdir()

    def test_actual_cargo_ignores_wrapper_compiler_and_flag_overrides(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.package(root)
            wrapper = root / 'override.sh'
            marker = root / 'unexpected-wrapper'
            wrapper.write_text('#!/bin/sh\nprintf used > "' + str(marker) + '"\nexec "$@" -Copt-level=0\n')
            wrapper.chmod(0o755)
            (root / '.cargo/config.toml').write_text(
                '[build]\nrustc="' + str(wrapper) + '"\nrustc-wrapper="' + str(wrapper) +
                '"\nrustc-workspace-wrapper="' + str(wrapper) + '"\nrustflags=["-Copt-level=0"]\n'
                '[profile.release]\nopt-level=0\ndebug-assertions=true\noverflow-checks=true\ncodegen-units=1\n')
            overrides = {key: str(wrapper) for key in ('RUSTC', 'CARGO_BUILD_RUSTC', 'RUSTC_WRAPPER',
                         'RUSTC_WORKSPACE_WRAPPER', 'CARGO_BUILD_RUSTC_WRAPPER', 'CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER')}
            overrides.update({'RUSTFLAGS': '-Copt-level=0', 'CARGO_ENCODED_RUSTFLAGS': '-Copt-level=0',
                              'CARGO_INCREMENTAL': '1', 'CARGO_PROFILE_RELEASE_OPT_LEVEL': '0'})
            with patch.dict(os.environ, overrides):
                binary, info = bench.rust_build(root / 'Cargo.toml', root, 'codec-build-control')
            self.assertFalse(marker.exists())
            self.assertEqual(bench.command([binary]).stdout, b'false\n')
            runtime = [unit for unit in info['invocations'] if unit['runtime']]
            self.assertEqual(len(runtime), 1)
            self.assertEqual(runtime[0]['codegen']['opt-level'], '3')
            self.assertNotIn('incremental', runtime[0]['codegen'])

    def test_actual_package_profile_override_is_rejected_before_timing(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.package(root)
            (root / '.cargo/config.toml').write_text('[profile.release.package.codec-build-control]\nopt-level=0\n')
            with self.assertRaisesRegex(AssertionError, 'Runtime compiler is not at opt-level 3'):
                bench.rust_build(root / 'Cargo.toml', root, 'codec-build-control')
            self.assertFalse((root / 'rust').exists())

    def test_actual_native_release_guard_rejects_unoptimized_or_debug_builds(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / 'main.cc'
            source.write_text('#include "release_checks.hh"\nint main() {}\n')
            cxx = [str(Path(shutil.which('c++')).resolve()), '-I' + str(bench.TOOLS), str(source), '-o', str(root / 'native')]
            for flags in [('-O0', '-DNDEBUG'), ('-O3',)]:
                result = subprocess.run([*cxx, *flags], capture_output=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(b'optimized release compiler invocation', result.stderr)
            subprocess.run([*cxx, '-O3', '-DNDEBUG'], check=True, capture_output=True)


if __name__ == '__main__':
    unittest.main()

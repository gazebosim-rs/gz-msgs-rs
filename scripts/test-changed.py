#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Exercise changed-source thresholds, exact inputs and real Git baselines."""
import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from changed import added_lines, check_changed, make_manifest, run_checks


class ChangedCoverage(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / 'src').mkdir()
        (self.root / 'build.rs').write_text('fn main() {}\n')
        (self.root / 'src/lib.rs').write_text('call();\n' * 100)
        self.manifest = make_manifest(self.root, self.root, None)

    def lcov(self, covered=100):
        return (f'SF:{self.root}/build.rs\nDA:1,1\n' +
                f'SF:{self.root}/src/lib.rs\n' +
                ''.join(f'DA:{n},{int(n <= covered)}\n' for n in range(1, 101)))

    def test_strict_threshold_and_no_borrowing_from_unchanged_lines(self):
        for covered, expected in [(94, False), (95, False), (96, True), (100, True)]:
            with self.subTest(covered=covered):
                if expected:
                    proof = check_changed(self.root, self.manifest, self.lcov(covered))
                    self.assertEqual(proof['files']['src/lib.rs']['lines'], [covered, 100])
                else:
                    with self.assertRaisesRegex(ValueError, 'must exceed 95%'):
                        check_changed(self.root, self.manifest, self.lcov(covered))
        # Overall file coverage is99%, but its sole newly added executable line misses.
        self.manifest['files']['src/lib.rs']['added_lines'] = [100]
        with self.assertRaisesRegex(ValueError, '0/1'):
            check_changed(self.root, self.manifest, self.lcov(99))

    def test_missing_source_empty_instrumentation_and_stale_hash_fail(self):
        for lcov in ['', f'SF:{self.root}/src/lib.rs\n', self.lcov().split('SF:')[1]]:
            with self.subTest(lcov=lcov[:40]), self.assertRaises(ValueError):
                check_changed(self.root, self.manifest, lcov)
        for mutation in ['missing', 'sha', 'negative', 'duplicate']:
            bad = copy.deepcopy(self.manifest)
            if mutation == 'missing':
                del bad['files']['src/lib.rs']
            elif mutation == 'sha':
                bad['files']['src/lib.rs']['sha256'] = 'wrong'
            else:
                bad['files']['src/lib.rs']['added_lines'] = [-1] if mutation == 'negative' else [1, 1]
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                check_changed(self.root, bad, self.lcov())
        (self.root / 'src/new.rs').write_text('pub fn new() {}\n')
        with self.assertRaisesRegex(ValueError, 'inventory mismatch'):
            check_changed(self.root, self.manifest, self.lcov())

    def test_duplicate_and_invalid_llvm_records_fail(self):
        bad = [self.lcov() + f'SF:{self.root}/src/lib.rs\nDA:1,1\n',
               self.lcov() + 'DA:100,1\n',
               self.lcov().replace('DA:1,1', 'DA:0,1', 1),
               self.lcov().replace('DA:1,1', 'DA:1,-1', 1)]
        for lcov in bad:
            with self.subTest(lcov=lcov[-40:]), self.assertRaises(ValueError):
                check_changed(self.root, self.manifest, lcov)

    def test_no_executable_change_is_explicit_not_a_coverage_pass(self):
        self.manifest['files']['src/lib.rs']['added_lines'] = []
        result = check_changed(self.root, self.manifest, self.lcov())
        self.assertEqual(result['files']['src/lib.rs']['state'], 'not_applicable_no_executable_additions')
        self.assertEqual(result['files']['src/lib.rs']['lines'], [0, 0])

    def test_real_git_parent_and_new_file_are_bound(self):
        def git(*args):
            return subprocess.check_output(['git', *args], cwd=self.root, stderr=subprocess.PIPE)
        git('init', '--quiet')
        git('add', '.')
        git('-c', 'user.name=Coverage test', '-c', 'user.email=coverage@example.invalid',
            'commit', '--quiet', '-m', 'base')
        parent = git('rev-parse', 'HEAD').decode().strip()
        (self.root / 'src/lib.rs').write_text('first();\n' + 'call();\n' * 99)
        (self.root / 'src/new.rs').write_text('pub fn new() {}\n')
        manifest = make_manifest(self.root, self.root, parent)
        self.assertEqual(manifest['base'], parent)
        self.assertEqual(manifest['files']['src/lib.rs']['added_lines'], [1])
        self.assertEqual(manifest['files']['src/new.rs']['added_lines'], [1])
        self.assertEqual(manifest['files']['build.rs']['added_lines'], [])
        with self.assertRaises(subprocess.CalledProcessError):
            make_manifest(self.root, self.root, 'missing-base')
        with self.assertRaisesRegex(ValueError, 'missing changed-source instrumentation: src/new.rs'):
            check_changed(self.root, manifest, self.lcov())
        self.assertEqual(added_lines('a\nb\n', 'a\n'), [])

    def fake_coverage(self, lcov, suffix=''):
        scripts = self.root / 'scripts'
        scripts.mkdir(exist_ok=True)
        module = str(Path(__file__).resolve().parent)
        code = f"""from pathlib import Path
import json,sys
sys.path.insert(0, {module!r})
from changed import check_changed
root=Path.cwd()
input=Path(sys.argv[sys.argv.index('--changed-lines') + 1])
data=input.read_bytes()
check_changed(root,json.loads(data),{lcov!r})
(root/'target').mkdir(exist_ok=True)
(root/'target/coverage-input.json').write_bytes(data)
{suffix}
"""
        (scripts / 'coverage.py').write_text(code)

    def test_real_tracked_target_collision_cannot_mask_new_miss(self):
        target = self.root / 'target/coverage-input.json'
        target.parent.mkdir()
        target.write_text('tracked collision')
        def git(*args):
            subprocess.check_call(['git', *args], cwd=self.root, stdout=subprocess.DEVNULL)
        git('init', '--quiet')
        git('add', '.')
        git('-c', 'user.name=Coverage test', '-c', 'user.email=coverage@example.invalid',
            'commit', '--quiet', '-m', 'tracked target collision')
        (self.root / 'src/lib.rs').write_text('call();\n' * 99 + 'missed();\n')
        manifest = make_manifest(self.root, self.root, 'HEAD')
        self.assertEqual(manifest['files']['src/lib.rs']['added_lines'], [100])
        masked = copy.deepcopy(manifest)
        for entry in masked['files'].values():
            entry['added_lines'] = []
        overwrite = [sys.executable, '-c',
                     "from pathlib import Path; Path('target/coverage-input.json').write_text(" +
                     repr(json.dumps(masked)) + ")"]
        self.fake_coverage(self.lcov(99))
        with self.assertRaises(subprocess.CalledProcessError):
            run_checks(self.root, manifest, [overwrite])
        # The same false input would pass if it were treated as authoritative.
        subprocess.check_call([sys.executable, 'scripts/coverage.py', '--changed-lines', str(target)],
                              cwd=self.root)
        self.fake_coverage(self.lcov(100))
        run_checks(self.root, manifest, [overwrite])
        self.assertEqual(json.loads(target.read_text()), manifest)

    def test_mutated_authoritative_input_or_source_fails_after_child_success(self):
        self.fake_coverage(self.lcov(), "input.write_text('{}')")
        with self.assertRaisesRegex(ValueError, 'authoritative changed-line input was modified'):
            run_checks(self.root, self.manifest, [])
        self.fake_coverage(self.lcov(), "(root/'src/lib.rs').write_text('modified')")
        with self.assertRaisesRegex(ValueError, 'source hash mismatch'):
            run_checks(self.root, self.manifest, [])


if __name__ == '__main__':
    unittest.main()

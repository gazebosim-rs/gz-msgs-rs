#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay the pinned native CLI option corpus without rewriting golden data."""
import argparse, hashlib, json, os, pathlib, subprocess
root = pathlib.Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('--fixture', type=pathlib.Path, default=root / 'tests/fixtures/cli-options.json')
parser.add_argument('--output', type=pathlib.Path)
args = parser.parse_args()
fixture_path = args.fixture if args.fixture.is_absolute() else root / args.fixture
fixture = json.loads(fixture_path.read_text())
reference = root.parent / 'gz-msgs'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=reference, text=True).strip()
assert revision == fixture['upstream_commit']
original = subprocess.check_output(['git', 'show', revision + ':core/cmd/msgs_main.cc'], cwd=reference)
assert hashlib.sha256(original).hexdigest() == fixture['native_source_sha256'] == sha(reference / 'core/cmd/msgs_main.cc')
binary = reference / 'build/bin/gz-msgs'
assert sha(binary) == fixture['native_binary_sha256']
assert sha(root / 'schemas/manifest.json') == fixture['schema_manifest_sha256']
env = dict(os.environ)
env.pop('GZ_DESCRIPTOR_PATH', None)
rows = fixture['cases'] if 'cases' in fixture else fixture['matching'] + fixture['formerly_unsupported_flag_assignments']
for row in rows:
    result = subprocess.run([fixture['native_argv0'], *row['args']], executable=str(binary), env=env, capture_output=True, text=True)
    assert result.returncode == row['status'] and result.stderr == row['stderr'], row['args']
    if row['mode'] == 'list':
        assert sorted(result.stdout.splitlines()) == sorted(row['stdout'].splitlines()), row['args']
    else:
        assert result.stdout == row['stdout'], row['args']
proof = dict(upstream_commit=revision, native_binary_sha256=sha(binary), native_source_sha256=sha(reference / 'core/cmd/msgs_main.cc'),
             schema_manifest_sha256=sha(root / 'schemas/manifest.json'), fixture_sha256=sha(fixture_path),
             matching_cases=len(rows), promoted_assignment_cases=len(fixture.get('formerly_unsupported_flag_assignments', [])),
             scope='Native statuses/stdout/stderr reproduced; unordered list compared as complete sorted sequence. Rust package version/help bindings and former unsupported assignments are promoted to native parity in Rust tests.')
output = args.output or root / 'artifacts' / (fixture_path.stem + '-reference.json')
output.parent.mkdir(parents=True, exist_ok=True)
output.write_text(json.dumps(proof, indent=2) + '\n')
print(f'{len(rows)} native option observations reproduced')

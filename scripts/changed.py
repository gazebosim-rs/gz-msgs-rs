# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Bind added production lines to the exact source sent to LLVM coverage."""
import difflib
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile


def production_files(root):
    paths = [root / 'build.rs', *(root / 'src').rglob('*.rs')]
    if not paths or any(not path.is_file() or path.is_symlink() for path in paths):
        raise ValueError('missing or linked production source')
    return {str(path.relative_to(root)): path for path in paths}


def added_lines(before, after):
    result = []
    for kind, _, _, start, end in difflib.SequenceMatcher(
            None, before.splitlines(), after.splitlines(), autojunk=False).get_opcodes():
        if kind in ('insert', 'replace'):
            result.extend(range(start + 1, end + 1))
    return result


def make_manifest(repository, source, base, tree=None):
    if base is not None:
        base = subprocess.check_output(
            ['git', 'rev-parse', '--verify', base + '^{commit}'],
            cwd=repository, text=True, stderr=subprocess.PIPE).strip()
    files = {}
    for name, path in production_files(source).items():
        old = b''
        if base is not None:
            result = subprocess.run(['git', 'show', base + ':' + name], cwd=repository,
                                    capture_output=True, check=False)
            if result.returncode == 0:
                old = result.stdout
            elif subprocess.run(['git', 'cat-file', '-e', base], cwd=repository,
                                capture_output=True, check=False).returncode:
                raise ValueError('missing coverage base')
        data = path.read_bytes()
        files[name] = {'sha256': hashlib.sha256(data).hexdigest(),
                       'added_lines': added_lines(old.decode(), data.decode())}
    return {'schema_version': 1, 'base': base, 'tree': tree, 'files': files}


def validate_manifest(root, manifest):
    files = production_files(root)
    if manifest.get('schema_version') != 1 or set(manifest.get('files', {})) != set(files):
        raise ValueError('changed coverage source inventory mismatch')
    for name, path in files.items():
        entry = manifest['files'][name]
        data = path.read_bytes()
        lines = entry.get('added_lines')
        if hashlib.sha256(data).hexdigest() != entry.get('sha256'):
            raise ValueError('changed coverage source hash mismatch: ' + name)
        if (not isinstance(lines, list) or any(type(n) is not int for n in lines)
                or lines != sorted(set(lines))
                or any(n < 1 or n > len(data.decode().splitlines()) for n in lines)):
            raise ValueError('invalid changed line inventory: ' + name)


def check_changed(root, manifest, lcov):
    validate_manifest(root, manifest)
    counts = {}
    current = None
    for line in lcov.splitlines():
        if line.startswith('SF:'):
            current = str(Path(line[3:]).resolve())
            if current in counts:
                raise ValueError('duplicate coverage source: ' + current)
            counts[current] = {}
        elif line.startswith('DA:'):
            if current is None:
                raise ValueError('coverage line has no source')
            number, count = map(int, line[3:].split(',')[:2])
            if number < 1 or count < 0 or number in counts[current]:
                raise ValueError('invalid or duplicate coverage line')
            counts[current][number] = count
    output = {}
    for name, entry in manifest['files'].items():
        filename = str((root / name).resolve())
        if filename not in counts or not counts[filename]:
            raise ValueError('missing changed-source instrumentation: ' + name)
        lines = sorted(set(entry['added_lines']) & counts[filename].keys())
        covered = sum(counts[filename][n] > 0 for n in lines)
        if lines and covered * 100 <= len(lines) * 95:
            raise ValueError(f'changed production lines must exceed 95%: {name}: {covered}/{len(lines)}')
        output[name] = {'state': 'passed' if lines else 'not_applicable_no_executable_additions',
                        'lines': [covered, len(lines)],
                        'uncovered': [n for n in lines if counts[filename][n] == 0],
                        'unmapped_added_lines': sorted(set(entry['added_lines']) - counts[filename].keys())}
    return {'base': manifest['base'], 'tree': manifest['tree'], 'files': output}


def run_checks(root, manifest, commands):
    """Keep the authoritative input outside build outputs and check its identity."""
    validate_manifest(root, manifest)
    encoded = (json.dumps(manifest, indent=2) + '\n').encode()
    with tempfile.TemporaryDirectory(prefix='gz-msgs-coverage-input-') as directory:
        authoritative = Path(directory) / 'changed-lines.json'
        authoritative.write_bytes(encoded)
        coverage = [sys.executable, 'scripts/coverage.py', '--changed-lines', str(authoritative)]
        for command in [*commands, coverage]:
            subprocess.run(command, cwd=root, check=True)
            if authoritative.read_bytes() != encoded:
                raise ValueError('authoritative changed-line input was modified')
            validate_manifest(root, manifest)
        if (root / 'target/coverage-input.json').read_bytes() != encoded:
            raise ValueError('retained changed-line input differs from authoritative source')

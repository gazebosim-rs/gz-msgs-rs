# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Verify exact immutable Git blobs without checkout-attribute substitutions."""
import hashlib
import os
from pathlib import Path
import subprocess
import sys
import tempfile


def tree(root):
    return subprocess.check_output(['git', 'write-tree'], cwd=root, text=True).strip()


def head(root):
    result = subprocess.run(['git', 'rev-parse', '--verify', 'HEAD'], cwd=root, capture_output=True, text=True)
    return result.stdout.strip() if result.returncode == 0 else None


def validate_snapshot(root, tree, candidate):
    algorithm = subprocess.check_output(['git', 'rev-parse', '--show-object-format'], cwd=root, text=True).strip()
    expected = set()
    for entry in subprocess.check_output(['git', 'ls-tree', '-rz', tree], cwd=root).split(b'\0'):
        if not entry:
            continue
        metadata, name = entry.split(b'\t', 1)
        mode, kind, oid = metadata.split()
        relative = os.fsdecode(name)
        expected.add(relative)
        path = candidate / relative
        if kind != b'blob':
            raise ValueError('Unsupported non-blob staged input')
        if mode == b'120000' and path.is_symlink():
            data = os.fsencode(os.readlink(path))
        elif mode in (b'100644', b'100755') and path.is_file() and not path.is_symlink():
            if bool(path.stat().st_mode & 0o100) != (mode == b'100755'):
                raise ValueError('Exported staged executable mode differs from Git tree')
            data = path.read_bytes()
        else:
            raise ValueError('Exported staged file type differs from Git tree')
        digest = hashlib.new(algorithm, f'blob {len(data)}\0'.encode() + data).hexdigest()
        if digest != oid.decode():
            raise ValueError(f'Exported staged blob differs from Git tree: {relative}')
    actual = {str(path.relative_to(candidate)) for path in candidate.rglob('*') if path.is_file() or path.is_symlink()}
    if actual != expected:
        raise ValueError('Exported staged paths differ from Git tree')


def verify_staged(root):
    frozen, original_head = tree(root), head(root)
    with tempfile.TemporaryDirectory(prefix='gz-msgs-staged-') as directory:
        candidate = Path(directory) / 'candidate'
        candidate.mkdir()
        # Private checkout includes export-ignore files; raw blob checks reject
        # EOL/smudge transformations before any exported build code can run.
        env = dict(os.environ, GIT_INDEX_FILE=str(Path(directory) / 'index'))
        subprocess.run(['git', 'read-tree', frozen], cwd=root, env=env, check=True)
        subprocess.run(['git', 'checkout-index', '--all', f'--prefix={candidate}/'], cwd=root, env=env, check=True)
        validate_snapshot(root, frozen, candidate)
        # Keep the hook's index context in the parent, never in foreign Git fixtures.
        child_env = dict(os.environ)
        for name in subprocess.check_output(
            ['git', 'rev-parse', '--local-env-vars'], cwd=root, text=True
        ).splitlines():
            child_env.pop(name, None)
        subprocess.run([sys.executable, 'scripts/verify.py'], cwd=candidate,
                       env=child_env, check=True)
        if tree(root) != frozen or head(root) != original_head:
            raise ValueError('Git index or HEAD changed during staged verification')

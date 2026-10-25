#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Check archive provenance or exercise a clean source distribution install."""
import argparse
import importlib.util
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import zlib
import sys

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('source_package', ROOT / 'scripts/source-package.py')
package = importlib.util.module_from_spec(spec)
spec.loader.exec_module(package)


class ArchiveTests(unittest.TestCase):
    def test_exact_git_blobs_modes_and_determinism(self):
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            repo = base / 'repo'
            repo.mkdir()
            package.git(repo, 'init', '-q')
            (repo / '.gitattributes').write_text('a export-ignore\n')
            (repo / 'a').write_bytes(b'raw\r\nbytes\n')
            (repo / 'exec').write_text('#!/bin/sh\nexit 0\n')
            (repo / 'exec').chmod(0o755)
            (repo / 'link').symlink_to('a')
            package.git(repo, 'add', '.')
            package.git(repo, '-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid', 'commit', '-qm', 'fixture')
            (repo / 'a').write_text('unstaged change')
            (repo / 'untracked').write_text('excluded')
            first = package.create(repo, 'HEAD', base / 'one')
            second = package.create(repo, 'HEAD', base / 'two')
            self.assertEqual(first.read_bytes(), second.read_bytes())
            source, manifest = package.extract(first, base / 'extracted')
            self.assertEqual((source / 'a').read_bytes(), b'raw\r\nbytes\n')
            self.assertEqual((source / 'exec').stat().st_mode & 0o777, 0o755)
            self.assertEqual(os.readlink(source / 'link'), 'a')
            self.assertFalse((source / 'untracked').exists())
            self.assertEqual(len(manifest['files']), 4)
            with patch.dict(os.environ, {'GIT_INDEX_FILE': '.git/index'}):
                package.extract(first, base / 'hook-relative-index')
            sentinel = base / 'foreign-index'
            sentinel.write_bytes(b'outside index must remain untouched')
            original_config = (repo / '.git/config').read_bytes()
            with patch.dict(os.environ, {'GIT_DIR': str(repo / '.git'), 'GIT_WORK_TREE': str(repo), 'GIT_INDEX_FILE': str(sentinel)}):
                package.extract(first, base / 'hook-foreign-context')
            self.assertEqual(sentinel.read_bytes(), b'outside index must remain untouched')
            self.assertEqual((repo / '.git/config').read_bytes(), original_config)
            with self.assertRaisesRegex(ValueError, 'expected commit'):
                package.extract(first, base / 'wrong-revision', '0' * len(manifest['revision']))
            # A self-consistent blob manifest still cannot relabel a different tree.
            changed = base / 'changed.tar.gz'
            with tarfile.open(first, 'r:gz') as original, tarfile.open(changed, 'w:gz') as tar:
                for member in original.getmembers():
                    data = original.extractfile(member).read() if member.isfile() else None
                    if member.name.endswith('/SOURCE_MANIFEST.json'):
                        value = json.loads(data)
                        value['files'].pop()
                        data = json.dumps(value).encode()
                        member.size = len(data)
                    elif member.name.endswith('/link'):
                        continue
                    tar.addfile(member, io.BytesIO(data) if data is not None else None)
            with self.assertRaisesRegex(ValueError, 'committed tree'):
                package.extract(changed, base / 'rejected')
            for change in ('blob', 'mode', 'extra'):
                bad = base / (change + '.tar.gz')
                with tarfile.open(first, 'r:gz') as original, tarfile.open(bad, 'w:gz') as tar:
                    for member in original.getmembers():
                        data = original.extractfile(member).read() if member.isfile() else None
                        if change == 'blob' and member.name.endswith('/a'):
                            data += b'changed'
                            member.size = len(data)
                        if change == 'mode' and member.name.endswith('/exec'):
                            member.mode = 0o644
                        tar.addfile(member, io.BytesIO(data) if data is not None else None)
                    if change == 'extra':
                        tar.addfile(tarfile.TarInfo('extra'))
                with self.assertRaises(ValueError):
                    package.extract(bad, base / ('reject-' + change))
            for path in ('./injected.rs', './a', '.git/config', 'dir//injected.rs', 'dir/./injected.rs', 'dir/../injected.rs'):
                forged = base / 'forged.tar.gz'
                data = b'not in trusted commit\n'
                oid = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
                with tarfile.open(first, 'r:gz') as original, tarfile.open(forged, 'w:gz') as tar:
                    for member in original.getmembers():
                        content = original.extractfile(member).read() if member.isfile() else None
                        if member.name.endswith('/SOURCE_MANIFEST.json'):
                            value = json.loads(content)
                            value['files'].append(dict(path=path, mode='100644', oid=oid, sha256=hashlib.sha256(data).hexdigest()))
                            content = json.dumps(value).encode()
                            member.size = len(content)
                        tar.addfile(member, io.BytesIO(content) if content is not None else None)
                    member = tarfile.TarInfo('gz-msgs-' + manifest['revision'] + '/' + path)
                    member.size = len(data)
                    member.mode = 0o644
                    tar.addfile(member, io.BytesIO(data))
                with self.assertRaises(ValueError):
                    package.extract(forged, base / 'reject-forgery', manifest['revision'])
            # A Git-authentic tree still cannot escape its extraction root.
            (repo / 'link').unlink()
            (repo / 'link').symlink_to('../../outside')
            package.git(repo, 'add', 'link')
            package.git(repo, '-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid', 'commit', '-qm', 'escaping link')
            unsafe = package.create(repo, 'HEAD', base / 'unsafe')
            with self.assertRaises(tarfile.FilterError):
                package.extract(unsafe, base / 'reject-link')
            (repo / 'link').unlink()
            (repo / 'link').symlink_to('../outside')
            package.git(repo, 'add', 'link')
            package.git(repo, '-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid', 'commit', '-qm', 'off-prefix link')
            outside = package.create(repo, 'HEAD', base / 'off-prefix')
            with self.assertRaisesRegex(ValueError, 'revision directory'):
                package.extract(outside, base / 'reject-prefix')



def installed(archive, expected_revision, fresh_cache=False):
    out = ROOT / 'artifacts/source-package-check'
    out.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='source-', dir=out) as directory:
        source, manifest = package.extract(archive, Path(directory), expected_revision)
        prefix = Path(directory) / 'install'
        env = dict(os.environ, CARGO_TARGET_DIR=str(out / 'build'))
        if fresh_cache:
            env['CARGO_HOME'] = str(Path(directory) / 'cargo-home')
            subprocess.run(['cargo', 'fetch', '--locked', '--manifest-path', str(source / 'Cargo.toml')], cwd=source, env=env, check=True)
        subprocess.run(['cargo', 'install', '--offline', '--locked', '--path', str(source), '--root', str(prefix)], cwd=source, env=env, check=True)
        binary = prefix / 'bin/gz-msgs'
        linked = subprocess.check_output(['ldd', str(binary)], text=True)
        assert 'libprotobuf' not in linked and 'libgz-' not in linked, linked
        rows = json.loads((source / 'tests/fixtures/cli-errors-reference.json').read_text())['rows']
        for row in rows:
            run = subprocess.run([str(binary), *row['args']], capture_output=True, text=True, check=True)
            assert (run.returncode, run.stdout, run.stderr) == (row['exit_code'], row['stdout'], row['stderr'])
        subprocess.run(['python3', 'scripts/test-downstream.py'], cwd=source, env=env, check=True)
        (out / 'proof.json').write_text(json.dumps(dict(revision=manifest['revision'], expected_revision=expected_revision, tree=manifest['tree'], archive_sha256=hashlib.sha256(archive.read_bytes()).hexdigest(), source_files=len(manifest['files']), installed_cli_observations=len(rows), fresh_cargo_home=fresh_cache, rustc=subprocess.check_output(['rustc', '-vV'], cwd=source, env=env, text=True), protoc=subprocess.check_output(['protoc', '--version'], text=True).strip(), python=sys.version.split()[0], zlib=zlib.ZLIB_RUNTIME_VERSION, runtime_libraries=sorted(Path(line.split()[0]).name for line in linked.splitlines() if line.strip()), downstream='signed zero/Any/nested unknown forwarding passed'), indent=2) + '\n')
        print('Clean archive install and downstream consumer pass for ' + manifest['revision'])


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path)
    parser.add_argument('--revision')
    parser.add_argument('--fresh-cache', action='store_true')
    args = parser.parse_args()
    if args.archive:
        if not args.revision:
            parser.error('--revision must name the trusted full Git commit')
        installed(args.archive.resolve(), args.revision, args.fresh_cache)
    else:
        unittest.main(argv=['source-package-tests'])

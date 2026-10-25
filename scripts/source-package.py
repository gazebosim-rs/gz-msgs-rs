#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Build a deterministic source distribution from immutable Git blobs."""
import argparse
import base64
import gzip
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import tarfile
import tempfile


LOCAL_GIT_ENV = subprocess.check_output(['git', 'rev-parse', '--local-env-vars'], text=True).splitlines()

def git(root, *args, **kwargs):
    # Hooks export repository-local variables; never apply them to foreign Git dirs.
    env = dict(os.environ)
    for name in LOCAL_GIT_ENV:
        env.pop(name, None)
    return subprocess.check_output(['git', *args], cwd=root, env=env, **kwargs)


def create(root, revision, output):
    revision = git(root, 'rev-parse', '--verify', '--end-of-options', revision + '^{commit}').decode().strip()
    tree = git(root, 'rev-parse', revision + '^{tree}').decode().strip()
    algorithm = git(root, 'rev-parse', '--show-object-format').decode().strip()
    prefix = 'gz-msgs-' + revision
    output.mkdir(parents=True, exist_ok=True)
    archive = output / (prefix + '.tar.gz')
    records = []
    with archive.open('wb') as destination, gzip.GzipFile(filename='', mode='wb', fileobj=destination, mtime=0) as compressed, tarfile.open(fileobj=compressed, mode='w|', format=tarfile.PAX_FORMAT) as tar:
        for entry in git(root, 'ls-tree', '-rz', tree).split(b'\0'):
            if not entry:
                continue
            metadata, raw_path = entry.split(b'\t', 1)
            mode, kind, oid = metadata.decode().split()
            path = os.fsdecode(raw_path)
            if kind != 'blob' or mode not in ('100644', '100755', '120000') or path == 'SOURCE_MANIFEST.json':
                raise ValueError('Unsupported source input: ' + path)
            data = git(root, 'cat-file', 'blob', oid)
            info = tarfile.TarInfo(prefix + '/' + path)
            info.mode = 0o755 if mode == '100755' else 0o644
            if mode == '120000':
                info.type = tarfile.SYMTYPE
                info.linkname = os.fsdecode(data)
                tar.addfile(info)
            else:
                info.size = len(data)
                tar.addfile(info, io.BytesIO(data))
            records.append(dict(path=path, mode=mode, oid=oid, sha256=hashlib.sha256(data).hexdigest()))
        manifest = dict(schema_version=1, revision=revision, tree=tree, commit_base64=base64.b64encode(git(root, 'cat-file', 'commit', revision)).decode(), object_format=algorithm, files=records)
        data = (json.dumps(manifest, indent=2, sort_keys=True) + '\n').encode()
        info = tarfile.TarInfo(prefix + '/SOURCE_MANIFEST.json')
        info.mode = 0o644
        info.size = len(data)
        tar.addfile(info, io.BytesIO(data))
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    archive.with_suffix(archive.suffix + '.sha256').write_text(digest + '  ' + archive.name + '\n')
    return archive


def extract(archive, output, expected_revision=None):
    with tarfile.open(archive, 'r:gz') as tar:
        manifests = [m for m in tar.getmembers() if m.name.endswith('/SOURCE_MANIFEST.json')]
        if len(manifests) != 1 or not manifests[0].isfile():
            raise ValueError('Missing or ambiguous source manifest')
        manifest = json.load(tar.extractfile(manifests[0]))
        if expected_revision is not None and manifest['revision'] != expected_revision:
            raise ValueError('Archive revision differs from expected commit')
        algorithm = manifest['object_format']
        if algorithm not in ('sha1', 'sha256') or not re.fullmatch('[0-9a-f]{' + str(hashlib.new(algorithm).digest_size * 2) + '}', manifest['revision']):
            raise ValueError('Invalid Git identity')
        commit = base64.b64decode(manifest['commit_base64'], validate=True)
        commit_oid = hashlib.new(algorithm, b'commit ' + str(len(commit)).encode() + b'\0' + commit).hexdigest()
        if commit_oid != manifest['revision'] or commit.splitlines()[0] != b'tree ' + manifest['tree'].encode():
            raise ValueError('Commit does not bind source tree')
        prefix = 'gz-msgs-' + manifest['revision']
        records = {r['path']: r for r in manifest['files']}
        expected = {prefix + '/' + p for p in records} | {prefix + '/SOURCE_MANIFEST.json'}
        members = tar.getmembers()
        if len(records) != len(manifest['files']) or len(members) != len(expected) or {m.name for m in members} != expected:
            raise ValueError('Archive path set differs from source manifest')
        blobs = []
        for path, record in records.items():
            if not path or '\0' in path or PurePosixPath(path).is_absolute() or any(p in ('', '.', '..') for p in path.split('/')):
                raise ValueError('Noncanonical source path')
            member = tar.getmember(prefix + '/' + path)
            mode = record['mode']
            if mode == '120000' and member.issym():
                data = os.fsencode(member.linkname)
            elif mode in ('100644', '100755') and member.isfile() and member.mode == (0o755 if mode == '100755' else 0o644):
                data = tar.extractfile(member).read()
            else:
                raise ValueError('Source file type or mode differs')
            oid = hashlib.new(algorithm, b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
            if oid != record['oid'] or hashlib.sha256(data).hexdigest() != record['sha256']:
                raise ValueError('Source blob differs: ' + path)
            blobs.append(data)
        # Delegate Git tree construction to Git, including directory ordering/modes.
        with tempfile.TemporaryDirectory(prefix='gz-msgs-tree-') as directory:
            repository = Path(directory)
            git(repository, 'init', '--bare', '-q', '--object-format=' + algorithm)
            for data in blobs:
                git(repository, 'hash-object', '-w', '--stdin', input=data)
            entries = b''.join(r['mode'].encode() + b' ' + r['oid'].encode() + b'\t' + os.fsencode(path) + b'\0' for path, r in records.items())
            git(repository, 'update-index', '-z', '--index-info', input=entries)
            indexed = set()
            for entry in git(repository, 'ls-files', '--stage', '-z').split(b'\0'):
                if entry:
                    metadata, path = entry.split(b'\t', 1)
                    mode, oid, stage = metadata.split()
                    indexed.add((mode, oid, stage, path))
            expected_index = {(r['mode'].encode(), r['oid'].encode(), b'0', os.fsencode(path)) for path, r in records.items()}
            if indexed != expected_index:
                raise ValueError('Git index differs from source manifest')
            if git(repository, 'write-tree').decode().strip() != manifest['tree']:
                raise ValueError('Source files do not match committed tree')
        tar.extractall(output, filter='data')
    source = output / prefix
    for record in records.values():
        if record['mode'] == '120000' and not (source / record['path']).resolve().is_relative_to(source.resolve()):
            raise ValueError('Source link escapes revision directory')
    return source, manifest


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--revision', default='HEAD')
    parser.add_argument('--output', type=Path, default=Path('artifacts/distribution'))
    args = parser.parse_args()
    print(create(Path(__file__).resolve().parents[1], args.revision, args.output))

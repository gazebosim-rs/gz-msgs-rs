#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Fail closed for transformed/omitted snapshots and concurrent Git mutations."""
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
import snapshot


class Snapshot(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='gz-msgs-snapshot-test-')
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name) / 'repo'
        (self.root / 'scripts').mkdir(parents=True)
        (self.root / 'src').mkdir()
        self.source = self.root / 'src/lib.rs'
        self.source.write_bytes(b'staged\n')
        self.git('init', '--quiet')

    def git(self, *args):
        return subprocess.check_output(['git', *args], cwd=self.root, stderr=subprocess.PIPE)

    def verifier(self, code):
        (self.root / 'scripts/verify.py').write_text(code)

    def test_exact_staged_bytes_include_export_ignore(self):
        (self.root / '.gitattributes').write_text('src/lib.rs export-ignore\n')
        self.verifier(f"from pathlib import Path\nassert Path('src/lib.rs').read_bytes()==b'staged\\n'\nPath({str(self.root / 'proof')!r}).write_text('checked')\n")
        self.git('add', '.')
        self.source.write_text('unstaged fix')
        snapshot.verify_staged(self.root)
        self.assertEqual((self.root / 'proof').read_text(), 'checked')
        self.assertEqual(self.source.read_text(), 'unstaged fix')

    def test_checkout_eol_transformation_is_rejected_before_verifier(self):
        (self.root / '.gitattributes').write_text('src/lib.rs text eol=crlf\n')
        self.verifier(f"from pathlib import Path\nPath({str(self.root / 'unchecked')!r}).touch()\n")
        self.git('add', '.')
        with self.assertRaisesRegex(ValueError, 'blob differs'):
            snapshot.verify_staged(self.root)
        self.assertFalse((self.root / 'unchecked').exists())

    def test_raw_blob_type_mode_and_path_identity(self):
        self.git('add', '.')
        frozen = snapshot.tree(self.root)
        candidate = Path(self.temporary.name) / 'candidate'
        shutil.copytree(self.root, candidate, ignore=shutil.ignore_patterns('.git'))
        snapshot.validate_snapshot(self.root, frozen, candidate)
        exported = candidate / 'src/lib.rs'
        exported.write_text('changed')
        with self.assertRaisesRegex(ValueError, 'blob differs'):
            snapshot.validate_snapshot(self.root, frozen, candidate)
        exported.write_bytes(self.source.read_bytes())
        extra = candidate / 'extra.rs'
        extra.touch()
        with self.assertRaisesRegex(ValueError, 'paths differ'):
            snapshot.validate_snapshot(self.root, frozen, candidate)
        extra.unlink()
        exported.chmod(0o755)
        with self.assertRaisesRegex(ValueError, 'executable mode differs'):
            snapshot.validate_snapshot(self.root, frozen, candidate)
        self.source.chmod(0o755)
        self.git('add', '.')
        executable_tree = snapshot.tree(self.root)
        exported.chmod(0o645)
        with self.assertRaisesRegex(ValueError, 'executable mode differs'):
            snapshot.validate_snapshot(self.root, executable_tree, candidate)
        exported.unlink()
        exported.symlink_to(self.source)
        with self.assertRaisesRegex(ValueError, 'file type differs'):
            snapshot.validate_snapshot(self.root, frozen, candidate)

    def test_symlink_blob_identity_and_unsupported_gitlinks(self):
        link = self.root / 'link'
        link.symlink_to('src/lib.rs')
        self.git('add', '.')
        frozen = snapshot.tree(self.root)
        candidate = Path(self.temporary.name) / 'symlinks'
        shutil.copytree(self.root, candidate, symlinks=True, ignore=shutil.ignore_patterns('.git'))
        snapshot.validate_snapshot(self.root, frozen, candidate)
        (candidate / 'link').unlink()
        (candidate / 'link').symlink_to('wrong')
        with self.assertRaisesRegex(ValueError, 'blob differs'):
            snapshot.validate_snapshot(self.root, frozen, candidate)
        self.git('-c', 'user.name=Gate test', '-c', 'user.email=gate@example.invalid', 'commit', '--quiet', '-m', 'fixture')
        commit = snapshot.head(self.root)
        self.git('update-index', '--add', '--cacheinfo', f'160000,{commit},submodule')
        with self.assertRaisesRegex(ValueError, 'non-blob'):
            snapshot.validate_snapshot(self.root, snapshot.tree(self.root), self.root)

    def test_concurrent_index_or_head_change_is_rejected(self):
        for mutation in ['index', 'head']:
            with self.subTest(mutation=mutation):
                action = (
                    "(root/'src/lib.rs').write_text('unchecked')\nsubprocess.run(['git','add','src/lib.rs'],cwd=root,check=True)"
                    if mutation == 'index' else
                    "subprocess.run(['git','-c','user.name=Gate test','-c','user.email=gate@example.invalid','commit','--quiet','-m','concurrent HEAD'],cwd=root,check=True)"
                )
                self.verifier(f"from pathlib import Path\nimport subprocess\nroot=Path({str(self.root)!r})\n{action}\n")
                self.git('add', '.')
                with self.assertRaisesRegex(ValueError, 'Git index or HEAD changed'):
                    snapshot.verify_staged(self.root)


if __name__ == '__main__':
    unittest.main()

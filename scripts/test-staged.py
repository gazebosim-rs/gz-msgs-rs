#!/usr/bin/env python3
"""Demonstrate that unstaged working-copy fixes cannot validate a broken index."""
import os, pathlib, shutil, subprocess, tempfile
source = pathlib.Path(__file__).resolve().parents[1]
with tempfile.TemporaryDirectory(prefix='gz-msgs-partial-stage-') as directory:
    root = pathlib.Path(directory)
    for path in source.iterdir():
        if path.name in ['.git', 'target', 'artifacts', '.cache']: continue
        if path.is_dir(): shutil.copytree(path, root / path.name, ignore=shutil.ignore_patterns('__pycache__'))
        else: shutil.copy2(path, root / path.name)
    def git(*args): subprocess.run(['git', *args], cwd=root, check=True, stdout=subprocess.DEVNULL)
    git('init', '-q')
    original = (root / 'src/lib.rs').read_text()
    broken = original.replace('(nanos % 1_000_000_000)', '(nanos / 1_000_000_000)')
    assert broken != original
    (root / 'src/lib.rs').write_text(broken)
    git('add', '.')
    (root / 'src/lib.rs').write_text(original)
    result = subprocess.run(['python3', 'scripts/verify.py', '--staged'], cwd=root, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    if result.returncode == 0 or 'time_contract ... FAILED' not in result.stdout:
        print(result.stdout)
        raise SystemExit('partial staging gate did not reject staged broken conversion for expected reason')
    print('Partial staging: broken index rejected by time_contract despite fixed working file')

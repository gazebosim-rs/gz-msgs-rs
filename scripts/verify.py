#!/usr/bin/env python3
import hashlib, json, os, pathlib, subprocess, sys, tempfile
root = pathlib.Path(__file__).resolve().parents[1]
if '--staged' in sys.argv:
    with tempfile.TemporaryDirectory(prefix='gz-msgs-staged-') as directory:
        archive = subprocess.Popen(['git', 'archive', subprocess.check_output(['git', 'write-tree'], text=True).strip()], cwd=root, stdout=subprocess.PIPE)
        subprocess.run(['tar', '-x', '-C', directory], stdin=archive.stdout, check=True)
        if archive.wait(): raise SystemExit(1)
        subprocess.run(['python3', directory + '/scripts/verify.py'], cwd=directory, check=True)
    raise SystemExit()
os.chdir(root)
manifest = json.loads(pathlib.Path('schemas/manifest.json').read_text())
assert {p['path'] for p in manifest['schemas']} == {str(p) for p in pathlib.Path('schemas').rglob('*.proto')}
for item in manifest['schemas']:
    assert hashlib.sha256(pathlib.Path(item['path']).read_bytes()).hexdigest() == item['sha256'], item['path']
for command in [[os.environ.get('CARGO', 'cargo'), 'fmt', '--all', '--', '--check'], [os.environ.get('CARGO', 'cargo'), 'clippy', '--locked', '--all-targets', '--', '-D', 'warnings'], ['python3', 'scripts/test-gates.py'], ['python3', 'scripts/coverage.py']]:
    subprocess.run(command, check=True)

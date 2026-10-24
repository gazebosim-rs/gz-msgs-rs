#!/usr/bin/env python3
"""Enforce nonempty >95% line and region coverage for every production source."""
import json, pathlib, sys

def check(report, root):
    wanted = {str(p.resolve()) for p in (root / 'src').rglob('*.rs')} | {str((root / 'build.rs').resolve())}
    files = {f['filename']: f for data in report['data'] for f in data['files'] if f['filename'] in wanted}
    if not wanted or set(files) != wanted:
        raise ValueError('missing production source or instrumentation')
    for name, file in files.items():
        for metric in ['lines', 'regions']:
            counts = file['summary'][metric]
            if counts['count'] == 0 or counts['covered'] * 100 <= counts['count'] * 95:
                raise ValueError(f'{name}: {metric} must exceed 95%: {counts}')
    return files

if __name__ == '__main__':
    check(json.loads(pathlib.Path(sys.argv[1]).read_text()), pathlib.Path(__file__).resolve().parents[1])

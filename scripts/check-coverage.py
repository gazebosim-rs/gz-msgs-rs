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



def check_patches(report, root, lcov):
    """Only unchanged vendored lines are outside ownership; count LLVM's executable patch lines."""
    counts = {}
    current = None
    for line in lcov.splitlines():
        if line.startswith('SF:'):
            current = line[3:]
            counts[current] = {}
        elif line.startswith('DA:'):
            number, count = line[3:].split(',')[:2]
            counts[current][int(number)] = int(count)
    results = {}
    for package in json.loads((root / 'vendor/manifest.json').read_text())['packages']:
        owned_lines = covered_lines = owned_regions = covered_regions = 0
        for item in package['files']:
            if 'owned_lines' not in item:
                continue
            filename = str((root / item['path']).resolve())
            if filename not in counts:
                raise ValueError('missing owned dependency instrumentation: ' + filename)
            changed = set(item['owned_lines'])
            # LLVM does not map closing delimiters or quoted proc-macro token text
            # as executable lines; no executable changed line may disappear.
            source = (root / item['path']).read_text().splitlines()
            for number in changed:
                if number not in counts[filename]:
                    if source[number - 1].strip() not in ['}', '};', '', 'if #non_default {']:
                        raise ValueError(f'uninstrumented changed line: {filename}:{number}')
                    continue
                owned_lines += 1
                covered_lines += counts[filename][number] > 0
            regions = [region for data in report['data'] for function in data['functions'] for region in function['regions']
                       if function['filenames'][region[5]] == filename and region[0] in changed and region[7] == 0]
            owned_regions += len(regions)
            covered_regions += sum(region[4] > 0 for region in regions)
        for name, covered, total in [('lines', covered_lines, owned_lines), ('regions', covered_regions, owned_regions)]:
            if total == 0 or covered * 100 <= total * 95:
                raise ValueError(f"{package['name']} owned patch {name}: {covered}/{total}")
        results[package['name']] = {'lines': [covered_lines, owned_lines], 'regions': [covered_regions, owned_regions]}
    return results

if __name__ == '__main__':
    root = pathlib.Path(__file__).resolve().parents[1]
    report = json.loads(pathlib.Path(sys.argv[1]).read_text())
    check(report, root)
    patches = check_patches(report, root, pathlib.Path(sys.argv[1]).with_suffix('.lcov').read_text())
    pathlib.Path(sys.argv[1]).with_name('owned-patches.json').write_text(json.dumps(patches, indent=2) + '\n')
    print(json.dumps(patches))

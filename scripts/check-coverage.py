#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Enforce nonempty >95% line and region coverage for every production source."""
import hashlib, json, pathlib, re, sys

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



def added_patch_lines(path, filename):
    current, number, added = None, 0, []
    old_left = new_left = 0
    for line in path.read_text().splitlines():
        if old_left or new_left:
            # Inside a counted hunk, +++ is added source text, not a header.
            if line.startswith('+'):
                if filename == current or filename.endswith('/' + str(current)):
                    added.append(number)
                number += 1
                new_left -= 1
            elif line.startswith('-'):
                old_left -= 1
            elif line.startswith(' '):
                number += 1
                old_left -= 1
                new_left -= 1
            elif line != r'\ No newline at end of file':
                raise ValueError('invalid patch hunk data')
            if old_left < 0 or new_left < 0:
                raise ValueError('patch hunk exceeds declared length')
        elif line.startswith('+++ '):
            current = line[4:]
        elif line.startswith('--- ') or line.startswith('diff --git '):
            current = None
        elif line.startswith('@@ '):
            header = re.match(r'@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@', line)
            if header is None or current is None:
                raise ValueError('invalid patch hunk header')
            _, old_count, start, new_count = header.groups()
            old_left, new_left = int(old_count or 1), int(new_count or 1)
            number = int(start)
        elif line.startswith(('+', '-', ' ')):
            raise ValueError('patch data outside declared hunk')
    if old_left or new_left:
        raise ValueError('truncated patch hunk')
    return added


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
            data = (root / item['path']).read_bytes()
            if hashlib.sha256(data).hexdigest() != item.get('sha256'):
                raise ValueError('owned source hash mismatch: ' + filename)
            source = data.decode().splitlines()
            changed = set(item['owned_lines'])
            if 'unmapped_syntax' in item or 'changed_lines' in item:
                added = added_patch_lines(root / item['patch'], item['path'])
                expected = {number for number in added
                            if source[number - 1].strip()
                            and not source[number - 1].lstrip().startswith('//')}
                if (added != item['changed_lines'] or changed != expected
                        or len(changed) != len(item['owned_lines'])):
                    raise ValueError('changed/owned patch accounting mismatch: ' + filename)
            # LLVM may omit syntax lines, but every owned code region that
            # intersects them remains measured, including boundary-spanning ones.
            syntax = {int(number): text for number, text in item.get('unmapped_syntax', {}).items()}
            # Exact, hash-bound annotations account for compiler-omitted syntax,
            # never executable lines. Regions on these lines remain measured.
            for number, text in syntax.items():
                simple = text in ['{', ')', '} else {', '};', '})', 'false,']
                field = text == 'integer_defaults: bool,'
                pattern = text.removeprefix('| ') in [
                    'KindIndex::Int32', 'KindIndex::Sint32', 'KindIndex::Sfixed32',
                    'KindIndex::Int64', 'KindIndex::Sint64', 'KindIndex::Sfixed64',
                    'KindIndex::Uint32', 'KindIndex::Fixed32',
                    'KindIndex::Uint64', 'KindIndex::Fixed64']
                if (number not in changed or number in counts[filename]
                        or source[number - 1].strip() != text or not (simple or field or pattern)):
                    raise ValueError(f'invalid nonexecutable syntax annotation: {filename}:{number}')
            for number in changed:
                if number not in counts[filename]:
                    if number not in syntax and source[number - 1].strip() not in ['}', '};', '', 'if #non_default {']:
                        raise ValueError(f'uninstrumented changed line: {filename}:{number}')
                    continue
                owned_lines += 1
                covered_lines += counts[filename][number] > 0
            regions = [region for data in report['data'] for function in data['functions'] for region in function['regions']
                       if function['filenames'][region[5]] == filename and region[7] == 0
                       and any(region[0] <= number <= region[2] - int(region[3] == 1 and region[2] > region[0])
                               for number in changed)]
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

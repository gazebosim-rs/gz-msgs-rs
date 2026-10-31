#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Prepare unmeasured public-archive native/default/append codec executables."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import time

import codec_benchmark as bench


def prepare(archive, revision, output, cpu):
    assert output.is_dir() and not any(output.iterdir()), 'Choose an empty output directory'
    spec = importlib.util.spec_from_file_location('source_package', bench.ROOT / 'scripts/source-package.py')
    package = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(package)
    source, manifest = package.extract(archive, output / 'source', revision)
    assert (source / 'src/encode_append.rs').is_file(), 'Archive predates the public helper'
    binaries, build_info = bench.build(output, append=True, source_root=source)
    observations = []
    fixtures, units = bench.verify(binaries, cpu, observations)
    assert fixtures == json.loads((bench.ROOT / 'docs/codec-benchmark.json').read_text())['fixtures'], 'Historical complete wire changed'
    for item in manifest['files']:
        path = source / item['path']
        data = os.fsencode(os.readlink(path)) if item['mode'] == '120000' else path.read_bytes()
        assert hashlib.sha256(data).hexdigest() == item['sha256'], 'Archive source changed during preparation'
    report = {
        'state': 'prepared_unmeasured',
        'utc_finished': time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
        'scope': 'Exact public append helper, default prost and native codecs; no timing or adoption qualification.',
        'archive': {'revision': manifest['revision'], 'tree': manifest['tree'],
                    'sha256': bench.sha(archive), 'source_files': len(manifest['files']),
                    'checker_sha256': bench.sha(bench.ROOT / 'scripts/source-package.py'),
                    'manifest_sha256': bench.sha(source / 'SOURCE_MANIFEST.json')},
        'identity': bench.identity(binaries, cpu, build_info),
        'tooling_sha256': {'tools/append_benchmark.py': bench.sha(Path(__file__))},
        'binaries': {name: str(path.relative_to(output)) for name, path in binaries.items()},
        'source': str(source.relative_to(output)),
        'checks': len(units),
        'counts': [1, 3],
        'warmup_iterations_per_process': 0,
        'verification_rows': observations,
        'rows': [{'fixture': fixture, 'mode': mode, 'unit_checksum': unit}
                 for (fixture, mode), unit in units.items()],
        'fixtures': fixtures,
        'verification': 'All three runtimes, all ten complete wires, all 45 cases at counts 1/3; every elapsed_ns=0; no warmup or clock path.',
    }
    (output / 'prepared.json').write_text(json.dumps(report, indent=2) + '\n')
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, required=True)
    parser.add_argument('--revision', required=True, help='Trusted complete public Git commit')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--cpu', type=int, default=min(os.sched_getaffinity(0)))
    args = parser.parse_args()
    assert args.cpu in os.sched_getaffinity(0)
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    result = prepare(args.archive.resolve(), args.revision, output, args.cpu)
    print(f'Prepared {result["checks"]} no-clock work controls and three executables; no measurement performed.')


if __name__ == '__main__':
    main()

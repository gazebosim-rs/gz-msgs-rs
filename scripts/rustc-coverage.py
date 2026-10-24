#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Instrument owned message code and patched vendors; leave upstream deps untouched."""
import json, os, pathlib, sys
root=pathlib.Path(os.environ['GZ_MSGS_COVERAGE_ROOT']).resolve()
packages=json.loads((root/'vendor/manifest.json').read_text())['packages']
owned={root}|{root/'vendor'/(p['name']+'-'+p['version']) for p in packages}
manifest=os.environ.get('CARGO_MANIFEST_DIR')
args=sys.argv[1:]
if manifest and pathlib.Path(manifest).resolve() in owned:
    args+=['-C','instrument-coverage']
os.execvp(args[0],args)

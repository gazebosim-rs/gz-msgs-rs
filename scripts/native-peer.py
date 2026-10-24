#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
import pathlib, subprocess
root = pathlib.Path(__file__).resolve().parents[1]
out = root / 'target' / 'native'
out.mkdir(parents=True, exist_ok=True)
subprocess.run(['protoc', '-I' + str(root / 'schemas'), '--include_imports', '--descriptor_set_out=' + str(out / 'messages.pb'), *map(str, sorted((root / 'schemas').rglob('*.proto')))], check=True)
subprocess.run(['c++', '-std=c++17', '-O2', str(root / 'tests/protobuf_peer.cc'), '-lprotobuf', '-pthread', '-o', str(out / 'protobuf-peer')], check=True)

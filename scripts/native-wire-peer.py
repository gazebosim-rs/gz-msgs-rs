#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Build actual generated C++ parsers from the unchanged canonical schemas."""
import pathlib, subprocess
root = pathlib.Path(__file__).resolve().parents[1]
out = root / 'target/native-wire'
out.mkdir(parents=True, exist_ok=True)
files = ['time', 'pose', 'parameter', 'param', 'any', 'double_v', 'header', 'color', 'quaternion', 'vector3d']
subprocess.run(['protoc', '-I' + str(root / 'schemas'), '--cpp_out=' + str(out), *['gz/msgs/' + f + '.proto' for f in files]], check=True)
subprocess.run(['c++', '-std=c++17', '-O2', '-I' + str(out), str(root / 'tests/wire_peer.cc'), *[str(out / 'gz/msgs' / (f + '.pb.cc')) for f in files], '-lprotobuf', '-pthread', '-o', str(out / 'peer')], check=True)

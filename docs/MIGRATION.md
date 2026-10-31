<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->
# Consumer migration

D29 changes Messages to definitions only. Preserve the existing historical
runtime commits and evidence; do not publish its removal before simulator
consumers replace their package dependency. D28 replaces the transport rewrite
with direct upstream Zenoh and requires C++ Gazebo Zenoh compatibility.

Simulator commit `5870373f0d8280601b051f2cbb1bb0aba264f70f` replaces the
historical Messages package and Rust Transport dependencies. Its server, camera
output, CLI and renderer share one consumer-owned binding set generated with
standard protoc 3.21.12 and prost-build 0.14.4. All 216 canonical schema files
match this repository byte for byte. No custom message runtime or transport
facade replaces the removed packages.

The simulator's complete local staged hook and independent artifact audit pass:
783 headless, 435 renderer and 217 tooling tests, real C++ Zenoh peers, software
Vulkan workers and strict owned production coverage. Exact hosted run
[41248418524](https://github.com/gazebosim-rs/gz-sim-rs/actions/runs/41248418524)
passed. Independent verification of all downloaded sources, inputs, native/GPU
receipts and both strict coverage profiles is complete. This qualifies the bounded
consumer migration and permits the schema-only release; it does not complete simulator parity,
all sensor families or release acceptance.

Retain actual clock/time/control, pose/IMU/rate/force and camera domain contracts:
Header.data is repeated records, not a map; repeated keys/order and values must
survive. Preserve enum values, timestamps, frame_id/sequence, RGB stride/data,
K/P/distortion/rectification arrays, native camera fixture and real peer tests.
Those contracts are exercised by the simulator consumer; schema validation here
does not substitute for its domain interoperability and coverage gates.

Standard runtime choices need their own consumer conformance. Stock prost0.14.4
uses numeric comparison for implicit scalar floating defaults and omits -0.0,
whereas native protobuf preserves its sign bit. Canonical Vector3d, Quaternion
and Double fields used by the simulator encounter this issue; repeated camera
matrices and IMU covariance use a different encoding path. Do not copy the old
vendor patch, mutate canonical schemas or silently declare that behavior equal.
The selected simulator consumer uses a reviewed two-predicate prost-derive
0.14.4 repair limited to scalar encode and encoded_len signed-zero behavior.
Original scalar kinds, NaN/default behavior and the standard runtime remain.
Native signed-zero and full Clock, IMU and CameraInfo fixtures pass; the stock
unpatched derive fails the retained signed-zero controls. The repair lives in the
consumer, carries upstream provenance and must be rechecked on upgrades. No
vendored runtime or generator is distributed by this schema-only repository.
The earlier Google Rust/upb probe was not adopted.

Generic SDK reflection/CLI, synthetic proto2 extension/default-loader and
byte-exact unknown-field envelope tests are historical retired runtime scope.
They do not all become simulator requirements. This does not remove retained
simulator field semantics or native domain assertions. Unknown-field forwarding
requirements, where actually needed by an endpoint, must be stated explicitly;
protobuf serialization is not universally canonical byte-for-byte.

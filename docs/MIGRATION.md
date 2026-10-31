<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->
# Consumer migration pending

D29 changes Messages to definitions only. Preserve the existing historical
runtime commits and evidence; do not publish its removal before simulator
consumers replace their package dependency. D28 replaces the transport rewrite
with direct upstream Zenoh and requires C++ Gazebo Zenoh compatibility.

The simulator currently pins historical Messages `2cea810e929adb4f925750b325265a99f9ae78b4`.
Its server and camera-output production crates and CLI/render-worker tests need
consumer-owned generated bindings. Use standard pinned tooling, one generated
type identity within the consumer workspace and the unchanged `schemas` include
root. Replace domain-local vector/quaternion construction and reexports without
recreating a reusable Messages SDK or transport facade.

Retain actual clock/time/control, pose/IMU/rate/force and camera domain contracts:
Header.data is repeated records, not a map; repeated keys/order and values must
survive. Preserve enum values, timestamps, frame_id/sequence, RGB stride/data,
K/P/distortion/rectification arrays, native camera fixture and real peer tests.
Code generation and endpoint migration are not complete or qualified here.

Standard runtime choices need their own consumer conformance. Stock prost0.14.4
uses numeric comparison for implicit scalar floating defaults and omits -0.0,
whereas native protobuf preserves its sign bit. Canonical Vector3d, Quaternion
and Double fields used by the simulator encounter this issue; repeated camera
matrices and IMU covariance use a different encoding path. Do not copy the old
vendor patch, mutate canonical schemas or silently declare that behavior equal.
A bounded official Google Rust/upb codegen probe is planned separately; it is not
an adopted dependency or a verified consumer migration.

Generic SDK reflection/CLI, synthetic proto2 extension/default-loader and
byte-exact unknown-field envelope tests are historical retired runtime scope.
They do not all become simulator requirements. This does not remove retained
simulator field semantics or native domain assertions. Unknown-field forwarding
requirements, where actually needed by an endpoint, must be stated explicitly;
protobuf serialization is not universally canonical byte-for-byte.

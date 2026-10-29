# Gazebo Messages in Rust

Canonical Gazebo schemas, generated Rust values, runtime descriptors, conversion
helpers and a standalone message tool. Apache-2.0; upstream notices are retained.

```rust
use gz_msgs::{Clock, time_from_nanos, prost::Message};
let clock = Clock { sim: Some(time_from_nanos(123)), ..Default::default() };
let bytes = clock.encode_to_vec();
```

Generated values discard unknown fields on decode. For lossless forwarding use
`WireMessage::<Clock>::decode_compatible(&bytes)`, inspect `value()` and forward `bytes()`
verbatim. See [the API contract](docs/API.md), [test ledger](docs/test-ledger.json)
and [runtime qualification](docs/RUNTIME_PROBE.md).
Direct prost decoding and reflective decoding reject known tags with unexpected
wire types that native C++ treats as unknowns; the compatible envelope handles
this boundary. See [wire qualification](docs/WIRE_COMPATIBILITY.md).

The MSG-01 message foundation is [formally accepted](https://github.com/gazebosim-rs/gz-sim-rs/blob/30ade168180f97aa0bcaf447b278aaf1c4c0a5ba/docs/acceptance/MSG-01.md).
All83 original message cases have independently reviewed Rust assertion mappings.
MSG-02 remains in progress: broader XML/formatting and dynamic tooling contracts,
packaging/release qualification and performance evidence are still required.
Complete Gazebo Messages parity is not claimed. The canonical architecture and acceptance policy live in
[gz-sim-rs](https://github.com/gazebosim-rs/gz-sim-rs/blob/main/docs/PLAN.md).

Requires Rust 1.94.1 and protoc (qualified baseline 3.21.12). `cargo test --locked`
also needs a C++17 compiler, libprotobuf and gtest development files. Run the quality
gate with `python3 scripts/verify.py`; install staged-tree enforcement using
`git config core.hooksPath .githooks`. The vendored dependency fixes apply
transitively without downstream Cargo patch configuration.
The [added-line coverage policy](docs/CHANGED_COVERAGE.md) also enforces >95%
coverage of newly added executable production lines and retains exact staged
coverage evidence.

New repository-owned contributions are Copyright (C) 2026 Open Source Robotics
Foundation, SPDX-License-Identifier: Apache-2.0. See [NOTICE](NOTICE). Imported
schemas/tests and vendored dependencies retain their original notices and licenses.

[Source packaging](docs/PACKAGING.md) preserves the reviewed vendored fixes and
licenses. Crates.io publishing remains disabled; a normal `.crate` archive loses
the path dependencies and is not a supported distribution.

External registry integer defaults follow a bounded, native-backed
[Linux LP64 policy](docs/INTEGER_DEFAULTS.md), preserving raw descriptor spelling.
Standard reflection entry points retain their separate limitations. The bounded
[integer declaration formatter](docs/INTEGER_DEFINITIONS.md) supports all ten ordinary
proto2 integer scalar kinds without changing raw descriptor metadata.

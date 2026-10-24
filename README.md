# Gazebo Messages in Rust

Canonical Gazebo schemas, generated Rust values, runtime descriptors, conversion
helpers and a standalone message tool. Apache-2.0; upstream notices are retained.

```rust
use gz_msgs::{Clock, time_from_nanos, prost::Message};
let clock = Clock { sim: Some(time_from_nanos(123)), ..Default::default() };
let bytes = clock.encode_to_vec();
```

Generated values discard unknown fields on decode. For lossless forwarding use
`WireMessage::<Clock>::decode(&bytes)`, inspect `value()` and forward `bytes()`
verbatim. See [the API contract](docs/API.md), [test ledger](docs/test-ledger.json)
and [runtime qualification](docs/RUNTIME_PROBE.md).

This is a validated initial MSG-01/MSG-02 slice, not complete Gazebo Messages
parity. Full conversion and descriptor-loading behavior remain
pending. The canonical architecture and acceptance policy live in
[gz-sim-rs](https://github.com/gazebosim-rs/gz-sim-rs/blob/main/docs/PLAN.md).

Requires Rust 1.94.1 and protoc (qualified baseline 3.21.12). `cargo test --locked`
also needs a C++17 compiler, libprotobuf and gtest development files. Run the quality
gate with `python3 scripts/verify.py`; install staged-tree enforcement using
`git config core.hooksPath .githooks`. The two vendored dependency fixes apply
transitively without downstream Cargo patch configuration.

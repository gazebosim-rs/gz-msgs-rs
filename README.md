# Gazebo Messages in Rust

Canonical Gazebo protobuf schemas, generated Rust types, runtime reflection and
small dependency-free domain conversion helpers. Apache-2.0; upstream notices
are retained with the schemas.

```rust
use gz_msgs::{Clock, time_from_nanos, protobuf::Message};
let clock = Clock { sim: Some(time_from_nanos(123)).into(), ..Default::default() };
let bytes = clock.write_to_bytes()?;
# Ok::<(), gz_msgs::protobuf::Error>(())
```

See [the API contract](docs/API.md) and [test ledger](docs/test-ledger.json).
This is a validated initial MSG-01 slice, not a complete Gazebo Messages rewrite.
Full upstream conversion/tooling test parity and packaging remain pending.

Requires Rust 1.94.1. `cargo test --locked` runs binary conformance against system
`protoc`, a C++17 compiler and libprotobuf development files. Production builds
use pinned Rust-only codegen and do not need native protobuf. Run all quality
gates with `python3 scripts/verify.py`; install the staged-tree pre-commit hook
with `git config core.hooksPath .githooks`.

The canonical architecture, scope and acceptance policy live in
[gz-sim-rs](https://github.com/gazebosim-rs/gz-sim-rs/blob/main/docs/PLAN.md).

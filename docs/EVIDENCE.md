# Initial MSG-01 slice evidence

Baseline: Gazebo msgs main 89c0c3d1474f545cb889335daf930bfdab6ab852.
216 canonical schemas copied byte-for-byte; SHA256 manifest checked by gate.
`source-inventory.json` also records baseline test/fixture hashes. Test ledger
contains static declarations, not claimed expanded or executed upstream tests.

2026-10-24 Linux x86_64: exact Rust 1.94.1, system protoc/libprotobuf 3.21.12,
C++17 compiler, protobuf/codegen/json-mapping 3.7.2. Installed toolchain binaries
were placed first in PATH because a concurrent shared rustup update caused the
proxy to attempt a write outside this repository. No shared tool changes here.

Commands: `cargo test --locked`; `python3 scripts/verify.py`.
Results: 7 integration tests pass, fmt/clippy (`-D warnings`) pass, coverage
negative tests pass, schema hashes pass. Instrumented coverage report is
`target/coverage/coverage.json`: handwritten src/lib.rs lines 37/37 and regions
37/37 (100%). This is one small production crate, not an upstream parity claim.
Generated schema bodies and schema-derived registry are not handwritten code.
Native peer is recompiled by the test, not silently skipped when unavailable.

Coverage and C++ outputs are reproduced by the verification command; target
artifacts are not committed as stale acceptance evidence. The staged hook reruns
checks on an exact `git write-tree` archive, independent of unstaged edits.

Remaining: full CTest/gtest runtime expansion and original native Gazebo Utility
reference runs; original assertion-preserving conversion ports; external descriptor
search/retry/concurrency behavior; CLI grammar/completion; packed point cloud
utilities; comprehensive bytes/float/Any/maps/duplicate-field corpus; performance,
packaging and historical schema qualification. These stay not_ported in the
ledger. New conformance tests do not retire or replace unmatched upstream tests.
No performance improvement or completed MSG-01/MSG-02 claim is made.

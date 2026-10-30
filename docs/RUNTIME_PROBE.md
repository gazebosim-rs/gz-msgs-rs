# Runtime decision probe (2026-10-24)

This is historical migration evidence. The maintained production stack and
current compatibility boundaries are documented in [API.md](API.md).

The current [rust-protobuf README](https://github.com/stepancheg/rust-protobuf)
announces approaching end of life. Its 3.7.2 release is a provisional baseline,
not a release-qualified foundation. The generator patch fixes typed binary
signed zero, but runtime reflection still drops it from JSON/text. Waiting for
an unspecified upstream release does not close this measured gap.

The maintained alternative evaluated is [prost 0.14.4](https://docs.rs/prost/0.14.4/prost/)
with [prost-reflect 0.16.5](https://docs.rs/prost-reflect/0.16.5/prost_reflect/).
These are exact versions with independent committed lockfiles in experiments.
The probe generates all 216 canonical schemas and Google Any from system protoc.
It also uses the independent C++ libprotobuf peer, not Rust-to-Rust agreement.

`python3 scripts/run-prost-probe.py` verifies upstream archive SHA256 from the
lockfile, extracts only into target/, applies exact count-checked predicates,
and records JSON/logs in target/prost-probe{,-patched}.{json,log}.

| Configuration | Required checks | Failures | Diagnostic limitation |
| --- | --- | --- | --- |
| Unmodified maintained crates | 29 | 8 signed-zero cases (f32/f64 generated, dynamic, JSON, text) | Plain generated types discard unknown fields |
| Target-only patched crates | 29 | 0 | Plain generated types still discard unknown fields |

Checks include real C++ Clock/Pose/IMU/Discovery binary+text+JSON roundtrips,
all dynamic descriptor instances, nested names, unknown enum values, malformed
input, dynamic unknown fields, and exact top-level/nested unknown forwarding via
a read-only WireMessage envelope. The unmodified executable exits 1 on its eight
required failures; the runner records these failures explicitly. Its diagnostic
of generated unknown-field loss is never described as passing. The envelope is
the proposed lossless API, and generated types are construction/inspection values.

Patch scope is two files, recorded under experiments/prost-probe/*.patch:
prost-derive scalar.rs changes the two plain scalar default predicates used by
encode and encoded_len to compare float bits; prost-reflect dynamic/mod.rs adds
a bitwise float-default helper and routes three default queries through it.
Neither changes a dependency's public API. Current probe copies are outside
release source and have not yet passed owned-patch coverage qualification.

Recommendation, authorized by /root: migrate while downstream users are few.
Vendor only those pinned crates with archives, original licenses, exact patch
provenance and a reviewed unchanged-third-party exclusion. Count and instrument
changed predicate logic as owned production/build code. Require the complete
existing message/native suite plus probe regressions under the new runtime.
Until that gate passes, the reviewed rust-protobuf commit stays the safe pin.

Proposed generated API changes: Option replaces MessageField; enum fields are
i32; Message::decode/encode_to_vec replaces parse_from_bytes/write_to_bytes;
Imu replaces IMU (a root alias may remain); Discovery oneof variant names use
ordinary Rust casing. Preserve module/root aliases where practical. An explicit
WireMessage<M> stores bytes plus a read-only typed value; forwarding returns the
original bytes exactly, including nested unknowns. It offers no silent mutable
edit/reencode surface. New typed messages encode normally; explicit dynamic
messages offer reflection/editing with their own preservation contract.

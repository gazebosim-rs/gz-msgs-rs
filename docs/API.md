# Version 0.1 contract

Package `gz-msgs` is independent of simulator, transport, Bevy and math libraries.
All 216 pinned `.proto` files are copied unchanged under `schemas/gz/msgs`;
`schemas/manifest.json` records their SHA256 and immutable upstream revision.
Historical schema snapshots are not yet bundled; unknown binary fields survive
read/write against this snapshot, but historical semantics have not been qualified.

Modules preserve schema file names (`gz_msgs::pose::Pose`). Clock, Pose, IMU and
Time are also root exports. Generated public fields use `MessageField<T>` for
message presence and `EnumOrUnknown<E>` for enums. `protobuf::Message` supplies
binary parse/write; `MessageFull` supplies static descriptors. `descriptor(name)`
accepts canonical full names with an optional leading dot, including nested types.
`files()` exposes all canonical file descriptors. No callback/runtime is created.

`time_from_nanos` converts signed integer nanoseconds to a normalized sec/nsec
pair using Euclidean division. `time_to_nanos` rejects invalid nanoseconds and
i64 overflow. This is a simulator integer-time convenience contract, not a claim
that arbitrary Gazebo Time wire values must have normalized nanos. Raw generated
Time retains every i64/i32 field value. `vector3` preserves XYZ doubles;
`quaternion` preserves XYZW doubles without normalization or default substitution.
Generated absent orientation differs from the domain conversion's identity
rotation; full upstream conversion API remains pending.

## Runtime choice

Pinned rust-protobuf runtime/codegen/json-mapping 3.7.2 supports static and dynamic
reflection, protobuf text/JSON, unknown binary fields and unknown enum integers.
Prost's generated ordinary messages discard unknown binary fields; prost-reflect
would require a second dynamic representation for preservation and formatting.
The single rust-protobuf representation avoids that mismatch. This choice is
supported by the checked-in C++ binary conformance cases, not by schema identity
alone. Generator and runtime are pinned together, Cargo.lock pins transitive deps.
The pure parser generates Rust; system protoc independently produces native-peer
descriptors. No build requires protoc; native tests do require protoc/libprotobuf.

Binary decode/reencode preserves unknown fields semantically; protobuf permits
field reordering and duplicate known scalar field canonicalization. Byte equality
is asserted only on controlled deterministic fixtures. JSON/text are tooling
representations and do not promise binary unknown-field lossless roundtrips.
Standard Any (`google.protobuf.Any`) and historical snapshot qualification,
external descriptor loading, broad conversion families and command compatibility
remain explicitly pending. Gazebo's `gz.msgs.Any` is its own canonical schema.

## Coverage provenance

Every handwritten production Rust file under `src` is required in coverage,
including newly added files; missing/zero denominators fail. LLVM source regions
attribute generated code to `target/**/out/generated/*.rs` or `registry.rs`.
These are exclusively build.rs output from the hashed schema corpus and pinned
protobuf-codegen; no handwritten policies/conversions live there. Dependencies,
tests and build/test tooling are outside the production runtime denominator.
Generated exclusions await independent review and are not a general source glob.

`python3 scripts/verify.py` checks hashes, format, clippy, gate-negative cases,
all Rust/native tests and >95% lines/regions. `.githooks/pre-commit` exports the
exact index via git archive into a fresh temporary tree before running that gate.
Install using `git config core.hooksPath .githooks`. CI runs the same verification.
The initial crate has a single production source file; file and crate totals are
equivalent. No GPU/native engine feature matrix exists in this message crate.

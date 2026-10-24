# Version 0.1 contract

The standalone `gz-msgs` package depends on no simulator, transport, Bevy or math
crate. It generates all 216 byte-for-byte canonical schemas from upstream
89c0c3d1474f545cb889335daf930bfdab6ab852. SHA256 provenance is checked by the gate.
Historical snapshots are not bundled or qualified yet.

## Rust values and wire forwarding

Canonical generated values live in `gz_msgs::msgs`; file-based aliases preserve
paths such as `gz_msgs::pose::Pose`, `gz_msgs::imu::IMU`, and
`gz_msgs::discovery::discovery::Publisher`. Clock/Pose/Time/IMU are root exports.
Generated prost values expose ordinary fields: messages are Option<T>, enum
fields are i32, and oneofs are Option of Rust enums. Discovery uses
DiscContents::{Sub,Pub}, publisher::PubType::{MsgPub,SrvPub}, Scope::{Process,Host,All}.
`gz_msgs::prost::Message` supplies decode/encode_to_vec/encoded_len. Generated
values implement prost::Name with canonical names and type.googleapis.com URLs.
Standard prost/prost_types traits and Any values interoperate downstream.

**Generated values discard unknown fields during decode.** Use
`WireMessage::<T>::decode(bytes)` for a typed read-only view and lossless
forwarding: value() borrows T and bytes() returns the original bytes exactly,
including nested unknowns, signed zeros, duplicates and field order. There is no
mutable view or implicit reencoding API. Constructing a new typed message and
encoding it is explicitly a new message, not a promise to preserve historical
unknown fields. Descriptor-based DynamicMessage supports explicit reflective
editing and preserves unknown fields; replacing/clearing a nested value removes
that value's contents as expected. Binary/text/JSON roundtrips have separate
contracts: protobuf JSON does not represent unknown binary fields.

`descriptor(name)` accepts canonical full names and legacy gz_msgs.Type spellings
with an optional leading dot; nested types are supported, unqualified names fail.
`descriptors()` exposes the pool, including imported Google types; files() returns
only canonical Gazebo files. No callbacks or runtime are created.

## Plain domain values

Time converts to/from signed integer nanoseconds using time_from_nanos and
checked time_to_nanos. The constructor truncates signed sec/nsec toward zero,
matching native Gazebo duration conversion (-1 ns is {0,-1}). The inverse sums
signed fields and accepts unnormalized i32 nanos, rejecting only true i64 total
overflow. Raw generated Time retains arbitrary i64/i32 values without imposing
a wall-clock range. Source/binary-hashed native positive/negative/boundary
fixtures verify fields and exact protobuf bytes.

StringMsg/Boolean/Int32/UInt32/Int64/UInt64/Float/Double implement From in both
directions for their standard Rust scalar/String values. New wrappers have no
header. Public data fields support assignment while retaining existing metadata.
Vector2d/Vector3d/Quaternion/Color convert to/from XY/XYZ/XYZW/RGBA arrays.
Arrays are raw values without math-domain normalization or color clamping.
vector3/quaternion are convenience constructors. pose(position, orientation)
constructs geometric fields with default metadata; pose_components returns XYZ
and XYZW, with absent translation zero and absent orientation identity. Explicit
zero quaternions remain zero. Broad math conversions remain pending.

## Runtime, tooling and provenance

The maintained stack uses prost/prost-build 0.14.4 and prost-reflect 0.16.5.
Two narrowly patched dependencies are direct path dependencies, preserving fixes
when this package is consumed without any consumer Cargo patch table. A supported
prost-build prost_path facade combines standard prost traits with the patched
derive macros; there is no generated-text rewrite. The independent downstream
consumer gate proves signed zero, Any traits and nested unknown forwarding.

The two source diffs under experiments/prost-probe fix float bit comparisons in
prost-derive encode/encoded_len and prost-reflect's three default-query paths.
All vendored files have original/archive and current SHA256 provenance in
vendor/manifest.json; original licenses are retained. The old rust-protobuf
baseline and failed unmodified maintained-runtime probes remain recorded in
RUNTIME_PROBE.md. Production now tests the corrected stack, including f32/f64
positive zero, negative zero, nonzero, NaN and proto2 extension default queries.

`gz-msgs list`, `info gz.msgs.Time`, and
`encode-json|decode-json|encode-text|decode-text gz.msgs.Type` use stdin/stdout.
Errors exit nonzero and stdout is explicitly flushed. Text/JSON use the new
runtime's compact protobuf formatting; info displays descriptor fields. This
additive standalone CLI does not claim upstream `gz msg` grammar/completion parity
or own the `gz` dispatcher. External descriptor environment loading, retry and
concurrency behavior, full Any formatting and point cloud utilities remain pending.

Production builds require system protoc (qualified baseline 3.21.12); C++ peer
tests additionally require C++17 and libprotobuf. `python3 scripts/verify.py`
checks schema/vendor hashes, fmt/clippy, negative gate cases, a clean downstream
consumer, all Rust/native tests and >95% line/region coverage. The staged hook
exports the exact index to a fresh tree before running that gate.

Every handwritten Rust file recursively under src and build.rs is measured. The denominator includes
only executable changed lines/regions in the two vendored files; unchanged
third-party lines are excluded by exact provenance, not a general vendor glob.
LLVM omits syntax-only delimiters and quoted proc-macro token text from line
instrumentation; missing executable changes fail. The changed build predicates
and all three reflection paths are measured separately per patched dependency.
No claim is made that whole third-party crates exceed 95%. Generated canonical
code and schema-derived aliases are excluded by OUT_DIR provenance only.

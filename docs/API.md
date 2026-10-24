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
Errors exit nonzero and stdout is explicitly flushed. Text/JSON use compact
protobuf formatting. Legacy --help/-h (including no arguments), --version/-v,
--list/-l and --info/-i TYPE... are supported. List emits the complete 271-name
native Factory set, including imported/nested/synthetic types, sorted by name;
native registration-map traversal order is not portable. Info accepts both canonical type
spellings and multiple types. Info exactly reproduces pinned native Name/File
headings and Descriptor::DebugString output, including qualified field types and
synthetic map entries with empty bodies. A schema-bound generated corpus covers
271 canonical/imported/nested/synthetic types; its native binary, protobuf version
and schema-manifest hashes are recorded. No handwritten proto formatter or native
runtime dependency is introduced. message_definition returns the descriptor body;
message_info returns complete CLI text. The original licensed completion script
is preserved. Five unchanged original
C++ CLI tests run against the Rust binary in the gate, with only package-version
and artifact-path bindings. This package does not own the gz dispatcher.
External descriptor environment loading, retry/concurrency, full Any formatting
and point cloud utilities remain pending.

Production builds require system protoc (qualified baseline 3.21.12); C++ peer
tests additionally require C++17 and libprotobuf; the unchanged CLI suite uses system gtest. `python3 scripts/verify.py`
checks schema/vendor hashes, fmt/clippy, negative gate cases, a clean downstream
consumer, all Rust/native tests and >95% line/region coverage. The staged hook
exports the exact index to a fresh tree before running that gate.

Every handwritten Rust file recursively under src and build.rs is measured. The denominator includes
only executable changed lines/regions in the two vendored files; unchanged
third-party lines are excluded by exact provenance, not a general vendor glob.
LLVM omits syntax-only delimiters and quoted proc-macro token text from line
instrumentation; missing executable changes fail. The changed build predicates
and all three reflection paths are measured separately per patched dependency.
No claim is made that whole third-party crates exceed 95%. Only owned package
manifests are instrumented by a content-fingerprinted wrapper; unrelated registry
and Git dependencies are excluded from instrumentation. LLVM export rejects all
warnings. Generated canonical
code, schema-derived aliases and pinned native descriptor-text lookup tables are
excluded by explicit corpus/OUT_DIR provenance only.

## Legacy enum names

`enum_names::EnumKind::{Joint,Geometry,Shader,PixelFormat,Discovery}` supplies
case-sensitive parse(name)->i32 and format(raw_integer)->String policy matching
Gazebo's native converters, including fallbacks and diagnostics. Canonical
numbers/names come from prost-reflect. Only legacy spelling and Geometry's
restricted conversion subset are handwritten: valid wire variants such as
TRIANGLE_FAN/EMPTY are still unsupported by the legacy converter, as upstream.
Unknown protobuf enum integers remain intact in generated message fields; these
human-name conversion functions deliberately retain upstream fallback behavior.
A source/binary-hashed native fixture checks 197 format/parse cases, including
unknown integers, bad/empty strings and opposite case.

## Packed-cloud metadata

`cloud::init_point_cloud_packed` resets the frame_id header entry and field list,
expands the literal xyz group, and sets native word-aligned or packed offsets.
Stamp, data and the cloud's other metadata remain unchanged. The size helper
returns native widths or -1 with the legacy diagnostic for unknown types.
Initialization likewise preserves native invalid-type zero-width field metadata
and diagnostics; it does not validate or rewrite the existing data blob. Offset
arithmetic is explicitly wrapping u32. No unsafe pointer/iterator API is exposed.
A native corpus verifies 30 complete protobuf layouts and ten size cases on the
recorded word-size target. Byte/scalar access, color aliases and iterator assertion
families remain pending; metadata parity does not claim those features.


`cloud::read_native<T: bytemuck::Pod>` and `write_native` safely expose the
legacy iterator's byte addressing without returning aliased references. `point`
advances by `point_step`, and `element` advances by `size_of::<T>()` contiguously,
including across point boundaries when still inside the blob. Declared field
count/datatype and width/height/row_step do not constrain that addressing. Direct
fields win over RGB/RGBA aliases; otherwise the first packed color field supplies
r/g/b/a offsets selected by `is_bigendian`. Scalar bytes always use host byte order,
matching the native iterator; these functions do not perform endian conversion.

Missing fields, zero-size types/stride, arithmetic overflow, point indices outside
the complete-record count and byte ranges beyond the data return `None`. Writes
then leave data unchanged. Safe unaligned Pod reads use pinned bytemuck 1.25.2;
no owned pointer casts or unsafe code are needed. Empty/invalid blobs cannot
reproduce the native iterator's undefined pointer operations. Borrowed mutable
iterator and pointer-identity operators are not exposed or claimed ported.


Factory-style creation needs no additional parser or registry wrapper:
`prost_reflect::DynamicMessage::new(descriptor(name)?)` constructs an empty value,
and `DynamicMessage::parse_text_format(descriptor(name)?, text)` constructs one
from protobuf text with a detailed parse error. These public reexports cover the
original well-formed and malformed Vector3d cases and instantiate all 271 native
registered names. External descriptors and their lazy/retry/concurrency contracts
remain separate unfinished work.

`registry::DescriptorRegistry::default()` owns a canonical descriptor snapshot.
`load_bytes` accepts binary FileDescriptorSet bytes and attempts files in their
encoded order. It returns every per-file descriptor error while retaining
successful files, so callers can provide a missing dependency and retry the
failed file. Malformed outer protobuf bytes return DecodeError. `load_file`
reads one explicit path, returning IO/InvalidData errors or the same validation
error list. Duplicate schema file names are ignored, including changed content;
this API does not replace schemas. The public pool supports normal maintained
reflection construction, field access, text/JSON and binary codecs. Existing
messages/descriptors retain their pool snapshots after registry mutation/drop.

Reflection also decodes the descriptor set itself, preserving nested custom
option bytes until their extension schema is available. A plain prost-types
FileDescriptorSet decode would discard those bytes. The registry has no global
mutation, native lifetime, path cache, directory scan, environment lookup or lazy
singleton behavior. Its canonical base is explicit; callers synchronize exclusive
mutation themselves when sharing it. Those remaining legacy factory contracts
are tracked separately, and default canonical descriptor APIs remain immutable.

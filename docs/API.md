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
`WireMessage::<T>::decode_compatible(bytes)` for a typed read-only view and lossless
forwarding: value() borrows T and bytes() returns the original bytes exactly,
including nested unknowns, signed zeros, duplicates and field order. There is no
mutable view or implicit reencoding API. Constructing a new typed message and
encoding it is explicitly a new message, not a promise to preserve historical
unknown fields. Descriptor-based DynamicMessage supports explicit reflective
editing and preserves unknown fields; replacing/clearing a nested value removes
that value's contents as expected. Binary/text/JSON roundtrips have separate
contracts: protobuf JSON does not represent unknown binary fields.

The compatible envelope is available for sealed `CanonicalMessage` types,
generated alongside the types from the same descriptors. It first uses ordinary
typed decoding. On failure, it derives a temporary known-field view with native
C++ handling of wrong-wire occurrences: these occurrences do not establish
presence, replace a oneof, or overwrite a valid value. Nested messages and map
entries share the native default 100-level recursion budget; standard Any fields
are decoded, while Any.value remains opaque bytes. Original bytes remain intact
on both paths. Invalid framing and invalid known-field data still return
`CompatibleDecodeError`; there is no error-triggered empty-message fallback.

`WireMessage::decode`, `prost::Message::decode` and `DynamicMessage::decode`
remain stricter for this boundary. Reflective editing supports ordinary unknown
field numbers; its storage does not represent a known value and a wrong-wire
unknown occurrence with the same field number simultaneously. The earlier
MSG-01 interoperability corpus did not cover that condition. Its bounded
acceptance remains distinct from complete wire/runtime qualification. See
[wire qualification](WIRE_COMPATIBILITY.md) for the new native corpus and limits.

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
native Factory set with no external paths, including imported/nested/synthetic
types, sorted by name. [Descriptor-aware listing](CLI_EXTERNAL_LIST.md) extends
that set with successfully loaded GZ_DESCRIPTOR_PATH types;
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
Attached/repeated info options, short flag groups, exclusions and observed native
error codes are qualified by the [bounded option corpus](CLI_OPTIONS.md). Native
long flag-value conversion has its [own corpus](CLI_FLAG_VALUES.md); full legacy
grammar remains unqualified. Bracketed info vectors and per-occurrence required
consumption follow the [bounded native rules](CLI_INFO_VECTORS.md).
Install-prefix/canonical-path-cache behavior, dynamic-descriptor CLI info and
broader Any/formatting qualification remain pending. Factory environment/lazy,
retry/concurrency and packed-cloud APIs are described below with their own limits.

Production builds require system protoc (qualified baseline 3.21.12); C++ peer
tests additionally require C++17 and libprotobuf; the unchanged CLI suite uses system gtest. `python3 scripts/verify.py`
checks schema/vendor hashes, fmt/clippy, negative gate cases, a clean downstream
consumer, all Rust/native tests and >95% line/region coverage. The staged hook
freezes the index, exports every blob via a private Git index, and checks raw
Git blob hashes, file/symlink types, owner-executable modes and complete path sets
before compiling. This includes export-ignore inputs and rejects EOL/smudge
transformations. It rejects concurrent index or HEAD changes after verification.
Five scratch-Git regressions run in the gate, alongside the existing full-build
partial-staging failure demonstration in CI.

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
recorded word-size target. Scalar access and the cursor contracts below are
qualified separately from metadata layout.


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
reproduce the native iterator's undefined pointer operations. Mutable pointer
aliases are replaced by exclusive indexed writes.

`cloud::NativeFieldCursor<T>` borrows the cloud immutably and copies its own
position. `get(element)` reads contiguous scalar values, `+ usize`/`+= usize`
advance by point strides, `end()` creates the complete-record end sentinel, and
standard Iterator yields copied scalar values. Equality compares current byte
addresses, so two fields aliasing the same byte compare equal; missing fields
compare as absent addresses. The borrow keeps backing bytes alive and fixed.
Out-of-range advancement clamps to end instead of creating native invalid
pointers; malformed accesses yield None. End-sentinel traversal is qualified for
valid layouts; use Iterator to stop at the first unreadable malformed record.
Zero stride and zero-size types are empty. Negative native pointer movement is
not exposed; indexed reads supply checked arbitrary point selection. No aliased
mutable references or unsafe code are introduced.


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
mutation, native lifetime, path cache, environment lookup or lazy
singleton behavior. Its canonical base is explicit; callers synchronize exclusive
mutation themselves when sharing it. Those remaining legacy factory contracts
are tracked separately, and default canonical descriptor APIs remain immutable.


`DescriptorRegistry::load_paths` accepts an explicit platform-separated path list.
`load_directory` scans one level. Native filename recognition searches the entire
path for `.desc`, `.gz_desc` or `.proto` (including `.proto.bin`), rather than
checking just the suffix. A directory's `gz-msgs.gz_desc` is attempted first;
remaining entries retain filesystem order. Each accepted file uses the existing
ordered loader. A completed scan returns per-file IO/descriptor errors with paths
and attempts every candidate. The own file loads before directory iteration;
other entries load progressively. A listing failure returns an IO error while
retaining loaded descriptors (earlier per-file diagnostics are not returned with
that listing error). No diagnostics are printed. Scanning does not recursively
search nested directories or automatically retry imports. Repeated files/schemas
remain idempotent by schema file name; there is still no canonical-path cache.
Install-prefix discovery and canonical-path caching remain unfinished. No
external generated Rust types are created by the registry.


`factory::MessageFactory` is an instance-owned synchronized constructor registry.
`new_message` returns a dynamic value; `new_text` returns a parsed value, a detailed
parse error, or None for an unknown type. Canonical generated-type lookup and
registration do not initialize external descriptors. First dynamic lookup, Types
or explicit loading reads GZ_DESCRIPTOR_PATH once per default instance. Explicit
`with_paths` avoids that process environment dependency. There is no background
runtime, process-global mutable registration or install-prefix probing; the
canonical descriptor base is bundled. Initial load errors remain available via
`take_initial_errors`; `load_paths` drains pending initial diagnostics before
returning explicit-load diagnostics. Returned messages own descriptor snapshots
and can outlive the factory.

`register` stores names literally, replacing previous constructors. Name/path
argument conversion, constructor invocation and replacement destruction happen
outside the state lock, allowing reentry. Constructors must be Send+Sync and
return DynamicMessage; no null callable/value registration is exposed. Native
Gazebo aliases are normalized on lookup, while other leading dots are rejected
unless registered literally. Text construction clears constructor-populated
fields as native ParseFromString does. Types returns a sorted union of canonical,
registered and loaded names; native unordered traversal order is not portable.

Plane and box component helpers use raw arrays without an additional math crate.
`plane(normal_xyz, size_xy, offset)` and `axis_aligned_box(min_xyz, max_xyz)`
create messages; `set_plane` and `set_axis_aligned_box` update existing geometric
fields while retaining outer and nested headers. Component readers return zero
for missing nested vectors. They copy values directly: plane normals are not
normalized and box corners are not sorted. In particular these represent already
assigned native math components, not the native AxisAlignedBox constructor's
corner ordering policy. Existing generated typed unknown-field limits still apply.

`set_vector2`, `set_vector3`, `set_quaternion`, `set_color` and `set_pose`
update raw components while retaining metadata. Pose updates also retain nested
position/orientation headers and the outer name/ID. Quaternion order remains XYZW;
no normalization is introduced, including explicitly zero or non-unit rotations.
These setters consume caller-computed math values; Euler conversion and color
clamping remain responsibilities of the caller's math implementation.

`MassMatrixComponents` stores mass, diagonal XX/YY/ZZ and off-diagonal XY/XZ/YZ.
`InertialComponents` adds raw XYZ/XYZW pose components and optional6x6 fluid mass.
`mass_matrix`/`inertial` construct fresh messages; component readers and setters
copy these values without physical validity checks or math operations. Native
conversion wrappers ignore math setter validity returns; callers retain that
validation responsibility. `set_mass_matrix` sets only orientation W=1 and keeps
all other existing pose components, metadata and fluid entries. `set_inertial`
updates pose geometry and **appends**21 upper-triangle coefficients when present;
None leaves old entries intact. Repeated Some therefore appends again. Reading a
fresh component value accepts fluid data only at exactly21 entries and reflects
the upper triangle into the lower; all other lengths yield None. This matches
native conversion, including discarded asymmetric lower-triangle values. Use
fresh construction when replacing the complete message is intended.

`coordinate_type_from_math` and `coordinate_type_to_math` explicitly translate
raw gz-math coordinate enum codes1..4 (SPHERICAL/ECEF/GLOBAL/LOCAL) and canonical
protobuf codes0..3. These numeric domains differ; direct integer casting is
incorrect. Both converters preserve native invalid-value diagnostics and LOCAL
fallback (protobuf3/native4). No spherical position transformation is performed.

`SphericalComponents` uses radians for latitude/longitude/heading and meters for
elevation; its typed surface is Earth, Moon, Custom axes, or an explicitly
unrecognized native code. `spherical_coordinates` creates fresh messages;
`set_spherical_coordinates` keeps header/entity and unused axis fields. Unknown
surfaces diagnose and retain old model/axes while updating other references.
`spherical_components` reads a fresh domain value: unknown protobuf models fall
back to Earth; invalid custom axes keep Custom but use Earth's parameters. Axis
validity follows native positivity/polar<=equatorial comparisons, including IEEE
nonfinite behavior. `.axes()` exposes native known/custom model axes. Scalar
radian/degree operation order matches native Angle rounding; there is no coordinate
transform/ellipsoid implementation or input quaternion math. As with other raw
components, caller-computed surface parameters are copied by setters rather than
rerunning native math constructors' physical normalization.

## Fuel configuration metadata

`fuel::from_xml(&str) -> Result<msgs::FuelMetadata, fuel::Error>` constructs a
fresh metadata value; `fuel::to_xml(&FuelMetadata)` constructs its legacy XML.
The new Rust API exposes structured errors instead of C++ boolean output mutation,
integer exceptions and stderr diagnostics. Failed conversions do not replace a
caller's existing owned value. Native stderr is captured in the reference corpus;
Rust stderr equivalence is not claimed for this structured-error API.

Model takes precedence over world among top-level elements. Names, descriptions,
contacts and file names use native six-character ASCII trimming; dependency URIs
retain whitespace. The parser uses pinned quick-xml for tokenization with a small
adapter for TinyXML's first-text/CDATA/comments, literal qualified names, NUL
C-string views and left-to-right CR/LF normalization. Namespace declarations do
not change literal element matching. Native byte-name grammar applies to elements
and attributes; permissive prolog text is retained. Terminal document text must
be terminated by markup, matching native acceptance of leading text. The captured native nesting boundary is
retained. Input is UTF-8; references requiring invalid UTF-8 or overflowing native
character arithmetic return Xml errors. Empty dependency URI text, which reaches
a native null-pointer setter, returns MissingUri safely.

Native SemanticVersion is not strict SemVer: abbreviated and numeric-prefix
components, leading zeros, arbitrary pre-release/build text and fourth/later
components overwriting major are retained. Resource versions use checked signed
32-bit integer prefixes. Malformed/range cases that throw in C++ return
InvalidVersion. The native SDF selection loop never updates its maximum, so the
last positive candidate wins rather than the highest version. This compatibility
rule is deliberate; a strict SemVer library cannot represent these inputs.

The serializer retains exact indentation, quote style, ordering, major.minor
version and unescaped text, including legacy output that may not be valid XML.
It requires file_format.name == "sdf" and ignores metadata fields the native
serializer ignores. Applications requiring escaped XML need a separate writer.
The native corpus qualifies the tested conversion behavior; complete XML parser
or diagnostic equivalence beyond those observations remains unclaimed.

## CLI info failures

For canonical/imported bundled types, legacy --info/-i reports missing names as
`Unable to create message of type[NAME]` on stderr, continues later names and
exits zero, matching native runMsgInfo. Only the explicit Gazebo leading-dot
aliases are accepted; a dot before a Google type or multiple leading dots fails.
The new `info` command alias has the same policy. Nine native complete-output and
exit-status observations qualify this boundary; full argument grammar and
external-descriptor info formatting remain separate unfinished contracts.

## Dynamic descriptor definitions

`descriptor_definition(&MessageDescriptor) -> Result<String, DefinitionError>`
formats the qualified common proto3 declaration subset. It preserves explicit
metadata and returns `Unsupported` for unhandled grammar/options. Existing
canonical `message_definition`/`message_info` output remains immutable; generated
and dynamic descriptors can differ in explicit JSON-name metadata. External CLI
info uses this bounded formatter after the existing factory resolves the type.
See [the definition contract](CLI_EXTERNAL_INFO.md) for supported features, native
qualification and CLI failure behavior.

This API formats reflection metadata. The maintained loader synthesizes absent
JSON names, so original raw omission provenance cannot be recovered from a
`MessageDescriptor`; the CLI shares that explicit gap for hand-built descriptor
sets without those names. The native-backed mismatch regression remains part of
the gate. Native-layout equality uses identical reflection metadata and the
qualified CLI corpus, not a claim of arbitrary raw-descriptor fidelity.


The formatter now includes reserved names/ranges and C-style escaping for UTF-8
JSON names. It preserves defined raw-range display quirks and rejects the two
message-boundary cases whose native signed arithmetic overflows. This does not
recover omitted JSON-name provenance or enable proto2/extension/advanced options.

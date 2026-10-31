# External descriptor definitions

`descriptor_definition(&MessageDescriptor)` formats the qualified common proto2/proto3
message declaration subset using native protobuf DebugString layout. The existing
`message_definition` and `message_info` canonical corpus stays immutable. Generated
C++ descriptors and dynamic descriptors can differ in explicit JSON-name metadata;
matching schemas alone does not make their declaration strings identical.

The formatter preserves declaration order, nested messages/enums, source-ordered
enum aliases and signed values, maps, real oneofs, proto3 optional fields, qualified
message/enum references, reserved ranges/names and arbitrary UTF-8 JSON names
(including the empty name). Native C-style byte escaping preserves quotes,
backslashes, controls and non-ASCII UTF-8 bytes using octal escapes. It supports message no_standard_descriptor_accessor/deprecated and explicit-false
[message_set_wire_format metadata](MESSAGE_SET_FALSE.md), enum
allow_alias/deprecated, enum-value deprecated and field packed/deprecated/lazy/weak/unverified_lazy boolean
options, including explicit false. [Field-option qualification](FIELD_OPTIONS.md)
records the message-only true lazy flags and metadata/runtime boundary.
[Enum-valued field options](ENUM_FIELD_OPTIONS.md) retain declared CType/JSType
labels with explicit shape and unknown-number guards. Synthetic map entries produce an empty body.
A definition is assembled before return, so a failure exposes no partial string.
[Ordinary proto2 extension ranges](EXTENSION_RANGES.md) retain raw source order
and native endpoints with explicit option and invalid-range guards. [Nested extension fields](NESTED_EXTENSION_DECLARATIONS.md) have a separate bounded
source-aware declaration contract, including declared [deprecated flags](EXTENSION_DEPRECATED.md).
[Packed extension flags](EXTENSION_PACKED.md) retain raw presence with native
repeated-packable true guards. [CType/JSType extension enums](EXTENSION_ENUM_OPTIONS.md)
retain declared presence and native kind constraints. Unknown labels, other
options, groups and true MessageSet remain unsupported.
[Optional boolean extension defaults](EXTENSION_BOOLEAN_DEFAULTS.md) preserve
explicit true/false before supported options on the source-aware paths; other
extension-default families retain their existing Unsupported boundary, except
[canonical INT32 defaults](EXTENSION_INT32_DEFAULTS.md) on the same source-aware
paths. Noncanonical integer text keeps existing loader/formatter boundaries.
[Standard message options](MESSAGE_OPTIONS.md) retain code-generation metadata
without changing Rust descriptor APIs or MessageSet wire support.

`DefinitionError::Unsupported` identifies invalid default shapes and groups,
unsupported extension-field declarations, unsupported standard/custom/unknown options, malformed
synthetic map entries,
empty oneofs, invalid/overflowing reserved boundaries, proto3 default/required fields or
nesting beyond 100 message levels. Reflection accepts the raw default/required
features and malformed map shapes that native validation rejects; checked access
returns errors instead of omitting them or calling panic-assuming map helpers.
This is a finite feature guard, not a complete native descriptor validator. The
formatter does not emit file declarations, comments or source locations.

Reserved declarations retain source order and native output placement. Message
ranges use raw signed descriptor integers rather than the reflection Range<u32>
view. Defined native behavior includes empty/reversed spans, negative ends and
over-max ends printed as `max`; enum bounds remain signed/inclusive. Native
DebugString performs signed int start+1/end-1 for message ranges, so starts at
INT_MAX and ends at INT_MIN return Unsupported instead of targeting overflow.
The pinned [protobuf source](https://github.com/protocolbuffers/protobuf/blob/v21.12/src/google/protobuf/descriptor.cc#L2928-L2941)
and native-backed controls bind that boundary. Invalid message starts are also
rejected; these checks do not constitute a complete descriptor validator.

Basic proto2 declarations preserve optional/required/repeated labels, oneofs,
maps, nested messages/enums (including a nonzero first enum value), reserved
declarations and the same supported boolean options. Oneof fields carry no
optional label. Ordinary optional/required scalar defaults are supported within
the family contracts below; invalid default shapes, groups and unsupported extension shapes return Unsupported;
legacy inputs remain tested, with their first unsupported-feature diagnostic now
identifying extensions rather than blanket proto2 rejection. Malformed required
or repeated oneof labels accepted by reflection but rejected by native return a
typed error. This does not qualify proto2 wire/value behavior or general validation.

Standard reflection pool entry points retain their raw proto2 integer-default
boundary: `010` is decimal ten and `0x2a` is rejected. The registry/factory now use
an [opt-in integer policy](INTEGER_DEFAULTS.md) matching the qualified native
Linux LP64/C23 effective values while preserving raw spelling. Protoc normalizes
source literals, so directly constructed descriptor bytes qualify this boundary.
The formatter prints effective values for all ordinary integer scalar defaults
in decimal, before json_name and boolean options. Oneof defaults remain unsupported.
[Enum defaults](ENUM_DEFINITIONS.md) preserve selected names and aliases;
[bytes defaults](BYTES_DEFINITIONS.md) format admitted reflected bytes with the
shared escaping helper. [Floating defaults](FLOATING_DEFINITIONS.md) use the pinned
protobuf precision/layout policy over reflected values, with Rust std rounding
and explicit existing decoder differences. [Boolean defaults](BOOLEAN_DEFINITIONS.md)
use the native `true`/`false` spelling with the same option order.
[String defaults](STRING_DEFINITIONS.md) reuse the native-backed quote/escape
helper; invalid UTF8 retains the existing Rust decoder rejection. See [integer declarations](INTEGER_DEFINITIONS.md).
No raw descriptor normalization or general native proto2 formatting contract is introduced.
Canonical proto3 schemas are unchanged.

The descriptor-only API formats metadata exposed by reflection. Its loader
synthesizes a JSON name when the raw field omitted one; that omission cannot be
recovered from a standalone `MessageDescriptor`. The native-backed regression
retains this distinction explicitly. The 271-definition differential supplies
identical reflection metadata to both formatters.

`DescriptorRegistry::definition(name)` instead formats the registry's own accepted
raw descriptor metadata. It preserves omitted versus explicit JSON names, including
empty/default names and nested/map/oneof fields. Metadata is added only after a
new file loads successfully; ignored duplicate filenames and failed imports cannot
change it. Clones retain the matching pool and source-metadata snapshot. The
canonical cached corpus retains precedence. This does not expose a general raw
source serializer or enable unsupported descriptor features.

`MessageFactory::registry_definition(name)` applies the existing Gazebo aliases,
then uses that source-aware registry snapshot. It does not invoke registered
constructor callbacks: callback-produced foreign descriptors have no registry
source identity. Canonical lookup stays lazy, and external formatting occurs
outside the factory lock. The CLI uses this source-aware path after resolving the
external message; hand-built raw JSON-name omissions now match native output in
the supported subset.

The CLI `--info`, `-i` and `info` paths keep the canonical cached output, headings,
alias policy, missing-name continuation and zero exit behavior. For other names,
they use the existing lazy `MessageFactory` and its accepted external descriptors,
then the source-aware registry formatter. A supported external type has native Name/File headings and
a complete body. An unsupported known external definition exits nonzero with its
feature diagnostic before printing that type; earlier types may already have
printed and later types are not processed. This explicit limit differs from the
native formatter, which can print those features. JSON/text conversion commands
retain their existing canonical-type scope.

Descriptor discovery/loading is a separate contract: see
[the listing contract](CLI_EXTERNAL_LIST.md) and [the factory API](API.md).
Immutable canonical-base precedence, structured loader failures and instance-local
lazy initialization remain in force. The bounded
[file-diagnostic candidate](CLI_LOAD_DIAGNOSTICS.md) covers open and malformed-wire
console output separately. Schema-validation native console diagnostics,
conflicting canonical redefinitions, install/global behavior and arbitrary
external serialization are not qualified by this formatter. Successful file
caching is qualified separately in [FACTORY_FILE_CACHE.md](FACTORY_FILE_CACHE.md).

`tests/definitions.rs` compares all 271 canonical dynamic message descriptors with
a freshly compiled C++ protobuf peer, checks supported custom descriptors and
the unchanged upstream binary fixture, asserts explicit errors, regenerates
descriptor fixtures and tests 33 complete CLI rows. Additional freshly compiled
native comparisons cover raw JSON-name presence, failed/duplicate loads, snapshot
lifetimes and omitted names in map/oneof/optional fields.
`scripts/definition-reference.py` independently replays those 33 native CLI rows,
regenerates fixtures and binds the upstream commit, source/binary/libraries,
schema manifest, peer and fixture hashes. Unsupported rows capture native output
but deliberately assert the Rust error instead of claiming output parity.
No original assertion mapping or broad MSG-02 acceptance changes in this slice.

The registry rejects the two known panicking byte-default escape boundaries
before invoking reflection. [The robustness contract](DESCRIPTOR_DEFAULT_ROBUSTNESS.md)
records exact native acceptance differences; admitted byte-default declarations
follow the [bytes formatting contract](BYTES_DEFINITIONS.md).

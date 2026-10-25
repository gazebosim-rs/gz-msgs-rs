# External descriptor definitions

`descriptor_definition(&MessageDescriptor)` formats the qualified common proto3
message declaration subset using native protobuf DebugString layout. The existing
`message_definition` and `message_info` canonical corpus stays immutable. Generated
C++ descriptors and dynamic descriptors can differ in explicit JSON-name metadata;
matching schemas alone does not make their declaration strings identical.

The formatter preserves declaration order, nested messages/enums, source-ordered
enum aliases and signed values, maps, real oneofs, proto3 optional fields, qualified
message/enum references and explicit JSON names containing ASCII letters, digits
or underscores (including the empty name). It supports message deprecated, enum
allow_alias/deprecated, enum-value deprecated and field packed/deprecated boolean
options, including explicit false. Synthetic map entries produce an empty body.
A definition is assembled before return, so a failure exposes no partial string.

`DefinitionError::Unsupported` identifies proto2, extension declarations, reserved
names/ranges, unsupported standard/custom/unknown options, other JSON-name bytes,
malformed synthetic map entries, empty oneofs, proto3 default/required fields or
nesting beyond 100 message levels. Reflection accepts the raw default/required
features and malformed map shapes that native validation rejects; checked access
returns errors instead of omitting them or calling panic-assuming map helpers.
This is a finite feature guard, not a complete native descriptor validator. The
formatter does not emit file declarations, comments or source locations.

The API formats the metadata exposed by reflection. Its loader synthesizes a
JSON name when the raw field omitted one, losing that omission provenance. A
hand-built descriptor set without JSON names can therefore produce extra
`json_name` options compared with native loading of those original bytes; the
CLI currently shares this known gap. A native-backed regression preserves the
difference explicitly. The 271-definition differential instead supplies exactly
the reflection metadata to both formatters. The 25 qualified CLI rows include
the unchanged upstream binary fixture and encoded JSON-name metadata. Arbitrary
raw-omission fidelity requires a separate descriptor-provenance extension.

The CLI `--info`, `-i` and `info` paths keep the canonical cached output, headings,
alias policy, missing-name continuation and zero exit behavior. For other names,
they use the existing lazy `MessageFactory` and its accepted external descriptors,
then this formatter. A supported external type has native Name/File headings and
a complete body. An unsupported known external definition exits nonzero with its
feature diagnostic before printing that type; earlier types may already have
printed and later types are not processed. This explicit limit differs from the
native formatter, which can print those features. JSON/text conversion commands
retain their existing canonical-type scope.

Descriptor discovery/loading is a separate contract: see
[the listing contract](CLI_EXTERNAL_LIST.md) and [the factory API](API.md).
Immutable canonical-base precedence, structured loader failures and instance-local
lazy initialization remain in force. Malformed-path native console diagnostics,
conflicting canonical redefinitions, install/global/cache behavior and arbitrary
external serialization are not qualified by this formatter.

`tests/definitions.rs` compares all 271 canonical dynamic message descriptors with
a freshly compiled C++ protobuf peer, checks supported custom descriptors and
the unchanged upstream binary fixture, asserts explicit errors, regenerates
descriptor fixtures and tests 25 complete CLI rows.
`scripts/definition-reference.py` independently replays those 25 native CLI rows,
regenerates fixtures and binds the upstream commit, source/binary/libraries,
schema manifest, peer and fixture hashes. Unsupported rows capture native output
but deliberately assert the Rust error instead of claiming output parity.
No original assertion mapping or broad MSG-02 acceptance changes in this slice.

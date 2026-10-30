<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Source-aware nested extension declarations

The registry, factory and external-info CLI format ordinary proto2 extension
fields declared within messages. This is declaration text only; it adds no
extension wire decoding, dynamic value API or canonical upstream test credit.
The maintained ExtensionDescriptor supplies kind, label, number, source path,
declaration owner, extendee and full raw options. No dependency changes or
synthetic FieldDescriptor conversion are used.

Optional and repeated non-group scalar, enum and message fields are supported
when the original descriptor omitted json_name and has no explicit default.
Declared deprecated and [packed booleans](EXTENSION_PACKED.md) are supported;
other nonempty options remain errors. Packed=true requires a repeated packable
kind; packed=false preserves explicit presence for all supported kinds/labels.
See also [deprecated extension options](EXTENSION_DEPRECATED.md). Empty option presence is retained in metadata. Declarations
follow their raw source order, after ordinary fields and extension ranges and
before reserved declarations. Consecutive declarations for one extendee share
an extend block; A/B/A reopens A rather than reordering declarations. Nested
owners retain their own blocks.

Reflection inserts a raw camelCase json_name when absent; its extension JSON
accessor instead returns a bracketed full extension key. The registry therefore
extends its existing original-absence bookkeeping to nested extension fields.
Explicit json_name remains Unsupported, including native-admitted explicit
ordinary default spellings. Custom names are native-negative. Standalone
`descriptor_definition` cannot recover original presence and keeps extension
declarations Unsupported. The formatter does not invent a case converter or
silently discard present metadata. Defaults, groups, required fields, maps,
proto3 declarations, true MessageSet extendees, oneof/proto3-optional shapes, empty extension names or bytes outside ASCII alphanumeric/underscore and
unhandled standard/custom/unknown options remain errors. This is a bounded feature
guard, not a complete native descriptor validator.

The native corpus has 125 complete peer and upstream CLI observations on pinned
protobuf 3.21.12: 115 are admitted natively and 109 match supported source-aware
registry/factory/CLI output. It covers all seventeen non-group kinds,
optional/repeated labels, nested owners, interleaved extendees, ranges/reserved
placement, explicit/default/custom JSON names and rejection boundaries. Six
native-admitted layouts remain unsupported after deprecated-option support. The maintained registry rejects the
oneof layout; other retained negative inputs reach explicit formatter errors.
Native also admits a leading digit in an extension name; that positive remains
supported. Space, newline, Unicode and empty names retain exact negative
observations and formatter guards. Native CLI paths are deterministic relative
paths with an explicit working directory, so errors replay without stripping or
normalizing stderr in independent source exports.
Tests compare status, stdout, stderr and unchanged full encoded descriptor
metadata. The original 14 definition tests, historical descriptors, native peer
and canonical 83-case ledger remain unchanged.

Author native replay, the new 125-row Rust control, unchanged definition tests and
all-target Clippy pass. Independent fresh source/target review passed all 15 Rust
checks and complete 125 native observations. Review corrected source-directory-
dependent CLI error paths without normalizing stderr, and removed an unintended
change to the existing range validator. That validator is byte-identical to the
accepted baseline; the name guard applies only to extension fields. Final nine
paths are independently approved. Strict staged coverage and exact hosted
qualification remain pending. This does not complete MSG-02.

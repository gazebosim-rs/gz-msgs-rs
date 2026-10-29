<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Enum-valued field options

The formatter retains declared `ctype` and `jstype` values using maintained
reflection enum descriptors. Labels, declaration order and explicit default-value
presence match protobuf 3.21.12 DebugString. It reuses the existing option iterator;
no descriptor parser, enum registry or runtime behavior is added.

Native permits every declared CType value on every field kind. JS_NORMAL is also
valid on every kind. Nonzero JS_STRING and JS_NUMBER require int64, uint64, sint64,
fixed64 or sfixed64. The formatter rejects other nonzero-JSType shapes after the
existing option validation and lazy guard. Proto3 required fields, unknown/custom
options and unrelated unsupported features keep their existing error precedence.
These are metadata declarations; Rust wire/value behavior does not acquire C++
string-storage or JavaScript integer semantics.

Unknown enum numbers remain Unsupported("option"). Reflection retains them as
EnumNumber, while native admits the descriptor and omits these option occurrences
from its declaration. Loader admission and raw metadata are preserved. This
bounded formatter guard deliberately retains its previous explicit error rather
than silently dropping reflected data.

## Native evidence

`scripts/enum-field-options-reference.py` replays 736 descriptors against a fresh
native peer and the pinned upstream CLI, binding source/binary/library/schema
identity. Native admits 472; 456 contain supported declared enum values. Sixteen
unknown-number controls remain explicit Rust errors. Native rejects the other
264 descriptors for nonzero-JSType shape or existing proto3-required rules.

The matrix covers all seventeen ordinary kinds, proto2/proto3 and all three
labels, combined boolean/enum options, maps, real oneofs and synthetic proto3
optional fields. Explicit JSON names give the direct and source-aware APIs the
same metadata. Every supported row checks exact registry/factory/direct/CLI
output. Negative rows check precise Rust errors separately from complete native
observations. Typed descriptors and encoded reflected options remain unchanged.

## Reviewed temporary Rust capability transitions

Independent review approved these exact transitions before the old predicates
changed. Prepared supplementary tests passed against unchanged old fixtures:

- The six `unsupported_ctype` rows in field-options.json retain their exact bytes
  and complete native observations. The three lazy=false rows become complete
  native-positive registry/factory/direct/CLI checks. The three lazy=true scalar
  rows retain failure and now identify the native-backed lazy shape error.
- StandardFieldOption in unsupported.desc retains its exact CORD input. Its old
  direct Unsupported predicate becomes an exact fresh-native declaration check.
- Historical CLI row 7 retains its descriptor path, arguments and captured native
  status/stdout/stderr. Its old Rust Unsupported predicate becomes a complete
  native-positive comparison; the fixture's historical annotation is unchanged.

`prepared_six_ctype_capability_transitions_retain_exact_native_controls` and
`prepared_original_ctype_descriptor_and_cli_use_their_unchanged_native_oracles`
exercised these replacements before the old predicates changed. Their assertions
are now folded into the existing tests, with no redundant prepared tests. No genuine
upstream assertion, fixture, tolerance or original mapping is removed. Unknown
and custom options and unrelated lazy errors remain explicit.

The new 736-row test and both prepared transition tests passed, along with the
complete native replay and Clippy. Independent fresh-source/target review replayed
all three tests and 736 complete native observations before approving the narrow
transition. Final independent review passed fourteen unchanged definition tests,
the 736-row enum test and the 354-row original option test, and verified the old
fixture/descriptor bytes are unchanged. Normal strict local and exact hosted
gates remain pending. All 83 original
mappings, dependencies, decoder boundaries and performance claims are unchanged.

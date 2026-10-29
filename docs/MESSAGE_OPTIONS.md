<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Standard message-option declarations

The formatter retains `no_standard_descriptor_accessor` beside the existing
message `deprecated` boolean through the maintained reflected option iterator.
Explicit false and true stay visible, absent options stay absent, and native
option order is preserved. Nested messages use the same path; synthetic map
entries retain their existing native empty declaration behavior. This formats
code-generation metadata without disabling Rust descriptors, changing generated
APIs or expanding wire/runtime behavior.

The pinned native 3.21.12 corpus has 98 complete peer/CLI observations. Ninety
ordinary cases cover proto2/proto3, empty/field/oneof/nested/map contexts and all
absent/false/true combinations with deprecated. All ninety compare exact direct,
registry, factory and complete CLI output. Typed descriptor metadata (including
nested messages) and encoded reflected message options are unchanged afterward.

Eight MessageSet controls preserve the existing explicit Unsupported option
boundary. Five admit in native and three are rejected by native, while all eight
remain ordinary descriptor admissions through the existing Rust loader. Their
native observations are recorded separately from the Rust formatter error; no
MessageSet implementation or general descriptor validation is claimed. Existing
custom/unknown options, extension declarations and other guards are unchanged.

## Reviewed exact capability transition

Independent fresh-source/target review approved the exact StandardMessageOption
direct and historical CLI row 6 transitions before the old predicates changed.
A prepared positive test retained unsupported.desc bytes, name and arguments,
checking direct/registry/factory against fresh native and complete CLI output
against its historical oracle. It is now folded into the existing native-positive
paths; no redundant prepared test remains. The fixture's old unsupported annotation
stays historical. No descriptor or native observation is rewritten.

This closes a concrete retained external-info behavior, not a new original-case
credit. All 83 upstream mappings, dependencies, canonical definitions, decoder
boundaries and performance evidence remain unchanged. The 98-case native replay,
supplementary tests and Clippy passed. Independent review replayed both prepared
tests and all 98 complete native observations before approval. Final independent
review passed all fourteen definition tests and the 98-row message-option test;
old descriptor/fixture bytes remain unchanged. Normal strict local and exact
hosted qualification remain pending.

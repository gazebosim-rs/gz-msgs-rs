<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Declared packed extension options

The bounded source-aware formatter retains declared packed=false and packed=true
alongside deprecated flags, using the existing maintained reflection option
accessor and option formatter. Explicit false presence remains distinct from
absence in typed and dynamic metadata. Options retain native ordering, and the
whole definition is returned only after successful formatting.

Pinned native validation admits packed=false for all seventeen non-group kinds
and both optional/repeated labels. Packed=true requires a repeated packable
primitive or enum. The formatter checks raw Some(true) against the maintained
ExtensionDescriptor::is_packed result after the existing declaration-shape guard;
it does not implement a new packability algorithm or relax descriptor loading.
This validates declaration text only, not extension wire/value behavior.

The existing source-aware proto2, name, original-JSON-name-absence and shape
boundaries remain. Standalone descriptors cannot recover original presence.
Explicit JSON names/defaults/groups/MessageSet, other standard options and custom/
unknown options remain unsupported. No dependency, vendor, runtime, schema or
canonical 83-case mapping changes accompany this formatter slice.

The new corpus contains 937 complete native peer/upstream CLI observations on
protobuf 3.21.12. Its 918-case matrix covers seventeen kinds, optional/repeated,
three owner layouts and independent absent/false/true packed/deprecated flags.
All 180 invalid true matrix inputs fail native validation and return explicit
formatter errors. Mixed presence, source order/extendee grouping, retained
boundaries and the exact historical false input are additional controls. Native
admits 749 inputs; 742 match the bounded registry/factory/actual CLI output.
Full typed option values, dynamic presence/default values, computed packed flags
and unchanged encoded file/options snapshots are asserted.

The old 224-row deprecated and 125-row declaration corpora, reference helpers,
native peer remain byte-identical. One prepared control copied the exact old224 other-option descriptor/native/CLI
observations and asserted literal packed=[false,None,None] with deprecated=true
on all three fields. Independent prior review replayed both prepared tests and
all 937 native observations before the exact temporary predicate was folded into
the complete positive branch. The old test now asserts literal packed presence,
absent remaining packed flags, deprecated=true and computed false packed state;
its supported count becomes 210 of the same 216 native admissions. Every unrelated
predicate/input remains. The redundant prepared test was removed after retaining
its exact historical binding inside the corpus control.

Author final 17 focused Rust checks, native replay and all-target Clippy pass.
Independent fresh-source/owned-target review passed all 17 checks and reproduced
all 937 complete native peer/CLI observations (749 admits). The raw Some(true)
maintained-property guard, exact old predicate fold, full false/None metadata,
immutable old corpora/helpers/peer/schema and additive bindings are reviewed with
no findings. All eleven final paths are approved. Normal strict staged checks
and exact hosted qualification remain pending; accepted Main remains 6150fd0.
No full MSG-02, runtime extension or performance acceptance is claimed.

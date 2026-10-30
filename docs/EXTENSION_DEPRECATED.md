<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Declared deprecated extension options

The source-aware nested-extension formatter retains declared `deprecated = false`
and `deprecated = true` using the existing maintained reflection option accessor
and common option formatter. Absent flags produce no option text. Explicit false
presence is retained in typed and dynamic metadata, and formatting changes neither
the raw descriptor nor encoded options. This is declaration metadata only.

The accepted declaration domain remains ordinary proto2 optional/repeated
non-group scalar, enum and message extensions nested within messages. Original
JSON-name absence is required, source order and consecutive extendee grouping
are preserved, and standalone descriptors still cannot recover original presence.
Defaults, groups, true MessageSet, explicit JSON names, other standard options,
custom/unknown options and previously rejected shapes remain unsupported. No
loader, dependency, wire or extension runtime behavior changes.

The new corpus records 224 complete pinned protobuf 3.21.12 peer and upstream CLI
observations. All seventeen non-group kinds, optional/repeated labels, ordinary/
nested/range-and-reserved layouts and mixed absent/false/true flags are covered.
216 inputs are native-admitted; 209 fit this bounded formatter. Negative cases
retain other-option, JSON-name, default, group/map/MessageSet, name, proto3 and
custom/unknown-option boundaries. Tests compare typed `Option<bool>`, dynamic
presence/value, full file/options bytes, registry/factory text and actual CLI
status/stdout/stderr.

The existing 125-row corpus and its reference helper remain byte-identical. A
prepared control preserves the exact old deprecated-false descriptor and full
native peer/CLI observations; its literal three false flags and complete output
are stronger replacements for one temporary Rust Unsupported predicate.
Independent prior review replayed both prepared tests and all 224 native controls
before the exact predicate was changed. The original input/native observations
remain byte-identical; the old test now also asserts literal Some(false) and
actual dynamic presence/value for all three fields. Its supported count becomes
109 of the same 115 native admits, with every unrelated guard retained. No original
upstream assertion or canonical 83-case mapping is changed.

Author final 16 focused Rust checks and all-target Clippy pass. Independent
fresh-source/owned-target review passed all 16 checks and reproduced every new
224 native observation (216 admits) plus the unchanged old 125 (115 admits),
including complete CLI outputs and pinned provenance. New verifier bindings,
final protected-predicate fold and all ten paths are independently reviewed with
no findings. The redundant prepared test was removed after retaining its exact
historical binding in the corpus control. Normal strict staged and exact hosted
qualification remain pending; accepted Main remains a2b0079. No full MSG-02 or
performance acceptance is claimed.

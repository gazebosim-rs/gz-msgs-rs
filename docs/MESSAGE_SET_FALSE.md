<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Explicit-false MessageSet metadata

The formatter emits message_set_wire_format only when its admitted value is
false. This is ordinary message metadata, not MessageSet wire support. It reuses
maintained reflected boolean option formatting and the typed descriptor flag.
True stays Unsupported("option"). Unknown/custom/unhandled option validation
runs before that true guard, preserving existing error precedence. The loader, stored
descriptor metadata, dependencies and message encoding/decoding are unchanged.

The immutable 98-case message-option corpus has four exact native-positive false
descriptors: empty/ordinary-field layouts in proto2/proto3, alongside explicit
no_standard_descriptor_accessor=true and deprecated=false. Their complete native
declaration and CLI status/stdout/stderr observations remain unchanged. Direct,
registry, factory and actual CLI paths now compare full native outputs, assert
literal Some(false), and retain full raw typed/dynamic metadata checks. All four
true controls keep their existing errors, including the one native-admitted empty
proto2 MessageSet case deliberately unsupported in Rust.

Independent fresh-source review approved these exact temporary Rust capability
transitions before the old predicates changed. The prepared control retained all
four original descriptor/CLI inputs and full observations; it also replayed the
four unchanged true guards. Its stronger checks are now folded into the existing
98-case test, with no redundant prepared test. The corpus's historical expected
annotations and native peer/reference bytes remain immutable. The helper's
summary still reports the historical 90 supported annotations; the current Rust
mapping supports 94 of the 95 native-admitted cases. No genuine upstream
assertion or any of the 83 mappings changes.

Author prepared validation passed the four-positive/four-true-guard test, all
fourteen unchanged definition tests, Clippy and the complete 98-case native
peer/CLI replay. Final folded source/assertion review and the normal strict staged
and exact hosted gates remain pending.

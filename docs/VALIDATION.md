<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->
# Schema validation

`tools/validate.py` checks all 216 manifest hashes and refuses missing, unlisted
or symlinked schemas and a Rust runtime tree. Standard `protoc` 3.21.12 compiles a
temporary public-import catalog of every canonical file with `--include_imports`.
The compiler's dependency output must resolve exactly the canonical files, the
catalog and the external standard Any definition. The deterministic descriptor
hash is checked against the manifest. This covers the complete compiled schema
structure; no handwritten protobuf grammar or descriptor decoder is used.

The standard Any dependency is external, pinned by source SHA-256 and provided
under `/usr/include` by default (`--protobuf-include` changes that include root).
The descriptor hash is bound to the pinned compiler and dependency bytes. An
intentional upstream/compiler/dependency update requires explicit manifest
changes and independent review, not regeneration during a passing gate.

The staged hook exports the intended index, runs negative tests and validates
that export. It preserves the parent's index and HEAD and sanitizes only local
Git environment variables in the child. `tools/test_staged.py` exercises an
unstaged edit, explicit Git directory/worktree, an alternate index and staged
failure. The regular schema tests exercise real compilation, source/hash drift,
unlisted/symlinked files, changed field tags, unresolved imports, changed standard
Any bytes and accidental runtime reintroduction.

CI runs both suites and exports the exact Git source archive with a checksum and
validation report. A successful schema check does not qualify any consumer's
code generator, runtime, wire semantics or Zenoh behavior. Generated/native
third-party coverage exclusions and retained consumer domain checks belong to
that consumer. This tree has no production runtime coverage denominator.

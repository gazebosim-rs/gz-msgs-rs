<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Declared CType/JSType extension options

Source-aware ordinary proto2 nested extension declarations format declared CType
and JSType values using the existing maintained dynamic-options accessor and
enum-name formatter. Explicit zero remains distinct from absence in typed and
dynamic metadata. Declared option order, extension source order and consecutive
extendee grouping remain unchanged. No synthetic field descriptors, parser,
case converter, vendor changes or runtime extension support are added.

Known CType labels STRING, CORD and STRING_PIECE are native-admitted on every
supported kind and optional/repeated label. JS_NORMAL is also admitted on every
kind; JS_STRING and JS_NUMBER require a 64-bit signed/unsigned integer kind,
including fixed and zigzag forms. The formatter mirrors the existing ordinary
field kind guard after the existing declaration-shape and packed guards.

Native protobuf 3.21.12 admits undeclared numeric enum options but silently omits
them from declaration output. Rust retains the exact typed number, dynamic
presence/value and encoded options, and returns Unsupported("option") rather
than dropping declared metadata. This is a named formatter difference, without
changing registry or standard reflection admission. Unknown/custom/unhandled
options retain their existing errors and precedence. Original json_name absence,
no explicit defaults, non-group shapes, proto2 and MessageSet boundaries remain.
Standalone descriptors still cannot recover original JSON-name presence.

The new corpus records 2,259 complete peer and actual upstream CLI observations.
It includes 1,632 single-option cases across all seventeen non-group kinds, both
labels, three owner layouts, absence/known labels and four undeclared numbers per
enum; 612 mixed presence/packed/deprecated layouts; and fifteen retained shape,
name, custom/unknown/unhandled-option controls. Actual CLI descriptors use a
relative path and explicit working directory; complete native CLI fixture
observations compare exactly without path stripping or normalization. Supported
Rust rows compare full CLI output; unsupported rows deliberately assert Rust
errors. Native peer rejects 371 rows while the upstream CLI still exits zero,
retaining its existing continuation/error-reporting contract rather than universal
Rust/native exit parity. Tests check registry/factory/actual CLI,
typed and dynamic option presence/numbers, complete encoded file/options
immutability and the standalone source-presence restriction.

The old 125/224/937 corpora, scripts, peers, descriptor bytes and assertions remain
unchanged. New verifier bindings retain their exact hashes and pinned native
binary/source/libraries/schema. No canonical 83-case ledger, dependency, wire,
runtime or performance credit accompanies this candidate.

Author checks pass eighteen focused Rust controls (fourteen unchanged definitions,
the unchanged 125/224/937 corpora and the new corpus), all-target Clippy and fresh
native generation. Of 2,259 native observations, 1,888 are admitted and 1,065
layouts are supported. All 816 undeclared-label cases retain explicit Rust errors
despite native omission; seven further native-admitted old shape/presence
boundaries remain unsupported. Independent fresh source/owned-target review passed all eighteen Rust controls
and reproduced all 2,259 complete native peer/CLI records with pinned identities.
All nine paths and retained old files were independently compared; no source or
assertion findings remain. Normal strict staged and exact hosted qualification
are pending.

```sh
python3 scripts/extension-enum-options-reference.py --upstream ../gz-msgs
cargo test --locked --test extension_enum_options
```

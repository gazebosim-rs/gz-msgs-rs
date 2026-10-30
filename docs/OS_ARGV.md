<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Linux OS-byte command arguments

The legacy message command reads `args_os` and passes owned OS values through
maintained `clap_lex::RawArgs`. Its existing ASCII option, bracket/vector and
boolean policy now works on bytes. Invalid UTF-8 in a type name remains an exact
missing-name diagnostic; later valid names still print. Unknown option and
conversion diagnostics preserve the original bytes. No replacement characters
or lossy path/name conversion are used. The external factory retains its OS-path
handling, including non-UTF-8 `GZ_DESCRIPTOR_PATH` filenames.

Rust-only encode/decode JSON/text commands require UTF-8 arguments and return a
controlled error when that requirement fails. This does not change their message
or wire semantics. Qualification is Linux/Unix byte argv; no Windows equivalence
is claimed.

`tests/fixtures/os-argv.json` records 56 complete native observations: invalid
byte sequences, attached/separate info names, aliases, mixed valid names, vectors,
unknown flags, boolean conversion, help/version/list precedence and an external
descriptor at a byte-valued filename. Exact status/stdout/stderr bytes are checked.
Help and version retain the existing Rust help/package-version bindings; list
compares the complete sorted names because native map order is unspecified.
These bindings are explicit and are not native help-text or list-order parity.

The reference helper pins the existing native binary, source, loaded libraries
and canonical schemas. It uses a relative descriptor path with an explicit cwd;
no stderr normalization is applied. The verifier also binds the new script and
all three unchanged legacy CLI corpora. Existing CLI tests, five unchanged native
CLI originals, external-descriptor controls and production gates remain required.
No dependency, runtime parser, wire format or original ledger entry changes.

Independent fresh-source review passed all 48 Rust controls (45 CLI, one
external, two OS-byte), all five unchanged native CLI originals against the
reviewer binary, and fresh 56-row native regeneration with pinned identities.
It verified the final lazy-source initialization, exact raw diagnostics and
unchanged old corpora/tests/reference harness. Author all-target Clippy also
passed. Normal strict staged and exact hosted qualification remain pending.

```sh
cargo test --locked --test cli --test cli_external --test os_argv
python3 scripts/os-argv-reference.py --upstream ../gz-msgs
```

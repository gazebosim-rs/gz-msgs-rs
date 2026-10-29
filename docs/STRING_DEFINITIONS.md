<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Proto2 string declarations

Ordinary optional and required proto2 string defaults use the existing
native-backed quote/escape helper. It preserves empty values, literal backslash
sequences, quotes, control characters including NUL, and Unicode bytes. Defaults
appear before explicit JSON names and supported boolean field options.
The parser, raw descriptor metadata and message wire paths are unchanged.

The registry, factory and CLI preserve omitted JSON-name metadata. The direct
formatter uses the supplied reflection metadata, including synthesized JSON
names, as before. No explicit default is invented when the input omits it.
Oneof, repeated and proto3 defaults, and unknown options, remain unsupported.
Bytes defaults have their separate escape/parser safety contract and remain
unsupported; floating and enum default declarations are also outside this slice.

The native corpus contains 54 controls, including 49 supported layouts. These
cover optional/required fields, empty and whitespace strings, every ASCII byte,
quotes and literal escapes, and composed/decomposed Unicode. The previous native
string-default input remains an exact positive assertion rather than being
removed from the suite. All 83 upstream mappings remain unchanged.

The invalid-UTF8 control records an existing difference: native protobuf accepts
the raw default with a warning, while Rust's descriptor decoder rejects it before
formatting. This change preserves that rejection and does not claim arbitrary
native string-byte compatibility.

The [fixture](../tests/fixtures/string-definitions.json) stores raw inputs,
complete native process outputs, compiler command and peer/library identities.
It reuses `tests/integer_definition_peer.cc` as the generating observer. Replay
installed protobuf 3.21.12 on Linux LP64 and the Rust paths with:

```sh
cargo test --locked --test string_definitions --test integer_definitions
```

Wider Messages behavior, other native library versions and performance remain
separate qualification work.

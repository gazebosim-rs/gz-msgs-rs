<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Proto2 enum declarations

Ordinary optional and required enum defaults retain their selected identifier,
including aliases of the same number. Reflection resolves membership in the
declared enum and stores its numeric value; it does not lexically validate enum
identifiers. The formatter checks the explicit default token's bounded ASCII
identifier shape: letter or underscore, then letters, digits or underscores.
It preserves the selected raw name rather than choosing a name by number.
Omitting a default emits no clause, even when the first enum value is nonzero.

This guard leaves reflection admission unchanged. Matching malformed declared
names, including valid-UTF8 Unicode and embedded newline/semicolon text, are
admitted by reflection but rejected by the formatter; native rejects them during
admission. Registry, direct formatter, factory and CLI tests preserve that
boundary. General malformed enum type/value metadata, especially without an
explicit default, is outside this guard. Invalid UTF8 descriptor bytes retain
the existing typed-decoder behavior. No general parser or enum validator is added.

Defaults precede explicit JSON names and supported field options. Registry,
factory and CLI source omissions remain distinct from direct reflection's
synthesized JSON names. Input descriptors and wire behavior are unchanged.
Oneof defaults and unknown options remain explicit formatter errors. Unknown
or scoped default names, numeric default strings and forbidden aliases fail
native and registry admission. Native rejects proto3/repeated defaults during admission; the existing
Rust reflection layer admits those shapes, and the formatter explicitly rejects
them. This slice preserves that admission difference. Bytes and floating declarations
remain unsupported; all 83 upstream mappings are unchanged.

The corpus records 96 native controls, including 69 supported layouts: top-level
and nested enum types, optional/required fields, bare/explicit options, positive,
negative, zero and signed 32-bit endpoint values, aliases and omitted defaults.
Every supported layout compares actual registry, factory and CLI output; explicit
JSON layouts also compare direct reflection. Tests assert raw metadata and the
separate reflected enum number. Twenty-seven controls exercise malformed names and
shape/option boundaries, including ten matching malformed declarations. Legal
underscore-leading names and names containing digits remain positive controls. The earlier enum-default observation remains an exact
positive native assertion.

The fixture stores complete native outputs, raw descriptors, compiler command and
peer/library identities for installed protobuf 3.21.12 on Linux LP64. The existing
observer is reused without changes. Replay with:

```sh
cargo test --locked --test enum_definitions --test integer_definitions
```

Other protobuf versions, wider Messages compatibility and equal-work performance
remain separate qualification work. Independent review, the mandatory staged
gate and exact hosted CI are required before acceptance.

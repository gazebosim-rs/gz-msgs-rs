<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Proto2 boolean declarations

Ordinary optional and required proto2 boolean defaults are formatted as `true`
and `false`, before an explicit `json_name` and supported boolean field options.
The existing reflection value and Rust's boolean formatter supply this spelling;
no parser, schema, wire codec or raw descriptor metadata changes.

The registry, factory and CLI preserve omitted JSON-name metadata. The direct
`MessageDescriptor` formatter uses the metadata supplied by reflection, including
its existing synthesized JSON names. Typed/raw fields remain unchanged after
formatting. Missing explicit defaults remain absent from declarations.

The native corpus covers thirteen supported layouts: optional/required true and
false, absent/explicit JSON names, deprecated true/false, and an omitted default.
Ten invalid literal spellings retain their parser rejection. Oneof, repeated and
proto3 defaults remain unsupported by formatting, including existing differences
between native and reflection admission. Unknown options still return an error.
The earlier native boolean control retains its exact input and expected native
declaration as a positive test. String/bytes, floating and enum defaults remain
outside this slice; all 83 upstream assertion mappings are unchanged.

`tests/integer_definition_peer.cc` is reused as the native declaration observer.
The [fixture](../tests/fixtures/boolean-definitions.json) records every raw input,
complete native status/stdout/stderr, peer identity and library provenance.
Replay all 28 native observations and the Rust paths with:

```sh
cargo test --locked --test boolean_definitions --test integer_definitions
```

Qualification is installed protobuf 3.21.12 on Linux LP64. Wider Messages parity,
other native library versions and performance remain separate requirements.

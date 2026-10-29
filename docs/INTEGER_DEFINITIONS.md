<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Proto2 integer declarations

The dynamic descriptor formatter supports explicit defaults on ordinary optional
and required proto2 int32/int64, uint32/uint64, sint32/sint64, fixed32/fixed64
and sfixed32/sfixed64 fields. It prints the parsed
effective value in decimal, before an explicit `json_name` and supported boolean
options, matching native protobuf declaration order. The registry, factory and
CLI info paths use the same formatter. Definitions are assembled before return,
so an unsupported field exposes no partial output.

Raw descriptor spelling is unchanged. For example, the qualified registry parser
interprets raw `010` as eight, while an ordinary reflection pool retains its prior
decimal-ten interpretation. Formatting uses the supplied descriptor's effective
value rather than reparsing the raw string. The original 990 parser observations,
raw metadata, explicit-presence wire and rejection assertions remain in force.

Defaults on oneof, repeated and proto3 fields remain outside this formatter's
supported subset. [Boolean](BOOLEAN_DEFINITIONS.md) and valid-UTF8
[string defaults](STRING_DEFINITIONS.md) are separate qualified families. Later
[bytes](BYTES_DEFINITIONS.md), [enum](ENUM_DEFINITIONS.md) and
[floating](FLOATING_DEFINITIONS.md) slices extend formatting without changing
this integer contract.
Unknown options remain errors.
This adds no parser policy, schema change, wire behavior, upstream test retirement,
performance result or complete MSG-02 acceptance.

`tests/integer_definition_peer.cc` is the generating native observer. The three
committed fixtures contain every raw descriptor input and all 1,066 native process
results, including status and complete stdout/stderr. The fresh C++ replay checks
that all observations reproduce byte for byte:

```sh
cargo test --locked --test integer_definitions --test integer_definition_kinds
cargo test --locked --test integer_defaults --test definitions
```

The [lineage record](../tests/fixtures/integer-definitions/lineage.json) records
the exact peer build command, protobuf source identity, installed-library hashes
and the separate historical qualification. Installed protobuf 3.21.12 on Linux
LP64 is qualified; other native parser/library versions require their own checks.
The initial four-kind slice remains published as
[28e69e6](https://github.com/gazebosim-rs/gz-msgs-rs/commit/28e69e6a494ad6c6a74ad944cb7b1737371952d0).
The remaining six kinds add 594 native observations from the unchanged 990-row
parser corpus; 384 admitted cases now check registry, direct descriptor, factory
and CLI declaration output. The original 76 controls and 396 rows remain unchanged,
including negative controls for unsupported defaults and malformed metadata.
The existing 83 reviewed upstream mappings do not change.

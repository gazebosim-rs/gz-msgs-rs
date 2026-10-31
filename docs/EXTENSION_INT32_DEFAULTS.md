<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->
# Canonical INT32 extension defaults

Source-aware registry, factory and CLI definitions format optional proto2 INT32
extension defaults when the raw text equals the maintained reflection runtime's
effective `Value::I32` rendered in decimal. This includes zero, negative values
and both INT32 limits. Default text precedes existing options. Omitted defaults
stay omitted; raw descriptor text and option presence remain unchanged.

This boundary deliberately excludes noncanonical text, including `+3`, `-0`
and leading zeros. Extension resolution uses standard decimal `i32` parsing;
ordinary fields' separately qualified LP64/C23 integer policy does not apply.
For example, reflection interprets `010` as ten while native protobuf prints
eight. Formatting returns Unsupported for that admitted descriptor. Hex, binary,
whitespace, NUL and overflowing defaults keep their existing loader behavior.
No parser, loader, vendor, dependency, schema, generator or wire API changes.

The native corpus records 97 complete descriptor/CLI observations against the
pinned Gazebo Messages source and installed protobuf 3.21.12. It covers 57
supported layouts, 23 formatting rejections and 17 reflection-loader rejections.
Ten of those loader rejections are accepted natively; this existing difference
is preserved. Required/repeated defaults, proto3, explicit extension JSON names,
other integer kinds and unknown/custom options retain their boundaries. A late
unsupported declaration exposes no partial CLI output. Boolean defaults retain
their accepted behavior. Standalone descriptor formatting retains its source
metadata limitation.

Six prepared rows retain exact historical descriptor bytes and complete native
observations from the nested, deprecated, packed, enum-option and Boolean
corpora. Existing fixtures are not regenerated. Their assertion dispositions
require independent prior approval before the old rejection predicates change.
All three Rust info spellings compare exact status/stdout/stderr. Loader errors
retain the existing status-zero missing-message diagnostic; full native
schema-validation diagnostic parity remains unqualified. Snapshot, clone,
duplicate-file and failed-load retry controls preserve raw presence and lifetime.

The original CmdLine.MsgInfo and FactoryTest.DynamicMessageWithEnv mappings
remain anchors. This supplemental behavior adds no original-test credit beyond
83, no general extension-default support and no runtime or performance claim.
MSG-02 remains open. Qualification status and evidence belong in the owner's
progress report.

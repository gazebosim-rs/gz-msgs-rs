<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->
# Canonical INT64 extension defaults

Source-aware registry, factory and CLI definitions format optional proto2 INT64
extension defaults using maintained `Kind::Int64` / `Value::I64` only when the
effective value's decimal spelling equals the raw default. Zero, negatives,
values wider than INT32 and both INT64 limits retain exact native declaration
text. Default text precedes supported options. Omitted defaults stay omitted;
raw text and option presence do not change. Boolean and canonical INT32 behavior
remain unchanged.

The extension resolver uses standard decimal `i64` parsing, not the ordinary
field loader's [LP64/C23 opt-in policy](INTEGER_DEFAULTS.md). Native protobuf's pinned descriptor.cc
assigns INT64 defaults with base-zero `strto64` and ignores range errors. Native
`010` therefore means eight, while the maintained extension parser sees ten;
native overflow saturates where reflection rejects the file. Noncanonical text,
radix, whitespace, embedded NUL and overflow boundaries remain unchanged. No
parser, loader, vendor, schema, generator, dependency, default codec or pin edit.

The native-first audit retains all ten native-positive INT32 loader gaps from
the accepted INT32 corpus. A separate baseline probe captures 48 full native
descriptor/CLI observations and the pre-change Rust values/errors, including
21 canonical INT64 cases across owner/extendee layouts. It identifies nine
additional native-positive INT64 loader differences; this formatting slice does
not close either loader gap family.

The committed 129-row corpus has 79 supported cases, 24 formatter rejections and
26 loader rejections. Nineteen loader rejections are accepted natively: nine
INT64 cases plus the ten exact historical INT32 inputs. All three info spellings
assert complete status/stdout/stderr; loader failures retain the existing
status-zero missing-message diagnostic rather than claiming native validation
prose parity. Required/repeated/proto3, explicit JSON names, unsigned and other
default kinds, unknown/custom options and late DOUBLE errors keep their guards.
Declared CType/JSType, packed false and deprecated false retain native ordering;
packed true on an optional extension remains rejected. Late errors expose no
partial output. Clone, raw presence, duplicate-file and
failed-load retry controls retain snapshot lifetime and metadata semantics.

The exact historical INT32-corpus `other-kind-3` descriptor and complete native
streams are preserved as a prepared positive. Its old rejection predicate must
receive independent prior disposition before changing; historical fixture bytes
and labels stay unchanged. Standalone descriptor formatting still lacks the
registry's source metadata and retains its existing extension error.

The CmdLine.MsgInfo and FactoryTest.DynamicMessageWithEnv mappings remain
anchors. Original-test credit stays at 83; there is no general extension-default,
native parser/runtime/wire or performance qualification claim. MSG-02 remains
open. Qualification status belongs in the owner's progress report.

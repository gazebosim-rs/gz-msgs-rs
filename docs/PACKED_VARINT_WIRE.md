<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Packed integer-vector wire boundaries

This supplement qualifies canonical Int32_V, UInt32_V, Int64_V and UInt64_V.
Each has Header at tag 1 and repeated data at tag 2. It adds no parser, codec,
wire admission or runtime optimization. Existing schemas, headers, shared C++
peer, fixed32/fixed64 fixtures, their builders and old assertions are unchanged.
The separate canonical builder links the four additional generated translation
units into its own peer directory, avoiding changes to earlier bound builders.

The corpus contains 208 complete observations. Fresh canonical and installed
protobuf 3.21.12 parsers agree on status, stdout and stderr: 140 accepts and
68 rejects. Generated Rust, reflection and strict WireMessage accept 112;
compatible WireMessage accepts 124. Twelve native-only strict admissions are
wrong known wire types already handled by compatible decoding. Sixteen further
native admissions remain explicit errors in both Rust modes: ten-byte values
whose terminating byte is 0x02 or 0x7f overflow the protobuf u64 varint domain.
Native truncates their high bits; maintained prost rejects them. This supplement
records that existing difference and does not relax it.

Controls cover signed/unsigned 32/64-bit boundaries, width truncation and signed
reencoding, packed/unpacked mixtures, repeated occurrences, the 127-byte length
prefix, nonminimal encodings, empty fields, invalid/unterminated/overflowing
values and lengths, truncation and unknown fields. Header controls retain
presence, an actual timestamp, nested unknowns and malformed known payloads.
Expected integer values are independently width/sign-normalized from the test
inputs, then checked against actual native known-field output as well as Rust.
Every common compatible success forwards its original bytes verbatim; typed
reencoding matches native after unknown-field removal. Native-only overflow
outputs are separately decoded and checked without pretending Rust admits the
original bytes.

A separate Rust-only control retains existing UInt64 partial vector writes,
errors and remaining cursors for contiguous buffers and every two-chunk split.
It records suffix consumption before a delimited-limit error and the different
fast/fragmented invalid-varint cursors. These are exact maintained-runtime
observations, not native partial-state parity or a replacement decoder.

The reference helper verifies the pinned upstream schemas and protobuf version,
records the actual installed command/library identities, and binds the shared
peer, separate canonical builder, reference script and schema manifest. This is
Linux x86_64 functional evidence; release-platform qualification, full MSG-02 and
performance acceptance remain separate. Author validation passed all eight old/new
Rust checks, all 208 native observations, unchanged 61-case fixed32 and 64-case
fixed64 native replays, and all-target Clippy. Independent fresh-source review
passed all eight Rust checks, all 208 new native observations, and both unchanged
61/64 native replays. The normal strict staged and exact hosted gates remain
pending; no upstream assertion mapping changes.

<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Packed Float_V wire boundaries

This qualification covers canonical `gz.msgs.Float_V` data at tag 2. Its optional
Header is tag 1; it is not the same layout as Double_V. Production Rust decoding,
encoding and compatible-envelope behavior are unchanged. The existing C++ peer,
schema/header sources, fixed64 corpus and old assertions are unchanged. The
canonical native builder adds only Float_V's generated translation unit.

The separate corpus has 61 complete native observations. Fresh canonical and
installed C++ parsers agree on full status, stdout and stderr: 31 accepts and
30 rejects. Generated Rust, reflection and strict WireMessage accept 28;
compatible decoding accepts the same 31 as native. The three differences are
wrong known wire types that native treats as unknown, already handled by the
existing compatible path. No decoder admission is broadened.

Controls retain exact float32 bits for positive/negative zero, subnormals,
minimum normal, finite extrema, infinities and quiet/signaling NaN payloads.
They cover packed/unpacked mixing, repeated packed occurrences, the 127-byte
length-prefix boundary, nonminimal lengths, wrong/unknown fields, misalignment,
truncation and invalid/overflowing lengths. Header controls include empty,
known-Time and nested-unknown payloads, a wrong Header wire type and a malformed
known Header. Corrupted known submessages remain errors.

Successful compatible values retain the expected bits and Header presence;
their raw bytes forward verbatim. Reencoding the typed value matches native
serialization after unknown-field removal. This distinguishes raw forwarding
from generated values, which discard unknown fields.

A separate Rust-only low-level control records existing partial vector writes,
error strings and remaining cursors for each contiguous input and every two-chunk
split. It includes malformed packed values that consume suffix bytes before
reporting the limit error, and the existing contiguous/fragmented invalid-varint
cursor difference. These are maintained-runtime observations, not native
partial-state parity or a new decoder implementation.

`scripts/packed-float-wire-reference.py` verifies the pinned upstream commit,
all canonical schema hashes and protobuf 3.21.12, then builds both native paths.
Its fixture binds the shared peer, canonical builder, reference script and schema
manifest. The reference report retains actual command/library identity. This is
Linux x86_64 functional qualification, not a performance or release-matrix claim.

Author checks passed both new Rust controls, four old fixed64/wire controls,
the unchanged 64-case native fixed64 replay, all 61 new native observations and
all-target Clippy. Independent fresh-source review passed the same six Rust
checks, all 61 new native observations and the unchanged 64-case native replay.
The normal strict staged and exact hosted gates remain pending. No upstream
mapping or full MSG-02 completion credit changes.

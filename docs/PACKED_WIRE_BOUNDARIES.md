<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Packed-double wire boundaries

This supplement adds 64 actual native Double_V cases without changing the existing
70-case wire fixture or its assertions. Installed generated C++ and fresh C++
generated from the canonical schemas agree on every complete output, status and
stderr observation. The source revision, schema/peer hashes and protobuf 3.21.12
identity are recorded in tests/fixtures/packed-wire.json. The reference tool also
records the actual installed runtime library hashes in its ignored proof file.

The cases retain exact float bits: signed zeros, subnormals, finite extrema,
infinities and distinct signed quiet/signaling NaN payloads. They cover packed and
unpacked values, split/mixed occurrences, payload lengths around 127 bytes,
nonminimal length varints, valid unknown suffixes, wrong-wire known tags,
misaligned packed payloads, truncation, missing/unterminated/overflow lengths and
a declared u64 maximum length. NaNs are compared by bits rather than PartialEq.

The Rust test replays all native status/output/stderr observations, direct typed
and reflective admission, strict envelopes and the compatible envelope. Accepted
known values match both the independently constructed bit patterns and the native
known-only wire. Compatible forwarding retains every original byte. Wrong-wire
known tags keep their existing strict rejection versus native-compatible behavior.
Malformed packed fields remain errors; no partial typed value is returned by the
envelope. This is bounded parser qualification, not changed admission or complete
malformed-input/native diagnostic equivalence.

A separate test preserves existing low-level Rust partial-state observations
across every two-chunk split: prior vector values, appended bits, error text and
remaining input. A malformed declared boundary can consume suffix bytes and append
a value before reporting delimited-length-exceeded. Invalid varints can leave a
different cursor in contiguous and small-chunk paths. These are observed prost
0.14.4 behaviors; no C++ partial-state equivalence or desirable API design is
claimed. Decode optimization remains deferred. No decoder, runtime dependency,
standard no-patch consumer contract or original ledger mapping changes.

```
cargo test --locked --test packed_wire
python3 scripts/packed-wire-reference.py --upstream /path/to/pinned/gz-msgs
```

The reference tool checks all canonical schemas against the pinned upstream source,
builds the unchanged C++ peer against both installed and fresh generated messages,
and compares every output/status. --write-fixture is an explicit regeneration
operation; the default only checks observations. Author focused tests, reference replay and all-target Clippy pass. Independent
review from fresh separate source/target passed both Rust tests and all 64 installed
and fresh canonical C++ outcomes. It verified exact bits, framing, strict/compatible
distinctions and partial vector/error/tail states; original assertions and
dependencies are unchanged. The normal strict staged hook and exact hosted CI
remain pending. This does not complete MSG-02 or qualify a performance improvement.

<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Reflected packed-field kind reuse

The reflected decoder previously resolved a field descriptor's kind again for
each repeated element, including every packed double in LaserScan ranges and
intensities. The existing Value merge entry point resolves the kind once and passes it to
an internal operation. Packed and unpacked repeated elements borrow that same
immutable kind. Ordinary-field, extension and map-value callers keep their
existing entry point.

Every scalar still uses its existing prost 0.14.4 merge helper. Packed parsing
still uses the existing prost merge_loop, including its length checks, chunked
input handling, partial writes and error cursor. No parser, scalar representation,
default, reflection editing API, dependency, generated-code path or standard
prost trait identity changes. The source diff against the pinned upstream file
is retained in experiments/prost-probe/prost-reflect-packed-kind.patch; all
executable additions remain owned vendor code for coverage.

Fourteen scalar-family controls compare reflected decoding with the unchanged
prost scalar helpers on packed, unpacked, mixed, empty and malformed occurrences.
Every two-chunk split compares complete reencoded values (including float bits),
errors, remaining input and clone output. Additional controls cover a 1,080-range
LaserScan with nested pose and signed zeros, JSON roundtrip, unknown-field
forwarding, independent clone editing, clear/default state, packed proto2
extensions and map values. The original native wire fixtures, assertions and
upstream ledger remain unchanged; these controls add no original-test credit.

```
cargo test --locked --test reflected_packed --test contracts --test signed_zero
```

Author focused controls and all-target Clippy pass. Independent review, the full
strict staged gate and exact hosted acceptance remain pending.

This change does not qualify a speedup. Diagnostic release probes used the
accepted native LaserScan1080 wire with complete value/wire checks outside their
loop. Those binaries, disassembly and profiles describe intermediate prototype
revisions, not this final candidate. perf is unavailable on this host; user-space
gprofng sampling reports an interval-timer warning, so its profiles are
unqualified. Concurrent-load timings are diagnostic only. Final-source binary,
compiler and native identity review and a suitable measurement phase remain
required before any performance claim.

The accepted [0.257–0.300 LaserScan ratios](CODEC_BENCHMARK.md) measure generated
typed prost codecs. They do not measure reflection, are unaffected by this
change, and remain an open performance blocker. The earlier packed encode-helper
experiment remains unadopted because of its standard consumer type-identity
boundary. No runtime fork, generator workaround or bytemuck encode path is added.

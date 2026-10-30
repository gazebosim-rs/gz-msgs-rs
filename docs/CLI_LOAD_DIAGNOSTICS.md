<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# CLI descriptor-file diagnostics candidate

The candidate CLI reports file-open and descriptor-wire failures when external
discovery first runs, using the qualified native file-level wording. It keeps path bytes, search order, continuation
and zero exit behavior. Canonical `--info` shortcuts, help and argument errors
remain lazy. The loader remains silent for library callers.

`MessageFactory::take_initial_errors_detailed` drains the same pending initial
errors as `take_initial_errors`, without starting discovery. Its optional
`LoadDiagnostic` identifies file-read/canonical-path failures or errors returned
by maintained reflective descriptor-set wire decoding. The original `io::Error`
and its inner error are preserved. No error-message parsing or second descriptor
read/decode is used. Schema validation, traversal and safe byte-default guards
have no console mapping. Existing explicit loading and cache APIs retain their
structured-error contracts.

The Linux native helper records 80 complete byte observations at the pinned source,
CLI binary and same-environment library identities. Cases cover missing and raw
non-UTF8 paths, malformed/truncated wire bytes, actual read failure after successful open, empty valid sets, ignored paths,
directory discovery, valid/failed combinations, generated shortcuts and option
precedence. It verifies the existing native SDK canonical-name baseline. This
does not add a Rust absent-variable install-prefix fallback.

Rust checks preserve the existing sorted-list contract and Rust-owned help text;
those are explicit bindings, not byte-for-byte native stdout claims. Other
stdout and all exits are compared directly. Stderr compares byte-for-byte in
68 observations. Six schema-validation observations retain explicit differences:
native libprotobuf prints additional validation errors and a pool summary. The
existing Rust loader also admits the tested `bad-name` message identifier rejected
by native; that exact list difference is asserted, not repaired by this slice.
Six further observations retain existing descriptor-wire strictness: native
ignores tag1 encoded as varint, fixed32 or fixed64 unknown data, while the
maintained Rust loader rejects it and the CLI reports its parse failure. Generated
shortcuts remain lazy in those cases too. No decoder/admission change is made;
these Native-positive/Rust-rejected records are not claimed as console parity.
The complete native records remain unchanged by these Rust dispositions.

A separate structured-error test distinguishes malformed wire from a byte-default
safety guard and checks original inner error types, lazy retrieval and one-time
draining. Existing factory/cache/registry tests are retained. The author
run passed all 186 regular Rust tests, the lazy-environment harness, 19 isolated
install-share workflows and all-target Clippy before the supplemental wrong-wire
controls; the expanded two tests are rerun separately. Independent fresh-source/owned-target
review passed two new tests plus 14 retained factory, cache and registry tests,
the lazy-environment harness and Clippy; independently regenerated all 65 complete native observations
and five unchanged CLI originals. Supplementary independent review reran both new tests and all 80 native
observations plus five originals, preserving the previous 65 observations.
Source and assertion review found no remaining issue. Normal strict staged and exact hosted qualification remain pending. No
additional original-test credit, runtime admission expansion or performance claim
is proposed.

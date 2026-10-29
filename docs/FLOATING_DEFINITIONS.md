<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Proto2 floating declarations

Ordinary optional and required float/double defaults use the pinned protobuf
number policy over their existing reflected values. The private declaration
adapter delegates every numeric digit and rounding operation to Rust std precision
formatting. It only selects native precision and fixed/scientific text layout.
It adds no general float conversion/parser, unsafe FFI, dependency or locale change.

[Protobuf v21.12 strutil](https://github.com/protocolbuffers/protobuf/blob/v21.12/src/google/protobuf/stubs/strutil.cc)
tries 15 significant digits for doubles and 6 for floats, then 17/9 if the result
does not roundtrip. On the qualified Linux GNU runner, native float subnormals
also take the nine-digit path because safe_strtof reports ERANGE. The adapter
reproduces that policy. The exponent after rounding selects scientific notation
below -4 or at the chosen precision. Fractional trailing zeros disappear;
exponents retain their sign and at least two digits. Signed zero remains signed,
infinities use inf/-inf and NaNs print nan without a payload claim.

## Maintained formatter evaluation

Rust std Display and Ryu 1.0.23 generate shortest representations, which differ
from this native policy. A float represented as 1.2345678 prints 1.23456776 natively.
Evaluated lexical-write-float 1.0.6 precision options pad shortest digits rather
than reconstruct the exact precision requested here: the selected double
0.8455124082255701 becomes 0.84551240822557010 at minimum precision 17, while
native prints 0.84551240822557006. Its [precision documentation](https://docs.rs/lexical-write-float/latest/lexical_write_float/struct.OptionsBuilder.html)
records that padding limitation. Neither crate is added. Rust std precision
formatting supplies the needed rounding; the adapter is restricted to this
protobuf declaration surface and the selected round-to-nearest environment.

## Existing decoder boundaries

Formatting does not reparse raw defaults. Registry, standard reflection, factory
and CLI preserve their existing parser behavior and raw metadata. The native
float builder parses decimal into double and then converts to float, while
reflection parses directly into float. Eight fixture layouts retain the resulting
double-rounding difference around two long decimal midpoints. The formatter prints
its actual reflected value; it does not change parsing or wire behavior to hide
the difference.

Native hexadecimal, leading-whitespace, empty and embedded-NUL default admission
is not imported into reflection. Existing Rust rejections stay explicit. Invalid
UTF8 descriptor strings retain DecodeError. Oneof/repeated/proto3 defaults and
unknown options remain formatter errors. Omitted defaults emit no default clause.
Source JSON-name omissions remain distinct from direct reflection's synthesized
names. The earlier float-default observation remains an exact positive assertion;
all 83 upstream mappings and earlier parser/wire/safety assertions are unchanged.

## Evidence

The numeric corpus contains 21,397 actual native formatter observations: fixed
seed bit patterns, an additional thousand float subnormals, extrema, signed zeros,
nonfinite values and neighbors around notation/precision thresholds. Private Rust
checks and a freshly compiled native peer both replay every result. The separate
280-descriptor corpus contains 252 positive layouts (244 native-exact and eight
named parser differences), 18 unchanged descriptor rejections, two UTF8 decode
rejections and eight unsupported-shape/option controls. Positive layouts check
effective bits, complete raw metadata and registry/factory/CLI declarations;
explicit JSON layouts also exercise the standard descriptor pool.

The [provenance](../tests/fixtures/floating-definitions.json) records actual input
bytes, complete native process outputs, peer/compiler/library identities and
upstream source hashes. Source inspection explains the installed library's
observed behavior; it does not claim a native source rebuild. Replay with:

```sh
cargo test --locked --lib float_default
cargo test --locked --test floating_definitions --test integer_definitions
```

Other native versions/platforms, changed floating environments, broader metadata,
Messages acceptance and equal-work performance remain separate work. Independent
review, the normal strict staged hook and exact hosted CI are required.

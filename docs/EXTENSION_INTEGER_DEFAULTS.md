<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->
# Canonical integer extension defaults

Source-aware registry, factory and CLI definitions format canonical decimal
optional proto2 integer extension defaults. This adds UINT32/64, SINT32/64,
FIXED32/64 and SFIXED32/64 to the accepted INT32/64 boundary. Boolean handling
is unchanged. The maintained runtime supplies the effective typed value:

| Runtime value | Declared integer kinds |
| --- | --- |
| I32 | INT32, SINT32, SFIXED32 |
| I64 | INT64, SINT64, SFIXED64 |
| U32 | UINT32, FIXED32 |
| U64 | UINT64, FIXED64 |

The formatter uses these existing typed getters and decimal display, and
requires the complete raw default string to equal that display. Zero, signed
negative values, both signed limits, unsigned maxima and values wider than
INT32/I64 retain exact native text. Omitted defaults stay omitted; explicit zero
stays declared. Raw text, option presence and descriptor lifetimes are preserved.
No parser, loader, vendor, schema, generator, dependency, default codec or pin edit.

The pinned native descriptor.cc selects signed/unsigned 32/64 conversion by
CPPTYPE, using base-zero strto*l and ignoring range errors. Extension reflection
retains standard decimal primitive parsing; the ordinary-field
[LP64/C23 policy](INTEGER_DEFAULTS.md) does not apply. Noncanonical signs/zeros,
radix, whitespace, NUL, unsigned negative wrapping, signed narrowing and
saturation boundaries remain unchanged. There is no general native loader
equivalence claim. All nineteen previously retained INT32/64 native-positive
loader gaps remain exact controls.

The native-first audit captures 276 complete descriptor/CLI observations and
baseline Rust effective values/errors before implementation. All 132 canonical
kind/range/layout cases match, including UINT64_MAX and values beyond I64.
Native accepts nonzero JSType for the 64-bit kinds and rejects it for 32-bit
kinds; the existing width guard preserves that distinction.

The 601-row corpus covers 283 supported cases, 165 formatter rejections and 153
loader rejections. Of those loader rejections, 97 are native-positive cases,
including the nineteen preserved historical rows. Every row compares complete
native status/stdout/stderr and all three Rust info spellings. Loader failures
retain the existing status-zero missing-message diagnostic; full native schema
validation prose remains unqualified. CType/JSType, declared false flags,
default-before-option ordering, nested owners and separated extendees have
controls. Required/repeated/proto3, packed true on optional fields, explicit JSON
names, unknown/custom options and noninteger defaults retain their guards.
Late DOUBLE errors expose no partial output. Standalone descriptor formatting
retains its source-metadata boundary.

Seven historical rejected rows in the INT32/64 corpora have prepared complete
positives using their exact existing descriptors and native streams. They
represent four distinct descriptor payloads. Their old predicates and historical
fixture bytes/labels remain untouched until independent prior disposition.
Clone, raw-presence, duplicate-file and failed-load retry controls supplement
the native matrix. Scope and qualification belong in the owner's progress
report; this is no runtime/wire/performance or MSG-02 completion claim, and adds
no original-test credit beyond the existing 83 mappings.

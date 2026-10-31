<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->
# Optional slice-backed encoding

`encode_append(&message, &mut output)` appends a standard `prost::Message`
without a length delimiter. The existing prefix stays intact. A caller can clear
and reuse its vector, or append another message while tracking its boundary.
The helper does not replace generated codecs or change the standard prost trait,
default APIs, schemas, field precision, unknown-field policy or consumer pins.

The helper computes the encoded length, checks its addition to the existing
length, calls `Vec::try_reserve`, then initializes the new tail with zeros and
passes that tail to ordinary `Message::encode`. That method performs its own
length traversal and capacity check. All reservation, zero-fill, repeated length
work and checks occur within the call. Sufficient existing capacity is retained;
an empty message appends no bytes.

`EncodeAppendError` distinguishes length overflow, reservation failure, an ordinary
prost encoding error and an incomplete successful write. Every returned error
restores the original logical length and prefix; capacity can have grown. A
custom encoder that advertises more bytes than it writes returns IncompleteWrite
instead of making initialized padding look like valid message output.

This is a returned-error guarantee. It does not catch panics, undo arbitrary
custom Message side effects, provide an allocation limit or make aborting
allocators transactional. An inconsistent encoder that writes beyond the slice
can panic under its normal BufMut contract. Capacity overflow is tested without
attempting a real out-of-memory allocation; allocator failures use the standard
fallible reservation API.

An independently reviewed precursor experiment used the same initialized-slice
strategy on canonical LaserScan messages containing headers and 360 or 1080
double ranges/intensities. It measured fresh/reused encode, decode and roundtrip
operations on one AMD Ryzen 9 5950X CPU, with 18 balanced native/Vec/slice triples
per case. Source/compiler/binary identities, full wire equality, work counts,
checksums, raw samples, dispersion and paired bootstrap intervals were checked.
The 1080-element paired median encode ratios versus Vec were 3.053 and 2.939,
with descriptive 95% bootstrap intervals [2.988, 3.082] and [2.849, 3.093]. Native
remained faster in those regimes. Unchanged decode paths remained slower than
native. These observations motivated an explicit opt-in helper, not a default
codec switch or a universal performance promise.

The public helper adds fallible reservation and returned-error handling; its
exact executable has not been separately timed. A second length traversal and
zero-fill can outweigh any encoding benefit, especially for other message shapes
or small payloads. There is no new decode implementation, native performance
parity, broad robotics/simulator speedup or allocation-count equivalence claim.
Fresh and reused operations retain the capacities each runtime supports.

Native conformance controls compare all fourteen packed scalar families,
empty/boundary lengths, nonfinite and signed-zero bits, unknown enum integers,
canonical LaserScan and standard Any wires. Further controls retain existing
prefixes/capacity, consecutive message boundaries, reflective unknown fields and
rollback after custom inconsistent-length or partially written returned errors.
The normal strict production/changed-source coverage, protected staging and exact
hosted checks remain required before acceptance.

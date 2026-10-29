# Native-compatible envelope decoding

Native libprotobuf 3.21.12 accepts a known field encoded with an unexpected wire
type as an unknown occurrence. For example, `Time` bytes
`09 00 00 00 00 00 00 00 00` contain a fixed64 occurrence of its varint `sec`
field. Native parsing succeeds without setting sec. Both generated prost and
prost-reflect reject it. The original MSG-01 corpus and 83 original cases did
not cover this boundary; they do not prove complete parser equivalence.

`WireMessage::<T>::decode_compatible` keeps the normal generated-decoder success
path. On failure it uses the bundled descriptor to remove unknown occurrences
from a temporary typed-view encoding, recursively visiting known messages. It
then decodes that view with the standard generated decoder. Original bytes are
always kept verbatim, including unknown nested data, duplicates and ordering.
There is no mutable view or promise that reencoding the typed value preserves
unknowns. No vendored runtime code or dependency version changes for this API.

The sealed `CanonicalMessage` implementations bind each generated Rust type to
its canonical descriptor. A caller cannot pair an arbitrary type with a same-name
descriptor. Google Any fields use their imported descriptor; the Any.value bytes
are opaque, independent of type_url. Maps retain entry boundaries and defaults;
known-message presence and oneof occurrence order are preserved. The same
100-level default recursion budget covers known messages and unknown groups.
Caller message-size limits still apply; this API adds no transport or allocation
quota. The fallback allocates temporary nested buffers and is not a performance
improvement claim.

`prost::Message::decode`, the existing strict `WireMessage::decode`, and general
`DynamicMessage::decode` remain stricter. Reflective field storage has one known
or unknown slot per field number; a routing-only patch cannot retain both a known
value and wrong-wire unknowns with that number. This envelope API avoids a broad
runtime fork. External descriptor messages, extensions and proto2 groups are not
part of the sealed canonical corpus. Consumers must opt into this API before
claiming the newly qualified boundary; transport discovery and request decoding
are not implicitly changed by the producer API.

The 70-case fixture records actual installed generated C++ outputs/statuses and
known-field views after DiscardUnknownFields. The independent descriptor-based
C++ oracle agrees on every acceptance outcome. Fresh canonical generated C++
parsers in the Rust test replay every complete output/status. The corpus covers
wrong-wire values before/after valid values, nested fields, optional empty
presence, oneof variants and merge order, map key/value/default/duplicate cases,
standard Any fields and opaque payloads, packed/unpacked doubles, unknown scalar
and group framing, UTF-8 errors, truncation, bad tags/wire types and the recursion
boundary at 100/101, including combined known-message/unknown-group depth.

Native known-only wire encodings of empty map values may differ from prost's
default omission; tests compare the decoded typed values, including map entries,
against the native known-only view. Raw forwarding still preserves every input
byte. Rejected native diagnostic text is not qualified. This corpus is finite,
Linux/protobuf-3.21.12 based; it is not a proof of all malformed-parser behavior,
all runtime formatting, all platforms or completed MSG-02.

Reproduce with `cargo test --locked --test wire_compatibility`. With the pinned
native gz-msgs installation available, `python3 scripts/wire-reference.py`
independently replays its generated parser and writes a local provenance proof.
Full source/coverage/staged/hosted gates and independent slice review are
required before publication.

The additional [packed-double qualification](PACKED_WIRE_BOUNDARIES.md) preserves
this original 70-case corpus and adds exact float-bit, length/error and
Rust-only partial-state observations. It changes no decoder or dependency.

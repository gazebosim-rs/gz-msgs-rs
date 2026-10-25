# JSON command framing

The additional `encode-json` command consumes exactly one JSON document from
stdin. A streaming serde deserializer previously returned after the first value
without validating remaining bytes. It accepted a second value, garbage, NUL or
invalid UTF-8 after a complete object. Calling the maintained deserializer's
`end()` before encoding now rejects those suffixes while allowing ordinary
trailing JSON whitespace. Errors produce no binary stdout.

The 14-row native fixture uses protobuf3.21.12 JsonStringToMessage for Time. The
CLI test freshly replays each native complete status/output. Twelve rows compare
Rust/native success and complete output, including duplicate fields, trailing
ASCII whitespace, invalid input and comments. Two separate native-supported
syntax rows (unquoted keys and a trailing comma) were left as explicit grammar
obligations by the framing slice. The
standalone format commands are Rust additions, not original gz-msgs CLI options.
No complete JSON grammar, native diagnostic or formatting equivalence is claimed.

The earlier eight CLI tests and all original mappings remain unchanged. This
bounded correction changes no protobuf runtime, dependency or coverage exclusion.
Independent review and the full strict staged/hosted gate remain required.

`python3 scripts/json-reference.py` reproduces all 14 native outcomes and records
fixture/schema/helper/binary hashes without rewriting the golden data. The full
local follow-up gate passes 83 Rust checks, five snapshot regressions, archive QA
and clean downstream compilation: 1258/1258 owned lines and 1933/1945 regions. CLI
coverage is 61/61 lines and 123/127 regions. These include the preceding frozen
wire implementation; this packet itself adds one CLI check and no wire changes.
Matching final metadata, exact staged publication and hosted CI remain required.

Independent review reproduced all 14 native outcomes and the new Rust framing
test in a fresh reviewer-owned target. An initial replay in the producer's normal
target failed: its executable was compiled from the earlier wire snapshot, as
shown by debug compilation-directory provenance, and Cargo reported it Fresh.
That target had been shared with a reviewer build. The failing binary/logs were
retained locally; cleaning only this package's outputs and rebuilding restored
the correct executable. All nine CLI tests then passed, including the five
unchanged original C++ assertions. Source and assertions were not changed to
resolve the discrepancy. Reviewer targets must remain isolated from producer
targets; mtimes alone do not establish checked-source identity.

## Bare keys and trailing object commas

The next bounded correction quotes bare ASCII field-name tokens and removes a
trailing comma before an object close. It operates outside quoted strings and
leaves comments, repeated commas and trailing document bytes
for the existing deserializer to reject. Dynamic message decoding, duplicate
field behavior and the complete-document check remain in their existing order.

The two former grammar-gap rows now compare Rust and native output. A separate
15-row native fixture covers combined syntax, whitespace, duplicate fields and
malformed controls. `scripts/json-reference.py` replays all 29 source-bound
native rows. General protobuf JSON grammar and diagnostics remain unqualified;
the [independent review](reviews/json-grammar.json) approved this bounded
mapping after a private full test replay and additional lexical controls. The
normal staged hook and hosted qualification remain required.

## Trailing array commas

The next bounded increment applies the same trailing-comma rule to arrays of
canonical repeated fields. It removes a comma only before `]` and only after a
value, while keeping array delimiters, quoted content and the final document
check intact. The 29 previously qualified Time rows and their fixture bytes are
unchanged. A separate 14-row protobuf 3.21.12 corpus covers Double_V,
StringMsg_V and Pose_V repeated fields, including bare keys, escaped strings,
nested objects, empty arrays, repeated commas and suffix rejection. It compares
actual native/Rust acceptance and full binary output. The
[independent review](reviews/json-array-grammar.json) approved this bounded
mapping after a private 43-row replay, full Rust test run and an array-rule
negative control. General JSON grammar, diagnostic text and all-message
qualification remain open; the normal staged hook and hosted CI are required.

## Single-quoted strings

Native protobuf 3.21.12 accepts single-quoted field names and string values.
The lexical adapter now converts those delimiters to JSON double quotes outside
existing double-quoted strings, escapes literal double quotes inside them and
preserves escaped apostrophes and JSON-compatible escapes. The existing dynamic
decoder and complete-document check still decide field and value validity.

A separate 21-row native corpus compares actual acceptance and full binary
output for Time, StringMsg and StringMsg_V. It covers combined bare names and
trailing commas, apostrophes, double quotes, backslash escapes, Unicode escapes
and malformed suffixes. At that slice, two additional native-positive rows were
explicit gaps: protobuf accepts an unknown `\x` escape and a raw newline within a
string, while the bounded adapter left both for serde_json to reject. The 43 earlier
fixtures are unchanged. `scripts/json-reference.py` replays all 66 native rows.
The [independent review](reviews/json-single-quote.json) approved the bounded
21-row mapping after a private 66-row replay, full Rust test run and a
single-quote-rule negative control. General protobuf JSON grammar, diagnostics
and all-message equivalence remain open; the normal staged hook and hosted CI
are required.

## Raw control bytes inside strings

Native protobuf 3.21.12 accepts every unescaped byte `0x00..0x1f` inside both
single- and double-quoted `StringMsg.data` values. The lexical adapter now
rewrites only those bytes to JSON `\u00XX` escapes before the maintained
deserializer; the decoded string keeps each original byte. Escape pairs,
outside-string bytes, invalid UTF-8 and the complete-document check retain
their existing behavior. The raw-newline row in the unchanged single-quote
fixture now matches native output. Its unknown `\x` row remains an explicit gap.

The source-bound 64-row positive corpus covers all 32 bytes under both quote
styles. Ten native-rejected controls exercise misplaced NUL bytes, a trailing
document, unterminated strings and invalid UTF-8. `scripts/json-reference.py`
replays all 140 native rows, including the prior 66 unchanged rows, and binds
the protobuf version, fixture hashes and native peer source/binary hashes.
Unknown escapes and broader protobuf JSON grammar remain MSG-02 work.
The [independent review](reviews/json-raw-controls.json) approved this bounded
mapping after a private 140-row native/Rust replay, full Rust suite and an
embedded-NUL branch-removal control. The exact six-file source gate passes all
strict checks. Final metadata verification, the normal hook and exact hosted
CI passed on public commit `dcdbb741630d38e21ba5271a50cae04be1aee6c4`.

## Escaped ASCII bytes inside strings

Pinned protobuf 3.21.12 accepts 254 of the 256 backslash-plus-ASCII-byte
combinations inside double- and single-quoted `StringMsg.data`; an incomplete
`\u` escape is rejected under each quote style. Native consumes the slash for
unknown pairs, preserves standard JSON escapes, maps `\v` to vertical tab and
preserves the byte after an escaped raw control. The adapter now performs only
those lexical rewrites before the same dynamic decoder and complete-document
check. Bytes outside strings, malformed Unicode escapes and invalid UTF-8 are
not accepted by this change.

The new source/binary-bound fixture asserts all 256 input combinations, native
acceptance and complete binary output. The prior 140 native rows and five
fixtures remain byte-for-byte unchanged. `scripts/json-reference.py` replays
all 396 native rows. The two earlier explicit single-quote gaps now match:
raw newline was corrected by the preceding slice and unknown `\x` by this one.
Broader protobuf JSON grammar, diagnostics and MSG-02 acceptance remain open;
the strict six-file source gate passed all checks. The
[independent review](reviews/json-escaped-bytes.json) approved the bounded
mapping after a private 396-row replay, full Rust suite and a separate
unknown-escape branch-removal control. Final metadata binding, the normal
hook and exact hosted CI passed on public commit
`5fa067f47273f2ac46c8af7ba27bef66b6a3251e`.

## Escaped UTF-8 sequences inside strings

Pinned protobuf 3.21.12 consumes the backslash before a complete UTF-8
sequence inside both quote styles of `StringMsg.data`. A source/binary-bound
boundary probe covers 12 valid sequences, including two-, three- and four-byte
scalar boundaries, and ten malformed sequences in each quote style. Native
accepts the 24 valid inputs and preserves their complete bytes; it rejects the
20 malformed inputs. The published CLI rejected the valid inputs. The lexical
adapter now consumes the backslash before the first non-ASCII byte, leaving
UTF-8 validity to the unchanged JSON deserializer. Outside-string bytes,
standard escapes, `\u` parsing and document completion retain their prior path.

The new 44-row fixture asserts exact input shapes, native acceptance and full
binary output. The prior 396 rows and six fixtures are unchanged.
`scripts/json-reference.py` replays all 440 native outcomes. The focused Rust
comparison passes after an actual before-failure on U+0080. Broader Unicode
grammar, diagnostic parity and MSG-02 acceptance remain open. The exact
six-file source gate passed all strict checks, and the
[independent review](reviews/json-escaped-utf8.json) approved the bounded
mapping after a private 440-row replay, full Rust suite and a separate U+0080
branch-removal control. Final metadata binding, the normal hook and hosted CI
remain required before publication.

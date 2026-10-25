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
and malformed suffixes. Two additional native-positive rows remain explicit
gaps: protobuf accepts an unknown `\x` escape and a raw newline within a string,
while the bounded adapter leaves both for serde_json to reject. The 43 earlier
fixtures are unchanged. `scripts/json-reference.py` replays all 66 native rows.
The [independent review](reviews/json-single-quote.json) approved the bounded
21-row mapping after a private 66-row replay, full Rust test run and a
single-quote-rule negative control. General protobuf JSON grammar, diagnostics
and all-message equivalence remain open; the normal staged hook and hosted CI
are required.

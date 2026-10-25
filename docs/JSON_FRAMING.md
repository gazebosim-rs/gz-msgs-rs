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
passed on public commit `f46db31af457a2a2c815c823e46389d2cc7aaf9b`.

## Bare numbers with a trailing decimal point

Pinned protobuf 3.21.12 accepts bare numeric forms such as `7.`, `-0.` and
`7.e2` in numeric fields. The published Rust CLI rejected them. Outside
quoted strings, the adapter now removes a dot immediately after an ASCII
digit only when the preceding numeric run has no dot or exponent and the next
byte is a JSON delimiter, whitespace or exponent marker. The existing
dynamic decoder still applies the field's integer or floating-point type and
complete-document syntax. Ordinary fractions, quoted numeric strings,
invalid comments and malformed exponents remain on their existing paths.
At this slice, bare `-0` floating-point sign and quoted hexadecimal numeric
strings remained separate MSG-02 gaps.

The source-bound 63-row fixture covers Time.sec and Double_V.data scalar
forms, signed zero, exponent variants, repeated commas, object closure,
quoted controls, malformed comments and whitespace before a dot. An
independent pre-approval audit caught a second-dot error: native rejects
`1.2.e3`, `7e2.` and related forms that the initial rule accepted. Fourteen
negative rows now guard fractions and prior exponents; five native-positive
`07.`, `0x7.` and `0x7.e3` cases remain explicit Rust-negative grammar gaps.
The fixture asserts native acceptance and complete binary output. The prior 440 rows and
seven fixtures are unchanged; `scripts/json-reference.py` replays all 503
native outcomes. The focused Rust comparison first failed on `{"sec":0.}`
and passes after the correction. The exact six-file staged gate passed 109
Rust harness checks plus FactoryLazy, five original C++ CLI tests, all 503
native rows, formatting, Clippy, strict coverage and negative gates. The
[independent review](reviews/json-trailing-dot.json) approved the corrected
source and assertions after a private 503-row replay, full Rust suite and
second-dot guard mutation. The normal hook and exact hosted CI passed on public
commit `c2e15df04e8983d07879fdef0344e207996d643d`. Broad numeric grammar
and MSG-02 acceptance remain open.

## Bare integer negative zero

Pinned protobuf 3.21.12 parses an unquoted integral `-0` as positive zero in
Double_V, Float_V and Vector3d numeric fields. The published Rust CLI
preserved a negative-zero bit, including an otherwise absent Vector3d.x field.
The adapter now removes the minus only at a value boundary before the exact
integral token `0` and a delimiter or standard JSON whitespace. A decimal
point, exponent or quoted value keeps its sign.
The maintained decoder still handles field types and complete-document syntax.

A source-bound 46-row fixture covers both repeated floating types, four
delimiter/whitespace contexts, signed decimal/exponent and quoted controls,
multi-value arrays, Vector3d field presence, malformed adjacent tokens and
trailing document data. Two native-positive quoted hexadecimal numbers remain
explicit Rust-negative gaps. The prior 503 native rows and eight fixtures are
unchanged; `scripts/json-reference.py` replays all 549 native outcomes and full
binary stdout. The selected comparison first failed on `{"data":[-0]}` and
passes after the correction. The strict staged gate passes 109 Rust harness
checks plus FactoryLazy, five original C++ CLI tests, all 549 native rows,
formatting, Clippy, coverage and negative gates. The
[independent review](reviews/json-negative-zero.json) approved the exact source
and assertions after a private 549-row replay, full Rust suite, rule-removal
mutant and 234 targeted boundary cases. It found no Rust overacceptance or
matched-wire mismatch; twelve native-positive cases containing vertical tab
or form feed remain Rust-negative outside this standard-whitespace slice.
The normal hook and hosted CI passed on public commit
`11ea55557d9a6a111a440f05b9ad3b65c9a8427a`. Broader numeric grammar,
diagnostics and MSG-02 completion remain open.

## Vertical tab and form feed outside strings

Pinned protobuf 3.21.12 treats literal vertical tab and form feed as JSON
token whitespace. A 42-row native probe found 32 accepted cases rejected by
the published Rust CLI. The adapter now emits ordinary spaces for only those
two bytes outside quoted strings. Its existing bare-key and trailing-comma
lookahead recognizes them, as do the bare `-0` and integral trailing-dot
guards. Quoted raw control bytes keep their existing string-value path.

The source-bound fixture covers both bytes at 21 positions each, including
leading and trailing document whitespace, object and array separators,
trailing commas, bare keys, negative-zero wire bits, trailing dots, Time
integers, raw quoted controls, and rejected adjacent tokens, suffixes and
comments. All 549 earlier rows and nine fixture files remain byte-for-byte
unchanged. The focused comparison first failed at leading vertical tab before
the parser change and now passes. The native script replays all 591 full
status and binary stdout outcomes. The working strict gate passes 109 Rust
harness checks plus FactoryLazy, five original C++ CLI tests, formatting,
Clippy, coverage, negative gates and downstream checks. The exact six-file
staged gate also passed. The [independent review](reviews/json-vtff.json)
approved the bounded source and assertions after a private 591-row replay,
full Rust suite, normalization-removal mutant and 138 supplemental placements
with no status or wire mismatch. Final metadata binding, the normal hook and
hosted run `40213285366` passed on public commit
`60e67ade72142d71c812b180bb7628c3bddcd4a1`. This slice does not claim complete protobuf JSON
grammar, diagnostic parity or MSG-02 acceptance.

## Quoted hexadecimal floating values

Pinned protobuf 3.21.12 accepts quoted C-style hexadecimal floats in
`Double_V.data` and `Float_V.data`, including binary exponents, signed zero,
subnormals and exponentless forms. The existing decoder rejected them. This
bounded adapter recognizes the `data` array only for those two message
descriptors, validates the native hex spelling, and uses pinned `fhex 2.0.2`
for IEEE conversion. It substitutes a valid zero string while the maintained
JSON decoder validates the complete document, then writes the exact parsed
float bits into the reflected repeated field. Other message types and string
fields are unchanged. Overflow remains rejected; underflow yields zero, and
negative zero keeps its sign.

The parser choice was measured against native behavior. `lexical-core 1.0.6`
panicked in debug and miscomputed binary exponents in release; `hexponent
0.3.1` lost signed zero and minimum subnormals; `hexf-parse 0.2.1` rejected
native-valid exponentless and rounded forms. `fhex 2.0.2` matched the tested
native bits, but permits whitespace and underscores that native rejects, so
the adapter validates the spelling and rejects nonfinite conversion before
calling it. `Cargo.toml` and `Cargo.lock` pin that dependency.

The new source-bound corpus has 78 initial numeric rows and 138 fresh boundary
rows across Double_V, Float_V, Time and StringMsg. It compares acceptance and
full wire bytes, including precision ties, overflow, underflow, signed zero,
malformed exponents and unchanged string payloads. All 591 prior rows and ten
fixture files retain their hashes. Two additional native-positive duplicate
`data` key rows remain explicit Rust-negative gaps: native appends both arrays,
while the existing reflected JSON decoder retains only the last repeated key.
The adapter rejects a hex patch in that ambiguous shape rather than returning
the wrong wire bytes. The native script replays all 809 rows. A deterministic
418-case supplemental float differential found no acceptance or wire
mismatch. Broader
hexadecimal numeric placement, repeated-key parity, diagnostics and MSG-02
completion remain open. The working strict gate passes 110 Rust integration
checks, formatting, Clippy, native C++ controls, downstream and negative gates.
Coverage is 2122/2128 production lines and 3378/3403 regions, including
357/362 CLI lines and 598/611 CLI regions. The exact eight-file staged gate
also passed on tree `278a47c644417657ca9dcf7d7de4c6865ed3a626`.
The [independent review](reviews/json-quoted-hex.json) approved this bounded
source and assertions after a private native 809-row replay, 110 Rust tests,
2,656 extra lexical variants without status or wire mismatch, and a
descriptor-handling negative control. The normal hook passed on public commit
`1c3c0b802cd17fd8c1f49fac4370db3103ba9028`, and hosted run `40216034480`
passed.

## Repeated float `data` keys

The quoted-hex slice is public at `1c3c0b802cd17fd8c1f49fac4370db3103ba9028`
and its exact hosted run `40216034480` passed. Its two retained native-positive
duplicate-key rows exposed a separate reflected-decoder behavior: native
protobuf appends every occurrence of a repeated `data` field in document order,
while the maintained decoder retains only the last key. This candidate applies
only to `gz.msgs.Double_V` and `gz.msgs.Float_V`. It decodes each top-level
`data` occurrence independently and combines the reflected lists after the
same decoder validates the other fields. A scalar occurrence contributes one
element and `null` contributes none, as observed in the pinned native probe.
Quoted hexadecimal elements retain the previous descriptor-aware bit patch;
canonical `NaN`, `Infinity` and `-Infinity` are accepted, while extra
nonfinite string spellings admitted by the maintained decoder are rejected.

The new 66-row source-bound corpus compares native status and complete wire
bytes for duplicate arrays, scalar/null occurrences, three keys, escaped/bare/
single-quoted names, signed zero, malformed and missing values, trailing
commas and nonfinite spelling. It has 38 native-positive and 28 native-negative
cases.
All 809 earlier native rows and 11 fixture files retain their exact hashes;
`scripts/json-reference.py` replays 875 outcomes. A separate deterministic
722-case two-value cross-product found no status or wire mismatch after the
nonfinite guard. General duplicate-field behavior, other descriptors, full
protobuf JSON grammar, diagnostics and MSG-02 acceptance remain open.
The [independent review](reviews/json-repeated-data.json) approved this
bounded source and assertions after a private 875-row native replay, 111 Rust
tests, 1,170 further variants with no mismatch and an append-removal control.
The exact seven-file staged gate, normal hook and hosted run `40218476323`
passed on public commit `0d267506915119344467a56956ddb4408777158c`.

## Repeated integer `data` keys

The same occurrence-by-occurrence reflected decoding now applies to top-level
`data` keys of `Int32_V`, `UInt32_V` and `Int64_V`. Native protobuf appends
each repeated occurrence in source order; the maintained decoder otherwise
keeps the last key. Arrays, scalar values and `null` retain native behavior,
while the full document still validates `header` and trailing input. A
117-row pinned C++ fixture covers 82 accepted and 35 rejected cases across
the three descriptors, including integer bounds, quoted numbers, permissive
keys, commas and malformed values. All 875 prior rows and 12 prior JSON
fixture hashes remain unchanged; the native replay now checks 992 outcomes.
The [independent review](reviews/json-repeated-integer.json) approved the
source and assertions after a private 992-row replay, 112 Rust tests, 270
supplemental pairs without mismatch and an Int32 admission-removal mutant.
The exact six-file staged gate passed on tree
`0b3832bd5eb01f40c036324b1d20125d0131cfba`. Final metadata binding,
the normal hook and hosted CI remain pending.
`UInt64_V` remains outside this slice because the maintained decoder saturates
an unquoted value above `u64::MAX` that native rejects. Other repeated fields,
single-key scalar values, general duplicate-field behavior and MSG-02 remain
open.

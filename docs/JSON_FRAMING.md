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
`07.`, `0x7.` and `0x7.e3` cases were explicit Rust-negative grammar gaps at
that slice.
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
The exact seven-file staged gate, normal hook and hosted run `40220925335`
passed on public commit `8ab936c491305e7c4517225d9e58ef162081ef9e`.
`UInt64_V` remains outside this slice because the maintained decoder saturates
an unquoted value above `u64::MAX` that native rejects. Other repeated fields,
single-key scalar values, general duplicate-field behavior and MSG-02 remain
open.

## Repeated string `data` keys

The same bounded occurrence path now handles `StringMsg_V.data`, retaining
each array or scalar string in document order and letting the maintained
decoder validate strings and the complete document. The pinned 46-row corpus
has 34 accepted and 12 rejected cases across empty/null values, escaped and
literal Unicode, raw controls, alternate key syntax, headers and malformed
input. All 992 earlier native rows and 13 fixture hashes remain byte-identical;
the native replay checks 1,038 outcomes. Other repeated fields and broader
JSON/MSG-02 contracts remain open. An additional 400-pair native/Rust sweep
found no acceptance or wire mismatch across the stated string forms.
The working strict gate passed 113 Rust tests and all five original C++ CLI
tests, with root and CLI production line and region coverage above 95%. An
isolated StringMsg_V admission-removal mutant failed the exact two-array wire
assertion.
The [independent review](reviews/json-repeated-string.json) reproduced all
1,038 native outcomes in a private source and target, passed the selected Rust
test, and independently failed the descriptor-removal mutant on `two_arrays`.
The exact six-file staged gate passed on tree
`88519e778897ad719cdf4a4605bb2b47c14cf1b9`. Final metadata binding
and the normal hook passed on public commit
`115b467f612c41b9c0b4706b5032a4280a731e41`; exact hosted run
`40225573548` passed.

## Single scalar repeated `data`

For the same six reviewed repeated descriptors (`Double_V`, `Float_V`,
`Int32_V`, `UInt32_V`, `Int64_V` and `StringMsg_V`), the occurrence path now
accepts one scalar `data` value. Native protobuf treats it as a one-element
repeated list; the maintained Rust decoder otherwise rejects it. One-key
arrays and `null` keep their previous decoder path. Quoted hexadecimal float
scalars reuse the existing `fhex` conversion and spelling guard. The complete
document is still checked by the maintained reflected decoder.

The pinned C++ fixture contains 95 matching rows (64 accepted, 31 rejected)
across all six descriptors, plus two explicit native-positive/Rust-negative
unquoted `0x7.` float scalar gaps. The native replay checks all 1,135 rows:
1,038 retained outcomes plus those 97 new controls. Fourteen earlier fixture
hashes are unchanged. Native rejects unquoted `p`-exponent hex; this slice
does not treat the quoted hex parser as a general unquoted numeric parser.
Broader repeated descriptors, JSON grammar, diagnostics and MSG-02 acceptance
remain open. The full working strict gate passed 114 Rust tests, the five
retained C++ CLI tests, root coverage 2239/2249 lines and 3599/3632 regions,
and CLI coverage 474/483 lines and 819/840 regions. A private raw-parent
negative control fails the new scalar assertion on `{"data":7}`.
The [independent review](reviews/json-single-scalar.json) approved the exact
six-file source freeze and privately replayed all 1,135 native outcomes.
Its 348-case supplemental differential found no status or accepted-wire
mismatch. The exact staged gate passed on tree
`6295fc2f7468ed452dbaf83898bcb996a649b0c5`.
Final metadata binding and the normal protected hook passed on public commit
`6dc8845eb8584c16832aebe19f635cc0e7d34275`; exact hosted run
`40227905407` passed.

## UInt64 array numeric precision and range

For `gz.msgs.UInt64_V` top-level `data` arrays, the pinned maintained
`serde_json` raw-value feature preserves numeric token spelling through the
existing protobuf JSON normalizer. Integer tokens use exact `u64` parsing.
Decimal and exponent tokens use Rust's float parser with native-style finite,
integral and `<2^64` checks. Only numeric array elements are replaced after
the maintained reflected decoder validates the whole document. This prevents
overflow saturation and large-decimal wire rounding differences. Scalars,
duplicate keys, nested null/empty elements and other descriptors retain their
prior behavior.

The pinned native fixture adds 107 matching and 33 explicitly unqualified
rows to 1,135 unchanged prior cases, for 1,275 native outcomes. Fifteen earlier
fixture hashes are unchanged. A supplemental 674-form numeric sweep found no
remaining status or accepted-wire mismatch for the bounded array-number path.
This prerequisite does not qualify UInt64 scalar/duplicate admission or broad
MSG-02 JSON behavior. The working strict gate passed 115 Rust checks, five
unchanged native C++ CLI tests, root coverage 2279/2290 lines and 3673/3709
regions, and CLI coverage 514/524 lines and 893/917 regions. The exact public
parent parser fails the new overflow assertion on
`{"data":[18446744073709551616]}`. The [independent review](reviews/json-uint64-range.json)
privately reproduced the 1,275-row native proof and 674 numeric controls,
then failed that exact overflow assertion against the public-parent parser.
The exact staged gate passed on tree
`932109277ea195f23f36284809bebdc07f59e06f`.
Final metadata binding and the normal protected hook passed on public commit
`cf0c44de2a40fe52310c03b7261abaea13103212`; exact hosted run
`40229507091` passed.

## UInt64 scalar, nested, null and repeated `data`

The bounded `UInt64_V` occurrence path now accepts one scalar or several
top-level `data` keys. It flattens nested arrays in source order, omits `null`
elements and appends values from repeated keys. Raw numeric tokens continue
through the reviewed range and precision parser before the maintained
reflected decoder validates each fragment and the complete document. The
path is limited to `UInt64_V`; other descriptors keep their existing decode
path.

The pinned native fixture has 185 parity forms (119 accepted and 66 rejected)
plus eight explicitly native-only unquoted hexadecimal trailing-dot forms
such as `{"data":0x7.}`. Those eight remain Rust-negative, including scalar,
array, nested and duplicate-key placements. The parity forms cover scalar
strings and numbers, nested arrays, nulls, duplicate keys, overflow,
exponents and unrelated-field errors. All 1,275 prior native rows and 16
prior fixture hashes are unchanged, bringing the replay to 1,468 outcomes.
The earlier UInt64 range fixture remains byte-identical: its 33 recorded
native-positive/Rust-mismatch rows now assert the historical mismatch and
require the new result to equal native status and wire bytes. Separate 171-form
and 568-form scalar/nested/duplicate characterizations found no mismatch
within their declared forms. The public-parent parser fails the first new
native-positive scalar assertion `{"data":"1"}`. The working strict
gate passed 116 Rust tests, the five unchanged native C++ CLI tests, root
coverage 2310/2322 lines and 3740/3779 regions, and CLI coverage 545/556
lines and 960/987 regions. Native-only hex forms, broader JSON grammar, other
descriptors and MSG-02 completion remain open. The
[independent review](reviews/json-uint64-occurrences.json) approved the V2
source and assertions after a separate 1,468-row native replay, eight hex-gap
checks and an admission-removal mutant that failed the first scalar case. The
exact six-file staged gate passed on tree
`b5bef5195548044f74a3d50265a28aac79f4cddc`. Final metadata binding and
the normal protected hook passed on public commit
`755c185e3f9cb4c341d07fc3c6a9092608c8ba7c`; hosted run `40232275992`
passed.

## UInt64 unquoted hexadecimal trailing-dot values

The pinned protobuf 3.21.12 [numeric lexer](https://github.com/protocolbuffers/protobuf/blob/v3.21.12/src/google/protobuf/util/internal/json_stream_parser.cc)
starts a number at an ASCII digit or `-`, scans digits plus `.`, `e`, `E`,
`+`, `-` and `x`, and stops at other letters. A dot or `e`/`E` selects
`safe_strtod`; undotted positive `0x` without an `e`/`E` digit is rejected. Its
[UInt64 conversion](https://github.com/protocolbuffers/protobuf/blob/v3.21.12/src/google/protobuf/util/internal/datapiece.cc)
checks the parsed double's integer conversion. The bounded corpus confirms
that this path accepts some unquoted hexadecimal
`UInt64_V.data` values with a lowercase `0x`, a mantissa of ASCII decimal
digits or `e`/`E` before a dot, and an integral finite result below `2^64`.
Here `e` and `E` are hex digits: native accepts `0xe.` and `0x7e1.`. It
rejects an undotted `0x7`, alphabetic digits other than `e`/`E` on either
side of the dot, uppercase `0X`, signs, `p` exponents and fractional results.
At that review checkpoint, the undotted `0xe` and `0x7e1` were explicit gaps.
The allowed fractional digits can round away, as `0x7.000000000000001`
and a larger `0x9878635522291.e1` do in the pinned native parser.

The `UInt64_V` top-level data adapter recognizes that spelling only outside
strings and uses the existing pinned `fhex 2.0.2` conversion followed by
finite, integral and range checks. It writes the resulting decimal integer
into the normalized JSON before the maintained raw-value and reflected
decoders validate each occurrence and the full document. Other descriptors,
quoted strings and unrelated fields retain their existing path.

The corrected fixture has 220 bounded native status and full-wire controls across
scalar, array, duplicate and nested data placements: 116 accepted and 104
rejected, plus two separately asserted native-only undotted gaps. Twelve
rejected concatenations such as `10x7.` ensure the rewrite
starts at a value boundary. The first frozen source omitted that guard; the
second omitted native `e`/`E` mantissas. Both were held without approval.
The native replay retains all 1,468 earlier outcomes and 17 earlier fixture
hashes. The prior occurrence fixture remains byte-identical; its eight
native-positive/Rust-negative hex observations remain historical evidence,
and their test now requires current native parity. Broader hexadecimal and
JSON grammar, diagnostics and MSG-02 acceptance remain open. The exact
six-file staged gate passed with 2356/2368 root lines and 3816/3857 regions;
the [independent review](reviews/json-uint64-unquoted-hex.json) approved the
bounded mapping after a private 1,690-row native replay and numeric-boundary
controls. The protected normal hook passed on public `387b859d0ca7b43555b5ef7d7a1d5db0711b8420`;
hosted run `40236392686` passed.

## UInt64 undotted hexadecimal values containing e/E

The pinned protobuf 3.21.12 numeric lexer selects its double conversion when
the scanned number contains `e` or `E`, even without a dot. For a lowercase
`0x` token, those bytes can be hexadecimal mantissa digits. Native therefore
accepts `0xe`, `0x7e1`, and matching uppercase-`E` digits, but rejects an
undotted `0x7`, other alphabetic hex digits, an uppercase `0X` prefix, signs,
and trailing exponent syntax such as `0x1e+0`. The adapter extends only the
existing `UInt64_V.data` value-start guard: an undotted token requires an
`e`/`E` digit, then the same pinned `fhex 2.0.2` finite, integral and
`<2^64` check. Quoted values and other descriptors are unchanged.

The new 152-row scalar, array, duplicate and nested fixture records 76 native
accepted and 76 rejected outcomes. It preserves all 1,690 earlier rows and
the 18 earlier fixture hashes. The two old undotted gap rows remain
byte-identical historical observations and now require exact native status and
wire parity. The combined native replay checks 1,842 outcomes. Broader
protobuf JSON grammar, diagnostics and MSG-02 acceptance remain open.

The exact six-file staged verifier passed 118 Rust tests with 2364/2376 root
lines and 3827/3868 regions. The
[independent review](reviews/json-uint64-undotted-hex.json) approved this
bounded source and fixture after a private 1,842-row native replay and a
failing undotted-branch mutant. The normal protected hook passed on public
`81f38c8225453a3c36b9b4a2bfee3fe7dc779a36`; hosted run `40237849828`
passed.

## Unquoted hexadecimal floating data

Pinned protobuf 3.21.12 starts numeric tokens at an ASCII digit or `-` and
selects its double conversion when the token contains a dot or `e`/`E`. For
`Double_V` and `Float_V` top-level `data`, a lowercase `0x` token can therefore
carry decimal digits and hex `e`/`E` digits around a dot, or an undotted
mantissa containing `e`/`E`. Native accepts signed `-0x7.`, `0x.7`, `0x7.e3`
and `0x7e1`. It rejects uppercase `0X`, other alphabetic hex digits,
`p` exponents, and malformed exponent suffixes. The adapter recognizes only
those descriptor and value positions and uses pinned `fhex 2.0.2` for finite
IEEE conversion. Scalar and repeated-array values use the existing reflected
bit-patch path, so signed zero and float precision survive JSON normalization.
Other descriptors, strings and unquoted decimal grammar retain their paths.

A new fixture binds 318 matching native status/full-wire observations, including
signed zero, precision, underflow and overflow boundaries, and six explicit
native-positive `07.` leading-zero rows outside this hex slice. The unchanged
historical Double_V/Float_V `0x7.` rows now require current parity. All 1,842
older native outcomes and 19 prior fixture hashes remain exact; the combined
native replay checks 2,166 outcomes. General JSON grammar, diagnostic parity
and MSG-02 acceptance remain open. The exact six-file staged gate passed 119
Rust checks and five original C++ CLI cases. An independent
[review](reviews/json-float-unquoted-hex.json) reproduced the 2,166 native
outcomes, found only the six declared leading-zero gaps in a separate 420-case
differential, and approved the bounded source and assertions. Final metadata
binding and the normal protected hook passed on public
`52895625cadb6a226b20579b3e83a8be0b6f9f12`; hosted run `40243304259`
passed.

## Leading-zero floating data values

Pinned protobuf 3.21.12 `JsonStreamParser::ParseNumberHelper` scans a numeric
token through digits, `.`, `e`, `E`, signs and lowercase `x`. A dot or `e`/`E`
selects `safe_strtod` before the plain-integer leading-zero rejection. Thus
`07.`, `007e2`, `-07.`, and `00.5` are accepted floating values, while bare
`07`, malformed exponents and an unquoted leading `+` are rejected. This slice
adapts only top-level `Double_V` and `Float_V` `data` values at scalar, array
and repeated-key starts. Rust's decimal float parser preserves the native
rounding on measured long-mantissa boundaries where the maintained JSON
decoder differs by one ULP. The adapter uses the same reflected-value patch
path as the preceding hexadecimal slice, retaining exact float bits and signed
zero. Other descriptors and unrelated JSON positions are unchanged.

The source-bound fixture has 252 status/full-wire rows, 180 native accepted and
72 rejected. The six earlier native-only `07.` rows remain byte-identical
historical observations and now require native parity. Six native-positive
`Time.sec` forms were explicit Rust-negative gaps at that slice. All 2,166 earlier native
observations and 20 earlier fixture hashes remain exact; the combined native
replay checks 2,424 outcomes. A 252-form quoted-input sweep found no behavior
changes against the public parent; six quoted-array overflow gaps predate this
unquoted slice. General JSON grammar, Time numeric parsing, diagnostics and MSG-02
acceptance remain open. The first staged attempt exposed a stale Rust-negative
expectation for the unchanged native-positive `Double_V 07.` row; the corrected
test now requires exact wire parity. The source staged gate passed 120 Rust
checks, five unchanged original C++ CLI cases and strict coverage. An
independent [review](reviews/json-leading-zero.json) reproduced all 2,424
native outcomes and verified the 252 quoted status/wire controls against
separately built parent and candidate binaries. The corrected source and
assertions are approved; the first candidate tree is superseded. The final
staged gate and protected hook passed on public
`17b0b9b20d3886fe19eb274c0525afbc844cc274`, with hosted run
`40245881573` successful.

## Time leading-zero numeric values

Pinned protobuf 3.21.12 scans an unquoted numeric token through digits,
`.`, `e`, `E`, signs and lowercase `x`. A dot or `e`/`E` selects double
conversion before the bare leading-zero rejection. The pinned object
writer passes that double through `DataPiece::ToInt64` for `Time.sec` and
`ToInt32` for `Time.nsec`, accepting integral values in the field's range.
This bounded adapter applies only to those two top-level fields and uses
the existing decimal and hexadecimal float token recognizers. It writes
the integral decimal value for the maintained descriptor decoder.

The new fixture records 139 distinct status/full-wire controls: 71 native
accepted and 68 rejected. Sixty-one accepted rows were Rust-negative on
the public parent; ten already matched. Combined and duplicate fields,
quoted values, unknown fields, fractions and measured overflow are controls.
The six native-positive `Time`
rows in the unchanged leading-zero fixture now require exact status and
wire parity. The unchanged trailing-dot fixture has two overlapping `Time`
rows; the first exact staged gate exposed their stale Rust-negative assertion,
which now requires native parity. Twenty-one prior fixture hashes and all 2,424
prior native outcomes remain protected; the combined pinned replay checks
2,563 observations, with 2,562 distinct type/input pairs because one new
`Time.sec` `7.0` observation overlaps an older fixture.
Quoted values, other descriptors, malformed suffixes and nonintegral or
measured out-of-range conversions stay on their existing paths. General JSON
grammar, diagnostics and full MSG-02 acceptance remain open. An independent
[review](reviews/json-time-float.json) rebuilt the pinned peer, reproduced
all 2,563 outcomes, passed 24 private CLI tests and found no mismatch in
120 additional bounded forms. The source gate passed 121 Rust tests and
strict coverage. The final staged gate and protected hook passed on public
`a6123a61606bedfa8a37427c32a0db4bbc155768`; hosted run `40249108460`
passed.

## Quoted floating data range and special-string admission

Pinned protobuf 3.21.12 accepts the exact strings `Infinity`, `-Infinity`
and `NaN`. For other quoted floating data it converts through `safe_strtod`;
`Float_V` also rejects a double value outside the finite float range before
narrowing. This adapter checks decoded top-level `Double_V` and `Float_V`
`data` strings in scalar, array and repeated-key positions for overflow,
noncanonical nonfinite spelling and the quoted `Float_V` pre-narrowing bound.
Hex strings keep the existing `fhex` conversion path. The maintained decoder
still validates the document and unrelated fields.

The fixture records 144 status/full-wire observations: 65 native accepted
and 79 rejected. Thirty-one rows exposed Rust-positive/native-negative
admission gaps on the public parent, including decimal overflow,
noncanonical nonfinite spellings and a `Float_V` boundary that would round
to finite max after narrowing. Canonical special strings, finite boundaries,
underflow, escaped strings, duplicate data and unrelated descriptors are
controls. A 108-form sweep inserts one decoded C-whitespace character using
JSON escapes at prefix, suffix and middle positions for both descriptors and
three data placements. Native accepts tab, LF, CR, VT and FF at the edges,
leaving 60 native-positive/Rust-negative controls; spaces at the edges and
all middle insertions form 48 exact rejected controls. Whitespace admission
remains outside this range/spelling slice. Twenty-two prior fixture hashes
and all 2,563 preceding native observations remain protected; the combined
replay checks 2,815 observations. Broader quoted numeric grammar, diagnostic
parity and MSG-02 acceptance remain open.
The exact six-file staged source gate passed 122 Rust checks and strict
production coverage. An independent [review](reviews/json-quoted-float.json)
rebuilt the pinned peer, reproduced all 2,815 observations, checked the
108-form whitespace disposition and showed that disabling the admission guard
fails an exact native-negative overflow row. The first two candidate packets
and a scheduling-interrupted gate are retained but superseded. Final metadata
and the normal protected hook passed on public
`47f44ddb30482ccd226d30f681ca5fc788da89e0`; hosted run `40253495539`
passed.

## Unquoted Float_V maximum rounding

The native unquoted number path parses finite double first, then
`DataPiece::DoubleToFloat` can round a value just above `float` max back to
finite max by halving and doubling. Native accepts unquoted `3.4028235e38`
but rejects that quoted string; `3.4028234663852886e38`, which parses to the
double value of f32 max, is accepted in both forms. The quoted path remains
unchanged. For top-level unquoted `Float_V.data`, the
adapter validates the JSON number token, applies the same bounded half-value
check and patches only the matched scalar or array element. Duplicate data
occurrences keep append order. The maintained decoder validates the rest of
the document.

Of 222 new native status/full-wire observations, 204 require exact parity.
The other 18 retained native-positive/Rust-positive `Double_V` one-bit
decimal differences in six tokens and three placements as historical parent
evidence; the subsequent bounded Double decimal slice requires their native
wire parity. The 23 older fixture hashes and 2,815 earlier
native observations remain unchanged; combined replay covers 3,037 rows.
Other floating precision boundaries, quoted whitespace, general grammar and
MSG-02 acceptance remain open. The first staged run exposed a test setup race
with another test's native peer build and is retained as a failed gate. The
corrected verifier replays all 3,037 native rows before instrumented Rust
tests; the exact staged source gate passed 123 Rust checks with strict
production coverage. An independent [review](reviews/json-float-max-range.json)
reproduced all 3,037 native observations, found no mismatch in 63 additional
unquoted `Float_V` boundary forms, kept 27 quoted forms byte-identical to the
public parent and made both parent and guard-removal variants fail an exact
max-range row. The final staged gate checks the metadata-integrated tree;
protected-hook and hosted results are reported in the public progress tracker.


## Cross-host native replay

The original fixture executable SHA identifies the machine that produced the
retained reference. The native replay builds the same pinned C++ source on
its current host, records that build's SHA, and checks all 3,037 status and
full-wire results against the fixtures. It also requires all 18 historical
fixture peer SHAs to agree. This keeps exact behavioral comparison while
allowing executable bytes to vary by compiler and build options. Hosted run
`40257819363` exposed the previous cross-host ELF equality assertion before
native replay began. The [independent review](reviews/json-reference-portable.json)
replayed the exact 3,037 rows with a differently hashed ELF and rejected a
changed historical fixture hash. The repaired run remains a separate hosted
qualification step.


## Unquoted Double_V decimal rounding

The pinned native JSON converter and protobuf's locale-independent double
parser agree with Rust's standard `f64` parser on a measured 427 positive
values from 460 deterministic dot/exponent tokens. The maintained JSON
number decoder differed by one ULP on 108. For top-level unquoted
`Double_V.data`, the adapter validates a complete JSON Number token and
patches the decoded value only when its bits differ from the finite standard
parser result. Scalar, array and repeated-key values retain order. Other
descriptors, quoted values and bare integers are untouched by this adapter.

The 135-row native fixture has 90 corrected, 18 already matching, 18
rejected and nine descriptor-isolation observations. The prior 18 Double
wire-gap rows stay unchanged as historical fixture bytes and now assert exact
native parity. The mandatory native replay covers 3,172 complete status/wire
outcomes. The 460-token candidate sweep found zero status or wire mismatch.
The exact source staged gate passed 124 Rust checks. An independent
[review](reviews/json-double-decimal.json) reproduced all 3,172 native
outcomes, rehashed the 24 earlier fixtures, and made a Double_V callsite
removal fail the first exact one-bit assertion. Broader JSON grammar, quoted
whitespace and MSG-02 remain open. Final metadata gate, protected hook and
hosted qualification remain separate.

## Quoted Float_V and Double_V edge whitespace

Pinned protobuf 3.21.12 rejects a literal ASCII space at either end of a
quoted numeric string before conversion. Its numeric conversion accepts tab,
line feed, carriage return, vertical tab and form feed at the edges of finite
decimal numbers. The CLI now admits those five edge bytes, including adjacent
spaces, only for top-level `Float_V.data` and `Double_V.data`. It parses the
trimmed decimal as f64, then patches the decoded field at native precision.
Literal edge spaces, interior whitespace, unrelated
descriptors, quoted hex values and broader quoted-number grammar are outside
this correction.

The 1,104-observation native fixture covers both descriptors, `1`, signed zero, a
small finite exponent and f32 midpoint boundaries, plus scalar, array and
repeated-key placements, six whitespace bytes and single-quote controls. It
has 720 native-positive full-wire rows and
384 jointly rejected rows. The earlier 60 native-positive whitespace rows
remain byte-identical historical evidence and now require exact parity; the
earlier 48 rejected controls remain unchanged. The revised native reference
replayed 4,276 exact status/full-wire outcomes and focused Rust tests pass.
The exact staged gate passed 28 CLI tests and strict greater-than-95% owned
coverage. An independent [review](reviews/json-quoted-float-whitespace.json)
replayed all 4,276 native outcomes from a private raw source export and
rehashed 25 older fixtures. The fixture has 1,032 distinct type-and-input
pairs. The previous source tree passed 4,060 native rows but failed the
one-bit midpoint witness, so it is excluded from acceptance. The final
metadata gate, protected hook and hosted qualification remain separate.

## Quoted hexadecimal float edge whitespace

Pinned protobuf 3.21.12 rejects a literal space as the first or last byte of
a quoted numeric string, then accepts the other ASCII C whitespace bytes at
the edges of a finite hexadecimal float. The descriptor-scoped CLI path now
trims those edges only for top-level `Double_V.data` and `Float_V.data`, uses
the existing maintained `fhex` conversion to f64, and narrows to f32 after
the double conversion. This ordering preserves the pinned Float midpoint
wire bits.

The 768-observation native fixture covers both descriptors and double- and
single-quoted strings in scalar, array and repeated-key placements. Its 360
native-positive and 408 rejected observations include lowercase
and uppercase prefixes, a signed value, a subnormal, the f32 midpoint, edge
spaces after another accepted whitespace byte, and rejected literal edge
spaces, malformed tokens, overflow and interior whitespace. All 26 older
JSON fixture hashes remain unchanged. The exact staged gate passed 5,044
native status/full-wire observations, 29 CLI tests and strict
greater-than-95% owned coverage. The independent
[review](reviews/json-quoted-hex-whitespace.json) replayed the full native
proof from a separate raw export and passed the selected Rust test. The
normal protected hook and hosted qualification follow separately. Other
descriptors, nonfinite spellings, unquoted hex and broader JSON grammar remain
outside this slice.

## Quoted Vector3d numeric components

Pinned protobuf 3.21.12 accepts finite quoted hexadecimal values and decimal
values with non-space ASCII C whitespace at the edges of top-level
`gz.msgs.Vector3d` `x`, `y` and `z`. The CLI now applies the existing
double-precision conversion only to those three fields, then patches their
decoded numeric values. It leaves `StringMsg` text and `Time` integers on
their existing paths. A later non-null occurrence of the same component
takes the last value; `null` leaves the earlier value in place, as in the
pinned native parser.

The 319-observation fixture records exact status and wire bytes for 126
native-positive observations and 193 rejected controls. It includes double
and single quotes, bare and escaped keys, edge whitespace, literal edge-space
rejection, signed zero, a decimal one-bit boundary, component presence,
three-component messages,
duplicate keys including a later `null`, overflow and malformed values. All
27 earlier JSON fixture hashes are pinned unchanged. The exact staged gate
passed 5,363 native status/full-wire outcomes, 127 Rust tests including 30
CLI tests, and strict greater-than-95% coverage for every production source.
An independent [review](reviews/json-vector3d-quoted.json) reproduced all
5,363 native outcomes from a separate raw export and passed the selected
319-row Rust test. The protected hook and hosted qualification remain
separate. Nested numeric fields, other message types and broader JSON grammar
remain open.

## Quoted Quaternion numeric components

Pinned protobuf 3.21.12 accepts the same finite quoted hexadecimal and
decimal conversion for top-level `gz.msgs.Quaternion` `x`, `y`, `z` and `w`.
The CLI extends the reviewed Vector3d component path to those four fields,
converts at double precision, and patches only the decoded component values.
Its descriptor gate leaves Vector3d's three-field behavior and StringMsg/Time
handling intact. A later non-null occurrence replaces an earlier value;
`null` retains the earlier component. Signed-zero presence is tested
separately.

The source-bound 408-observation fixture records 155 native-positive exact
wires and 253 rejected controls. It covers all four components, double and
single quotes, finite hex and decimal spellings, edge and interior whitespace,
literal edge spaces, overflow, signed zero, a one-bit decimal boundary,
multiple components, duplicate/null keys, bare and escaped keys, and
Vector3d/StringMsg/Time controls. All 28 earlier JSON fixture hashes remain
unchanged. The selected 408-row Rust test and fresh pinned native fixture
generation passed. The exact staged gate passed 5,771 native status/full-wire
outcomes, 128 Rust tests including 31 CLI tests, and strict greater-than-95%
owned coverage. An independent [review](reviews/json-quaternion-quoted.json)
replayed all native outcomes and the selected 408-row Rust test from a
separate raw export. The first gate stopped at formatting before tests and
was corrected before this passing gate. Protected hook and hosted
qualification remain separate. Nested numeric fields and general JSON
grammar remain open.

## Quoted numeric components in one Pose child object

The pinned protobuf 3.21.12 parser accepts quoted finite hexadecimal and
decimal values under one top-level `gz.msgs.Pose.position` Vector3d object
and one `Pose.orientation` Quaternion object. The CLI extends the reviewed
component conversion to those two nested paths, then patches the decoded
double-precision child fields. It leaves unrelated nesting and other
descriptors on their existing paths. A later non-null occurrence of one
child field replaces its earlier value; `null` retains it.

The 694 matching native observations cover `position.x/y/z` and
`orientation.x/y/z/w`, double and single quotes, finite hex and decimal,
edge/interior whitespace, literal spaces, overflow, signed zero, one-bit
decimal rounding, multiple children, bare/escaped keys, duplicate child
fields and null. They include 257 native-positive exact wires and 437
rejections; all 29 earlier JSON fixture hashes remain unchanged. At the
`896073f` source boundary, four native-positive repeated-parent message-merge
forms were explicitly pending: two quoted forms were rejected, while two
plain or mixed forms retained the decoder's wrong wire. The fixture pins
those four native outcomes and the Rust dispositions at that boundary.
That slice did not establish repeated-parent merge, deeper Pose containers
or general JSON grammar parity. The exact source staged gate and independent
[review](reviews/json-pose-quoted.json) passed: 129 Rust tests including 32
CLI, strict owned coverage above 95%, and all 6,469 pinned native status
and full-wire outcomes reproduced. The protected hook and hosted
qualification remain separate.

## Direct scalar fields across repeated top-level Pose child objects

The maintained reflection decoder keeps the final duplicate JSON message
key. Native protobuf preserves direct scalar fields across repeated
`Pose.position` or `Pose.orientation` objects; a later non-null scalar field
replaces the earlier value, while `null` leaves it intact. The scoped adapter
records the normalized byte range of each top-level parent value and, only
when that parent repeats, combines object members in occurrence order into
the final parent object before decoding. Invalid earlier objects remain in
the input and are still rejected. Quoted direct scalar values retain the
existing double-precision patch path.

The 54 distinct Pose type/input observations include 47 native-positive
exact wires and seven rejected controls. They cover direct `x/y/z/w` fields,
numeric and quoted values, null and empty objects, three occurrences, both
parent types, escaped/bare/single-quoted keys, cross-parent interleaving,
and invalid earlier values. The four native-positive scalar merge cases
recorded as pending in the prior Pose fixture now match exactly. All 30 prior
JSON fixture hashes remain unchanged. The native peer exits with code 6 for
seven rejected controls while the Rust CLI exits with code 1; acceptance and
wire parity are qualified here, while diagnostic-code parity remains open.
The exact staged source gate passed 130 Rust tests including 33 CLI, strict
owned coverage above 95% and 6,523 pinned native outcomes. The independent
[review](reviews/json-pose-parent-merge.json) reproduced the native proof and
selected Rust tests from separate raw exports with exclusive Cargo targets.

At public `65958f65`, this correction did not merge nested Header fragments. For example,
`{"position":{"header":{"stamp":{"sec":1}}},"position":{"header":{"stamp":{"nsec":2}}}}`
is accepted by both parsers, but native emits `22080a060a0408011002` and Rust
emits `22060a040a021002`. Rust loses the earlier `sec` field, just as the
public parent did. Independent orientation and combined-parent witnesses
confirm the same gap. Nested Header/stamp merging remains pending; the
qualified scope is direct scalar-child fields. The normal protected hook and
exact hosted run `40291464209` passed on public
`65958f65a6592a10b40c368ced5e480bce8e4724`. Deeper Pose containers, other message
descriptors and general JSON grammar remain open.


## Header stamp fragments under Pose component parents

The next bounded adapter combines `header` objects and their `stamp` objects
under top-level `Pose.position` and `Pose.orientation`. It retains ordered raw
members through the maintained serde map visitor rather than a JSON value map,
which would discard duplicate keys. Every earlier fragment remains available
for reflection validation. A null fragment contributes no members; an empty
object retains message presence. The adapter only descends along these two
Header/stamp paths. The direct serde dependency uses the already locked
`1.0.229` version; no package version changes.

The 73 distinct native observations contain 57 accepted full-wire matches and
16 rejected controls. They cover both parents, repeated parent/header/stamp
objects, scalar replacement and null, empty-message presence, signed and quoted
integer boundaries, direct component coexistence, escaped/bare/single-quoted
keys and invalid earlier fragments. The public scalar-merge parent produced
37 accepted wrong wires in this corpus. All 31 earlier JSON fixture hashes are
retained. Three selected Pose CLI tests pass, and cumulative native replay
covers 6,598 outcomes, including the two pending forms below. The exact Header source gate passed 131 Rust tests including 34 CLI; independent
[review](reviews/json-pose-header-stamp-merge.json) reproduced the three selected
Rust tests, all native outcomes and the 37 parent wrong-wire controls. Retained
coverage measured 2,976/2,996 owned production lines, 4,884/4,948 regions and
63/63 newly added executable CLI lines. Publication also requires the reviewed
[automatic changed-line policy correction](CHANGED_COVERAGE.md), whose earlier
absence is not hidden by whole-file coverage. The combined exact source gate passed 131 Rust tests including 34 CLI, all
6,598 native outcomes, strict owned/vendor coverage and the new automatic
63/63 added-line gate; its retained evidence is complete. Final metadata binding,
normal protected hook and exact hosted run `40302314150` passed on public
`4cc842e3a267e9b1e305c92eb924c9d48b8cabd8`, tree `3e7a5837`.

At the `4cc842e3` Header/stamp source boundary, repeated `Header.data` lists
were outside that correction. For example,
`{"position":{"header":{"data":[{"key":"a","value":["one"]}]}},"position":{"header":{"data":[{"key":"b","value":["two"]}]}}}`
is accepted by both parsers, but native preserves both entries and Rust keeps
only `b` at that boundary. The fixture retains these two historical wrong-wire
observations; they were not counted among the 73 matching stamp observations. Full nested
message/list behavior, other descriptors, richer nested numeric grammar and
native rejection diagnostic codes remain unqualified. MSG-02 stays open.


## Ordinary Header data arrays under Pose component parents

The next bounded adapter appends ordinary repeated `Header.data` arrays under
`Pose.position` and `Pose.orientation`. It collects raw array entries in encounter
order and replaces only the final data occurrence. Earlier arrays remain in the
input so invalid earlier message entries still fail reflection validation. Null
data contributes no entries; an empty array does not erase earlier entries.
This is scoped to ordinary arrays of Header data messages. It does not normalize
singleton objects, nested arrays, null entries or duplicate inner `value` keys.

The new fixture has 55 distinct inputs: 43 matching observations comprise 35
accepted exact wires and eight joint rejections. Twelve native-positive forms
were explicitly pending at that source boundary: ten singleton/nested-array/null-entry
forms and two inner Header data `value` append forms. The public Header/stamp parent has 21
accepted wrong wires among the matching observations. Cases include ordering,
repeated equal keys, empty arrays/messages/values, null fields, multiple entries,
stamp coexistence, both parents and invalid earlier entries.

All 32 prior JSON fixture hashes are retained. The two ordinary-array witnesses
previously pending in the stamp fixture now assert native full-wire equality;
their historical fixture bytes remain unchanged. The selected four Pose CLI
tests pass and fresh native replay reproduces 6,653 outcomes. An initial selected
run still asserted the old wrong Rust wire for those two witnesses and failed;
that historical-gap assertion was replaced by exact native equality before the
passing run. The exact source gate passed 132 Rust tests including 35 CLI, all
6,653 native outcomes, strict 3,010/3,030 owned lines and 4,935/5,001 regions,
and automatic 33/33 added executable lines with complete retained evidence.
Independent [review](reviews/json-pose-header-data-append.json) rebuilt candidate
and parent separately, confirmed the 21 corrected wrong wires and all 12 pending
boundaries, and bound the retained changed-line proof. Final metadata binding,
normal protected hook and exact hosted run `40307095086` passed on public
`12814b4af826c90a79a1b347c35c2e28318b460c`, tree `37ec62c5`. Native rejection diagnostics,
wider nested grammar, other descriptors and full MSG-02 remain unqualified.


## Ordinary value arrays inside Header data messages

This candidate appends duplicate ordinary `value` arrays within Header data
messages under `Pose.position` and `Pose.orientation`. It uses serde RawValue
arrays and ordered message members, replaces only the final array occurrence,
and retains earlier fields for semantic validation. Null fields contribute no
entries and empty arrays do not erase earlier strings. Each ordinary Header.data
array is visited before the existing outer array append, including when there
is only one outer array. Other Header keys and item fields remain available to
reflection unchanged.

The 65-input fixture records 57 matching observations: 41 accepted exact wires
and 16 joint rejections. Cases include order, equal values, multiple strings,
null/empty fields, Unicode and escaped keys, invalid earlier or later entries,
multiple data messages, repeated outer arrays/headers/parents and stamp coexistence.
The accepted parent has 31 wrong-wire observations among these rows. Eight
native-positive inner singleton/nested-array/null-entry forms remain pending;
the ten earlier outer Header.data shape witnesses remain pending as well.

All 33 prior JSON fixtures retain their bytes. The two old inner-value pending
assertions are strengthened to native wire equality. Five selected Pose tests
and fresh native replay of 6,718 outcomes pass. The exact source gate passed
133 Rust tests including 36 CLI, strict 3,032/3,052 owned lines and 4,978/5,047
regions, and automatic 25/25 added executable lines with complete retained
evidence. Independent [review](reviews/json-pose-header-value-append.json)
confirmed the 31 parent wrong wires, all 18 explicit inner/outer pending rejection
witnesses, fixture preservation and the retained changed-line proof. An earlier
unsupported-edition formatting failure is preserved and excluded; the corrected
source passed without an edition or policy change. Final metadata binding, normal
protected hook and exact hosted run `40314819306` passed on public
`5e147a1da763cf894cab193b6eecde2b210c7d78`, tree `f655c397`. This slice does not
qualify native reject diagnostics, other descriptors, richer nested grammar or
MSG-02 as a whole.


## Singleton Header data objects under Pose component parents

Native protobuf also accepts a single Header.data message object where the
standard array form would contain one entry. The adapter converts only those
objects under Pose position and orientation. It uses the maintained ordered
member parser and RawValue serialization. Arrays, null fields and other shapes
are left to the existing adapters and reflection validation.

Every header occurrence in a component fragment is visited. Earlier component
fragments retained for validation receive the same structural conversion before
the final merged component is emitted. The conversion never appends entries or
inner values itself; the existing final-fragment append runs once. This prevents
valid earlier singleton objects from being rejected and avoids double appending
inner duplicate values. Malformed objects, unknown fields and wrong key/value
types remain validated.

The 81 native observations include 71 matching rows: 51 accepted exact wires and
20 rejections. The public parent rejected 49 matching positive rows. Cases cover
empty objects, absent/empty/null fields, Unicode and escaped keys, repeated data
keys, headers and parents, mixed singleton/array occurrences, stamp coexistence
and malformed earlier or later inputs. Ten richer-shape inputs remain explicit
native-positive pending observations. The prior inner-string grammar cases and
four prior outer nested-array/null-entry witnesses remain pending as well.

All 34 previous JSON fixtures retain their bytes. Six old singleton rejection
assertions now require native equality; other retained expectations are preserved.
Six selected Pose tests and a fresh 6,799-outcome native replay pass. The exact
source gate passes 134 Rust tests including 37 CLI, strict 3,063/3,083 lines and
5,052/5,126 regions, and automatic 31/31 added executable lines. Independent
[review](reviews/json-pose-header-data-singleton.json) rebuilds candidate, parent
and native peer, confirms all 49 corrected parent rejections, checks the explicit
pending cases and recomputes the added-line proof. Four supplemental null-header
combinations also match native; they are outside the fixture outcome count.
Final binding, the normal protected hook and exact hosted run `40324275712`
passed on public `94fd57149ef46f2f3342500fd7ba43d27da58ef6`, tree `8fae1e5f`.
Other descriptors, native reject diagnostics and MSG-02 as a whole are unqualified.

## Scalar strings in Header entry values

Native protobuf accepts a scalar string for the repeated Header.Map.value field.
The adapter wraps only string scalars under Pose position/orientation;
ordinary arrays and existing singleton Header.data objects use the same ordered
structural normalization. It does not append during that step. Earlier fragments
remain validated, and existing final-fragment append applies once. Invalid types
and unknown fields are preserved rather than discarded.

The 80-input pre-edit characterization contains 40 matching native positives,
32 joint rejections and eight explicit richer-shape pending forms. All 35 older
JSON fixtures retain their hashes; six older rejection expectations are
strengthened to exact native equality. Seven selected Pose tests and fresh native
replay of 6,879 outcomes pass. Eight supplemental repeated-header/parent controls
match native (four accepted, four rejected), outside the fixture count. The exact
source gate passes 135 Rust tests including 38 CLI, strict 3,100/3,120 lines and
5,111/5,190 regions, and automatic 40/40 added executable lines. Independent
[review](reviews/json-pose-header-value-scalar.json) rebuilds candidate, parent
and native peer, confirms all corrected parent cases and recomputes the retained
coverage proof. Final binding and the normal protected hook passed on public
`65d8b53c8a2ea94d36360d7a43c974e6c2d7232f`, tree `2fdf91c0`; exact hosted
run `40330145997` succeeded. Nested arrays, null entries, other descriptors,
native rejection diagnostics and full MSG-02 are unqualified.

## Direct null elements in Header entry values

Native protobuf skips direct null elements in Header.Map.value arrays. The
bounded Pose adapter removes those elements using maintained RawValue array
parsing and serialization, before the existing ordered append. Literal strings
such as "null" remain strings. Every non-null token, including invalid types and
nested arrays, remains available for validation. The same structural step visits
earlier parent/header fragments, preventing a later valid occurrence from hiding
an invalid earlier value and avoiding duplicate append.

The pre-edit corpus has 100 distinct inputs: 64 direct-null native positives,
28 joint rejections and eight native-positive nested-array cases kept pending.
The public parent rejects all 64 bounded positives. Both Pose components,
ordinary and singleton Header.data, duplicate value/header/parent occurrences,
empty/Unicode strings, escaped keys and whitespace are covered. Repeated-fragment
controls are included in these counts. All 36 older JSON fixtures retain their
bytes; eight old direct-null expectations are strengthened to native wire equality.
Eight selected Pose tests and fresh native replay of 6,979 status/full-wire
outcomes pass. Candidate/native/parent comparison confirms all 64 corrected
parent rejections; all eight nested forms remain Rust rejections. The canonical
source gate passes 136 Rust tests including 39 CLI, strict 3,115/3,135 lines and
5,133/5,214 regions, and automatic 17/17 added executable lines. Independent
[review](reviews/json-pose-header-value-null.json) rebuilds candidate, parent and
native peer, verifies all 36 fixture pins and recomputes the coverage checks.

A controlled run with an additional `CARGO_INCREMENTAL=0` setting fails owned
vendor region coverage at 205/222; the canonical configuration passes 205/206.
The unchanged source and toolchain produce 16 additional zero-count generic
placeholder regions in the disabled-incremental report. Both failed runs and
full coverage datasets remain retained. No source, threshold, exclusion or
coverage policy changed, and no general compiler cause or all-configuration
qualification is claimed. Final binding and the canonical normal protected hook
passed on public `a5112cd90ccd433fedb6207eab7f2bd8dbdc4b8d`, tree `0a2e32f7`;
exact hosted `40345454360` attempt 2 succeeded. The cancelled first attempt stalled
in dependency setup before verification; all logs remain retained and the same-SHA
rerun kept every required check. Nested arrays, outer Header.data null elements,
other descriptors, native rejection diagnostics and full MSG-02 remain unqualified.


## Bounded nested Header entry value arrays

The next adapter draft flattens nested string/null array structure for Header.Map.value
under Pose components, preserving leaf order and every invalid non-null leaf for
validation. It shares the existing UInt64 raw-value traversal and maintained JSON
serialization. Duplicate value/header/parent fragments are all visited before the
existing final append. Outer Header.data nested/null structure remains separate.

Pre-edit native/current-parent characterization records 144 intended matching
forms (84 native positives rejected by the parent and 60 joint rejects) plus four
outer-data pending forms. All 37 prior fixture bytes and earlier UInt64 assertions
are retained; 16 older inner-array expectations strengthen to native wire equality.
Formatting passes after a retained edition-syntax correction. All 40 CLI tests
and fresh native replay of 7,127 outcomes pass. The 148-input comparison confirms
all 84 corrected parent rejections, 60 joint rejections and four outer-data gaps.
The exact canonical source gate passes 137 Rust tests including 40 CLI, strict
3,112/3,132 owned lines and 5,116/5,200 regions, unchanged owned vendor checks and
automatic 22/22 added executable lines. All five staged evidence files are complete.
Independent [review](reviews/json-pose-header-value-nested.json) reproduces fresh
candidate/parent/native results, additionally compares 656 prior UInt64 inputs
byte-for-byte with the parent, and recomputes the unchanged full coverage policies.
Final metadata binding, the normal protected hook and exact new hosted success
remain required. Outer-data grammar, other descriptors, native rejection diagnostics
and broader MSG-02 parity remain unqualified.

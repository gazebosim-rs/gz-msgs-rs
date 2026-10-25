# Native flag-value conversion

Long list/version assignments now follow the observed native CLI callbacks.
The final assignment wins, including a later plain flag after an invalid earlier
value. English true/false spellings and single-character forms are case-insensitive;
empty values and {} use the flag's true default. Version conversion executes
first and a true version callback exits before list conversion. List conversion
then precedes help, exclusions and extras. An invalid converted value returns
104 with the observed diagnostic. A false list flag still counts as present for
its exclusion with info. Help is presence-based and ignores its assigned value.

Numeric flag values follow the pinned GNU reference's base-zero signed-integer
profile, including decimal, octal, hexadecimal and binary prefixes, leading C
ASCII whitespace and complete-consumption checks. Checked standard Rust integer
parsing supplies range detection; no new native ABI or dependency is introduced.
The reference's overflow path tests the original first character for a minus,
even with leading whitespace or a trailing suffix. That observable quirk is
retained; it is not normalized to a more intuitive sign rule. This profile is
qualified against the existing pinned Linux reference binary, not all C runtimes.

All 75 previous matching invocations and all fifteen former unsupported inputs
remain. Those fifteen now use exact native outcomes instead of asserting an
unsupported error. The added 171 observations cover the three flags, spellings,
radices, signed/range boundaries, suffix/whitespace errors, repeated values,
false-flag exclusion, required-value errors and callback precedence. Info/error
bytes and complete sorted list results remain exact; existing Rust help/package
version bindings remain as documented. No original C++ assertion is weakened or
retired, and no full CLI11 grammar or MSG-02 acceptance is claimed.

Run `cargo test --locked --test cli native_` for Rust/native corpus assertions.
`python3 scripts/cli-options-reference.py` replays the original 90 observations;
add `--fixture tests/fixtures/cli-flag-values.json` to replay the additional 171.
Both verify pinned source/binary/schema identities and fix the native argv[0]
usage binding. Non-UTF-8 argv, broader legacy grammar, dynamic-descriptor CLI
info, JSON grammar differences and full formatting/platform/performance/release
qualification remain open.

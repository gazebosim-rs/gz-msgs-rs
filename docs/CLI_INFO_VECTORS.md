# Native info vector values

The info option expands bracket-wrapped comma lists using the pinned CLI11
`Option::_add_result` rule. Each nonempty comma fragment is expanded again only
when both its first and last characters are brackets. This is literal splitting:
whitespace and quotes remain, and commas do not track nested bracket depth.
An explicit stack preserves the native order without recursive Rust calls.

Empty vector fragments contribute no names. Each info occurrence must contribute
at least one name, so an empty vector consumes further argv values, including
flag-looking values, until that occurrence has a name. An empty attached long
assignment likewise consumes the next argv value; an explicitly separate empty
string is itself a name. A later empty info occurrence still requires its own
value even if an earlier occurrence supplied names. Missing required values keep
the native 114 error before callbacks and exclusions.

The 119 pinned native observations compare complete stdout, stderr and status.
They include nested/malformed brackets, comma/empty fragments, preserved spaces,
attached/short/repeated options, required flag-looking values, aliases, exclusions
and the positional marker. Existing option and flag-value observations remain
unchanged. An additional ignored development probe exercised 264 generated nested
inputs; it is supplementary evidence rather than a replacement for the checked
corpus. This increment does not claim complete CLI11 grammar, non-UTF-8 argv,
external descriptor info, or MSG-02 acceptance.

Run `cargo test --locked --test cli native_info_vector_values_and_required_consumption`
and `python3 scripts/cli-options-reference.py --fixture tests/fixtures/cli-info-vectors.json`.
The reference script checks source/binary/schema hashes and does not rewrite data.

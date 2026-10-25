# Bounded legacy option combinations

The standalone CLI now uses maintained clap_lex 1.1.1 for GNU tokenization, with
native message-command policy layered on top. Attached `--info=TYPE`/`-iTYPE`,
repeated info options, multiple names, short flag groups and repeated list flags
are supported. Missing info values return 114, list/info conflicts return 108,
and unexpected arguments return 109, matching the observed native parser.
Version and help selection precede conflicts/extras; a missing required value
still errors first. The first required info value may itself look like a flag;
subsequent flag tokens end the names. Digit-prefixed negative tokens remain
literal names while collecting info, including nonnumeric suffixes such as -1x.
These rules are native observations, not numeric conversion or a CLI11 clone.

The 75 matching invocation rows compare complete statuses and exact successful
info/diagnostic outputs, including empty/Unicode names, aliases and short-group
tails. Repeated lists compare the complete sorted sequence because native map
iteration order is unspecified. Help selects this Rust tool's existing text,
including its additional format commands; version binds CARGO_PKG_VERSION, as in
the unchanged original C++ tests. The native reference fixes argv[0] to gz-msgs so
help's executable-name field is reproducible. Rust word aliases list/info share
the same option policy and are checked against corresponding native flag forms.

The fifteen formerly unsupported long flag assignments are retained verbatim
and now compare against native outcomes. A further 171-case corpus qualifies
boolean words, numeric radices/range behavior and callback precedence; see
[flag values](CLI_FLAG_VALUES.md). Full CLI11 grammar,
non-UTF-8 argv, every diagnostic/help formatting case and dynamic-descriptor CLI
info remain unqualified. External descriptor installation/global-cache decisions,
full message formatting and performance/release matrix work remain MSG-02 scope.
No original assertion mapping or broad packet is retired by this increment.

`cargo test --locked --test cli native_legacy_option_combinations` exercises the
new corpus plus Rust aliases. `python3 scripts/cli-options-reference.py` replays
all 90 original native observations and verifies the immutable source/binary/schema
identities, without rewriting golden data. The native environment is the existing
pinned Linux build, not a newly reproduced reference image. All previous CLI,
wire, conversion, coverage and staged gates remain mandatory.

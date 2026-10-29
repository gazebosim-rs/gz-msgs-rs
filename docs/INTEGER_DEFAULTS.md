# Registry integer defaults

`DescriptorRegistry::load_bytes` and the registry used by `MessageFactory` interpret
eligible proto2 integer defaults using the qualified native Linux LP64 policy.
This covers int32/int64, uint32/uint64, sint32/sint64, fixed32/fixed64 and
sfixed32/sfixed64. Nonrepeated ordinary fields are eligible; oneof fields must
be optional. A required field outside a oneof can have an effective default.
The canonical schemas, generated values, sealed message trait and wire codecs
are unchanged.

The integration adds one opt-in method to vendored prost-reflect,
`decode_file_descriptor_proto_with_integer_defaults`. The registry uses it after
its existing byte-default safety checks. Its build flag exists only during that
file's resolution; it is not stored as mutable global or pool policy. Standard
`DescriptorPool` entry points retain their prior behavior. Extension and proto3
defaults, other scalar kinds and constructor-produced foreign descriptors keep
their existing paths. The opt-in method alone does not add the registry's
byte-default safety preflight.

Only effective integer values change. The accepted raw default string remains
intact in field/file descriptor metadata and descriptor exports, including sign,
radix, whitespace and embedded NULs. No descriptor normalization occurs. Failed
files do not change the pool or source-provenance snapshot; successful earlier
files, ignored duplicate filenames and later retries keep their existing
semantics. This does not add a general byte-for-byte serializer for arbitrary
malformed or unknown descriptor fields.

The native policy is bound to protobuf 3.21.12, commit
`f0dc78d7e6e331b8c6bb2d5283e06aa26883ca7c`, on the recorded Linux LP64 platform.
[DescriptorBuilder's integer default assignments](https://github.com/protocolbuffers/protobuf/blob/f0dc78d7e6e331b8c6bb2d5283e06aa26883ca7c/src/google/protobuf/descriptor.cc#L5602-L5627)
use base-zero strtol/strtoul and strto64/strtou64, assign to the field width and
ignore errno. Its later check tests the original string's emptiness and the byte
at the returned end pointer. The installed binary imports the C23 libc symbols
at GLIBC_2.38, running with Ubuntu GLIBC 2.39. The committed lineage records the
primary-source hash, peer source/binary and native library hashes and symbols.
These are installed-library observations, not a fresh build of protobuf.

The safe Rust parser reproduces that measured policy with primitive radix
parsing, explicit saturation and defined narrowing:

- Leading C-locale ASCII whitespace and one sign are accepted. A leading zero
  selects octal; 0x/0X selects hex; this C23 binding also accepts 0b/0B binary.
  Trailing whitespace, separators, invalid digits and trailing non-NUL junk fail.
- The first NUL terminates native parsing while remaining in the raw descriptor.
  An initial NUL accepts zero because the original string is nonempty and the
  end byte is already zero. A sign or whitespace followed by NUL does not consume
  a number and is rejected. Empty raw strings are rejected.
- Signed parsing saturates at the 64-bit signed limits. Unsigned parsing wraps a
  negative in-range magnitude modulo 2^64, but overflow saturates at UINT64_MAX.
  Native range errors are ignored. The resulting value is then narrowed to the
  field width; Rust's defined casts reproduce the observed native 32-bit result.

For example, `010` means eight, `0x2a` means 42 and `0b10` means two. An int32 raw
`2147483648` becomes -2147483648, and raw `4294967296` becomes zero. Raw
`18446744073709551616` becomes INT64_MAX for int64, -1 for int32 and UINT64_MAX
for uint64. These results deliberately match the measured parser; they are not
strict numeric range validation. Other native libc/platform bindings require
separate qualification.

There are 990 exact scalar observations, 640 accepted and 350 rejected, with 99
literal spellings for each of the ten field types. All 190 original study bytes,
statuses and accepted values are retained. The original 190 standard-reflection
observations and errors also remain assertions, proving that ordinary pool
entry points have not changed. Fresh C++ peers replay native build status,
effective value and explicit-presence wire bytes. Rust checks the registry,
empty optional-message decode, full-buffer decode after setting the default,
raw field/file exports, factory construction/text paths and CLI failure limits.
The clean downstream consumer proves the opt-in patch survives direct path
dependencies without a consumer Cargo patch table.

Eight additional native controls distinguish labels. Raw native BuildFile accepts
optional-oneof and ordinary-required int32 defaults `010`/`0x2a`, yielding 8/42.
It rejects repeated and required-oneof defaults. Those rejected shapes remain on
the old reflection path, including its existing acceptance of decimal `010` and
rejection of hex. This preserves an explicit native/legacy validation gap rather
than expanding malformed descriptor acceptance. Required-message initialization,
complete oneof validation and arbitrary synthetic map shapes are not qualified.

`descriptor_definition`, registry/factory definitions and CLI info support ordinary
proto2 int32, int64, uint32 and uint64 defaults. They print the parsed effective
value in decimal while preserving raw descriptor metadata; see
[integer declarations](INTEGER_DEFINITIONS.md). Other explicit default kinds and
oneof defaults remain Unsupported.
A rejected numeric descriptor remains unavailable through the existing missing
info/continuation behavior. Floats, strings/bytes, enums, groups,
extensions and general native proto2 wire/formatting behavior are not newly
qualified. The published byte-default rejection remains in force. No original
assertion mapping is retired and MSG-02 remains open.

Run `cargo test --locked --test integer_defaults` and its `--release` counterpart.
`python3 scripts/integer-defaults-reference.py` regenerates the raw corpus and
lineage using the installed native reference; it needs Python protobuf, a C++17
compiler, libprotobuf, ldd and readelf. The normal gate uses committed bytes and
freshly compiles its C++ peer. Invalid native stderr is retained as evidence;
Rust/native diagnostic text equivalence is not claimed.

The new parsing code is linear in default-text length and runs only while loading
a descriptor. No performance improvement is claimed. All executable additions
in the three vendored descriptor files are owned and measured alongside the
existing signed-zero fixes. Exact source hashes, unified-patch additions and
owned-line accounting are checked. LLVM-omitted syntax receives no covered-line
credit; every intersecting owned code region remains measured. The strictly
higher-than-95% thresholds and original vendor attribution remain unchanged.

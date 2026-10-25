# Descriptor byte-default robustness

`DescriptorRegistry::load_bytes` checks raw byte-field defaults before sending a
new file to the pinned prost-reflect 0.16.5 loader. Octal escapes above 255 and
hex escape slices that split a UTF-8 character return the existing `DecodeError`
type. `load_file` and factory path diagnostics translate that to `InvalidData`.
The check covers ordinary and extension fields at file/message/nested-message
scope. It does not rewrite raw metadata, descriptor bytes or the vendored runtime.
Previously accepted files and snapshots remain available when a later file fails;
the failing file is not inserted, and corrected input can be retried. A safety
error stops that load call before later files. Already-loaded file names retain
keep-first behavior and are not checked again.

This is a deliberate bounded rejection, not native default-value compatibility.
On the pinned baseline, `\400` and `\777` panic in debug builds but wrap to 00 and
ff in release builds. `\x€` and `\x1é` panic in both modes because a Rust string
slice ends inside a multibyte character. Native libprotobuf 3.21.12 accepts all
four: its effective bytes are respectively 00, ff, e282ac and 01c3a9. Rust now
rejects these four inputs consistently in both modes. The literal escaped slash
in `\\777` still yields 5c373737. Earlier ordinary malformed escapes retain the
loader's `DescriptorError` disposition, even when followed by an unsafe sequence;
the loader never reaches that suffix. Other structurally invalid descriptors may
be rejected by this preflight before native/reflection structural validation.
This check is not a complete malformed-descriptor validator.

The original exact raw descriptor inputs, native statuses, output bytes, source
and binary hashes, and explicit Rust dispositions are retained in
`tests/fixtures/byte-defaults/native.json`. Each gate freshly compiles the byte
value C++ peer and compares its complete outputs on those inputs. The fixture
also preserves the original independent baseline's descriptor-formatting outputs
and hashes; those historical binary hashes are provenance, not a claim that a
fresh compiler emits an identical executable. The earlier isolated 143-case
scalar-default study remains preserved outside production inputs.

Run `cargo test --locked --test byte_defaults` and
`cargo test --locked --release --test byte_defaults`. Four tests cover exact native
observations and Rust dispositions, ordinary escapes/raw metadata, nested and
extension traversal, duplicate files, partial progress, snapshots, retry, file and
factory error paths, and actual CLI list/info continuation. The complete gate
also keeps custom-option bytes, descriptor definitions, original cases and all
coverage policies mandatory. The CLI retains its existing behavior: list omits
failed files; info reports an unavailable type and continues subsequent names.
It does not add native malformed-path diagnostic equivalence.

Prost 0.14.4 deprecates its only arbitrary-description `DecodeError` constructor.
A documented allowance is limited to the owned preflight function so this
maintenance correction preserves the public return type without a dependency
fork or a fabricated error. Revisit that boundary when migrating prost.

Explicit defaults remain unsupported by declaration formatting. Integer radix,
float/enum/string/byte interpretation differences beyond this rejection remain
unqualified, and arbitrary native proto2 default/wire parity and MSG-02 completion
are not claimed. There is no performance improvement claim; the additional scan
is linear in new-file default text and adds no per-message decode work.

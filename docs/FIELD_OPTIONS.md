# Standard field-option declarations

The external declaration formatter now retains explicit `lazy`, `weak` and
`unverified_lazy` booleans alongside `packed` and `deprecated`. It uses the
existing reflected option iterator, native option order and boolean formatter.
Absent options stay absent; explicit false stays visible. This formats metadata;
it does not add lazy decoding or weak-link runtime semantics.

Pinned protobuf 3.21.12 accepts true lazy flags only on message fields. Reflection
admits these flags on scalars too, so the formatter returns
`Unsupported("lazy option on non-message field")` for those inputs. False lazy
flags and either weak value remain supported where the existing declaration
shape is supported. Existing option validation runs first: unsupported standard
options such as `ctype`, custom options and unknown options retain their errors.
Proto3 required fields retain their existing rejection. This is a bounded output
guard, not a complete descriptor validator or a change to loader admission.

`tests/field_options.rs` replays 354 descriptors against a freshly compiled native
peer, comparing complete status, stdout and stderr. Native admits 275; 272 have
supported declaration metadata. Three native-valid `ctype` controls deliberately
remain unsupported. The other native rejections distinguish scalar true lazy
flags and proto3 required fields. All 354 still load through the existing Rust
registry; 82 then return explicit formatter errors.

The matrix covers proto2/proto3, optional/required/repeated labels, scalar and
message kinds, absent/false/true flag combinations, maps, real oneofs, synthetic
proto3 optional fields and omitted/explicit JSON names. Every supported row checks
registry, factory, direct descriptor and complete Rust CLI output. The direct
descriptor API retains its computed JSON name; its comparison uses the paired
native descriptor with that explicit name. Source-aware paths preserve raw
omissions. Typed descriptors and encoded dynamic options are unchanged after
formatting, including the reflected `unverified_lazy` flag absent from the typed
prost-types options struct.

`scripts/field-options-reference.py --upstream ../gz-msgs` independently replays
the checked-in corpus against the pinned upstream CLI and a fresh peer. It checks
upstream source, binary, schema and loaded library identity. Invalid native CLI
inputs can exit zero with empty output and diagnostics; those observations are
retained separately from descriptor admission. Rust reports its existing nonzero
formatter error instead. No native error-output parity is claimed for those rows.

Production parsing, wire behavior, canonical definitions, dependencies and all 83
upstream test mappings are unchanged. Full retained MSG-02 behavior and performance
acceptance remain open. Independent review passed fresh source/target tests and
all 354 native peer/CLI observations. Author focused tests (32) and all-target
Clippy passed. Normal strict local and exact hosted gates remain pending.

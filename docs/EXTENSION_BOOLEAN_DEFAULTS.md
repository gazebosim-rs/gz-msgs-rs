<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->
# Boolean defaults in nested extension declarations

The source-aware registry, factory and CLI definition paths format explicit
optional proto2 boolean extension defaults as `default = true` or
`default = false`, before the existing supported options. An omitted default
stays omitted, even though its effective boolean value is false. Ordinary and
nested extension owners retain source order and separate extendee sections.

The formatter reads `ExtensionDescriptor::default_value()` from the maintained
reflection runtime and leaves the raw FieldDescriptorProto and option presence
unchanged. No loader, boolean parser, value/wire codec, schema, generator,
dependency or consumer-pin change is introduced. A standalone MessageDescriptor
still lacks original JSON-name omission metadata and retains its existing
extension-declaration error; explicit extension JSON names remain unsupported.

Required/repeated defaults, other scalar-default families, groups, maps,
proto3/MessageSet shapes, invalid literals, unsupported options and existing
integer/default boundaries retain their guards. A late unsupported declaration
does not expose an earlier partial definition or CLI heading. The native loader
rejects some shapes that reflection admits; the Rust formatter continues to
return a typed Unsupported error for those admitted shapes.

The bounded native corpus records 56 complete descriptor/CLI observations at
Gazebo Messages commit 89c0c3d1474f545cb889335daf930bfdab6ab852 and installed
protobuf 3.21.12, including binary/library/source identities. It includes 32
supported layouts, 14 retained formatter rejections and 10 invalid-literal
loader rejections. The unchanged native definition observer is freshly compiled
by the Rust test and every status/stdout/stderr is compared. Supported CLI
observations retain full Name/File/body bytes. Clone, duplicate-file, failed-load
retry and raw-metadata controls preserve the existing source snapshot contract.
All three Rust info spellings assert exact status, stdout and stderr for every
policy. Loader rejections preserve the existing missing-message diagnostic and
status zero; complete native schema-validation diagnostics remain unqualified.

The original CmdLine.MsgInfo and FactoryTest.DynamicMessageWithEnv behaviors
remain anchors. All 83 original mappings were already ported and do not advance
for this supplemental retained behavior. There is no general extension-default,
native descriptor-validation, runtime/wire or performance qualification claim.
Independent source/assertion review, normal strict staged coverage and exact
hosted qualification remain pending for this private candidate.

```sh
python3 scripts/extension-boolean-defaults-reference.py --upstream /path/to/gz-msgs
cargo test --locked --test extension_boolean_defaults
```

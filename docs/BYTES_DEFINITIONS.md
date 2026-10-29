<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Proto2 bytes declarations

Ordinary optional and required proto2 bytes defaults format their existing
reflected bytes through the byte-wise ASCII/octal quoting helper already used by
strings. No escape parser is added. Raw descriptors, default presence, loader
admission and message wire behavior remain unchanged. Defaults precede explicit
JSON names and supported options. Omitted defaults emit no clause.

Registry, factory and CLI retain their existing byte-default preflight. Unsafe
octal values above 255 and UTF8-splitting hexadecimal escapes remain DecodeError;
ordinary malformed escapes remain DescriptorError. Native accepts these examples
with its own effective bytes. Invalid UTF8 descriptor strings retain typed-decoder
rejection. The [loader safety contract](DESCRIPTOR_DEFAULT_ROBUSTNESS.md) and its
original tests/fixtures are unchanged.

Direct descriptor formatting accepts an already-built reflection descriptor. It
does not add registry preflight to standard DescriptorPool decoding. Tests exercise
standard reflection only on safe admitted inputs; they do not feed known panic
inputs to the dependency. A caller decoding arbitrary raw descriptors should use
the existing registry boundary rather than infer new safety from this formatter.

The corpus records 327 native controls. Of 308 positive Rust layouts, 300 match
native byte values and declarations exactly. Eight explicitly retain existing
loader differences. In four layouts, native
interprets `\x4142` as byte `0x42` (`B`); reflection consumes two hex digits and
retains `42`, giving `A42`. In four more, native truncates a literal NUL-prefixed
default to empty bytes, while reflection preserves NUL/newline/tab. The formatter
prints its
actual reflected bytes without reparsing raw text. This is not expanded native
byte-default parsing or wire compatibility. Oneof/repeated/proto3 defaults and
unknown options keep explicit formatter errors.

Controls cover every individual byte, all 256 bytes together across optional and
required fields and bare/explicit options, short octal, hexadecimal, adjacent
Unicode, literal controls, composed/decomposed Unicode, literal escape text and
omitted defaults. Positive tests compare effective bytes, raw metadata, direct
reflection, registry, factory and CLI output. Nineteen remaining controls preserve
unsafe, malformed, invalid UTF8 and unsupported-shape dispositions. Every native
status/stdout/stderr observation is replayed by the new scoped C++ observer.
Existing byte-default, parser, wire and 83 upstream assertion mappings are unchanged.

The fixture records raw descriptors, native process output, observer/compiler
identity and linked-library hashes for protobuf 3.21.12 on Linux LP64. Replay:

```sh
cargo test --locked --test bytes_definitions --test byte_defaults
```

The later [floating declaration policy](FLOATING_DEFINITIONS.md) has a separate
contract. Other library versions, general malformed metadata, wider Messages
acceptance and equal-work performance remain separate work.
Independent review, the normal staged hook and exact hosted CI are required.

<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->
# Gazebo message schemas

This repository distributes 216 canonical Gazebo `.proto` definitions, their
inventory and validation tooling. Consumers generate their own bindings with
standard pinned protobuf tooling. There is no Rust package, generated binding,
codec, reflection library, conversion API or standalone Messages command here.

The schemas are unchanged from Gazebo Messages commit
`89c0c3d1474f545cb889335daf930bfdab6ab852`. Their original OSRF notices remain in
every file. See [LICENSE](LICENSE), [NOTICE](NOTICE) and the
[schema manifest](schemas/manifest.json).

Use `schemas` as the compiler include root. Compile the files listed in the
manifest, resolving the standard external `google/protobuf/any.proto` from your
pinned protobuf installation. All canonical files use proto3; the inventory has
no proto2 defaults, extensions or custom option definitions. Discovery schemas
remain canonical data even though the standalone transport rewrite is retired.

For schema validation, install standard `protoc` 3.21.12 with its matching include
files and run:

```sh
python3 tools/test_validate.py
python3 tools/test_staged.py
python3 tools/validate.py
```

Install the staged hook with `git config core.hooksPath .githooks`. It checks the
intended Git index in a temporary source export, including alternate indexes.
See [validation](docs/VALIDATION.md) and [consumer migration](docs/MIGRATION.md).

This schema-only release follows the independently verified consumer migration.
Simulator commit `5870373f0d8280601b051f2cbb1bb0aba264f70f` now
uses the identical canonical schemas and consumer-generated bindings. Its normal
local staged gate and independent audit pass. Exact hosted run
[41248418524](https://github.com/gazebosim-rs/gz-sim-rs/actions/runs/41248418524)
passed, including independent downloaded source, input and coverage verification.
See [consumer migration](docs/MIGRATION.md) for the precise scope.
Runtime history and evidence remain available at commit
`a39a9455d731923bd316745c2dda21071408dcbf` and its ancestors. The owner retired
that runtime release responsibility under D29; this does not mean the historical
full runtime task was completed.

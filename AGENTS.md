# Schema repository guidance

Owner decision D29 supersedes the former Rust runtime scope. This repository
contains canonical `.proto` sources, a minimal inventory, notices, documentation
and standard-compiler validation. Do not restore a generated Rust package,
custom codec, reflection/conversion framework or standalone Messages CLI here
or hide a successor runtime in simulator consumers. Consumers generate locally
with pinned maintained tooling. D28 uses upstream Zenoh directly in the simulator.

Preserve canonical source bytes, packages, field numbers, types, presence,
options, imports and original OSRF copyright/license notices. Use the standard
protobuf compiler to validate the complete import closure and descriptor identity;
do not write a custom protobuf parser. Synthetic historical proto2/default-loader
fixtures do not expand the 216-file canonical release inventory.

Before changing a canonical definition, inspect the pinned upstream source and
coordinate the compatibility decision. Require independent review, staged-index
validation, negative controls, hosted CI and exact published source verification.
Never weaken assertions, thresholds or inventory checks to make a gate pass.
There is no production runtime crate in this tree; empty coverage denominators
are not a >95% pass. Consumer-owned production code retains its real coverage and
native domain interoperability requirements.

Use isolated worktrees and preserve other owners' work. Do not edit Fuel. Do not
publish this transition until the coordinator confirms consumers have migrated;
existing historical pins must keep working. Keep all existing Git history and
runtime evidence. List non-regenerable files before any physical deletion.

Staged validation preserves the parent Git context/index and removes only
`git rev-parse --local-env-vars` from the exported child. Preserve nonlocal Git
settings. Test real worktree, explicit context and alternate-index behavior.

All new owned source uses Copyright (C) 2026 Open Source Robotics Foundation and
SPDX-License-Identifier: Apache-2.0. Preserve third-party notices unchanged.
Keep this owner's public status current through the sibling rewrite-progress
updater; historical runtime metrics are not proof this transition is complete.

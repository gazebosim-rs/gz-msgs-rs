<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->
# Explicit runtime install-share discovery

This local prototype adds the explicit runtime `GZ_MSGS_INSTALL_PREFIX` domain to
`MessageFactory::default()`. On first dynamic use, descriptor-environment paths
load first, followed by the existing `share/gz/protos` path under that prefix.
A subsequent explicit `load_paths` request follows both. Existing successful-file
cache, schema-name precedence and per-file error ordering remain unchanged.
Generated lookups and registered constructors do not initialize discovery; an
initialized instance does not rescan when the environment changes.

An empty override is meaningful: it selects relative `share/gz/protos` under the
current directory. Prefixes retain OS bytes, including non-UTF8 names. The full
joined location is checked before passing it to the existing `load_paths` API,
which retains native path-list separator behavior. On Unix, an existing prefix
containing a colon therefore does not act like one uninterrupted directory.
Absent locations are ignored; scanning and filename recognition use the existing
one-level loader, including its native descriptor-file priority.

`with_paths` remains the explicit isolated Rust constructor. It reads neither
descriptor-environment paths nor install-prefix discovery. Native has no such
constructor; comparison of native automatic ordering uses its default factory
followed by an explicit load, not a claim about this Rust-only constructor.

When the runtime variable is absent, this prototype preserves existing Rust
behavior. Native instead uses a generated CMake-configured installation prefix.
Cargo release layout and that absent-variable default remain open. This change
does not hardcode an SDK location or infer a prefix from the executable.

A failure determining whether the share path exists becomes an initial I/O
diagnostic after earlier descriptor-path diagnostics. A native symlink-loop
profile aborts via its filesystem exception; Rust retains safe diagnostics and
prior state. No abort or post-abort equivalence is claimed.

The checked native fixture records 16 workflows: 15 normal completions and one
explicit prefix-loop abort. The helper verifies pinned source, generated getter,
headers, actual execution-environment libraries and complete native output bytes.
An additional actual CLI observation retains full stdout/stderr/exit. Temporary
directory occurrences are explicit input bindings; output is not stripped or
normalized. The broader private characterization also records absent paths,
symlinks, one-level traversal and native descriptor-name priority.

The harness-free Rust test runs 19 isolated, single-thread subprocess workflows
so environment changes do not race other tests. Fifteen bind the normal native
states and message file identities; one abort profile asserts safe Rust errors.
The absent-variable and two isolated-constructor controls are explicitly Rust-only.
An actual Rust CLI invocation matches the native positive output byte-for-byte.
The full author run passed 184 standard Rust tests, both environment/workflow
harnesses and all-target Clippy. Unchanged native cache replay retained all 46
observations and five originals. Independent fresh-source review passed 14
retained regular Rust tests, the unchanged lazy-environment harness, all 19 new
workflows and the actual CLI control. Its fresh native helper reproduced all 16
workflow observations, complete CLI bytes and pinned source/library identities.
Normal local staged and exact hosted/public qualification remain pending. No whole MSG-02, new original-test credit,
runtime-wire change or performance claim is made.

<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# MSG-02 retained completion work

All 83 original cases are ported. This closes the original test ledger, not the
full behavior, distribution or performance requirements. This checklist compares
Rust baseline `9dd9c0d` with pinned native `89c0c3d`; the OS-byte fix
is now accepted at `ed19951`. Corpus sizes and coverage are evidence,
not progress percentages.

| Area | Retained work and evidence |
| --- | --- |
| CLI OS bytes | Baseline `std::env::args` panics on invalid UTF-8. The bounded [OS-byte slice](OS_ARGV.md) closes that proven gap, accepted after independent review, the normal strict gate and exact hosted CI. |
| Successful factory-file cache | The [file-cache slice](FACTORY_FILE_CACHE.md), accepted in `c979e87` through exact hosted `8939b42`, closes the demonstrated same-process A→B symlink rewrite workflow. Successful weakly canonical files cache; failed/partial imports retry and explicit byte loading retains its semantics. |
| Install-share discovery | The [explicit-runtime-prefix slice](FACTORY_INSTALL_SHARE.md), accepted in `8939b42`, closes the demonstrated `GZ_MSGS_INSTALL_PREFIX` workflow, including lazy ordering and OS bytes. Native absent-variable fallback comes from generated CMake code; Rust compiled-prefix/release-layout policy remains open. No executable-relative inference is introduced. |
| CLI loader diagnostics | The [file-diagnostic candidate](CLI_LOAD_DIAGNOSTICS.md) closes missing-file and malformed-wire console output using actual loader stages. Native schema-validation console output, the tested invalid-name admission difference and broader failure domains remain explicit. Independent/staged/hosted qualification is pending. |
| Declaration formatting | Source-valid groups, true MessageSet, extension defaults/explicit JSON names and remaining options need implementation or explicit release dispositions. Current boundaries are in [CLI_EXTERNAL_INFO.md](CLI_EXTERNAL_INFO.md). Preserve invalid-input safety and raw metadata; descriptor combinations alone do not close a family. |
| Performance | The accepted [codec baseline](CODEC_BENCHMARK.md) retains LaserScan1080 native/Rust ratios around 0.257–0.300. This is a real blocker to a performance-completion claim. The reviewed maintained-runtime helper experiment remains unadopted: standard consumer package/type identity and external publication are unresolved. No bespoke Gazebo codec workaround is authorized. |

Next retained work: qualify the bounded CLI loader-diagnostic candidate, then
resolve remaining schema-validation/runtime and release-layout boundaries.
Retry, deletion and alias behavior remain qualified independently from
install-prefix policy.

Runtime compatibility also needs an explicit release-domain conclusion. Typed
values discard unknown fields; the read-only canonical envelope forwards original
bytes. Strict and compatible wrong-wire behavior, overflowing varints and proto2
default differences already have controls. Do not claim universal native parser
identity or silently loosen safety guards.

All sixteen conversion families included by native `Utility.hh` have Rust
counterparts: scalars/time, vectors/pose/color, plane/box/inertial/spherical,
enum names and Fuel metadata. Packed-cloud APIs and the 83 original mappings add
further coverage. This audit established no new missing conversion family; it
replaces the stale blanket “broad math/XML pending” statement without claiming
all possible conversion inputs are qualified.

Git/path and immutable source-archive distribution are supported; exact CI checks
fresh-cache installation, downstream use and runtime closure. Ordinary `.crate`
publishing stays disabled because it drops reviewed path dependencies. The
package-identity/publication decision is separate from Linux source distribution.
No external prost fork or PR is authorized. Broader platform/binary release scope
must remain explicit.

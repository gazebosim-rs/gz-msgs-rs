<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Successful descriptor-file cache

`DescriptorRegistry::load_file`, path/directory scanning and `MessageFactory`
now retain successful file identity keys. Replacing a loaded descriptor file with
another set does not import new messages through the same key. IO errors,
malformed protobuf and any per-file validation errors leave the path retryable;
partially successful imports still retain their previously admitted descriptors.
A valid empty set is successful and is cached. Schema file-name identity and raw
option/default handling are unchanged.

`load_bytes` remains an explicit uncached byte operation. It can add previously
unseen schema files after a path has been cached; it does not replace existing
schema definitions. Registry clones retain aligned pool/cache snapshots and use
copy-on-write cache storage. Factories remain independent and their existing
mutex/constructor/lazy-initialization/error-order contracts stay in force. No
background state or global filesystem cache is introduced.

The pinned GNU native implementation uses `std::filesystem::weakly_canonical`
before opening a candidate. The adapter delegates existing-prefix resolution to
Rust std and lexical cleanup to exact `path-clean` 1.0.1. The cache stores
`OsString`, preserving bytes and trailing directory separators rather than using
lossy strings or Path equality. Existing paths resolve symlinks. Missing tails
retain native lexical behavior; for example a deleted cached `a.desc` reports an
open error when retried as that bare relative name, while `./a.desc` or its
absolute path can still hit the prior key. A dangling symlink does not become
its missing target's cached key; retargeting a live symlink can admit a new file.
Non-missing filesystem resolution failures remain IO errors before the cache
lookup, including when the failing absolute filename was previously cached.
A cached key must not mask a later symlink loop or permission error. This identity
computation is not a security sandbox or atomic filesystem transaction. Qualification is the pinned Linux GNU environment.

The one added dependency is a registry package, MIT OR Apache-2.0, with no runtime
dependencies and `forbid(unsafe_code)`. Its checksum is pinned in Cargo.lock;
all older locked packages remain unchanged. No vendor fork or protobuf package
identity changes. [The maintained helper](https://docs.rs/path-clean/1.0.1/path_clean/)
performs only lexical cleanup; the filesystem/presence policy stays explicit.
[GNU's recorded relative-prefix behavior](https://gcc.gnu.org/pipermail/gcc-bugs/2024-January/847733.html)
explains why plain canonicalization or an always-absolute soft resolver alone is
insufficient. Native directory-tail string semantics are retained separately.

The checked fixture has 46 primary native workflows: aliases, absolute and
relative deletion, dot/parent paths, missing parent components, dangling and
retargeted symlinks, malformed/partial retries, empty sets and distinct non-UTF-8
filenames (including a valid replacement-character filename). Forty-four complete
normally; two native symlink-loop cases terminate via uncaught filesystem
exceptions. Rust keeps its existing IO-error/refusal behavior and prior state;
no native-abort or post-abort-state parity is claimed. Three further private
non-root permission probes also observed native exception aborts. These are
bounded diagnostic evidence, not additional supported behavior.

Native replay compares complete untouched stdout/stderr and statuses. Absolute
paths necessarily use the supplied fresh case directory: fixture expectations
contain literal byte segments and that single explicit input binding. This is
not stderr stripping or normalization. Complete raw actual observations, peer
hash and compiler identity are retained in ignored reference output. Native
source/header and all resolved library identities (including libstdc++) are
checked; original parser/schema/peer fixtures remain unchanged.

Three new Rust tests exercise the workflow corpus in isolated child working
directories, byte-loader and cloned-cache independence, and initial/explicit
error ordering with retries. The cached-target loop control failed before the
IO-error propagation fix and passes with it. All 184 author Rust tests, all-target
Clippy and the clean standard no-patch downstream consumer passed. The native
helper also reruns five unchanged factory/retry/lazy/concurrency originals and
records their binary/library identities independently from the probe. All old
locked packages and original tests/fixtures remain unchanged. Independent review
passed 14 focused Rust checks, all 46 native workflows and five unchanged originals,
and a clean downstream consumer in a fresh source export and owned target. The
review also verified the preserved lockfile records and exact execution/linkage
environment. Normal strict staged and exact hosted qualification remain pending.
No additional original-test credit, performance improvement or full MSG-02
acceptance is claimed. Install-share discovery is a separate next slice.

```sh
cargo test --locked --test factory_cache --test registry --test registry_paths --test factory --test factory_env
python3 scripts/factory-cache-reference.py --upstream ../gz-msgs
```

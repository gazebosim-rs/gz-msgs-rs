# Initial MSG-01 slice evidence

Baseline: Gazebo msgs main 89c0c3d1474f545cb889335daf930bfdab6ab852.
216 canonical schemas copied byte-for-byte; SHA256 manifest checked by gate.
`source-inventory.json` also records baseline test/fixture hashes. Test ledger
contains static declarations, not claimed expanded or executed upstream tests.

2026-10-24 Linux x86_64: exact Rust 1.94.1, system protoc/libprotobuf 3.21.12,
C++17 compiler, protobuf/codegen/json-mapping 3.7.2. Installed toolchain binaries
were placed first in PATH because a concurrent shared rustup update caused the
proxy to attempt a write outside this repository. No shared tool changes here.

Commands: `cargo test --locked`; `python3 scripts/verify.py`.
Results: 7 integration tests pass, fmt/clippy (`-D warnings`) pass, coverage
negative tests pass, schema hashes pass. Instrumented coverage report is
`target/coverage/coverage.json`: handwritten src/lib.rs lines 37/37 and regions
37/37 (100%). This is one small production crate, not an upstream parity claim.
Generated schema bodies and schema-derived registry are not handwritten code.
Native peer is recompiled by the test, not silently skipped when unavailable.

Coverage and C++ outputs are reproduced by the verification command; target
artifacts are not committed as stale acceptance evidence. The staged hook reruns
checks on an exact `git write-tree` archive, independent of unstaged edits.

Remaining: full CTest/gtest runtime expansion and original native Gazebo Utility
reference runs; original assertion-preserving conversion ports; external descriptor
search/retry/concurrency behavior; CLI grammar/completion; packed point cloud
utilities; comprehensive bytes/float/Any/maps/duplicate-field corpus; performance,
packaging and historical schema qualification. These stay not_ported in the
ledger. New conformance tests do not retire or replace unmatched upstream tests.
No performance improvement or completed MSG-01/MSG-02 claim is made.

## Reviewed follow-up slice

Independent root review required including owned build.rs in coverage; this is
now measured, with no exclusion. Instrumentation derives the host target from
rustc and cleans only this crate before execution to force build-script profiles.
`scripts/test-staged.py` demonstrated a staged broken time conversion fails even
when the working file is fixed. CI repeats this demonstration.

CLI tests exercise every command, malformed input, unknown commands/types,
invalid UTF-8 and Linux IO failures. /dev/full exposed a swallowed buffered-write
failure; explicit stdout flush fixes it. Expanded native tests cover 1 MiB image
bytes, unknown pixel enum, Any oneofs, infinities, NaN, signed zero, empty message
presence and duplicate scalar fields. A signed-zero incompatibility was found in
the third-party generator and corrected in measured build logic. The native peer
compares every canonical top-level DescriptorProto exactly against system protoc.

Native reference expansion: existing local pinned build contains 10 integration
executables. `python3 scripts/reference-tests.py` ran all 82 original gtests:
82 passed, zero failures/skips. `ctest --test-dir ../gz-msgs/build
--show-only=json-v1` enumerated 21 entries (10 gtest binaries, 10 XML existence
wrappers, 1 Python test); wrapper commands were not separately rerun. Running
`/usr/bin/python3 -m pytest ../gz-msgs/python/test/basic_TEST.py` with native-build
PYTHONPATH, owned XML/cache destinations and bytecode writing disabled passed
1/1 (three deprecation warnings). This is an existing local build against the
pinned source, not a reproduced immutable image. Binary SHA256, exact commands,
XML and logs are in target/reference; ledger records native status separately
from Rust migration status. No unmatched test was marked ported.

Reviewed-slice coverage: build.rs 39/39 lines, 59/59 regions; src/lib.rs 37/37
lines and regions; CLI 53/53 lines, 95/99 regions (95.96%). All 11 integration
tests pass. `python3 scripts/verify.py` passes including owned build logic.

Primitive-conversion follow-up: 15 integration tests pass; preserved original
assertions map 8 scalar conversion cases, time conversion, Python serialization,
and 4 static factory cases (14 mappings). These are `ported_pending_review`, not
self-approved test retirements. Eighty-three original native tests are reconciled
(82 gtests including the macro-guarded Linux CLI test plus 1 Python test).
The signed-zero postprocess now fails on drift from its expected 546 checks.
Coverage: build.rs 42/42 lines, 63/63 regions; lib.rs 40/40 lines, 41/41 regions;
conversions.rs 9/9 lines, 6/6 regions; CLI 53/53 lines, 95/99 regions (95.96%).

Geometry slice: 18 integration tests pass. Array conversions cover XY, XYZ,
XYZW and RGBA; pose extraction distinguishes absent identity from present zero
orientation. Added original primitive assignment assertions. Fifteen additional
mappings remain pending independent review; the prior 14 mappings are marked
ported only after /root's explicit b941c01 review. Coverage: build.rs 42/42 lines,
63/63 regions; lib.rs 46/46 lines, 64/64 regions; conversions.rs 13/13 lines,
12/12 regions; CLI 53/53 lines, 95/99 regions (95.96%).

Separate runtime-dependency experiment: run-prost-probe.py records 30 checks:
29 required plus one unsupported typed-unknown diagnostic. Unmodified maintained
crates fail 8 required signed-zero cases; patched target-only crates pass all 29.
See RUNTIME_PROBE.md and exact dependency patches. The production runtime is not
changed by this experiment. Neither this probe nor current formatting gap is
misreported as completed MSG-02 compatibility.

## Maintained-runtime migration

/root approved the exact two dependency patches, provenance-specific unchanged
upstream exclusions, owned-predicate coverage accounting, supported prost_path
facade and direct path dependency design. The production package now uses
prost/prost-build 0.14.4, direct patched prost-derive 0.14.4 and prost-reflect
0.16.5. No root Cargo patch table is required. Publishing stays disabled pending
separate vendor/package release qualification; immutable Git consumption works.

`python3 scripts/test-downstream.py` builds a clean separate Cargo consumer with
no patches. Standard prost::Message, prost_types::Any packing/unpacking, canonical
Parameter+Clock, generated/reflected signed zero and nested unknown forwarding
pass. The gate repeats this test. `scripts/test-staged.py` still rejects staged
broken time conversion despite an unstaged working fix under the new stack.

`python3 scripts/verify.py` passes: 22 integration tests, schema/vendor hashes,
fmt/clippy -Dwarnings, strict threshold negative tests, clean downstream and
coverage. Additional tests cover all three reflection-default methods including
real proto2 extension descriptors, f32/f64 positive zero/negative zero/nonzero/NaN,
generated encoded_len, explicit dynamic edits preserving unknown fields, and
standalone CLI negative-zero JSON/text roundtrips. Existing C++ descriptor, 1 MiB
image, unknown enum, malformed and four-message peer tests all remain enabled.
CLI compact JSON/text layout changed with the runtime; values and protobuf syntax
remain equivalent. The original gz command-grammar ledger remains not_ported.

Owned coverage: build.rs 63/63 lines, 82/82 regions; lib.rs 67/67 lines, 93/93
regions; conversions.rs 13/13 lines, 12/12 regions; CLI 47/47 lines, 90/92 regions
(97.83%). Changed vendored predicates: prost-derive 6/6 executable lines, 8/8
regions; prost-reflect 9/9 executable lines, 28/28 regions. These are not whole
third-party crate coverage claims. LLVM excludes delimiter-only and quoted token
lines from executable line counts; every modified line remains provenance-listed
and missing executable instrumentation fails. Combined host/runtime LLVM export
also emits shared-function mismatch diagnostics; acceptance uses required owned
source and owned-predicate counters, not third-party aggregate percentages.

## Signed Time compatibility correction

A root review identified that normalized convenience time helpers did not match
native negative duration fields. `python3 scripts/time-reference.py` compiled
against the actual pinned StdTypes.hh and installed native gz-math/protobuf,
then captured 13 duration encodes (including signed i64 limits) and 9 Time decodes.
The committed fixture records source/header/math and native binary hashes plus
exact serialized hex. -1 ns encodes sec=0,nsec=-1; -1,000,000,001 ns encodes
sec=-1,nsec=-1. Native inverse accepts unnormalized signed nanos, including i32
limits. The default Rust helpers now match that behavior, truncating toward zero
and summing signed fields with a checked i128 intermediate. True i64 total overflow
returns None; C++ undefined signed overflow is explicitly not a compatibility
claim. Nonnegative integration behavior is unchanged.

23 integration tests pass, including exact native fields/bytes and inverse values.
The staged partial-source rejection demonstration was updated for the corrected
operator spelling. Existing original ConvertTimePoint assertions stay unchanged;
the new negative/boundary corpus is additional conformance evidence.

## Original CLI workflow slice

Restored legacy --help/-h/noargs, --version/-v, --list/-l and --info/-i TYPE...
with canonical/underscore type spellings, multiple types and the unchanged
licensed completion script. Info declarations come from canonical SourceCodeInfo
spans generated at build time; no proto parser or descriptor-to-proto formatter
was invented. The source/declaration corpus is still hash-gated.

The original gz_TEST.cc is copied byte-for-byte with its two original test-macro
headers and source SHA256 manifest. A small test-only harness binds package
version and executable/completion paths, then runs all 5 original C++ gtests
unchanged against the Rust binary: 5/5 pass. CI installs libgtest-dev for this
required test. Rust assertion ports independently retain the same five cases.
The mapping family remains pending /root review; no dispatcher ownership claim.

Full gate passes 26 Rust integration tests, including the unchanged 5-case C++
suite, native peers, downstream consumer and coverage negative tests. build.rs
107/107 lines, 164/164 regions; CLI 59/59 lines, 116/119 regions (97.48%). Other
owned production Rust and the two narrow dependency changes stay at 100%.

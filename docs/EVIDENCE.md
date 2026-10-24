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

## Legacy enum conversion family

`python3 scripts/enum-reference.py` compiled actual pinned JointType, GeometryType,
ShaderType, PixelFormatType and DiscoveryType converters and captured 197 cases
from the canonical descriptors. The fixture includes all known integers, -1/100,
valid/unsupported Geometry values, both cases, bad/empty names and native stderr.
Rust output values match every case. EnumKind reuses descriptor name/number
lookup and implements only observed legacy casing, restricted Geometry subset,
fallbacks and diagnostic policy. No numeric enum table is transcribed.

Original roundtrip/fallback assertions for Joint/Geometry/Shader/PixelFormat and
Discovery's ten strings/min/max/count are preserved. Image.PixelFormat's default
and assignment assertions are also ported. Six original mappings stay pending
independent review. The new module measured 94/94 lines and 95/95 regions; all
other production/owned-patch coverage remains above 95%. The final staged gate
runs 29 integration tests, including the unchanged original five-case CLI suite.

## Complete canonical --info fidelity correction

Independent SDF-owner review found a real gap beyond the original five CLI
assertions: raw SourceCodeInfo text omitted Name/File headings, left types
unqualified and retained comments; synthetic map-entry Info failed. The earlier
CLI slice therefore stayed in review. Source-span generation is now removed.

`python3 scripts/info-reference.py` captures complete actual native --info output
for 271 canonical/imported/nested/synthetic descriptors and verifies combined
output equals concatenated individual results. The 160 KiB generated corpus is
bound to the pinned schema manifest, native binary SHA256 and protobuf 3.21.12.
Build-time Rust emits lookup tables from this schema-derived data; runtime stays
Rust-only, with no incomplete handwritten generic formatter. External descriptor
formatting remains explicitly pending rather than being inferred from this corpus.

Full byte-for-byte stdout comparisons now cover all 271 in one request plus
WorldControl, underscore alias, Discovery.Publisher, Param.ParamsEntry and Google
Any individually. Map-entry headings/empty body and qualified names match native.
All five unchanged C++ tests remain enabled. Full local gate passes 30 integration
tests and all required owned line/region thresholds; the CLI family remains pending
independent acceptance. Tracker publication instructions are persisted in AGENTS.md.

## Owned instrumentation and reviewed mapping records

Coverage now instruments only this package and the two exact vendored packages
selected by their manifests. Registry/Git dependencies stay outside ownership and
instrumentation. A wrapper-content digest enters Cargo's flag fingerprint; owned
packages are forced to rebuild, and only the owned proc-macro object is exported.
LLVM export uses --fatal-warnings and completes cleanly, eliminating prior shared
dependency counter mismatch warnings. Selector tests cover root, both vendors,
foreign crates and compiler-query invocations. All 30 integration tests, the
unchanged CLI C++ suite, native info corpus and owned-predicate thresholds pass.

/root reviewed and approved this gate update and the mapping records. Six enum
mappings now record /root's source-by-source review; five CLI mappings record
/root and /root/sdformat acceptance after complete-output native comparisons.
Diagnostic text in the enum fixture is captured for reference and source-inspected;
the 197-case automated comparison checks returned values. The ledger now has
25 reviewed ports, 15 pending-review ports and 43 not-ported cases out of 83.

Hosted GitHub CI passed revision 22a0482fbf17e6469b59be4d217875262f916f94:
https://github.com/gazebosim-rs/gz-msgs-rs/actions/runs/40111717896 . This hosted
result does not qualify later unpushed gate changes; those have separate local
staged-tree verification and await their own linked CI result.

## Packed-cloud metadata slice

The native packed-cloud initializer/size oracle captured 30 full encoded layouts
and ten datatype-size results, with header/native-binary SHA256 and actual
sizeof(size_t) recorded. Inputs cover all eight datatypes, packed/aligned groups,
xyz+rgb/rgba, mixed widths, empty field/frame data, frame spaces and unknown types.
The Rust initializer matches every wire byte while retaining stamp/data/other
metadata and replacing frame/field layout. Diagnostic literals are source-inspected;
returned sizes and encoded layout bytes are compared automatically. There are no
unsafe accesses or iterator completeness claims in this bounded slice.

Original UtilityTest.InitPointCloudPacked and PointCloudPackedUtilsTest.SizeOfPointField
assertions are ported and marked pending independent review. Full local gate
passes 32 integration tests, the five unchanged CLI C++ cases, native peers,
complete info corpus and downstream consumer. cloud.rs measures 47/47 lines and
56/56 regions (100%); all other owned source/predicate thresholds remain above 95%.
New cloud code/probes carry the user-requested OSRF 2026 / Apache-2.0 notices.

## Safe packed-cloud byte access

Pinned bytemuck 1.25.2 supplies Pod byte conversion and safe unaligned reads;
legacy addressing/color selection is the narrow owned logic. Ten new native
complete-wire fixtures cover all eight scalar types and both RGBA endian-flag
orders. `python3 scripts/cloud-scalar-reference.py` compiles the C++ reference
against the pinned upstream headers and captures compiler-output binary SHA,
header SHA and host byte order. The reference uses aligned fields; no native
unaligned pointer dereference is required for this corpus.

`python3 scripts/verify.py` passes 35 integration tests, unchanged five native
CLI workflow tests, native interoperability, downstream no-patch consumption,
fmt/clippy and strict coverage gates. Cloud production code: 103/103 lines and
145/149 regions; owned generator/reflection patch denominators remain measured.
Tests also cover all scalar widths, unaligned f32 access to declared FLOAT64,
signed zero and NaN payload bits, direct/duplicate/first-packed field precedence,
contiguous component indexing, invalid layout/ranges/overflow and failed-write
atomicity. No original iterator test is retired by these new conformance tests:
mutable pointer aliases, pointer equality and C++ iterator operators remain
unexposed. Missing/invalid native pointer operations are undefined and are not
reproduced; the bounded Rust Option contract is documented explicitly.

## Full native Factory set and text construction

The actual pinned Factory revealed that CLI `--list` omitted 48 imported/nested/
synthetic names: Rust traversed only 223 top-level canonical messages, while the
native Factory registers 271. Listing now uses the maintained descriptor pool's
all_messages iterator. Complete output is compared to the native name set sorted
by name, retaining the documented deterministic Rust ordering. Native Types uses
an unordered set (MessageFactory.cc:156); its traversal order is not portable.

`python3 scripts/factory-reference.py` instantiates every native registered type
and records all 271 names, original valid Vector3d text's exact wire bytes, and
the original malformed text's null result/diagnostic. Binary/header SHA256 and
upstream SHA are recorded in the fixture. The Rust tests use existing public
prost-reflect construction/parsing rather than implementing another parser.
Original NewWithWellFormedData, NewWithMalformedData and NewAllRegisteredTypes
assertions are preserved and mapped pending independent review. Existing five
unchanged C++ CLI tests remain unchanged and pass.

`python3 scripts/verify.py` passes 37 integration tests, native interop, clean
consumer, fmt/clippy and strict coverage: 422/422 owned lines, 595/603 regions.
The smaller CLI denominator reflects replacing its nested top-level-only scan
with a direct all_messages iterator, without new coverage exclusions. External
factory descriptor loading/retry/lazy/concurrency scope remains unfinished.

Independent review receipt: /root/sdformat compared every original initializer/
size assertion against 9916179 and reproduced the 30 metadata, 10 size and 10
follow-on scalar/color native fixtures with matching provenance. No functional
defect found in either cloud slice. /root independently compared the three
Factory cases against a7b99ec and reran the original text/all-type and complete
native-list Rust tests. All five mappings are now reviewed; the coordinator
accepted the chain through ab3fc90 for publication. This accepts bounded slices,
not completed MSG-01/MSG-02 packets or unimplemented iterator operators.

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

## Explicit descriptor loading and missing-import retry

New instance-owned DescriptorRegistry reuses the existing patched maintained
reflection pool. It loads descriptor files in encoded order, retains partial
successes and returns per-file errors for retry. The descriptor set is itself
reflected so custom options remain nested unknown bytes until schemas load;
no custom protobuf parser, unsafe code or additional dependency is introduced.

`python3 scripts/registry-reference.py` reruns the exact original native retry
sequence and captures exact A/B descriptor-set wire inputs, original source SHA,
upstream SHA and native helper binary SHA. The Rust port preserves each original
missing/dependency/reload assertion. Supplementary tests cover partial success
within a set, duplicate names, malformed and IO errors, cloned/surviving descriptor
snapshots, and an actual protoc-generated custom MessageOptions extension whose
string value survives loading. Only the retry case is mapped, pending independent
review; environment/directory/lazy/concurrent factory scope remains unported.

`python3 scripts/verify.py` passes 40 tests, downstream consumer, native interop,
fmt/clippy and unchanged strict coverage gates. This explicit API starts with
canonical descriptors; it does not claim the native dynamic pool's install-prefix
or path-cache behavior. No new coverage exclusion is introduced.

/root/sdformat independently approved the exact implementation/test diff
`aa6e1fabfff654058458d2bc9dd21e45b80918b57173271a41534aced2595af6`.
It compared the original descriptor fields and all three construction outcomes,
reran the native helper and three Rust tests, and verified custom-option retention,
partial retry/snapshots and provenance. Review requested explicit Bar construction
beyond lookup; the final port includes it. Diagnostics now use a relative fixture
directory, preserving native output without capturing a host-specific path. The
single retry mapping is marked reviewed; remaining factory scope is unchanged.

## Exact staged Git blob hardening

The old git archive export could honor export-ignore attributes and omit owned
inputs from the staged denominator. The hook now freezes the index tree and
checks out a private index, including every frozen blob. The independently
reviewed transport 2bbbf8c validation helper is reused: exported path sets, raw
SHA1/SHA256 Git blob identities, file/symlink types and owner-executable bits must
match before build code executes. Checkout EOL/smudge transformations fail closed.
After the full verifier returns, both the real index tree and HEAD must still
match the captured values. No Rust source, coverage denominator, exclusions or
threshold changes are part of this quality slice.

Five scratch-Git tests cover exact staged bytes despite an unstaged fix and
export-ignore, CRLF substitution rejection before the verifier runs, content/
extra-path/file-type/mode substitutions (including mode0645 versus100755), valid
and changed symlinks, unsupported gitlinks, and separate index/HEAD mutations.
They run inside scripts/verify.py. The existing full-build broken Time conversion
versus unstaged fix demonstration remains unchanged in CI.

/root/transport independently approved exact five-file diff
`a39637e54f778c54b65992885358f5f9eeb71f887ba3432e180eb4d96c52146a`
after source review and independently rerunning all five snapshot regressions.
The full-build staged broken Time/unstaged fix demonstration also passes with
the new snapshot implementation. The full 40-test local gate passes with
unchanged owned Rust coverage, 451/451 lines and 645/653 regions. A matching
strict staged gate remains required for this commit.

## Explicit descriptor path lists and one-level directories

The next registry slice preserves the retained DynamicFactory path sequence and
NoDuplicateDescriptors assertions with six upstream descriptor/source fixtures
copied byte-for-byte and pinned in tests/reference/manifest.json. The exact native
C++ path/missing/extension/content/file/directory outcomes and absence of duplicate
errors are captured by scripts/registry-paths-reference.py. Original differing
Bytes/BarBytes/FooBytes queries are retained rather than normalized away. Rust
constructs each accessible dynamic message and isolates the duplicate test in a
child process to preserve the original stderr-substring assertion.

Supplementary tests cover native gz-msgs.gz_desc priority, duplicate directory
reloads, empty path lists, recognized substrings in parent directory names,
missing/invalid directory errors, and platform-reported permission errors. The
API intentionally returns structured path-associated errors rather than printing
native diagnostics. It has no environment/global/lazy or canonical-path-cache
contract yet; those cases remain retained.

The first fully generic traversal failed the strict per-file region gate at
127/135 regions despite merged source views showing hits from different path
argument instantiations. Traversal now shares one concrete implementation behind
thin ergonomic argument adapters, avoiding duplicated traversal code and keeping
all executable logic instrumented. The directory-entry failure region remains in
the denominator. No gate or exclusion changed. Full verification passes 43 tests
and five snapshot regressions: owned Rust 502/502 lines, 731/740 regions; registry
80/80 lines and 136/137 regions. Both original mappings await independent review.


Independent transport review identified an ordering side effect: native loads
its known own descriptor before directory iteration and retains each successful
entry before a listing failure. The eager Rust entry collection could lose that
progress. Loading now attempts the own file first and streams entries. A native
execute-only-directory probe confirms own type availability after the iterator
throws; the Rust test preserves that state and checks the actual OS listing
permission result. No permission-dependent assertion is skipped, including on
systems where the current user can still read the directory.

/root/transport independently approved corrected implementation diff
`0d7e1fc027f17a523b8c5b7174db622e6daaeb1e76c7b182662ab6d0ad2c78ef`.
It reran all three Rust path tests and the exact native binary with a fresh
execute-only directory, verified all six fixtures against immutable Git objects,
and compared both original assertion families including stderr behavior. The
two mappings are marked reviewed. Source, scope and coverage remain as above;
strict staged verification is required before publication.

## Instance factory, lazy environment and parallel retained behavior

New MessageFactory reuses DescriptorRegistry behind std Mutex. Canonical lookup
and registration avoid external initialization; first dynamic lookup/Types/load
reads a default instance's GZ_DESCRIPTOR_PATH once. Explicit paths avoid ambient
environment. Callbacks and argument conversion run outside the lock; replaced
constructors are destroyed after release. Load diagnostics stay observable, and
returned messages retain descriptor snapshots. No worker runtime or process-global
mutable registry is introduced. Install-prefix scanning/path caching are still
explicitly outside this instance API's qualified scope.

The original explicit two-path load, late environment setter and complete
ParallelUse thread/iteration/final assertions are preserved. The environment port
is a harness=false process with no background threads, so its standard environment
mutation occurs safely on the sole thread. It also proves first dynamic use freezes
paths. Four ordinary factory tests cover aliases/errors/text, initial diagnostics,
constructor replacement/reentry and value lifetime. ParallelUse retains four times
10000 generated creates, twice1000 dynamic creates,1000 registrations,100 type
enumerations and20 repeated loads, followed by every original final assertion.

scripts/factory-state-reference.py reruns the three selected original native cases,
requiring exact gtest XML IDs and no failures/skips, and pins source/binary hashes.
A supplemental actual native probe verifies replacement, text clearing, reentry,
literal-dot registrations and enumeration. It found a real maintained-reflection
lookup difference: reflection strips a leading dot; native Factory accepts only
its three Gazebo aliases unless a literal constructor is registered. The facade
preserves native lookup and dynamic-initialization timing without a runtime patch.
Native helper invocations clear only the descriptor-path environment to isolate
these baseline observations; originals may set it inside their own process.

Full local verification passes47 harness tests plus the isolated environment probe
(48 Rust checks), five snapshot regressions, native peers, clean consumer, fmt/
clippy and strict coverage:597/597 owned Rust lines,874/883 regions; the factory
is95/95 lines and143/143 regions. No dependency or exclusion added. Three original
mappings await independent review; MSG-01's separate coordinator acceptance is
linked in README while MSG-02 remains incomplete.

/root/sdformat independently approved final implementation/test diff
`e4ddb1143900595fdf75fe47ee7b3800cbe33afc0443c1c3f0959ca3a7ea428a`.
It compared the three original cases, reran four Rust factory tests plus the
single-thread environment executable, compiled/executed the actual native probe,
and reran all three original native tests with exact one-run/no-failure XML.
Source/native hashes, complete thread counts, aliases, text clearing, callback
reentry/destruction, lifetimes and one-shot diagnostics were checked. An explicit
reentrant-name enumeration assertion was added after review. Three mappings are
marked reviewed; no TSan, global singleton, install-prefix or cache parity claim
is made. The matching strict staged gate remains required for publication.

The next bounded ports retain `FactoryTest.MultipleMessagesInAProto` and
`Messages.NestedHeaders`. The former enumerates and constructs all three messages
from serialized_map.proto. Its twelve nullable C++ unique/shared/const ownership
checks map to typed nullable Rust Box/Arc owners and immutable borrows; these are
Rust ownership forms rather than generated C++ pointer-alias names. The latter
retains every successful serialization and exact 0/0/6/8/equal-inner size check.
The native reference script reruns both exact original cases alongside the three
previous factory cases, pinning sources and native binaries. Both mapping
acceptance and this ownership translation await independent review.

/root/sdformat approved exact corrected staged diff
`666971ade045be1370754721b6b189779f897c3b4c7ea80fcfcb354ba416ed47`,
independently comparing complete original assertions and rerunning both exact
native cases plus both targeted Rust ports. It checked source/binary hashes and
the documented nullable Box/Arc and immutable-borrow mapping. Review corrected
the embedded header construction to be independent of the standalone wrench,
as the original does. Both mappings are now reviewed; final staged gate required.

Plane/AABB conversion slice: public raw-array constructors, component readers and
setters mirror the native conversion headers' pure field assignments. Setters
retain outer/nested metadata, including headers on preexisting normal/size/corner
messages. Missing nested values read as zero; signed zeros and inverted corners
are preserved. This does not implement native math algorithms or the bounding-box
constructor's sorting policy. The original box test explicitly assigns inverted
corners through mutable Min/Max references; our helper retains those values.

`geometry-reference.py` compiles the actual native conversion probe and records16
complete protobuf wire messages across plane/box/default/non-unit/inverted/
signed-zero inputs and both fresh/seeded nested-header messages. It checks reverse
native values and absent-message zero defaults, pins helper/header/original-source
and binary hashes, and reruns four exact original native cases with one-run/
zero-failure XML. Four Rust tests preserve every original plane/box assertion and
compare all16 native bytes, supplementary raw component bits, defaults and header
retention. Four mapping dispositions remain pending independent review.

/root/sdformat approved exact staged plane/AABB diff
`8acacd9cc0a70c59ae27e9360a6eacc27007388ab82884a21ce5a38d63c8708f`.
It independently compared every original assertion, reran four exact original
native cases, freshly compiled the native probe and matched all16 complete wire
rows and default reverse checks, checked all source/header/helper/binary hashes,
and reran four Rust tests. Array component mapping and explicit sorting limits
are accepted. Original scalar tolerances are retained or strengthened. All four
mappings are reviewed. Full local gate passes54 Rust checks,5 snapshot regressions,
fmt/clippy/clean consumer,643/643 owned Rust lines and955/964 regions (geometry
46/46 lines,81/81 regions). No dependency or exclusion changed. Final strict
staged gate remains required for publication.

Safe packed-cloud accesses now exercise the complete original missing-field,
mixed-field and XYZRGBA behavioral assertions through indexed read/write calls.
Some/None current/end observations replace native iterator/end comparisons;
exclusive cloud borrows replace simultaneous aliased mutable iterator references.
All original metadata, seven typed reads/writes, five-record counts, mutable and
const-equivalent passes and all end observations are retained. The original
FLOAT64 field accessed as float remains f32. The Operators test's numeric reads,
copied/advanced positions and arithmetic values are covered, but object identity
and overloaded-operator assertions remain outside the API; its mapping stays
not_ported with partial evidence, not silently retired. No cursor wrapper or unsafe
reference is introduced. Initialization retains all9 scalar default assertions;
absent nested Rust Option fields are explicitly viewed through default values
without insertion. Four full mappings await independent review.

`cloud-original-reference.py` reruns the four exact native cloud cases and
UtilityTest.Initialization, checks one-run/zero-failure XML and saves source/binary
hashes plus logs under target/reference. The existing full-wire scalar/color
corpus remains unchanged. Five targeted new Rust tests and all five native cases
pass; the next full gate will include them before publication.

/root/sdformat approved exact cloud/default staged diff
`86a7ef17bcb9e96512cdefdac08de38ca4eca3a41d3d01e5feb839cefe952d2f`:
complete original source/assertions compared, five exact native tests and five
Rust tests independently rerun; source/native hashes verified. Four full mappings
are reviewed. Operators remains not_ported with explicitly partial numeric evidence.
The full local gate passes59 Rust checks with unchanged643/643 owned lines and
955/964 regions; no production Rust, dependency, exclusion or threshold changed.

Separately /root/transport independently reviewed all15 older pending vector,
color, missing-orientation identity and primitive Set mappings against immutable
Utility_TEST.cc. It verified source SHA, reran exactly15 native original cases and
all9 Rust upstream_ports tests, all passing. All original finite literals,
components, identity/state and two-string overwrite assertions are retained;
Rust exact comparisons strengthen the original bounds. All15 mappings are now
reviewed; direct generated field assignment is accepted as the Rust Set API.
No broader math/color clamping semantics are added to this raw component API.

Component setter slice adds metadata-retaining vector2/vector3/quaternion/color
setters through the existing array conversion macro and a pose setter retaining
outer/nested headers/name/ID. The existing pose constructor delegates to this
setter; plane/AABB setters reuse the vector3 setter rather than duplicate field
assignment. Raw array component semantics are unchanged, without math algorithms,
quaternion normalization or color clamping.

`component-reference.py` compiles the actual pinned C++ conversions and generates
18 complete wire records: five message kinds, fresh/seeded headers and three
rotations (original Euler input, non-unit and explicitly zero). Native reverse
quaternion and pose conversions must preserve component bits. Original Euler
components are generated by the native math library, not reimplemented in Rust.
The script source/binary/header hashes are pinned and nine exact original native
cases rerun with one-run/no-failure XML. Three Rust tests retain all original
component/position/assertion literals and <=1e-6 quaternion bounds, roundtrip both
ways, and compare every native wire row and raw rotation bits. A local Clippy
allowance preserves original long assertion literals without altering their
values. Nine original mappings await independent review; no dependency or
exclusion added. The prior full geometry corpus remains passing.

/root/transport independently approved exact component-setter staged diff
`affdc82edd66fff16aec1be6ff8bde11bee7cddbd5a3759b6bb0436c1da91665`.
It compared every assertion in the nine original cases and five native headers,
reran all three Rust tests and exact nine native originals with matching hashes,
and reproduced18 complete wire rows and three orientation inputs from the probe.
All nine mappings are reviewed. Full local gate passes62 Rust checks,5 snapshot
regressions, fmt/clippy/clean consumer and643/643 owned lines,966/975 regions.
No dependency/exclusion changed; final matching strict staged gate required.

Inertial/mass-matrix slice supplies raw domain component values and fresh/message
setter/read APIs without implementing physical math algorithms or normalizing
quaternions. Native math setter return values are ignored by the original
conversion wrappers and assignments still occur; negative mass/moments are
copied faithfully, with physical validation left to the caller. Source review
identified stateful details: mass-only Set forces only nested orientation W=1,
retains other pose fields and fluid data; full Set appends21 upper coefficients
for Some, leaves old fluid data for None; decode accepts exactly21 entries and
symmetrizes, ignoring all other lengths. These are explicit API contracts.

The actual pinned native probe records24 complete wire and reverse-state rows:
fresh/seeded outer+nested metadata, mass-only/full/optional symmetric/asymmetric
fluid values, repeated appends, and valid/negative physical values. It compares
reverse mass/pose/matrix IEEE bits, not only schema agreement. Header/helper/
source/native binary hashes are recorded; seven exact original native cases
pass with one-run/zero-failure XML. Five Rust tests preserve every original
mass/moment/pose/default/21-coefficient/matrix assertion, compare all24 native
wires and reverse-state bit patterns, and cover absent pose/orientation and bad
fluid lengths. The original <=1e-3 pose component bounds, <=1e-1 zero-pose bound,
and all scalar assertions are retained or strengthened; no quaternion-sign
relaxation is used. Seven mappings await independent review. No dependency or
exclusion changed.

/root/sdformat approved exact inertial staged diff
`6438ccb9577d1eb6ddef0141d978cdf0f014dc2f22ceed6d5f2b108f03445322`.
All seven original assertions and native Inertial/Pose source were independently
compared. It verified fixture/source/binary hashes, freshly compiled the native
probe and reproduced24 complete wire/reverse-bit rows, reran seven exact native
original cases with one-run/zero-failure XML and all five Rust tests. All seven
mappings are reviewed. Full local gate passes67 Rust checks,5 snapshot regressions,
fmt/clippy/clean consumer and714/714 owned lines,1052/1061 regions; inertial values
are71/71 lines,86/86 regions. Physical/math validation remains caller-owned.
No dependency/exclusion changed; final strict staged gate remains required.

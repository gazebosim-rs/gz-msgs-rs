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

Coordinate enum slice explicitly translates native math codes1..4 to canonical
protobuf0..3 and back; malformed values diagnose and fall back to LOCAL. The
actual native probe captures14 return-value/stderr rows and reruns both exact
original coordinate cases, including original500000 invalid inputs. Source/
header/helper/native binary hashes are pinned. Two Rust tests retain every
original enum/fallback assertion and compare all14 returned values. Native
stderr is captured; Rust diagnostic wording is source-inspected, not claimed as
an automatic subprocess equality test. This is numeric interoperability only,
without coordinate transformation algorithms or a new math dependency. Both
mapping dispositions await independent review.

The coordinate enum work is grouped with complete spherical-reference conversion
and assignment into one family. Typed Earth/Moon/Custom/Unrecognized surfaces and
raw radian reference components replace math objects. Fresh/message setters and
readers retain native surface code behavior, stale axes/header/entity retention,
custom-axis fallback (keep CUSTOM, Earth radii), unknown model fallback (Earth),
negative zero, and operation-order-sensitive degree/radian IEEE rounding. This
uses constants/simple unit conversion and the native parameter predicate, without
reimplementing geodetic coordinate transforms or adding a math dependency.

The native spherical probe captures16 complete fresh/seeded setter wires and11
reverse parameter/axis bit rows, including nonpositive, reversed, NaN and infinite
axes, unknown surfaces, negative zeros and overflowing angle values. Caller math
construction diagnostics are separate from conversion diagnostics. The fixture
pins message/core/helper/native binary and math source revision/file/library
hashes. Three Rust tests retain every assertion in the two complete originals,
including their enum checks and four-ULP EXPECT_DOUBLE_EQ bounds, then compare all
wires and reverse bits. Native diagnostics are captured; Rust diagnostic text is
source-inspected rather than claimed as a subprocess equivalence test. Both
original native cases pass exactly1run/0fail. Alongside the enum slice, four
mappings await independent review; no dependency or exclusion changed.

/root/transport approved exact spherical-family staged diff
`5313d24b676d65f90cf26dd48da0410e175eb893f58deaf980bed68b34377428`.
It independently compared all four originals, conversion header and native
Angle/custom fallback, reran five Rust tests and four exact native originals,
and replayed hash-matched probes for14 enum rows,16 wires and11 reverse states.
Source/probe/original/math-library hashes match. Literal values/four-ULP bounds,
500000 fallback, caller-raw constructor scope, stale metadata, custom/nonfinite
semantics and exact operation order are retained. Rust diagnostics are explicitly
source-compared. All four mappings are reviewed; full local gate passes72 Rust
checks,5 snapshot regressions, fmt/clippy/clean consumer and788/788 owned lines,
1123/1132 regions. Coordinate helpers are18/18 lines/regions, spherical helpers
56/56 lines and53/53 regions. No dependencies/exclusions changed. Final strict
staged gate remains required.

## Borrowed packed-cloud cursors (pending independent review)

NativeFieldCursor adds copied cursor identity, contiguous get, point-stride +/+=,
end sentinels and standard Iterator values over an immutable cloud borrow. The
original Operators field/loop/value/copy/advanced assertions are retained; writes
use the existing exclusive indexed API instead of aliased mutable references.
Three Rust tests include all eight native scalar fixture types and both color
orders, unaligned values, missing/malformed fields and bounded end advancement.
The original native four cloud cases plus initialization rerun unchanged and pass.
Negative pointer movement and native malformed-pointer undefined behavior are not
exposed; the Rust safe boundaries are explicit. No dependencies or exclusions change.
Working coverage passes839/839 owned lines and1197/1209 regions, with cursor43/43
lines and67/70 regions; strict final staged gate and independent review remain.

/root/transport independently approved exact staged cursor diff
`ccf35439e757a671e19c6646c1b79d649358d5e8712bb2b02615a6b5feb56a40`.
Every original Operators41–109 assertion was compared; a private-index exported,
raw-blob-validated staged-only snapshot passed three new cursor and four existing
original Rust tests without unstaged Fuel sources/dependencies. All four native
cloud originals and Initialization passed five exact XML runs; source/header and
fixture lineage hashes match. The immutable cursor/exclusive-write API mapping,
byte-alias identity, checked reads and bounded end semantics have no blocking
finding. Operators is now reviewed, bringing original mappings to82/83; Fuel and
all remaining MSG-02 packet obligations remain open. Matching strict staged gate
is required before publication.

## Fuel metadata conversion

The final unported original is UtilityTest.ConvertFuelMetadata558–683. Its four
parse failures, complete model/world fields, invalid file-format failures,
intermediate valid model serialization and exact final model/world XML strings
are retained in tests/fuel.rs. The original C++ case reruns unchanged, exactly
one run with zero failures. New Rust APIs return structured errors/new values;
native boolean mutation, exceptions and stderr are explicitly mapped rather
than claimed as identical language APIs.

A fresh C++ conversion probe captures173 native observations with complete
protobuf wires, exact serialized XML and stderr. It includes first-text/CDATA,
comments/children, entities/NUL views, raw dependency whitespace, resource integer
prefix/range exceptions, SemanticVersion quirks, last-positive SDF selection,
model precedence/multiple roots, namespace spelling, malformed attributes and
captured nesting limits. All30 CR/LF combinations up to four characters prove
left-to-right pair precedence. Hashes bind upstream sources, helper/binary,
original native binary, installed/build-identical math library and SemanticVersion
source, and TinyXML header/runtime. Four Rust tests retain original assertions,
compare native wires/XML, and check explicit safe null-URI/invalid-Unicode errors.
Pinned quick-xml0.38.3 supplies maintained tokenization; native text/attribute and
version quirks have bounded owned adapters. No unsafe code or exclusions added.
Final full gate and independent review remain required; MSG-02 broader obligations
remain open even when the last original mapping is accepted.

Working full verifier passes79 Rust checks,5 snapshot regressions, fmt/clippy,
clean downstream consumer and1136/1136 owned lines with1725/1736 regions. Both
Fuel production files are100% covered (297/297 lines,527/527 regions). All owned
vendored predicates remain measured; no denominator or threshold changed.

Independent SDF review identified four false accepts outside the initial corpus:
terminal document text and malformed element/attribute names. The corrected
adapter applies pinned TinyXML byte-name grammar only to elements/attributes and
requires document text to terminate before markup. All reviewer inputs and
additional leading/terminal comment/CDATA/reference, permissive prolog/control
and valid/invalid name controls are bound into the173-row native oracle. The
corrections pass full gate; independent re-review remains required.

/root/sdformat approved corrected exact staged diff
`e16abdcdee3b27845ec11116a82e81699b2473777d96de59dedfaf2b9516246b`.
It independently compared every original parse/field/format/intermediate/final XML
assertion, freshly replayed all173 native full-wire/XML observations and11 extra
edge controls, and ran four Rust tests in an independent consumer excluding the
separate unstaged CLI work. The exact original C++ case passes one run/zero
failures; source/probe/header/original-binary hashes agree. Name/terminal-text
corrections and all declared structured-error/safety/API limits have no blocking
finding. All83 original mappings are reviewed. Full XML/formatting, descriptor
installation/global/cache behavior, packaging and equal-work performance remain
open MSG-02 obligations; no whole-packet acceptance is claimed. Matching final
strict staged gate is still required.

## CLI missing-type continuation (pending independent review)

Native runMsgInfo reports each unknown type to stderr, keeps processing later
names and exits successfully. The Rust tool previously returned early/nonzero;
two supplementary Rust assertions incorrectly expected that behavior. They are
replaced with exact native stdout/stderr/status comparisons for nine invocations,
including mixed valid/invalid names, repeated failures, empty/Unicode names,
short flags and the precise leading-dot alias whitelist. All original five C++
CLI tests and271 successful descriptor outputs remain unchanged and pass.
New source/helper/native binary hashes bind the oracle. Working full gate passes
80 Rust checks,5 snapshot regressions, fmt/clippy/clean consumer and1143/1143
owned lines,1731/1742 regions. Full CLI grammar/dynamic descriptor formatting,
packaging and other MSG-02 obligations remain open. Independent review and matching
strict staged gate are required before publication.

/root/transport approved exact CLI staged diff
`d09bfe16e13b7e80f9e2410ce9051d331ca472f09f9474647eee15c0fcab513f`.
It independently compared native runMsgInfo and MessageFactory alias conversion,
replayed all nine native stdout/stderr/status observations, verified source and
binary hashes, and ran all eight Rust CLI tests including271 unchanged successful
goldens and the unchanged five C++ originals (five run/zero failures). No blocking
finding within the documented bundled-type/grammar scope; original assertions
remain unchanged. Matching strict staged gate remains required.

## Immutable source packaging (pending independent review)

A normal Cargo `.crate` probe confirmed the known unsupported release path:
normalized dependencies lose both reviewed vendor paths. Extracted registry-form
source fails both signed_zero tests (zero passed/two failed), so publish=false
remains enforced. That deliberate negative probe is not a qualified distribution.

The supported candidate uses raw immutable Git blobs, preserving all vendored
source/licenses and unmodified Cargo paths. Its manifest binds modes, object and
SHA256 hashes to the original Git commit/tree; independent reconstruction delegates
tree ordering to Git. Standard tar/gzip metadata is deterministic and extraction
checks closed paths/types/modes/content and expected revision. Regression evidence
includes CRLF/export-ignore/executable/symlink preservation, untracked/unstaged
exclusion, deterministic bytes and wrong-revision/blob/mode/path/tree/link rejection.

For public4357fc1c1f8a245a822e97f61a012125e69f86a4 the611-file archive is432KiB,
SHA2562a722e029dba1ab785622aa471eab862882e0fb503d47a31c7178e864dc983f4.
Two output locations produce identical bytes under Python3.12.3/zlib1.3. A fresh
extracted source plus empty private Cargo home fetches locked dependencies, then
installs offline into a local prefix. All nine installed CLI observations and a
clean downstream signed-zero/standard-Any/nested-unknown consumer pass without
Cargo patches. The actual executable links only libc/libgcc/loader/vdso, with no
Gazebo or protobuf C++ runtime. Rust1.94.1/protoc3.21.12 versions are recorded in
ignored artifacts/source-package-check/proof.json. No shared configuration or
installation is changed. This is a Linux source-install proof, not a full binary
or clean-OS release matrix. CI artifact publication and independent exact-source
review remain pending for the packaging implementation itself.

Independent packaging review found that Git silently ignored a forged ./ path,
allowing an uncommitted member despite an authentic tree. Before publication,
validation now rejects raw noncanonical components and compares every manifest
entry against the complete reconstructed index (mode/OID/stage/path). Six forged
path/overwrite/ignored-.git variants and the reviewer's independent injected
archive all reject. The implementation remains under re-review; no vulnerable
packaging implementation was published.

/root/transport_recovery approved corrected full packaging diff
`7feace21776e3b9ac00846b52f4fd487ef578d9f984ceef985e8c23495882a27`.
It independently reran the archive suite/six injection variants and rejected the
preserved exploit with its authentic expected revision, reproduced the exact
611-file4357 source archive/tree/SHA, then performed an isolated offline install,
nine CLI observations and clean downstream consumer with no native Gazebo/protobuf
runtime links. Fresh-cache evidence was inspected without claiming a second fetch;
full80-check coverage was not independently repeated. The qualified receipt is
[recorded](reviews/source-package.json). Matching strict staged and hosted gates
remain required; source-install qualification does not enable crates.io publishing
or accept the full release matrix/MSG-02 packet.

The first mandatory staged commit gate rejected a hook-context-only archive test
failure: inherited GIT_INDEX_FILE targeted a nonexistent .git/index inside the
private bare tree verifier. Git child calls now clear Git's own --local-env-vars
list before selecting their explicit repository. Regressions run under relative
hook index and foreign directory/worktree/index environments and prove the
foreign sentinel index/config remain untouched. No hook was bypassed; the
corrected source must pass a new strict staged gate and independent delta review.

/root/transport_recovery independently approved the hook-context correction
`5109ba7e6fd1886199fb2186938dd0c72e8ec5d9aacb6383cf1b0ce365909d3e`.
Standalone, Git-less relative-index and independent foreign directory/common-dir/
worktree/index checks pass without touching the caller sentinel or creating a
foreign repository. Authentic archive bytes remain exactly2a722e... . The prior
full source/install review remains applicable; no repeated fetch/full coverage
claim is made by this delta review. The [receipt](reviews/source-package-context.json)
and full local80-check gate pass precede the new mandatory staged hook.

## Known-tag/wrong-wire qualification

Native probing after source-package publication exposed a boundary absent from
the original MSG-01 interoperability corpus and all83 original assertion cases:
libprotobuf3.21.12 accepts known tags with unexpected wire types as unknowns;
generated prost and DynamicMessage reject them. Earlier acceptance is bounded
evidence, not complete wire equivalence. No original mapping is reclassified or
weakened, and MSG-02 remains open.

The new compatible WireMessage path retains the normal typed success path and
builds a temporary descriptor-bound known-field view only after an error. A
sealed generated trait binds canonical types and schemas. Original bytes remain
verbatim on both paths; nested Any fields/maps/oneofs are visited with a shared
100-level native recursion budget and byte payloads remain opaque. No vendor,
dependency or coverage exclusion changes are involved. Direct prost, strict
WireMessage::decode and general reflection remain explicitly stricter.

All70 native observations agree between actual installed generated C++ and
descriptor-based C++ acceptance. The freshly compiled canonical generated C++
test replays each complete status/output, including native known-only outputs.
The compatible Rust values match those decoded native known-only values across
presence, duplicate merge/order, maps/defaults, scalar packing, nested Any,
malformed framing/UTF-8 and recursion-limit cases. Rejected diagnostic formatting
is unqualified. The native regeneration script verifies all216 schema blobs
against pinned Git, records probe/library/schema/corpus hashes, and writes proof
without rewriting the checked-in fixture. All70 installed-parser outputs/statuses
were freshly reproduced. The clean downstream standard-Any consumer now also
checks compatible decoding without any Cargo patches.

Working coverage passes:1258/1258 owned Rust lines and1925/1937 regions; wire.rs
88/88 lines157/158 regions; measured build.rs106/106 lines148/148 regions. The
normal full and staged gates, independent review and hosted CI remain required.
Consumers must opt into this API after publication before claiming the newly
qualified boundary. No complete parser/runtime/platform or MSG-02 claim is made.

/root/transport_recovery approved exact staged implementation c0bec749 after
independently exporting and validating all 623 raw Git blobs. Both new Rust tests
and all 70 fresh generated C++ outputs/statuses pass from that snapshot, excluding
the unstaged JSON follow-up. Its installed-native proof equals the author's proof,
including all 216 schema checks; the clean downstream consumer also passes without
Cargo patches. No independent full coverage rerun is claimed. The qualified
[receipt](reviews/wire-compatibility.json) retains strict/reflection, allocation
quota, recursion, map-normalization and lossy reencoding boundaries. Matching
strict staged and hosted verification remain required before adoption.

## JSON document framing

`encode-json` now calls the maintained deserializer's end check before encoding
the parsed message. The previous streaming parse accepted trailing garbage, a
second value, NUL or invalid UTF-8 after a valid document. The native 14-row corpus
and fresh helper replay compare 12 complete status/output observations; two
native permissive-grammar examples remain explicitly unsupported. No runtime,
dependency, original assertion mapping or coverage exclusion changes are made.
The full local gate passes 83 Rust checks, five snapshot regressions, archive QA
and clean downstream compilation: 1258/1258 owned lines and 1933/1945 regions.

The independent review caught a stale normal-target executable that still
accepted suffixes. Its debug compilation directory identified the earlier
reviewer's wire snapshot; a shared producer target had overwritten the normal
binary, and Cargo's timestamp-based freshness did not detect the source change.
Fresh isolated reviewer and coverage targets passed. The stale binary, hashes,
debug provenance and failed replay logs were retained locally. Cleaning only
owned package outputs restored a working-source executable and all nine CLI tests
passed, including the five unchanged C++ originals. No assertions or source were
altered to hide the failure. Repository guidance now requires separate reviewer
targets and source/binary provenance beyond a Fresh message or mtime.

Independent /root/sdformat approved the bounded source correction and explicit
grammar limits after its fresh-target test and native-proof replay. The matching
qualified receipt, normal staged hook and hosted CI remain required; no full
JSON/CLI grammar or MSG-02 acceptance is implied.

The independent [receipt](reviews/json-framing.json) binds the corrected source,
metadata and reviewer-target policy. Its fresh isolated target passes all nine
CLI tests, including the unchanged C++ originals, and reproduces every native
fixture status/output. It records the initial cache failure and its proven
resolution; the author's full coverage gate is not misreported as an independent
rerun. Source and assertions remain unchanged after review.

## Bounded native option combinations

The CLI now uses pinned maintained clap_lex 1.1.1 for GNU tokenization and a small
native message-option policy. Attached/repeated info, multiple names, short
flag groups, callback priority, list/info exclusions and native missing/extra
argument statuses are covered by 75 observed invocation rows. Tests preserve
exact info/error bytes where meaningful; list comparison retains the complete
sequence after sorting, while help/package-version bind this Rust tool as in the
original C++ assertions. Fifteen separate long flag-assignment observations
remain explicitly unsupported. No full CLI11 grammar, external-descriptor info,
formatting, performance or MSG-02 acceptance is claimed.

The native reference script freshly replays all 90 observations and verifies the
pinned original source, binary and schema identities. Native argv[0] is fixed to
gz-msgs so its help usage name is reproducible. The prior original CLI and message
assertions remain unchanged. One maintained lexer dependency is added; no other
locked version changes or coverage exclusions are introduced.

The first full gate rejected an LLVM source filename alias containing bin/../.
That helper was measured at 120/120 lines and 217/217 regions but correctly failed
the exact source-accounting gate. Moving it to the standard nested Rust module
layout removes the alias without changing the checker or assertions. Cargo
metadata confirms exactly one executable. The revised full gate passes 84 Rust
checks, five snapshot regressions, archive QA and clean downstream compilation:
1380/1382 owned lines and 2155/2169 regions; the new helper remains 120/120 and
217/217. Independent exact review and strict staged/hosted gates remain required.

The [wire isolation follow-up](reviews/wire-isolation-followup.json) retains the
original review receipt and explicitly corrects its shared producer-target
mistake. The reviewer reran both wire tests and all 70 generated-native outputs
in a fresh private target and reverified all 623 source blobs. The downstream
consumer target was snapshot-owned. No original receipt was silently rewritten,
no producer source was changed to hide the initial JSON cache failure, and the
published wire staged/hosted checks remain green.

Independent /root/transport_recovery approved exact staged 62eafc52 after raw
export and verification of all 633 blobs. Its fresh private target passes all ten
CLI tests and the five unchanged original C++ assertions. The native script
freshly reproduces all 90 observations with an identical proof, including fixed
argv[0] and source/binary/schema/corpus hashes. The [receipt](reviews/cli-options.json)
reviews the lexer-only dependency, native option policy and explicit unsupported
assignments; no producer target was shared. Full coverage remains author evidence.
Matching final metadata, strict staged and hosted verification remain required.

## Native long flag assignments

Long list/version values now follow the observed native boolean callback rules,
including words, signed base-zero integers and range behavior. Help remains
presence-based. Version conversion runs before list conversion, then help and
validation; the final assignment wins and even a false list value retains its
exclusion with info. Native overflow's original-first-character sign test and
range-before-trailing-suffix behavior are explicitly preserved. Standard checked
Rust integer conversion supplies the numeric result; no new dependency, native
ABI or unsafe code is introduced.

The original fifteen unsupported inputs remain verbatim, promoted to exact
native outcomes instead of asserting an unsupported error. All 75 previous
matching inputs remain. An additional 171 native observations cover the three
flags, word/numeric forms, radix prefixes, signed boundaries, whitespace/suffix
failures and callback precedence. Fresh native replay verifies both corpora and
source/binary/schema identities. The pinned GNU binary accepts binary integer
prefixes; that measured profile is retained rather than assuming an older C
strtoll grammar. Existing Rust help/version and complete-list bindings remain.

The full local gate passes 85 Rust checks, five snapshot regressions, archive QA
and clean downstream compilation: 1448/1450 owned lines and 2275/2289 regions.
The helper is 188/188 lines and 337/337 regions. No original C++ assertions or
retained mapping are removed, and no coverage threshold/exclusion changes occur.
Independent review and exact staged/hosted gates remain required. Full CLI11
container/grammar, non-UTF-8 argv, external-descriptor info, formatting and wider
MSG-02 qualification remain open.

Independent /root/transport_recovery approved staged 2de39b13 after raw export
and verification of all 636 blobs. Its fresh private target passes all eleven
CLI tests plus five unchanged C++ cases. Both native corpora reproduce all 90
and 171 observations with complete proof equality. It verified all original
75 matching and fifteen formerly unsupported rows remain unchanged, the stronger
list-stderr assertion, and checked radix/range conversion against installed CLI11
source. The [receipt](reviews/cli-flag-values.json) distinguishes targeted review
from the author's full coverage gate. No producer target was shared. Matching
strict staged and hosted verification remain required; broader grammar remains
open.

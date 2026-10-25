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

## Native info vector values

The info option now uses literal bracket/comma expansion in native order, with
an explicit Rust stack rather than recursive calls. Empty expansions do not
satisfy a required value; every info occurrence consumes further argv, including
flag-looking strings, until it contributes a name. An empty attached assignment
also falls through to argv, while a separate empty string remains a name. The
119 native rows compare complete outputs/statuses, including whitespace, nesting,
malformed brackets, aliases, short/attached/repeated options and required-value
precedence. All earlier option/flag corpora are unchanged. An ignored prototype
also matched 264 generated nesting cases; those do not replace checked fixtures.

The full working gate passes 86 Rust checks, five snapshot regressions, archive
QA and the clean downstream consumer. Owned coverage is 1465/1467 lines and
2303/2317 regions; the CLI helper is 205/205 lines and 365/365 regions. No new
dependency, unsafe code, threshold/exclusion change or original mapping retirement
is introduced. Independent review and matching staged/hosted gates remain pending.
Full CLI11 grammar, non-UTF-8 argv and external descriptor info remain open.

Independent /root/transport_recovery approved the immutable vector tree 22a3e9bf
and its bounded c4f7228b diff after verifying all 638 raw blobs and eight source
bindings, including equality to the earlier reviewed flag parent. A fresh private
target passes all twelve CLI tests plus five unchanged C++ cases. All 119 native
observations reproduce the complete author proof. It checked literal untrimmed
comma splitting, iterative order and per-occurrence required consumption against
installed CLI11 source, and verified the prior 90 and 171 corpora remain unchanged.
The [receipt](reviews/cli-info-vectors.json) excludes separate external-list work
and distinguishes this targeted replay from the author's full coverage gate.
Final integration retains the now-public flag review metadata unchanged; matching
strict staged and hosted checks remain required. No broad MSG-02 claim is made.

## Descriptor-aware CLI listing

The list command reuses the existing MessageFactory inventory to combine the
immutable canonical base with accepted GZ_DESCRIPTOR_PATH types. This removes
its separate canonical-only enumeration without adding a registry or dependency.
Eleven native complete-name-set/status/stderr cases cover unset/empty paths,
direct files, combinations/order/duplicates, directories/priority and ignored
paths. Rust tests exercise --list, -l and list per case. Imported Time, custom
nested types and a synthetic map entry are present. The checked source regenerates
exact descriptor bytes through protoc. Shared canonical expected names are stored
once; each case still compares its complete sorted output without dropping names
or allowing duplicates. Original descriptor fixture bytes remain unchanged.

Native replay binds the CLI source/commit/binary, linked Gazebo/protobuf library
hashes, schema manifest and each fixture input. Fresh replay reproduces all eleven
sets. The full working gate passes 87 Rust checks, five snapshot regressions,
archive QA and the clean downstream consumer: 1460/1462 owned lines and 2297/2311
regions. No dependency, unsafe code, original mapping or threshold/exclusion
changes occur. The final provenance guard and factored-fixture checks are included
in this full pass. Independent review and matching staged/hosted gates remain
required before publication.

The qualified extension is accepted descriptor names. External info formatting,
native malformed-path diagnostics, environment-first conflicting canonical file
redefinitions, install/global/path-cache behavior, non-UTF-8 argv and broader
formatting/release/performance contracts remain explicit unfinished scope; see
[the listing contract](CLI_EXTERNAL_LIST.md). No MSG-02 acceptance is claimed.

Independent /root/transport_recovery approved staged efd6fe09 within the stated
valid-descriptor scope. It verified all 646 raw blobs from index tree 08e47951,
used a fresh private target, and passed all twelve existing CLI tests, the new
external-list test and five unchanged C++ cases. All eleven complete native
lists and their full proof reproduce exactly, including linked library/fixture
hashes and the regenerated nested/map/import descriptor. It checked all three
Rust aliases and the existing production factory reuse. The exact authored
[receipt](reviews/cli-external-list.json) distinguishes targeted replay from the
author's full gate and preserves diagnostic/conflict/info/install/cache limits.
No producer target was reused. Matching final metadata, strict staged and hosted
qualification remain required before claiming a published revision passed.

## Bounded external descriptor definitions

The new maintained-reflection formatter compares all 271 canonical dynamic
message definitions with freshly compiled C++ protobuf DebugString output. This
uses identical descriptor inputs; it does not substitute dynamic JSON-name
metadata for the existing immutable generated canonical CLI goldens. Custom
fixtures qualify maps, nested declarations, source order, aliases, oneofs,
proto3 optional fields, explicit false options and JSON-name presence. Proto2,
reserved/extension declarations and unsupported options return typed errors.
Twenty-five complete native CLI observations include supported output, explicit
unsupported cases and missing/canonical/external continuation. Descriptor fixture
regeneration and source/schema/binary/library bindings are checked independently.
See [the contract](CLI_EXTERNAL_INFO.md). The final working gate passes 94 Rust
checks (93 harness tests and the standalone factory environment executable),
five snapshot regressions, archive QA and a clean downstream consumer. Owned
production coverage is 1642/1646 lines and 2633/2651 regions; the new formatter
measures 183/185 lines and 351/356 regions. A follow-up raw-descriptor guard
rejects proto3 defaults and Required labels that reflection accepts but native
validation rejects; both native failure paths
are tested. Independent review found a malformed-map panic in reflection
convenience accessors; checked map layout/type access now rejects it. One valid
and eight malformed raw map cases have matching native acceptance/rejection
controls. Four unchanged upstream descriptor-fixture types extend actual CLI
coverage beyond the custom corpus. A further native-backed regression retains
the observed JSON-name synthesis gap rather than comparing only normalized
metadata and hiding the original-byte difference. Its complete gate passes with
the same production coverage. Existing patched-vendor ownership remains covered.
No original mapping or MSG-02 acceptance changes are claimed.


Independent /root/transport_recovery approved staged 9d4d3fa5 within the partial
normalized-reflection/native-layout scope. It raw-exported and verified all 660
blobs, used a fresh private source/target and passed 25 focused Rust checks,
including the unchanged probe that previously exposed the map panic. All 271
dynamic definitions and 25 complete native CLI rows/proof bindings reproduce.
Additional mixed good/unsupported/later requests pass for all three aliases.
The exact authored [receipt](reviews/cli-external-info.json) preserves the original
panic evidence, raw JSON-name omission mismatch and remaining grammar/loading
limits. Its focused checks remain distinct from the author's full 94-check gate.
Matching final metadata, normal strict staged verification and hosted success
remain required before declaring a published revision qualified.

## Worktree hook Git-context isolation

The staged exported verifier previously inherited repository-local Git variables
from its parent hook. A real linked-worktree pre-commit regression reproduced a
foreign bare-fixture initialization failure under absolute `GIT_DIR`/common-dir
context. The failure is retained in ignored evidence; it ran only in disposable
repositories. The exported child now removes Git's official local-variable list,
while the parent retains the hook context needed to capture and recheck the actual
intended index. A deliberately different normal index proves that distinction.
Six snapshot tests pass, including foreign normal/bare creation, unchanged caller
HEAD/index/config, preserved nonlocal settings and the previous identity/mutation
checks. A full production normal pre-commit hook in a disposable linked
worktree passes, preserving its config and source tree. The complete working
gate also passes 94 Rust checks, six snapshot regressions, archive QA and the
clean downstream consumer, with unchanged 1642/1646 owned lines and 2633/2651
regions. Independent review and matching staged/hosted qualification remain
pending. No Rust/dependency/ledger/threshold/exclusion changes occur.

Independent /root/transport_recovery approved staged e4aa1af6. It raw-exported
and verified all 661 blobs, passed all six snapshot regressions, and confirmed
that substituting only the old private driver makes the real-worktree test fail
foreign bare initialization. Restoring the reviewed source passes all six again.
It checked intended alternate-index preservation, unchanged parent guards and
child-only official variable removal. The exact authored
[receipt](reviews/worktree-git-isolation.json) distinguishes those independent
checks from the author's full production-worktree-hook and working coverage
runs; no live repository reproduction or shared Cargo target was used. Matching
final metadata and normal staged/hosted qualification remain required.


## Proto3 reserved declarations and byte escaping

The formatter extends its bounded scope to reserved message/enum ranges/names and
arbitrary UTF-8 JSON names using Rust's standard ASCII escape iterator, with the
native byte-octal form substituted for hex escapes. Existing native output/status
for all previous 25 CLI rows remains byte-identical; three formerly Unsupported
rows now compare their full native output. Two new complete CLI rows cover
range/name/escaping combinations. Five source descriptor fixtures regenerate
exactly, and the 271 dynamic canonical definitions remain unchanged.

Native raw-range probes exposed permissive behavior that a strict Rust Range
interpretation would lose: empty/reversed ranges, negative ends and over-max
ends are accepted and have specific display rules. Nine defined native controls
are preserved. Two additional source-grounded message-boundary guards reject
native signed arithmetic overflow; observed undefined output is not a target.
Earlier stricter prototype failures remain in ignored evidence. Existing malformed
maps/default/Required/options/depth guards and the raw JSON-name omission mismatch
regression remain. The full working gate passes 96 Rust checks (95 harness
tests plus the standalone environment executable), six snapshot regressions,
archive QA and downstream/owned-patch checks. Owned coverage is 1704/1708 lines
and 2715/2734 regions; the formatter measures 245/247 lines and 433/439 regions.
Independent review and exact staged/hosted qualification are pending; no original mapping or MSG-02 acceptance changes occur.

Independent /root/transport_recovery approved staged 195dc86e within these bounds.
It raw-exported and verified all 666 blobs, passed all nine definition tests and
132 additional ASCII/Unicode native descriptor comparisons for JSON/reserved
names, and reproduced the complete 27-row CLI proof. All prior 25 native outputs
remain byte-identical. It independently checked the pinned message-range signed
arithmetic source; the two overflow guards remain outside native execution.
The exact authored [receipt](reviews/reserved-definitions.json) records the
reviewer-only probe compile correction and restored candidate bytes, distinguishes
focused checks from the author's full 96-check gate, and retains the raw-name
provenance/advanced-grammar limits. Matching final metadata and normal exact
staged/hosted qualification remain required.

## Source-aware external definitions

The registry now retains only accepted raw JSON-name omission metadata alongside
its descriptor pool. `DescriptorRegistry::definition` binds formatting to that
snapshot; `MessageFactory::registry_definition` clones it under the lock and
formats outside the lock without executing constructor callbacks. Canonical
cached definitions and aliases retain precedence. The CLI uses this source-aware
path. The standalone descriptor formatter remains a normalized reflection view,
and its native-backed mismatch assertion remains visible rather than rewritten.

New native comparisons cover omitted/explicit empty/default names, nested fields,
maps, oneofs and optional fields. Both duplicate load orders preserve the first
accepted file; failed imports cannot poison a later accepted file; clones retain
their matching descriptor/provenance snapshots. Existing custom-option retention
and ordered retry tests pass, since original reflective bytes still feed the pool.
An initial supplemental fixture used repeated explicit JSON names and was rejected
by reflection; the corrected unique-name fixture is retained with the earlier
failure log in ignored evidence. No production behavior or assertion was weakened.

The full working gate passes 99 Rust checks (98 harness tests plus the standalone
environment executable), six snapshot regressions, archive QA, clean downstream
and owned-patch gates. Root handwritten production measures 1763/1766 lines and
2815/2834 regions; the two owned vendor patches add 15/15 lines and 36/36 regions,
for 1778/1781 owned lines and 2851/2870 regions. Registry/factory/formatter/CLI
files each exceed both 95% thresholds. All 27 previously qualified native CLI
rows and all 271 same-metadata dynamic definitions remain tested. Independent
review and exact staged/hosted qualification are pending. Advanced descriptor
features, broader CLI/loading/install/performance scope and MSG-02 remain open;
all 83 reviewed original mappings remain unchanged.

Independent /root/transport_recovery approved staged 8d71cb2c/tree 275209ce.
It raw-exported and verified all 667 blobs into private source/target directories,
passed all 20 definition/factory/registry checks and a supplementary rollback,
constructor-collision and actual native CLI omission matrix. Fresh native replay
reproduced the complete 27-row proof with identical bindings. It reviewed new-file
acceptance, failed-build rollback, duplicate filtering, COW snapshots, by-name
source binding and factory lock release. The authored
[receipt](reviews/source-aware-definitions.json) records a reviewer
reference-link setup race and overwritten initial log honestly; corrected replay
passed without changing producer source or targets. Focused independent evidence
is distinct from the author's full 99-check/coverage gate. The descriptor-only
normalized view and remaining advanced-feature limits remain explicit. Final
metadata binding, normal strict staged hook and hosted qualification are required.

## Basic proto2 declaration subset

Ordinary proto2 optional/required/repeated labels, nested messages/enums, oneofs,
maps and reserved declarations now use the existing native layout. The new fixture
includes a nonzero first enum value and packed/deprecated options. Six new complete
CLI observations bring the native corpus to 33; all prior 27 native outputs remain
unchanged. Explicit defaults, groups, extensions and advanced options stay
Unsupported. The legacy default/extension input remains rejected at its first
extension guard rather than the former blanket syntax guard. Known malformed
required/repeated oneof fields accepted by reflection but rejected by native
return a typed error. This does not add a proto2 wire/value contract.

Isolated directly mutated descriptor probes expose a separate maintained-loader
boundary: raw default `010` yields native eight versus reflection ten; native
accepts raw `0x2a` as 42 while reflection rejects it. Protoc normalizes those
literals before writing descriptor sets, so ordinary generated fixtures hide the
difference. Fresh C++ output and isolated Rust assertions are retained in ignored
evidence. No descriptor normalization, runtime fork or general native proto2
default-value equivalence is claimed; explicit defaults remain unsupported here.

The focused 14-test definition replay and fresh 33-row native script pass. An
initial prototype harness accidentally included the default-bearing fixture in
the reserved-only equality loop; that original loop was restored unchanged, and
the dedicated proto2 matrix asserts either complete native output or the explicit
Unsupported result for every message. The failed prototype log is retained.
The full working gate passes 101 Rust checks (100 harness tests plus standalone
environment executable), six snapshot regressions, archive QA, downstream and
owned-patch gates. Root production measures 1774/1777 lines and 2834/2852 regions;
owned patches add 15/15 lines and 36/36 regions, yielding 1789/1792 owned lines
and 2870/2888 regions. The formatter measures 270/271 lines and 468/471 regions.
Independent review and exact staged/hosted qualification remain pending. Canonical
schemas/cache, runtime codecs, dependencies, exclusions and all 83 reviewed
original mappings are unchanged; broad MSG-02 remains open.

Independent /root/sdformat approved staged cce80bcc/tree 684579a3 within this
subset. Its raw-Git export and fresh private target passed all 100 harness tests
plus the actual standalone FactoryLazy executable (101 checks), including all 14
definition tests. Fresh native replay reproduced all 33 CLI observations and the
entire author proof; all prior 27 native status/stdout/stderr triples remain
unchanged. It reviewed the declaration labels and default/group/oneof guards,
canonical paths and generated omitted-proto2 syntax. The legacy Rust diagnostic
change is explicitly approved while its input remains rejected. The authored
[receipt](reviews/proto2-definitions.json) preserves the explicit default/loader,
group/extension, complete-validation and codec limits. Source, tests, fixtures and
native inputs remain unchanged by this review metadata. Matching final binding,
normal strict staged hook and hosted qualification remain required.


## Descriptor byte-default robustness

The independent raw-default study reproduced public registry panics at pinned
baseline d5f4416: octal `\400`/`\777` overflow only in debug (release wraps), while
`\x€`/`\x1é` split UTF-8 slices in both modes. Native libprotobuf 3.21.12 accepts
all four, with effective bytes 00, ff, e282ac and 01c3a9. The exact original
serialized inputs, native outputs/statuses and provenance are retained in the
new five-row fixture; `\\777` is an accepted literal-slash control. This is an
explicit safe rejection, not a claim of native default parity.

An owned preflight returns the existing DecodeError before those operations for
new-file byte defaults, including file/message extensions and nested fields.
It preserves raw reflective bytes/custom options, keep-first duplicate names,
prior successful files, pool/provenance snapshots and retry. Ordinary malformed
escapes still reach the dependency and return DescriptorError; unsafe suffixes
after an earlier ordinary error are unreachable. No vendor, schema, codec,
dependency or coverage exclusion changes. A local documented deprecation
allowance preserves the current public DecodeError API using prost's only
arbitrary-description constructor. The contract records the added linear scan,
structural-validation ordering caveat and lack of performance/default-parity
claims. Existing explicit-default formatter rejection remains in force.

`cargo test --locked --test byte_defaults` and its `--release` counterpart pass
all four new tests, including freshly compiled C++ default values on exact raw
inputs, valid/raw/invalid escape controls, traversal, duplicate/partial/retry and
actual factory/file/CLI paths. The initial debug run passed with the two prost
constructor deprecation warnings; the documented narrowly scoped allowance was
added before the passing strict gate. The normal full working gate passes 104
harness tests plus the actual standalone FactoryLazy test (105 checks), all native
and existing custom-option tests, six snapshot regressions, archive and downstream
checks. Root production measures 1837/1840 lines and 2925/2943 regions; unchanged
owned patches add 15/15 lines and 36/36 regions, totaling 1852/1855 lines and
2961/2979 regions. Registry coverage is 172/172 lines and 289/291 regions.
The fixture hash guard was added after the first full working gate loaded its
script; the mandatory exact staged hook will qualify that final input as well.
Independent review, final exact binding, normal strict staged hook and hosted
qualification remain required. The earlier isolated 143-case study and all 83
original assertion mappings remain preserved; broad MSG-02 is still open.

Independent /root/transport_owner approved frozen tree b4f9e43e/diff 2a494abe
with no findings. A raw 675-blob export and nine exact file hashes were verified;
a fresh reviewer target passed four debug and four release tests, plus 22
registry/factory/definition checks. Both default suites compiled their own native
byte-value peer. The [receipt](reviews/descriptor-default-robustness.json)
approves the explicit native rejection, error disposition, scanner/duplicate/raw
metadata behavior, retry and constructor allowance while keeping wider default
and MSG-02 limitations open. Only this additive evidence and the unchanged
receipt follow the reviewed source; final binding and normal staged/hosted
qualification remain required.

## Source-preserving registry integer defaults

An opt-in prost-reflect file loader now interprets eligible ordinary proto2
integer defaults with the measured native Linux LP64/C23 rules while retaining
raw default strings in descriptor metadata and exports. The registry calls that
loader after the published byte-default preflight. Standard reflection entry
points, extensions, proto3 defaults and noninteger scalar paths remain unchanged.
Nonrepeated fields are eligible; oneof labels must be optional. Declaration
formatting still returns Unsupported for explicit defaults. The
[integer contract](INTEGER_DEFAULTS.md) records exact radix/sign/whitespace/NUL,
range saturation and 32-bit assignment narrowing behavior, including native
acceptance of binary prefixes through C23 libc imports. No general native
validator, proto2 wire/formatter, group/extension or performance claim is added.

The 990-row scalar corpus retains all 190 original input bytes, native statuses
and accepted values, plus the original 190 standard-pool values/errors as actual
assertions. Fresh native C++ compares build outcomes, effective defaults and
explicit-presence wire bytes. There are 640 accepted and 350 rejected scalar
rows. Eight native label controls show optional-oneof and ordinary-required
int32 defaults accepted as 8/42, while repeated and required-oneof defaults fail.
The latter stay on the legacy reflection path, preserving the documented
native/reflection structural-validation gap. New tests also check empty optional
message decoding, raw field/file bytes, snapshots, ordered errors, duplicate
keep-first, import retry, factory/text and actual CLI unsupported/missing paths.
The clean downstream consumer verifies native octal 8 with raw 010 through direct
vendored dependencies, preserving all prior signed-zero/Any/envelope assertions.

Pinned primary descriptor.cc bytes, SHA and commit were verified from preserved
source lineage. Fresh peer/source/binary, installed libprotobuf/libc/libstdc++
hashes and C23 imports are recorded without claiming a fresh protobuf rebuild.
The reference script regenerates 990 scalar and 8 shape inputs and binds its own
source, raw corpus and original-study hashes. Invalid native stderr is retained;
Rust/native diagnostic text parity is not claimed. A first prototype omitted
C23 binary-prefix support and used a missing test-only extension dependency;
its failing log is preserved. Native observations corrected the parser and an
independent local extendee corrected the test fixture. No original assertion
was weakened or removed.

Coverage accounting was independently reviewed before adopting the new checker.
The initial proposal lacked source-hash verification inside the direct checker
and used only region starting lines; the reviewer reproduced a stale-source
counterexample and identified five covered boundary-spanning regions. V2 binds
source hashes, exact unified-patch additions and complete noncomment ownership,
rejects mapped-code syntax annotations and omitted executable lines, and counts
all intersecting owned LLVM code regions with end-column handling. Missing-DA
syntax receives no covered executable line credit. The defensive impossible-kind
error remains executable and uncovered. The unchanged authored
[gate-only receipt](reviews/integer-default-gate.json) approves this accounting
extension only, preserving all semantic/final-gate/publication limits. The
original thresholds, coverage inputs and unchanged third-party exclusions remain.

Five integer tests pass in debug and release on the final narrowed scope,
including all 990 scalar and 8 shape native observations. The full strict working
gate passes 109 harness tests plus the actual standalone FactoryLazy executable,
110 checks total, six snapshot regressions, source archive QA, negative coverage
fixtures, native originals and clean downstream verification. Root production
measures 1839/1842 lines and 2925/2943 regions. Owned patches measure 6/6 lines and 8/8 regions
for prost-derive and 96/97 lines and 205/206 regions for prost-reflect, totaling 1941/1945
owned lines and 3138/3157 regions. Scratch-consumer Apache license/header
metadata followed that first full run; the downstream check and then the complete
normal gate were rerun successfully on all 25 exact frozen source inputs. The
exact final normal gate exits zero with log SHA256
20dbeb2712eb9d84b66e2e2f995b29055a529f292e2d98d1aadd128557ee584d.
Final exact binding, the normal mandatory staged hook and hosted qualification
remain required. The canonical schemas,
codecs, dependency versions and all 83 reviewed original mappings are unchanged;
MSG-02 remains open.


Independent /root/transport_fallback approved the 25-file semantic packet at
frozen tree a0e166e0/diff d4307a33 with no remaining findings. Its raw-Git export
and separate reviewer target passed all 110 Rust checks, five integer tests in
each debug/release mode, fmt/Clippy, negative gates, snapshot/archive/downstream
checks and byte-identical native corpus/shape/lineage regeneration. It independently
verified source hashes, narrowed label/extension/proto3 limits, all 190 legacy
standard-pool observations, native casts/NUL/overflow values, raw file spelling,
rollback/snapshots/retry, and exact final DA/LLVM accounting. The defensive
impossible-kind error remains uncovered and included. The unchanged authored
[semantic receipt](reviews/registry-integer-defaults.json) distinguishes the earlier
gate-only approval and requires final metadata binding, normal strict staged
verification and hosted qualification. Only this additive evidence, its exact-gate
update and the unchanged receipt follow the reviewed inputs; production, tests,
fixtures, scripts, API/contract and provenance inputs remain unchanged.

## Bounded protobuf JSON bare-key and trailing-object-comma candidate

Native protobuf 3.21.12 accepts the two previously retained grammar-gap inputs
`{sec:7}` and `{"sec":7,}`. A byte-level adapter now quotes bare ASCII field
names and removes a trailing object comma outside strings before the existing
DynamicMessage deserializer. The complete-document check remains mandatory.
The original 14 fixture rows remain byte-for-byte unchanged; both former gap
rows now compare Rust/native status and full binary output. Fifteen additional
native rows cover combined syntax, duplicate fields, whitespace, escaped string
content, array and malformed boundary controls. The refreshed native proof
replays all 29 rows against the pinned peer and binds both fixture hashes,
protobuf version, schema manifest and peer source/binary hashes.

The working strict gate passes formatting, Clippy, policy/negative gate tests,
snapshot and source-package checks, clean downstream compilation and all owned
Rust/native checks. Root production coverage is 1913/1916 lines and 3047/3065
regions; the CLI source is 148/150 lines and 267/273 regions. Patched vendor
ownership remains 6/6 lines and 8/8 regions for prost-derive and 96/97 lines
and 205/206 regions for prost-reflect. The exact working gate log SHA256 is
`6608e5ac3e95d45ed4d426174c828fe953cb6358d2167a0cfae0e31232514c03`.
An initial parallel native-peer build in a separate new test hit an executable
busy race; all rows now execute in the existing single JSON test. A subsequent
working gate missed CLI region coverage by one region; native-backed escaped
string and array-boundary rows cover those lexical branches. Neither failed
run is treated as a passing qualification. The exact frozen seven-file staged
export gate also passes at tree `430ceb4e645aad1ca3cf5de9a90401b3e3e7f155`;
its log SHA256 is `812af20cfc84dc4fc2a0aeff86e506f4edbadb46126fc8d0a36c5e3abbe8dbf2`.

Independent /root/transport_fallback approved the bounded source, API and
assertions. Its private Git export and target passed all 109 harness checks,
including the complete 29-row actual native/Rust replay. Fresh native proof
bytes matched the author proof; 277 additional lexical variants had no
acceptance or binary-output mismatch. Private formatting, Clippy, negative
gates, snapshots and source-package checks passed. A separate no-normalizer
control failed on the exact `{sec:7}` original gap. The unchanged
[semantic receipt](reviews/json-grammar.json) binds the seven source/test/docs
inputs and distinguishes earlier failed attempts from qualified evidence.
Final metadata binding, normal staged hook and hosted CI remain required.
General protobuf JSON grammar, diagnostics and MSG-02 completion remain open.

## Bounded protobuf JSON trailing-array-comma candidate

The previously reviewed 29 Time framing/object-grammar rows and both fixture
files remain byte-for-byte unchanged. Native protobuf 3.21.12 accepts a trailing
comma in repeated-field arrays; the published CLI rejected it. The lexer now
removes only a comma before `]` after a value, outside quoted strings, and
leaves the existing descriptor deserializer and final-document check intact.
A 14-row source-bound corpus compares complete native/Rust acceptance and
binary output across Double_V, StringMsg_V and Pose_V. It includes combined
bare-key syntax, strings containing `,]`, nested objects, repeated commas,
empty arrays and document suffixes. The native script replays all 43 previous
and new rows and binds the unchanged prior fixture hashes, new fixture,
schema manifest, protobuf version and peer source/binary hashes.

The full working strict gate exits zero with 109 harness checks plus the
FactoryLazy executable, source/archive/downstream and negative gate checks.
Root production coverage is 1917/1920 lines and 3053/3071 regions; the CLI
source is 152/154 lines and 273/279 regions. Owned vendor patches remain
6/6 lines and 8/8 regions for prost-derive, 96/97 lines and 205/206 regions
for prost-reflect. Working gate log SHA256 is
`2130fba4a24a922801333bcdeed2a6159f6079ef55b7715ff06dcaa106e91cdf`.
The exact seven-file staged export gate also exits zero at tree
`0c25a021e6986ee00751ceb5e10c5b6622044d92`; its log SHA256 is
`57db9b9b574e556348850900c7011f3340f8d8a048ba8c6e76f419c9badf2834`.
The prior 29-row fixture hashes remain unchanged. A separate author control
removing only the array trailing-comma rule fails at the exact native-positive
`{"data":[1,2,]}` row.

Independent /root/transport_fallback approved the frozen bounded source, API
and assertions. Its private Git archive and separate target regenerated the
43-row native proof byte-for-byte, passed all 109 Rust harness checks and failed
an independent array-rule removal control on the same native-positive row. The
unchanged [semantic receipt](reviews/json-array-grammar.json) binds the seven
reviewed inputs and qualifies only repeated-field array commas. Final metadata
binding, normal staged hook and hosted qualification remain required. General
protobuf JSON grammar, diagnostics and MSG-02 completion remain open.

## Bounded protobuf JSON single-quoted-string candidate

Native protobuf 3.21.12 accepts single-quoted keys and string values rejected
by the published CLI. A bounded lexical correction converts quote delimiters,
escapes literal double quotes and preserves escaped apostrophes before the
existing dynamic deserializer and final-document check. The 43 previously
reviewed JSON rows and three fixture files remain byte-for-byte unchanged.
A new 21-row source-bound corpus compares native/Rust acceptance and complete
binary output across Time, StringMsg and StringMsg_V. Two further native-positive
rows remain explicit negative Rust dispositions: unknown `\x` escapes and raw
newlines inside strings. The native proof replays all 66 rows and binds fixture,
schema, peer source and binary hashes.

The full working strict gate exits zero with 109 harness checks plus FactoryLazy,
negative gates, snapshots/archive/downstream and source package checks. Root
production coverage is 1939/1942 lines and 3086/3104 regions; the CLI source
is 174/176 lines and 306/312 regions. Owned vendor patches remain 6/6 lines
and 8/8 regions for prost-derive, 96/97 lines and 205/206 regions for
prost-reflect. Working gate log SHA256 is
`e23fb1468719655a5e399e05470962cec0b1d8aa7a755ca3b1cd698a7d980f3e`.
An initial formatting check found two test layout differences and was corrected;
it is not positive gate evidence. A parallel native peer rebuild caused a
temporary executable-permission race in the reference script; sequential replay
then passed without a source or assertion change. The exact seven-file staged
export gate passes at tree `04d88ac5c056e58a668318953810ba5f8f6e7408`;
its log SHA256 is
`e82cca648b482a1fc8811eabe75f125cb34fc315fa0ee45e477c037ea64f1056`.
A separate author control removing only the single-quote branch fails at the
exact native-positive `{'sec':7}` row.

Independent /root/transport_fallback approved the frozen bounded source, API
and assertions. Its private Git archive and separate target regenerated the
66-row native proof byte-for-byte, passed all 109 Rust harness checks and failed
an independent single-quote-rule removal control at the same row. The unchanged
[semantic receipt](reviews/json-single-quote.json) binds the seven reviewed
inputs and preserves both explicit native-positive gaps. Final metadata binding,
normal staged hook and hosted qualification remain required; MSG-02 remains open.

## Bounded protobuf JSON raw-control-string candidate

The 66 previously accepted native rows and all four fixture files remain
byte-for-byte unchanged. A protobuf 3.21.12 source/binary-bound probe found
all 32 unescaped control bytes accepted inside each quote style of
`StringMsg.data` while the published Rust CLI rejected them. The adapter now
encodes only those in-string bytes as JSON `\u00XX` escapes before serde_json;
escape pairs, bytes outside strings, invalid UTF-8 and the final-document
check retain their prior path. The formerly pending raw-newline row now matches;
the unknown `\x` row remains a native-positive Rust-negative gap.

A new fixture has all 64 positive combinations and ten native-rejected
placement, suffix, unterminated and invalid-UTF-8 controls. The native script
replays all 140 rows and binds the unchanged old fixture hashes, new fixture,
schema manifest, protobuf version and native source/binary hashes. The focused
Rust/native comparison passes. The exact six-file staged gate exits zero with
109 Rust harness checks, five unchanged original C++ CLI assertions, formatting,
Clippy, negative coverage gates, snapshots, source package and downstream checks;
strict root/CLI and owned vendor coverage thresholds remain intact. Its log
SHA256 is `7f4aa71b3da8d39ad2ccc387af029fdfe06c25989c018d105c6d6b6ed09ba072`.
A private branch-removal control fails the native-positive double-quoted NUL
row `7b2264617461223a22610062227d`.

Independent /root/transport_fallback approved the frozen source/API/assertions.
Its private raw export and exclusive target regenerated the 140-row native proof
byte-for-byte, passed all 109 Rust checks and failed its own NUL branch-removal
control. Six direct escaped-raw probes remain native-positive/Rust-negative,
outside this unescaped-byte scope. The unchanged [semantic receipt](reviews/json-raw-controls.json)
binds the six source/test/docs inputs and four retained fixture hashes. Final
metadata binding, normal hook and exact hosted CI passed on public commit
`dcdbb741630d38e21ba5271a50cae04be1aee6c4`. General protobuf JSON grammar
and MSG-02 completion remain open.

## Bounded protobuf JSON escaped-byte candidate

A pinned protobuf 3.21.12 source/binary-bound probe enumerates every ASCII
byte following a backslash inside each quote style of `StringMsg.data`. Native
accepts 254/256 combinations; only incomplete `\u` under each quote style is
rejected. The bounded lexical adapter consumes unknown escape slashes, maps
`\v` to vertical tab, rewrites escaped raw controls to equivalent JSON Unicode
escapes, and preserves standard JSON escapes. The existing deserializer and
complete-document check still decide field/value validity. The 140 prior native
rows and all five old fixtures remain byte-for-byte unchanged.

The new 256-row fixture asserts every input shape, native acceptance and full
binary stdout. `scripts/json-reference.py` replays all 396 native rows and
binds the protobuf version, schema, fixture and native peer hashes. The focused
Rust/native comparison passes all 256 new rows and all 140 retained rows.
Incomplete Unicode, invalid UTF-8, non-ASCII escape bytes and broader grammar
remain outside this slice. The exact six-file staged gate exits zero with 109
Rust harness checks, five original C++ CLI tests, formatting, Clippy, strict
coverage and negative gates; its log SHA256 is
`bbccc88a24d7dcaa923d0c7f7c796c5721e7665be9f843f878e75993ad08d139`.
Independent /root/transport_fallback approved the frozen source/API/assertions
after a private 396-row replay and a separate unknown-escape mutant failure.
The unchanged [semantic receipt](reviews/json-escaped-bytes.json) binds the six
reviewed inputs and five retained fixture hashes. Final metadata binding,
normal hook and exact hosted CI passed on public commit
`5fa067f47273f2ac46c8af7ba27bef66b6a3251e`.

## Bounded protobuf JSON escaped-UTF-8 candidate

A source/binary-bound protobuf 3.21.12 probe checks 12 valid and ten malformed
UTF-8 sequences after a backslash within each quote style of
`StringMsg.data`. It includes scalar width boundaries, a BOM, common
multibyte characters, isolated continuation/lead bytes, overlong encodings,
surrogate encoding, out-of-range scalars and truncated sequences. Native
accepts the 24 complete valid inputs, preserving full decoded bytes, and
rejects all 20 malformed inputs. The published Rust CLI rejected all 24 valid
rows. The selected Rust/native comparison first failed at escaped U+0080; a
one-line lexical change consumes the slash before a non-ASCII byte and lets
the unchanged JSON deserializer enforce UTF-8 validity. The selected test now
passes all 440 rows.

The 396 accepted earlier native rows and all six old fixtures remain
byte-for-byte unchanged. The new 44-row fixture asserts input structure,
native acceptance and full binary stdout; `scripts/json-reference.py` replays
all 440 source-bound native outcomes. The exact six-file staged gate exits zero
with 109 Rust harness checks plus FactoryLazy, five original C++ CLI tests,
formatting, Clippy, strict coverage and negative gates; its log SHA256 is
`53f782fc85dcac71ddfda8f3979b784cf423dc0cdda3641bdf2dc23e4b6cc93e`.
Independent /root/transport_fallback approved the frozen source/API/assertions
after a private 440-row replay and separate U+0080 mutant failure. The
unchanged [semantic receipt](reviews/json-escaped-utf8.json) binds the six
reviewed inputs and six retained fixture hashes. Final metadata binding,
normal hook and exact hosted CI passed on public commit
`f46db31af457a2a2c815c823e46389d2cc7aaf9b`. Broader protobuf
JSON/Unicode grammar, diagnostics and MSG-02 acceptance remain open.

## Bounded protobuf JSON trailing-dot-number candidate

A pinned protobuf 3.21.12 probe found bare `0.`, `-0.`, `7.`, `-7.` and
`7.e2` native-positive in Time.sec and Double_V.data, while the published
Rust CLI rejected them. A 63-row source/binary-bound corpus covers 18 scalar
forms for each type plus two-field, repeated-comma, trailing-comma, comment
and whitespace-before-dot controls. It compares native acceptance and
complete binary stdout.
The selected test first failed at native-positive `{"sec":0.}`. The lexical
adapter now drops only a dot immediately after a digit and before a numeric
terminator, JSON whitespace or exponent marker, provided the preceding
numeric run contains no earlier dot or exponent. The maintained dynamic
decoder and full-document check remain in place. The selected native/Rust
comparison passes the matching rows after that correction.

An independent pre-approval audit rejected the first frozen candidate:
it accepted native-rejected second-dot forms after a fraction or exponent,
including `1.2.e3`, `7e2.` and `1e+2.`. Fourteen new negative rows enforce
that boundary. Five native-positive `07.`, `0x7.` and `0x7.e3` rows under Time
and Double_V remain explicit Rust-negative gaps, rather than silently
broadening the accepted grammar. The revised native script replays all 503 rows.

The prior 440 native rows and seven fixture files remain byte-for-byte
unchanged. Quoted numeric strings, ordinary fractions, malformed exponents
and comments retain their old path. At this slice, bare `-0` Double_V sign
and quoted hexadecimal numeric strings remained explicit separate gaps. The
corrected six-file exact staged gate exits zero with 109 Rust harness checks
plus FactoryLazy, five original C++ CLI tests, native 503-row replay,
formatting, Clippy, strict coverage and negative gates; its log SHA256 is
`7e90468b8a047ec35cc3b261a5c3994b38060c94cd031338dd1355e9b3c4762b`.
Independent /root/transport_fallback approved the corrected source and
assertions after a private 503-row replay, full Rust suite, additional numeric
probes and a second-dot guard mutant. The unchanged
[semantic receipt](reviews/json-trailing-dot.json) binds the six reviewed
inputs and seven retained fixture hashes. The normal hook and exact hosted CI
passed on public commit `c2e15df04e8983d07879fdef0344e207996d643d`.
Broad numeric grammar and MSG-02 acceptance remain open.

## Bounded protobuf JSON negative-zero candidate

Pinned protobuf 3.21.12 converts a bare integral `-0` into positive floating
zero; published Rust emitted negative-zero wire bits for Double_V and Float_V
and an extra negative-zero Vector3d.x field. The lexical adapter now removes
the sign only for exact unquoted `-0` at an array/object value boundary before
a JSON delimiter or standard JSON whitespace. Fractional `-0.`, exponent
`-0e0` and quoted `"-0"` remain negative as native requires. Adjacent `1-0`
and `-0-0` stay invalid; integer Time.sec and StringMsg controls keep their
old output.

The new source/binary-bound fixture asserts 28 scalar rows, 16 combined or
malformed rows and two native-positive quoted-hex rows left Rust-negative.
It checks actual acceptance and complete binary stdout; the prior 503 rows
and eight fixtures remain byte-for-byte unchanged. The native script replays
all 549 rows. The focused Rust/native comparison first failed at bare
`{"data":[-0]}` with opposite sign bits; the retained before-failure log
SHA256 is `ffe8f7b44de324ae0dd76c3508985a42dad41a178c202bd30f4fb45cb7b4ac9b`.
The selected comparison passes after the correction. The strict staged gate
passes 109 Rust harness checks plus FactoryLazy, five original C++ CLI tests,
all 549 native rows, formatting, Clippy, coverage and negative gates. The
separate raw-source rule-removal mutant fails the exact bare `-0` wire row.
Independent /root/transport_fallback approved the exact source and assertions
after a private 549-row replay, full Rust suite, rule-removal mutant and 234
targeted boundary cases. No Rust overacceptance or matched-positive wire
mismatch appeared. Twelve native-positive vertical-tab/form-feed cases remain
Rust-negative outside this standard-whitespace slice. The byte-identical
[semantic receipt](reviews/json-negative-zero.json) binds all six reviewed
files and eight unchanged prior fixtures. The normal hook and hosted CI passed
on public commit `11ea55557d9a6a111a440f05b9ad3b65c9a8427a`; broader
numeric grammar and MSG-02 acceptance remain open.

## Bounded protobuf JSON vertical-tab/form-feed candidate

A pinned protobuf 3.21.12 source/binary probe covers 42 literal VT/FF rows:
32 native-positive cases rejected by published Rust, two already matching
raw-string controls, and eight matching native rejections. The focused test
with the new fixture failed against the preceding parser at leading VT. The
adapter now normalizes only unquoted VT/FF to ASCII spaces, preserving quoted
control bytes. Native token lookahead and the existing bare `-0` and
trailing-dot guards recognize both bytes before the rewrite. Complete-document
validation remains with the maintained decoder.

The new fixture binds the unchanged native probe source/binary and asserts
all 42 exact inputs, native statuses and complete binary stdout. The nine
prior fixtures and 549 rows remain byte-for-byte unchanged. The native script
replays 591 rows. Invalid split digits, split sign, suffix data and comments
remain rejected. The working strict gate exits zero with 109 Rust harness
checks plus FactoryLazy, five original C++ CLI tests, all 591 comparisons,
formatting, Clippy, strict negative and downstream gates. Root production
coverage is 2004/2007 lines and 3182/3200 regions; CLI coverage is 239/241
lines and 402/408 regions. The working gate log SHA256 is
`db5b451ba636c58adfbc7c040821078b5ee39f64975c56d868230d81e27bcb60`.
The exact six-file staged gate also exits zero on tree `b178e1ff67e8aaf92a04321adb7649f6a441f047`;
its log SHA256 is
`c687ef3b93978a05040c6f221aa3366df52a0aa1b1706c2c032d420adc79e2c5`.
Independent /root/transport_fallback approved the exact source and assertions
after a separate raw-source 591-row replay, full 109-check Rust suite,
normalization-removal mutant and 138 insertion placements with no status or
wire mismatch. The unchanged [semantic receipt](reviews/json-vtff.json) binds
all six reviewed files and nine retained fixture hashes. Final metadata
binding, the normal hook and hosted run `40213285366` passed on public commit
`60e67ade72142d71c812b180bb7628c3bddcd4a1`; broader JSON grammar,
diagnostics and MSG-02 remain open.

## Bounded protobuf JSON quoted-hex candidate

The preceding published parser rejected quoted hexadecimal floats accepted by
pinned protobuf 3.21.12. A source/binary-bound fixture now covers 216
Double_V, Float_V, Time and StringMsg forms: 78 initial characterization rows
and 138 boundary rows. It asserts exact native status and full binary output,
including f32/f64 rounding, subnormals, overflow, signed zero and non-float
field isolation. The selected Rust test first failed on `"0x7"`, then passes
after the descriptor-scoped conversion. All 591 retained rows and ten prior
fixture hashes remain intact. Two native-positive duplicate-key rows stay
explicit Rust-negative gaps to avoid a wrong accepted wire value.

The pinned `fhex 2.0.2` dependency performs bit-level conversion behind a
strict native spelling and finite-result guard. Other evaluated parsers had
concrete precision, grammar or panic failures recorded in
`docs/JSON_FRAMING.md`. `scripts/json-reference.py` freshly replays all 809
native outcomes. A separate deterministic 418-case f32/f64 differential has
zero acceptance or wire mismatches. The working full strict gate exits zero:
110 Rust integration checks, native C++ controls, formatting, Clippy, negative
and downstream gates; root production coverage 2122/2128 lines and
3378/3403 regions, CLI 357/362 lines and 598/611 regions. Its retained log
SHA256 is `790b8f4b474f790fcc6edd788561d5c8701d9ba0710d502c20aa8e762658a5a1`.
The exact eight-file staged gate also exits zero on tree
`278a47c644417657ca9dcf7d7de4c6865ed3a626`; log SHA256
`ccf5d7d84ac38fb9086f128c4dcd3868de8647d0cfc1f377fa53b9915c30f054`.
Independent /root/transport_fallback approved the source, assertion mapping
and duplicate-key disposition after a private 809-row native replay, 110 Rust
tests, 2,656 supplemental numeric/quote variants with no mismatch and a
descriptor-disable mutant. The copied
[semantic receipt](reviews/json-quoted-hex.json) binds all eight source files,
the ten retained fixture hashes and the independent proof. Final metadata
binding and the normal hook passed on public commit
`1c3c0b802cd17fd8c1f49fac4370db3103ba9028`; hosted run `40216034480`
passed. This candidate does not complete MSG-02.

## Bounded repeated float `data` keys

The previously retained two native-positive duplicate-key rows now compare
the complete Rust and native wire output. The adapter remains limited to
`Double_V.data` and `Float_V.data`; it uses the maintained reflected decoder
for every occurrence and for the full document's other fields, then appends
the repeated values in source order. Native scalar and `null` occurrences,
quoted hex bits, signed zero, and canonical nonfinite strings are covered.
The native/Rust cross-product initially found an overacceptance of
lowercase `nan`; a post-freeze malformed-input audit found an empty value
accepted as an empty array. The corrected candidate rejects both cases.

The new 66-row native fixture contains 38 accepted and 28 rejected cases. Its
manifest binds the unchanged pinned C++ probe and all 11 earlier JSON fixture
hashes; the native replay now covers 875 rows, retaining all 809 prior ones.
An additional 722-case value cross-product found zero acceptance or wire
mismatches. The strict working gate passes formatting, Clippy, native C++
controls, downstream/source-package/negative checks and 111 Rust integration
tests. Measured handwritten production coverage is 2217/2232 lines and
3562/3608 regions; the CLI has 452/466 lines and 782/816 regions. The exact
six-file staged gate passed on tree `000a6531db853b6f092c3eb10d43a0a5533472e1`.
The [independent review](reviews/json-repeated-data.json) approved this
bounded source after a private 875-row native replay, all 111 Rust tests,
1,170 additional variants with no mismatch and an append-removal mutant that
fails the original two-decimal wire assertion. The superseded first candidate
is excluded from the approval. Final metadata binding and the normal hook
passed on public commit `0d267506915119344467a56956ddb4408777158c`;
hosted run `40218476323` passed.
Other descriptor types and broader duplicate-field/JSON behavior remain
MSG-02 work.

## Bounded repeated integer `data` keys

The same reflected occurrence decoder now appends duplicate top-level `data`
values for `Int32_V`, `UInt32_V` and `Int64_V`. This adds no integer parser:
the maintained decoder still validates each value and the complete document.
The pinned C++ probe recorded 117 rows, with 82 accepted and 35 rejected,
covering arrays, scalars, nulls, three keys, header preservation, native
permissive syntax, type bounds and malformed input. All 875 prior rows and
12 fixture hashes remain unchanged; the native replay checks 992 outcomes.
A separate 495-case boundary differential across these three types found zero
acceptance or wire mismatch. The strict working gate passed 112 Rust tests,
the retained native C++ controls, formatting, Clippy, downstream checks and
>95% coverage. Measured production coverage is 2228/2242 lines and
3575/3619 regions; the CLI has 463/476 lines and 795/827 regions. The log
SHA256 is `53ee727497bd26d8012a9fc6284cbd56419c851f080536ff517ec6abadaaaa03`.
An isolated source/target mutant that removes `Int32_V` admission fails the
first native wire assertion, `[1]` plus `[2]`, as expected. The
[independent review](reviews/json-repeated-integer.json) approved the six
frozen source files after a private raw export and 992-row native replay,
112 Rust tests, 270 further duplicate pairs without mismatch, and a separate
Int32 admission mutant. The exact six-file staged gate passed on tree
`0b3832bd5eb01f40c036324b1d20125d0131cfba` with log SHA256
`638054bf8b1fc4fad6a310e9a337abccd1666b62b3d1c6f54fd3a0c7e8ad006b`.
`UInt64_V` remains pending: the maintained
decoder saturates an unquoted integer above `u64::MAX`, while native rejects
it. Single-key scalar values, other repeated types, broader JSON grammar and
MSG-02 acceptance remain open. Final metadata binding and the normal hook
passed on public commit `8ab936c491305e7c4517225d9e58ef162081ef9e`;
hosted run `40220925335` passed.

## Bounded repeated string `data` keys

The occurrence decoder now appends duplicate top-level `StringMsg_V.data`
values in source order using the maintained reflected string decoder. The
source-bound C++ fixture covers arrays, scalar strings, nulls, header retention,
permissive keys and escapes, Unicode/control bytes and malformed input. Its
46 rows contain 34 native-positive and 12 native-negative outcomes. All 992
prior JSON rows and 13 fixture hashes remain unchanged; the native replay now
checks 1,038 outcomes. A reproducible 400-pair sweep across Unicode, escape,
null, malformed and scalar/array forms found no status or wire mismatch. The
working strict gate passed 113 Rust tests and all five original C++ CLI tests;
root production coverage was 2,228/2,242 lines and 3,575/3,619 regions,
and CLI coverage was 467/479 lines and 798/828 regions. In an isolated
source/target mutant, removing StringMsg_V admission failed the exact
`two_arrays` wire assertion (one element instead of two).
The [independent review](reviews/json-repeated-string.json) approved the six
frozen files after a private 1,038-row native replay, selected Rust assertion
and separate descriptor-removal mutant. The exact six-file staged gate passed
on tree `88519e778897ad719cdf4a4605bb2b47c14cf1b9` with log SHA256
`7a17b561a664a21f4980cc51ab697ce0cadaecde4ca3c10ba4a5da67b50d85f6`.
Single-key scalar values, other repeated descriptors, general duplicate-field
behavior and MSG-02 acceptance remained open at this review point. Final
metadata binding and the normal hook passed on public commit
`115b467f612c41b9c0b4706b5032a4280a731e41`; exact hosted run
`40225573548` passed.

## Bounded single-key scalar repeated `data`

The existing occurrence decoder also handles one scalar `data` value for the
six already admitted repeated descriptors: `Double_V`, `Float_V`, `Int32_V`,
`UInt32_V`, `Int64_V` and `StringMsg_V`. A one-key array or `null` stays on
the maintained decoder's prior path. The occurrence path wraps a scalar in an
array, lets the maintained reflected decoder validate that element and the
complete document, then restores the repeated list. Quoted hexadecimal float
values reuse the pinned `fhex` adapter; it adds no general numeric parser.
The pinned C++ probe recorded 95 parity rows, 64 accepted and 31 rejected,
including integer bounds, nonfinite float spellings, quoted hex, signed zero,
literal and escaped strings, permissive keys, `header`, arrays, `null` and
unrelated-field errors. All 1,038 prior rows and 14 fixture hashes are retained
unchanged; the native replay checks 1,135 outcomes. Two native-positive
unquoted hex scalar rows (`0x7.` for each float type) remain explicit
Rust-negative gaps. Native rejects unquoted `p`-exponent hex forms that the
quoted float adapter accepts, so this slice does not extend that adapter to
unquoted hex. Other repeated descriptors, broader JSON grammar and MSG-02
acceptance remain open. The full working gate passed 114 Rust tests, five
original C++ CLI tests, formatting, Clippy, downstream and strict coverage
checks. Root production coverage was 2239/2249 lines and 3599/3632 regions;
the CLI had 474/483 lines and 819/840 regions. A private raw-parent source
with this exact new test and fixture fails the first native-positive scalar
`{"data":7}` control, confirming the assertion detects the added path.
The [independent review](reviews/json-single-scalar.json) approved the exact
six-file source freeze and private replay of all 1,135 native outcomes. Its
348-case supplemental differential found no status or accepted-wire mismatch.
The exact staged gate passed on tree `6295fc2f7468ed452dbaf83898bcb996a649b0c5`.
Final metadata binding and the normal protected hook passed on public commit
`6dc8845eb8584c16832aebe19f635cc0e7d34275`; exact hosted run
`40227905407` passed.

## Bounded UInt64 array numeric precision and range

Native protobuf rejects numeric array elements above `u64::MAX` and decimal
values whose double conversion reaches 2^64. The maintained reflected decoder
can saturate those values; its JSON number conversion can also round large
in-range decimals differently or round a fractional exponent to an integer.
For `UInt64_V` top-level `data` arrays only, pinned `serde_json` 1.0.149
`raw_value` retains each numeric token. Rust's maintained float parser checks
decimal/exponent integrality and the native range; exact in-range integer tokens
use `u64` parsing. The reflected decoder still validates the complete document,
then receives the exact numeric array values. This change does not admit scalar
or repeated `data` keys.

The new source-bound native fixture has 107 parity rows and 33 explicit
unqualified scalar, nested-null or duplicate-key rows. All 1,135 prior native
outcomes and fifteen fixture hashes stay unchanged, so the replay covers 1,275
native outcomes. A separate 674-form numeric boundary differential had zero
status or accepted-wire mismatches after the change; its raw rows include
overflow, underflow, exponent, precision and signed-zero controls. This is a
bounded prerequisite for future UInt64 admission. General duplicate-key JSON,
other descriptors and MSG-02 completion remain open. The full working gate
passed 115 Rust checks, all five original C++ CLI tests, formatting, Clippy,
downstream and strict coverage checks. Root coverage was 2279/2290 lines and
3673/3709 regions; the CLI had 514/524 lines and 893/917 regions. A private
raw-parent parser with the new test and fixture fails on the native-rejected
`{"data":[18446744073709551616]}` control. The [independent review](reviews/json-uint64-range.json)
approved the exact seven-file source freeze after a private 1,275-row native
replay and 674-form differential; it also failed the exact overflow control
against the public-parent parser. The exact staged gate passed on tree
`932109277ea195f23f36284809bebdc07f59e06f`. Final metadata binding and
the normal protected hook passed on public commit
`cf0c44de2a40fe52310c03b7261abaea13103212`; exact hosted run
`40229507091` passed.

## Bounded UInt64 scalar, nested, null and repeated `data`

Only `gz.msgs.UInt64_V` top-level `data` values take the new occurrence path.
It uses pinned `serde_json` raw values to flatten nested arrays, skip `null`
elements and append repeated field occurrences in source order. Numeric leaves
reuse the independently reviewed exact integer and native-style decimal/range
conversion; reflected decoding still checks each occurrence and the complete
document. Other descriptors do not enter this branch.

A new source-bound fixture records 185 parity cases (119 accepted, 66
rejected) for scalar, nested, null, duplicate, exponent, overflow and
unrelated-field forms. Eight additional native-positive unquoted hexadecimal
trailing-dot forms, including `{"data":0x7.}`, remain Rust-negative and are
asserted separately across scalar, array, nested and duplicate-key placements.
The native replay reproduces 1,468 outcomes, retaining all 1,275 prior rows
and 16 unchanged prior fixture hashes. The old UInt64 range fixture's 33
native-positive/Rust-mismatch observations remain verbatim: its test first
checks that old mismatch, then requires current Rust to match the stored native
status and wire result. Separate 171-case and 568-case characterizations had
zero status or accepted-wire mismatches within their declared forms; the exact
public-parent parser fails new case `{"data":"1"}`. This is bounded UInt64
repeated-field parity, not general duplicate-key JSON support or completion of
MSG-02. The native-only hexadecimal gap remains explicit. The full
working strict gate passed 116 Rust tests, five unchanged native C++ CLI tests,
formatting, Clippy, downstream and coverage checks. Root coverage was
2310/2322 lines and 3740/3779 regions; CLI coverage was 545/556 lines and
960/987 regions. The [independent review](reviews/json-uint64-occurrences.json)
approved the V2 six-file freeze after a separate 1,468-row native replay and
eight explicit hex-gap checks. A private removal of UInt64 occurrence admission
failed the exact first scalar assertion. The initial source review was
superseded when the hex gap was found; the V2 receipt binds this disposition.
The exact staged gate passed on tree
`b5bef5195548044f74a3d50265a28aac79f4cddc`. Final metadata binding and
the normal protected hook passed on public commit
`755c185e3f9cb4c341d07fc3c6a9092608c8ba7c`; hosted run `40232275992`
passed.

## Bounded UInt64 unquoted hexadecimal controls

The pinned C++ JSON peer now checks 220 bounded `UInt64_V.data` trailing-dot
hexadecimal cases and two native-only undotted `e`/`E` cases in scalar, array,
duplicate and nested placements, for 1,690 native status and full-wire
outcomes total. All 17 older fixture hashes are
held exact, and the previous eight native-only hex observations remain in their
original fixture as historical negatives before the current parity check.
The corrected adapter reuses pinned `fhex 2.0.2` under descriptor,
token-spelling and value-start guards. Pinned protobuf 3.21.12
`JsonStreamParser::ParseNumberHelper` scans `e`/`E` but stops at other
alphabetic hex digits, then `DataPiece::ToUint64` checks the floating
conversion. The first source freeze overaccepted `10x7.`; the second missed
native-positive `0xe.` and `0x7e1.`. Both are
excluded. Undotted `0xe` and `0x7e1` remain native-only gaps. Other
descriptors and general JSON parsing are unchanged. The exact six-file staged
gate passed with 2356/2368 root lines and 3816/3857 regions. The
[independent review](reviews/json-uint64-unquoted-hex.json) approved the
bounded mapping after a separate 1,690-row native replay and explicit
undotted-gap checks. Final metadata binding, the normal protected hook and
hosted CI remain pending.

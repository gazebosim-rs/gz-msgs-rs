<!-- Copyright (C) 2026 Open Source Robotics Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Representative codec baseline

Two preliminary timing runs lacked effective Rust compiler/configuration evidence.
They remain explicitly [unqualified](codec-benchmark-unqualified.json); no override
is alleged or established. The final matched run below followed independent
approval of the corrected compiler-fidelity controls. Its final measurement review and normal staged/hosted qualification passed at
`45f29b7b333e9187f16e5a542241eacf80cb6ff7`.

This is a bounded single-threaded message-codec baseline. It preserves slower
results and establishes an optimization target; it does not accept MSG-02,
the balanced robotics suite or a simulator performance budget.

Both executables independently construct Clock, IMU, Pose_V and LaserScan values.
The driver requires identical complete wire bytes before timing. Fixtures include
headers, names, timestamps, all three IMU covariance arrays, signed zeros, raw quaternion
components and repeated double ranges/intensities. Pose counts are 1/100/1,000;
scan counts are 360/1,080. They are message values, not simulated sensor outputs.
Every fresh/reuse path checks complete decoded values/wire outside the timer,
including two consecutive reuse cycles and the final reused state. Timed checksums
and iteration/byte counts must also agree. No field, precision or payload is dropped.

## Equal work

Fresh encode creates and drops a new owned output buffer per iteration. Reused
encode clears an existing buffer, initially reserved to the exact payload size.
Fresh decode constructs and drops a typed message from the same borrowed input.
Reused decode clears/parses the existing object using each runtime's public API:
Rust Message::clear + merge, C++ ParseFromString. Each implementation retains the
capacities its generator supports; no manual field-reuse optimization is added.
Roundtrip combines the corresponding decode and encode lifetimes. Both escape
the complete message/output through compiler barriers, then read the same small
projection. Payload generation, descriptor initialization, full fidelity checks,
64 in-process warmup operations and final validation are outside the timer.

Three Time envelope cases stay separate: ordinary input, an unknown field, and
a known tag with the wrong wire type. Each iteration owns the original bytes,
parses a fresh typed Time and reads its known values. Both forward the exact
original wire. Native generated values additionally retain parsed unknown-field
metadata; Rust's typed value omits it while its envelope preserves the wire.
No equivalence of mutable unknown-field APIs or internal allocation is claimed.

## Build and measurement

C++ types are generated from the exact canonical schema closure with protoc
3.21.12, then compiled using C++17, -O3 and -DNDEBUG against the identified system
libprotobuf 3.21.12. This avoids assuming the flags of an installed gz-msgs binary.
The packaged libprotobuf's original optimization flags are not reconstructed.
Rust uses the separate release consumer with exactly the root lock's runtime
versions and the same reviewed vendor patches. The driver pins the actual Cargo/rustc binaries from the repository's
named toolchain, clears wrapper aliases and encoded Rust flags, sets explicit
release profiles and uses a fresh temporary target. It captures actual verbose
compiler arguments and artifact profiles, refusing another compiler/wrapper,
unoptimized measured runtime units or instrumentation. Host build scripts and
proc macros are recorded separately. Only the measured executable is retained;
regenerable target products are removed. Actual C++ compiler identity and an
optimized-release preprocessor guard supplement its explicit command flags.
Binaries, dynamic libraries, generated C++ sources and owned Rust inputs retain
identities. All 77 fresh compiler invocations/profiles are retained, including 21 measured
runtime units; independent build-fidelity review passed before final timing.
It changes no runtime source, dependency, schema or decoder policy.
[Cargo configuration precedence](https://doc.rust-lang.org/cargo/reference/config.html#command-line-overrides)
is handled with explicit settings plus observed compiler arguments, rather than
inferring effective optimization from the nominal --release command.

Each case calibrates a shared iteration count for an approximately 50 ms sample.
Ten rounds visit all cases, with independently shuffled case and C++/Rust pair
order using seed 73119. Child processes run on the same allowed CPU. Reports retain
every elapsed value, checksum, iteration count, order and host load observation.
The ratio is C++ nanoseconds divided by Rust nanoseconds for equal counts:
above one favors Rust; below one favors C++. The paired median and 10,000-resample
percentile interval describe this run. They are not a noisy CI latency gate or
proof against frequency drift, temporal correlation or other hardware effects.

The recorded run is 2026-10-29T17:12:14Z through 2026-10-29T17:13:22Z on AMD Ryzen 9 5950X 16-Core Processor, CPU 0, governor powersave. One-minute host load ranged 1.79–3.22; the shortest actual sample was 29.16 ms. The host was not reserved or frequency/temperature controlled.

## Results

| Fixture | Encode fresh | Encode reuse | Decode fresh | Decode reuse | Roundtrip fresh | Roundtrip reuse |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| clock | 0.969 | 0.928 | 1.750 | 1.821 | 1.415 | 1.532 |
| imu | 0.880 | 0.862 | 1.248 | 1.317 | 1.124 | 1.170 |
| poses_1 | 0.940 | 0.940 | 1.532 | 1.514 | 1.330 | 1.337 |
| poses_100 | 0.762 | 0.757 | 1.082 | 1.271 | 0.990 | 1.098 |
| poses_1000 | 0.761 | 0.766 | 0.829 | 1.320 | 0.755 | 1.146 |
| scan_360 | 0.390 | 1.244 | 0.495 | 0.577 | 0.471 | 0.498 |
| scan_1080 | 0.262 | 0.257 | 0.282 | 0.300 | 0.291 | 0.295 |

| Time envelope | C++ / Rust median | Resampling interval |
| --- | ---: | --- |
| time_plain | 0.831 | 0.821–0.836 |
| time_unknown | 2.038 | 2.000–2.043 |
| time_wrong | 0.424 | 0.421–0.430 |

The larger packed LaserScan cases regress substantially in this baseline.
Those results must remain visible. Verify release flags and packed representation,
then profile actual encoding/decoding hot paths before proposing an optimization
or dependency change. Do not change precision, buffering, lifetime or workload to
make the comparison favorable. Allocation/RSS, end-to-end transport, sensor
accuracy, parallelism, other hardware and the release matrix remain open.

The final [raw report](codec-benchmark.json) contains all 45 cases, full fixtures,
per-sample data, uncertainty and provenance. The older exploratory Time-only
artifact remains historical and is not substituted for this representative run.

## Reproduce and verify

```sh
python3 tools/test_codec_benchmark.py
python3 tools/codec_benchmark.py --verify-only
python3 tools/codec_benchmark.py --samples 10 --sample-ms 50
```

An explicit --output directory must not contain a prior results.json; raw
measurements are never overwritten. Build products are regenerable. The normal
staged gate runs the failure/sampling/compiler controls, consumer formatting/Clippy
and all 45 native/Rust fidelity/work controls without timing. The standalone
consumer is measurement/test tooling, outside the shipped production crate's
coverage denominator. All existing production coverage, native, assertion and
packaging gates remain unchanged. No performance threshold is added.

Independent compiler-fidelity review passed before final timing. Final review
replayed all 45 fidelity controls, recomputed every summary/interval/table cell
and checked all 450 paired samples against the independently rebuilt executable.
The single normal Main gate and exact hosted qualification remain required.

## Public append-helper preparation

The current verifier has a separate `verify` action in both executables. It
performs no warmup and never enters the harness clock path; every verification
row must report zero elapsed time. Counts 1 and 3 still execute the complete
operation and compare work, bytes and checksums. `--verify-only --append` also
builds an opt-in public-helper consumer, checks all three runtimes against the
ten complete historical wires, and retains all 270 verification rows. The normal
staged gate uses this path and checks both consumer feature configurations.

`tools/append_benchmark.py --archive ARCHIVE --revision FULL_PUBLIC_SHA
--output EMPTY_DIRECTORY` prepares independent native, default prost and public
append executables from a source distribution. The trusted full revision is
mandatory. The maintained source-package checker verifies every archive blob,
mode, path and the complete Git commit/tree before compilation. A separate
consumer links that extracted library through its public API; its source, lock
and manifest are identical for the two Rust builds. The `append` feature is
their only build selection difference. Generated native types use the same
extracted schema closure. Compiler arguments/profiles, raw Rust build logs,
consumer inputs, generated sources, executable/library identities and source
manifest identity are retained in the output. No private precursor helper code
is used as runtime provenance.
The hosted workflow repeats this preparation against its own immutable source
archive and uploads the report, raw build logs, consumer/generated inputs and
three executables alongside the existing source and coverage evidence.

Fresh default encoding calls `Message::encode_to_vec`; fresh append encoding
creates an empty vector and calls the shipped `gz_msgs::encode_append`, including
fallible reservation, zero-fill, both length traversals and all returned-error
checks. Reused encoding clears the output vector inside the operation before
calling its selected encoder. Native fresh encoding uses `SerializeAsString`;
`SerializeToString` resets the retained output inside each reused operation.
Decode and roundtrip keep the original public clear/parse lifetimes. Buffer
capacity retention and allocation counts may differ between runtimes; this is
equal caller work, not a claim of identical allocation internals.

Public controls additionally check Any, prefix/append boundaries, reflective
unknown fields and exact signed-zero/NaN bits. The source-bound public-helper
output is preparation only: this command contains no measurement option. A later
quiet comparison must bind its raw balanced samples to these exact executables
and inputs and receive independent review. The historical default-codec result
and the distinct precursor experiment remain unchanged; shipped-helper timing,
default adoption and MSG-02 completion remain unqualified.

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
staged gate runs the eight failure/sampling/compiler controls, consumer formatting/Clippy
and all 45 native/Rust fidelity/work controls without timing. The standalone
consumer is measurement/test tooling, outside the shipped production crate's
coverage denominator. All existing production coverage, native, assertion and
packaging gates remain unchanged. No performance threshold is added.

Independent compiler-fidelity review passed before final timing. Final review
replayed all 45 fidelity controls, recomputed every summary/interval/table cell
and checked all 450 paired samples against the independently rebuilt executable.
The single normal Main gate and exact hosted qualification remain required.

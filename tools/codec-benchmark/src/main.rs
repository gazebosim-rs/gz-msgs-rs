// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use gz_msgs::{
    msgs::{Clock, FloatV, Header, Imu, LaserScan, Pose, PoseV, Time},
    prost::Message,
    quaternion, vector3, WireMessage,
};
use std::{hint::black_box, io::Write, time::Instant};

fn header() -> Header {
    Header {
        stamp: Some(Time {
            sec: 123,
            nsec: 456,
        }),
        data: vec![gz_msgs::msgs::header::Map {
            key: "frame_id".into(),
            value: vec!["robot/base".into()],
        }],
    }
}

fn pose(i: usize) -> Pose {
    Pose {
        header: Some(header()),
        name: format!("robot/{i}"),
        id: (i + 1) as u32,
        position: Some(vector3([(i + 1) as f64 * 0.125, -0.25, -0.0])),
        orientation: Some(quaternion([-0.0, 0.0, 0.5, 1.0])),
    }
}

trait Sample: Message + Default + PartialEq + std::fmt::Debug {
    fn sample(&self) -> u64;
}
impl Sample for Clock {
    fn sample(&self) -> u64 {
        self.sim.as_ref().unwrap().nsec as u64
    }
}
impl Sample for Imu {
    fn sample(&self) -> u64 {
        self.angular_velocity.as_ref().unwrap().z.to_bits()
    }
}
impl Sample for PoseV {
    fn sample(&self) -> u64 {
        self.pose.len() as u64 + self.pose.last().unwrap().id as u64
    }
}
impl Sample for LaserScan {
    fn sample(&self) -> u64 {
        self.ranges.len() as u64 + self.count as u64
    }
}

fn encoded_sample(bytes: &[u8]) -> u64 {
    bytes.len() as u64 + u64::from(bytes[0]) + u64::from(*bytes.last().unwrap())
}

fn encode<T: Message>(value: &T, output: &mut Vec<u8>) {
    #[cfg(feature = "append")]
    gz_msgs::encode_append(value, output).unwrap();
    #[cfg(not(feature = "append"))]
    value.encode(output).unwrap();
}

fn encode_fresh<T: Message>(value: &T) -> Vec<u8> {
    #[cfg(feature = "append")]
    {
        let mut output = Vec::new();
        gz_msgs::encode_append(value, &mut output).unwrap();
        output
    }
    #[cfg(not(feature = "append"))]
    value.encode_to_vec()
}

fn public_controls() {
    let time = Time { sec: -1, nsec: 42 };
    let any = gz_msgs::prost_types::Any::from_msg(&time).unwrap();
    let mut output = vec![0xaa, 0xbb];
    for _ in 0..2 {
        let start = output.len();
        gz_msgs::encode_append(&any, &mut output).unwrap();
        assert_eq!(
            gz_msgs::prost_types::Any::decode(&output[start..]).unwrap(),
            any
        );
    }
    assert_eq!(&output[..2], &[0xaa, 0xbb]);
    let mut wire = time.encode_to_vec();
    wire.extend([0xa0, 6, 123]);
    let dynamic = gz_msgs::prost_reflect::DynamicMessage::decode(
        gz_msgs::descriptor("gz.msgs.Time").unwrap(),
        wire.as_slice(),
    )
    .unwrap();
    assert_eq!(dynamic.unknown_fields().count(), 1);
    output.clear();
    gz_msgs::encode_append(&dynamic, &mut output).unwrap();
    assert_eq!(output, dynamic.encode_to_vec());
    assert_eq!(output, wire);
    let floats = FloatV {
        header: None,
        data: vec![-0.0, 0.0, f32::from_bits(0x7fc01234)],
    };
    output.clear();
    gz_msgs::encode_append(&floats, &mut output).unwrap();
    assert_eq!(output, floats.encode_to_vec());
    let decoded = FloatV::decode(output.as_slice()).unwrap();
    assert_eq!(
        decoded.data.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        floats.data.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
    );
}

fn run<T: Sample>(value: T, action: &str, mode: &str, count: u64) {
    let bytes = value.encode_to_vec();
    {
        let decoded = T::decode(bytes.as_slice()).unwrap();
        assert_eq!(decoded, value);
        assert_eq!(decoded.encode_to_vec(), bytes);
    }
    assert_eq!(encode_fresh(&value), bytes);
    if action == "dump" {
        std::io::stdout().write_all(&bytes).unwrap();
        return;
    }
    {
        let mut check = T::default();
        let mut encoded = Vec::with_capacity(bytes.len());
        for _ in 0..2 {
            check.clear();
            check.merge(bytes.as_slice()).unwrap();
            assert_eq!(check, value);
            encoded.clear();
            encode(&check, &mut encoded);
            assert_eq!(encoded, bytes);
        }
    }
    let mut reused = T::default();
    let mut output = Vec::with_capacity(bytes.len());
    let mut operation = || match mode {
        "encode_fresh" => {
            let encoded = encode_fresh(black_box(&value));
            let sample = encoded_sample(&encoded);
            black_box(encoded);
            sample
        }
        "encode_reuse" => {
            output.clear();
            encode(black_box(&value), &mut output);
            let sample = encoded_sample(&output);
            black_box(&output);
            sample
        }
        "decode_fresh" => {
            let decoded = T::decode(black_box(bytes.as_slice())).unwrap();
            let sample = decoded.sample();
            black_box(decoded);
            sample
        }
        "decode_reuse" => {
            reused.clear();
            reused.merge(black_box(bytes.as_slice())).unwrap();
            let sample = reused.sample();
            black_box(&reused);
            sample
        }
        "roundtrip_fresh" => {
            let decoded = T::decode(black_box(bytes.as_slice())).unwrap();
            let encoded = encode_fresh(&decoded);
            let sample = decoded.sample().wrapping_add(encoded_sample(&encoded));
            black_box((decoded, encoded));
            sample
        }
        "roundtrip_reuse" => {
            reused.clear();
            reused.merge(black_box(bytes.as_slice())).unwrap();
            output.clear();
            encode(&reused, &mut output);
            let sample = reused.sample().wrapping_add(encoded_sample(&output));
            black_box((&reused, &output));
            sample
        }
        _ => panic!("unknown codec mode"),
    };
    if action == "run" {
        for _ in 0..64 {
            black_box(operation());
        }
    }
    let start = (action == "run").then(Instant::now);
    let mut checksum = 0_u64;
    for _ in 0..count {
        checksum = checksum.wrapping_add(operation());
    }
    let elapsed = start.map_or(0, |start| start.elapsed().as_nanos());
    if mode == "decode_reuse" || mode == "roundtrip_reuse" {
        assert_eq!(reused, value);
        assert_eq!(reused.encode_to_vec(), bytes);
    }
    if mode == "encode_reuse" || mode == "roundtrip_reuse" {
        assert_eq!(output, bytes);
    }
    println!(
        "{{\"elapsed_ns\":{elapsed},\"checksum\":{checksum},\"iterations\":{count},\"bytes\":{}}}",
        bytes.len()
    );
}

fn envelope(fixture: &str, action: &str, count: u64) {
    let value = Time { sec: -1, nsec: 42 };
    let mut bytes = value.encode_to_vec();
    match fixture {
        "time_plain" => (),
        "time_unknown" => bytes.extend([0xa0, 6, 123]),
        "time_wrong" => bytes.extend([9, 0, 0, 0, 0, 0, 0, 0, 0]),
        _ => panic!("unknown envelope fixture"),
    }
    let check = WireMessage::<Time>::decode_compatible(&bytes).unwrap();
    assert_eq!(check.value(), &value);
    assert_eq!(check.bytes(), bytes);
    if action == "dump" {
        std::io::stdout().write_all(&bytes).unwrap();
        return;
    }
    let operation = || {
        let decoded = WireMessage::<Time>::decode_compatible(black_box(bytes.as_slice())).unwrap();
        let sample = decoded.value().nsec as u64 + decoded.bytes().len() as u64;
        black_box(decoded);
        sample
    };
    if action == "run" {
        for _ in 0..64 {
            black_box(operation());
        }
    }
    let start = (action == "run").then(Instant::now);
    let mut checksum = 0_u64;
    for _ in 0..count {
        checksum = checksum.wrapping_add(operation());
    }
    let elapsed = start.map_or(0, |start| start.elapsed().as_nanos());
    println!(
        "{{\"elapsed_ns\":{elapsed},\"checksum\":{checksum},\"iterations\":{count},\"bytes\":{}}}",
        bytes.len()
    );
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert!(
        args.len() == 3 || args.len() == 5,
        "dump FIXTURE | run|verify FIXTURE MODE ITERATIONS"
    );
    let (action, fixture) = (args[1].as_str(), args[2].as_str());
    assert!(action == "dump" || action == "run" || action == "verify");
    let mode = args.get(3).map(String::as_str).unwrap_or("");
    let count = args.get(4).map(|v| v.parse::<u64>().unwrap()).unwrap_or(0);
    if action != "dump" {
        assert!(count > 0 && count <= 1_000_000_000);
    }
    if action == "verify" {
        public_controls();
    }
    if fixture.starts_with("time_") {
        assert!(action == "dump" || mode == "envelope");
        envelope(fixture, action, count);
        return;
    }
    match fixture {
        "clock" => run(
            Clock {
                header: Some(header()),
                system: Some(Time { sec: -1, nsec: 42 }),
                real: Some(Time { sec: 12, nsec: 34 }),
                sim: Some(Time { sec: 56, nsec: 78 }),
            },
            action,
            mode,
            count,
        ),
        "imu" => {
            let covariance = || FloatV {
                header: Some(header()),
                data: vec![0.125, -0.0, 0.0, 0.0, 0.25, 0.0, 0.0, 0.0, 0.5],
            };
            run(
                Imu {
                    header: Some(header()),
                    entity_name: "robot/imu".into(),
                    orientation: Some(quaternion([-0.0, 0.0, 0.5, 1.0])),
                    orientation_covariance: Some(covariance()),
                    angular_velocity: Some(vector3([0.125, -0.25, 0.5])),
                    angular_velocity_covariance: Some(covariance()),
                    linear_acceleration: Some(vector3([-0.0, 0.0, -9.75])),
                    linear_acceleration_covariance: Some(covariance()),
                },
                action,
                mode,
                count,
            );
        }
        "poses_1" | "poses_100" | "poses_1000" => {
            let n = fixture.strip_prefix("poses_").unwrap().parse().unwrap();
            run(
                PoseV {
                    header: Some(header()),
                    pose: (0..n).map(pose).collect(),
                },
                action,
                mode,
                count,
            );
        }
        "scan_360" | "scan_1080" => {
            let n: usize = fixture.strip_prefix("scan_").unwrap().parse().unwrap();
            run(
                LaserScan {
                    header: Some(header()),
                    frame: "robot/lidar".into(),
                    world_pose: Some(pose(0)),
                    angle_min: -1.0,
                    angle_max: 1.0,
                    angle_step: 0.015625,
                    range_min: 0.125,
                    range_max: 64.0,
                    count: n as u32,
                    vertical_angle_min: -0.0,
                    vertical_angle_max: 0.0,
                    vertical_angle_step: 0.0,
                    vertical_count: 1,
                    ranges: (0..n).map(|i| 1.0 + (i % 128) as f64 * 0.125).collect(),
                    intensities: (0..n).map(|i| (i % 256) as f64).collect(),
                },
                action,
                mode,
                count,
            );
        }
        _ => panic!("unknown fixture"),
    }
}

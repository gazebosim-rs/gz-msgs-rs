//! Gazebo's canonical messages. Generated modules retain upstream file names.
include!(concat!(env!("OUT_DIR"), "/generated/mod.rs"));
pub use clock::Clock;
pub use imu::IMU;
pub use pose::Pose;
pub use protobuf;
pub use time::Time;
include!(concat!(env!("OUT_DIR"), "/registry.rs"));

/// Find a canonical descriptor by fully qualified protobuf name.
pub fn descriptor(name: &str) -> Option<protobuf::reflect::MessageDescriptor> {
    let name = format!(".{}", name.trim_start_matches('.'));
    files()
        .into_iter()
        .find_map(|file| file.message_by_full_name(&name))
}

/// Convert integer nanoseconds without imposing wall-clock timestamp limits.
pub fn time_from_nanos(nanos: i64) -> Time {
    Time {
        sec: nanos.div_euclid(1_000_000_000),
        nsec: nanos.rem_euclid(1_000_000_000) as i32,
        ..Default::default()
    }
}

/// Reject non-normalized or out-of-range input instead of wrapping.
pub fn time_to_nanos(time: &Time) -> Option<i64> {
    if !(0..1_000_000_000).contains(&time.nsec) {
        return None;
    }
    i64::try_from(i128::from(time.sec) * 1_000_000_000 + i128::from(time.nsec)).ok()
}

pub fn vector3(value: [f64; 3]) -> vector3d::Vector3d {
    vector3d::Vector3d {
        x: value[0],
        y: value[1],
        z: value[2],
        ..Default::default()
    }
}

/// Quaternion component order is x, y, z, w; values are not normalized.
pub fn quaternion(value: [f64; 4]) -> quaternion::Quaternion {
    quaternion::Quaternion {
        x: value[0],
        y: value[1],
        z: value[2],
        w: value[3],
        ..Default::default()
    }
}
pub use protobuf_json_mapping;

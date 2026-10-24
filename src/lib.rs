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
    let name = name.trim_start_matches('.');
    let name = match name.strip_prefix("gz_msgs.") {
        Some(local) => format!(".gz.msgs.{local}"),
        None => format!(".{name}"),
    };
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
    value.into()
}

/// Quaternion component order is x, y, z, w; values are not normalized.
pub fn quaternion(value: [f64; 4]) -> quaternion::Quaternion {
    value.into()
}

/// Constructs only a pose's geometric fields; names/IDs/headers remain default.
pub fn pose(position: [f64; 3], orientation: [f64; 4]) -> Pose {
    Pose {
        position: Some(position.into()).into(),
        orientation: Some(orientation.into()).into(),
        ..Default::default()
    }
}

/// Missing orientation means identity, matching Gazebo's domain conversion.
/// An explicitly present all-zero quaternion remains all zero.
pub fn pose_components(pose: &Pose) -> ([f64; 3], [f64; 4]) {
    let position = pose.position.as_ref().map_or([0.; 3], |p| [p.x, p.y, p.z]);
    let orientation = pose
        .orientation
        .as_ref()
        .map_or([0., 0., 0., 1.], |q| [q.x, q.y, q.z, q.w]);
    (position, orientation)
}

pub use protobuf_json_mapping;

mod conversions;

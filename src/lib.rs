// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Canonical Gazebo values and an explicit lossless wire envelope.
pub use {prost, prost_reflect, prost_types, serde_json};
/// Standard prost traits/types with the reviewed derive macro fixes.
#[doc(hidden)]
pub mod runtime {
    pub use prost::*;
    pub use prost_derive::{Enumeration, Message, Oneof};
}
pub mod msgs {
    include!(concat!(env!("OUT_DIR"), "/gz.msgs.rs"));
}
include!(concat!(env!("OUT_DIR"), "/aliases.rs"));
include!(concat!(env!("OUT_DIR"), "/definitions.rs"));
pub use msgs::{Clock, Imu as IMU, Pose, Time};
use std::sync::OnceLock;

pub fn descriptors() -> &'static prost_reflect::DescriptorPool {
    static POOL: OnceLock<prost_reflect::DescriptorPool> = OnceLock::new();
    POOL.get_or_init(|| {
        prost_reflect::DescriptorPool::decode(
            include_bytes!(concat!(env!("OUT_DIR"), "/messages.pb")).as_ref(),
        )
        .expect("generated descriptors")
    })
}

/// Canonical files only; imported Google types remain available in the pool.
pub fn files() -> Vec<prost_reflect::FileDescriptor> {
    descriptors()
        .files()
        .filter(|f| f.name().starts_with("gz/msgs/"))
        .collect()
}

pub fn descriptor(name: &str) -> Option<prost_reflect::MessageDescriptor> {
    let name = name.trim_start_matches('.');
    let canonical = match name.strip_prefix("gz_msgs.") {
        Some(local) => format!("gz.msgs.{local}"),
        None => name.to_owned(),
    };
    descriptors().get_message_by_name(&canonical)
}

/// A read-only typed view plus the original bytes, including all nested unknowns.
/// Forward bytes() verbatim. Generated typed decode/encode alone drops unknowns.
pub struct WireMessage<M> {
    bytes: Vec<u8>,
    value: M,
}
impl<M: prost::Message + Default> WireMessage<M> {
    pub fn decode(bytes: &[u8]) -> Result<Self, prost::DecodeError> {
        Ok(Self {
            bytes: bytes.to_vec(),
            value: M::decode(bytes)?,
        })
    }
    pub fn value(&self) -> &M {
        &self.value
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Match Gazebo duration conversion: signed seconds/remainder truncate toward zero.
pub fn time_from_nanos(nanos: i64) -> Time {
    Time {
        sec: nanos / 1_000_000_000,
        nsec: (nanos % 1_000_000_000) as i32,
    }
}

/// Sum signed fields as Gazebo does, accepting unnormalized nanos; reject overflow.
pub fn time_to_nanos(time: &Time) -> Option<i64> {
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
        position: Some(position.into()),
        orientation: Some(orientation.into()),
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

pub mod cloud;
mod conversions;
pub mod enum_names;

pub mod registry;

pub mod factory;

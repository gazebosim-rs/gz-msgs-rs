// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Packed-cloud metadata helpers matching the native initializer.
use crate::msgs::{
    header,
    point_cloud_packed::{field::DataType, Field},
    PointCloudPacked,
};
fn width(datatype: i32) -> Option<u32> {
    Some(match DataType::try_from(datatype).ok()? {
        DataType::Int8 | DataType::Uint8 => 1,
        DataType::Int16 | DataType::Uint16 => 2,
        DataType::Int32 | DataType::Uint32 | DataType::Float32 => 4,
        DataType::Float64 => 8,
    })
}
/// Native datatype size in bytes; unknown values log and return -1.
pub fn size_of_point_field(datatype: i32) -> i32 {
    match width(datatype) {
        Some(size) => size as i32,
        None => {
            eprintln!("PointCloudPacked::Field of type [{datatype}] does not exist");
            -1
        }
    }
}
/// Reset fields/frame metadata while retaining stamp, data and other cloud state.
///
/// `xyz` expands to three fields; alignment follows the native machine word.
/// Unknown datatypes retain legacy zero-width metadata and diagnostics. This
/// initializer does not validate the existing data blob or provide unsafe access.
pub fn init_point_cloud_packed(
    cloud: &mut PointCloudPacked,
    frame: &str,
    aligned: bool,
    fields: &[(&str, i32)],
) {
    cloud.header.get_or_insert_default().data = vec![header::Map {
        key: "frame_id".into(),
        value: vec![frame.into()],
    }];
    cloud.field.clear();
    let mut offset = 0_u32;
    let word = std::mem::size_of::<usize>() as u32;
    for &(name, datatype) in fields {
        let names = if name == "xyz" {
            &["x", "y", "z"][..]
        } else {
            std::slice::from_ref(&name)
        };
        for name in names {
            cloud.field.push(Field {
                name: (*name).into(),
                offset,
                datatype,
                count: 1,
            });
            if let Some(size) = width(datatype) {
                offset = offset.wrapping_add(size);
            } else {
                eprintln!("PointCloudPacked field datatype of [{datatype}] is invalid.");
            }
        }
        if aligned {
            offset = offset.wrapping_add(word - 1) & !(word - 1);
        }
    }
    cloud.point_step = offset;
}

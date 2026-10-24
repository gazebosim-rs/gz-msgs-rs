// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Packed-cloud metadata helpers matching the native initializer.
mod cursor;
pub use cursor::NativeFieldCursor;

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

/// Resolve direct fields before legacy packed RGB/RGBA byte aliases.
fn field_offset(cloud: &PointCloudPacked, name: &str) -> Option<usize> {
    if let Some(field) = cloud.field.iter().find(|field| field.name == name) {
        return Some(field.offset as usize);
    }
    let channel = match (name, cloud.is_bigendian) {
        ("r", false) | ("g", true) => 2,
        ("g", false) | ("r", true) => 1,
        ("b", false) | ("a", true) => 0,
        ("a", false) | ("b", true) => 3,
        _ => return None,
    };
    let field = cloud
        .field
        .iter()
        .find(|field| matches!(field.name.as_str(), "rgb" | "rgba"))?;
    (field.offset as usize).checked_add(channel)
}
fn byte_range<T: bytemuck::Pod>(
    cloud: &PointCloudPacked,
    name: &str,
    point: usize,
    element: usize,
) -> Option<std::ops::Range<usize>> {
    offset_range::<T>(cloud, field_offset(cloud, name)?, point, element)
}
fn offset_range<T: bytemuck::Pod>(
    cloud: &PointCloudPacked,
    offset: usize,
    point: usize,
    element: usize,
) -> Option<std::ops::Range<usize>> {
    let stride = cloud.point_step as usize;
    let size = std::mem::size_of::<T>();
    if stride == 0 || size == 0 || point >= cloud.data.len() / stride {
        return None;
    }
    let start = point
        .checked_mul(stride)?
        .checked_add(offset)?
        .checked_add(element.checked_mul(size)?)?;
    let end = start.checked_add(size)?;
    (end <= cloud.data.len()).then_some(start..end)
}
/// Read native-endian bytes, matching the legacy typed iterator.
///
/// `point` advances by `point_step`; `element` advances contiguously by the
/// requested Rust type's size, independent of the declared datatype/count.
/// The endian flag only selects packed color aliases. Dimensions and row_step
/// are not consulted. Missing fields, zero stride/size, arithmetic overflow,
/// points beyond the complete-record count and out-of-blob reads return None.
/// Unaligned fields are supported; no reference into the blob is created.
pub fn read_native<T: bytemuck::Pod>(
    cloud: &PointCloudPacked,
    name: &str,
    point: usize,
    element: usize,
) -> Option<T> {
    bytemuck::try_pod_read_unaligned(&cloud.data[byte_range::<T>(cloud, name, point, element)?])
        .ok()
}
/// Write with the same byte addressing and failure contract as [`read_native`].
/// Failure leaves the blob unchanged. The exclusive borrow prevents concurrent
/// mutable iterator aliases; message metadata is untouched.
pub fn write_native<T: bytemuck::Pod>(
    cloud: &mut PointCloudPacked,
    name: &str,
    point: usize,
    element: usize,
    value: T,
) -> Option<()> {
    let range = byte_range::<T>(cloud, name, point, element)?;
    cloud.data[range].copy_from_slice(bytemuck::bytes_of(&value));
    Some(())
}

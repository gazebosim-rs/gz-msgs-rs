// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Plane and bounding-box components without a simulator math dependency.
use crate::{
    msgs::{AxisAlignedBox, PlaneGeom},
    set_vector3,
};

/// Creates a plane with default metadata; components are not normalized.
pub fn plane(normal: [f64; 3], size: [f64; 2], offset: f64) -> PlaneGeom {
    let mut message = PlaneGeom::default();
    set_plane(&mut message, normal, size, offset);
    message
}

/// Updates geometry while retaining outer and nested message headers.
pub fn set_plane(message: &mut PlaneGeom, normal: [f64; 3], [x, y]: [f64; 2], offset: f64) {
    set_vector3(message.normal.get_or_insert_default(), normal);
    let size = message.size.get_or_insert_default();
    size.x = x;
    size.y = y;
    message.d = offset;
}

/// Missing nested vectors read as zero, matching native domain conversion.
pub fn plane_components(message: &PlaneGeom) -> ([f64; 3], [f64; 2], f64) {
    let normal = message.normal.as_ref().map_or([0.; 3], |v| [v.x, v.y, v.z]);
    let size = message.size.as_ref().map_or([0.; 2], |v| [v.x, v.y]);
    (normal, size, message.d)
}

/// Creates an axis-aligned box with default metadata; corners are not reordered.
pub fn axis_aligned_box(min: [f64; 3], max: [f64; 3]) -> AxisAlignedBox {
    let mut message = AxisAlignedBox::default();
    set_axis_aligned_box(&mut message, min, max);
    message
}

/// Updates both corners while retaining outer and nested message headers.
pub fn set_axis_aligned_box(message: &mut AxisAlignedBox, min: [f64; 3], max: [f64; 3]) {
    set_vector3(message.min_corner.get_or_insert_default(), min);
    set_vector3(message.max_corner.get_or_insert_default(), max);
}

/// Missing corners read as zero; explicitly inverted corners remain inverted.
pub fn axis_aligned_box_components(message: &AxisAlignedBox) -> ([f64; 3], [f64; 3]) {
    let min = message
        .min_corner
        .as_ref()
        .map_or([0.; 3], |v| [v.x, v.y, v.z]);
    let max = message
        .max_corner
        .as_ref()
        .map_or([0.; 3], |v| [v.x, v.y, v.z]);
    (min, max)
}

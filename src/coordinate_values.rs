// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2023 Open Source Robotics Foundation
//! Native gz-math coordinate enum codes differ from canonical protobuf numbers.
use crate::msgs::SphericalCoordinatesType;

/// Convert raw gz-math codes (SPHERICAL=1, ECEF=2, GLOBAL=3, LOCAL=4).
/// Invalid values diagnose and fall back to LOCAL, matching native ConvertCoord.
pub fn coordinate_type_from_math(value: i32) -> SphericalCoordinatesType {
    match value {
        1 => SphericalCoordinatesType::Spherical,
        2 => SphericalCoordinatesType::Ecef,
        3 => SphericalCoordinatesType::Global,
        4 => SphericalCoordinatesType::Local,
        _ => {
            eprintln!("Invalid coordinate type passed");
            SphericalCoordinatesType::Local
        }
    }
}
/// Convert canonical protobuf codes to raw gz-math codes, with native LOCAL fallback.
pub fn coordinate_type_to_math(value: i32) -> i32 {
    match value {
        0 => 1,
        1 => 2,
        2 => 3,
        3 => 4,
        _ => {
            eprintln!("Invalid coordinate type passed");
            4
        }
    }
}

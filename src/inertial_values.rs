// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2023 Open Source Robotics Foundation
//! Raw inertial components; physical validity and math operations belong to callers.
use crate::msgs::Inertial;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MassMatrixComponents {
    pub mass: f64,
    /// XX, YY, ZZ.
    pub diagonal: [f64; 3],
    /// XY, XZ, YZ.
    pub off_diagonal: [f64; 3],
}
#[derive(Clone, Debug, PartialEq)]
pub struct InertialComponents {
    pub mass_matrix: MassMatrixComponents,
    pub position: [f64; 3],
    /// XYZW; not normalized.
    pub orientation: [f64; 4],
    /// Only the upper triangle is serialized; decoding reflects it below the diagonal.
    pub fluid_added_mass: Option<[[f64; 6]; 6]>,
}
impl Default for InertialComponents {
    fn default() -> Self {
        Self {
            mass_matrix: MassMatrixComponents::default(),
            position: [0.; 3],
            orientation: [0., 0., 0., 1.],
            fluid_added_mass: None,
        }
    }
}
/// Create a mass-matrix message, including the native explicit identity W field.
pub fn mass_matrix(value: MassMatrixComponents) -> Inertial {
    let mut message = Inertial::default();
    set_mass_matrix(&mut message, value);
    message
}
/// Copy mass/moments and set only pose.orientation.w to one, retaining other fields.
pub fn set_mass_matrix(message: &mut Inertial, value: MassMatrixComponents) {
    message.mass = value.mass;
    [message.ixx, message.iyy, message.izz] = value.diagonal;
    [message.ixy, message.ixz, message.iyz] = value.off_diagonal;
    message
        .pose
        .get_or_insert_default()
        .orientation
        .get_or_insert_default()
        .w = 1.;
}
pub fn mass_matrix_components(message: &Inertial) -> MassMatrixComponents {
    MassMatrixComponents {
        mass: message.mass,
        diagonal: [message.ixx, message.iyy, message.izz],
        off_diagonal: [message.ixy, message.ixz, message.iyz],
    }
}
/// Create a fresh message; component values are copied without physical validation.
pub fn inertial(value: &InertialComponents) -> Inertial {
    let mut message = Inertial::default();
    set_inertial(&mut message, value);
    message
}
/// Retain metadata and append an upper triangle when present, matching native Set.
/// None leaves existing fluid entries untouched; repeated Some appends another21.
pub fn set_inertial(message: &mut Inertial, value: &InertialComponents) {
    set_mass_matrix(message, value.mass_matrix);
    crate::set_pose(
        message.pose.get_or_insert_default(),
        value.position,
        value.orientation,
    );
    if let Some(matrix) = &value.fluid_added_mass {
        for (row, values) in matrix.iter().enumerate() {
            message.fluid_added_mass.extend_from_slice(&values[row..]);
        }
    }
}
/// Read a fresh domain value. Exactly21 fluid entries form a symmetric matrix;
/// other lengths are ignored. Absent pose/orientation follows native identity defaults.
#[allow(clippy::needless_range_loop)] // Both matrix triangles are assigned together.
pub fn inertial_components(message: &Inertial) -> InertialComponents {
    let (position, orientation) = message
        .pose
        .as_ref()
        .map_or(([0.; 3], [0., 0., 0., 1.]), crate::pose_components);
    let fluid_added_mass = if message.fluid_added_mass.len() == 21 {
        let mut matrix = [[0.; 6]; 6];
        let mut offset = 0;
        for row in 0..6 {
            for col in row..6 {
                matrix[row][col] = message.fluid_added_mass[offset];
                matrix[col][row] = message.fluid_added_mass[offset];
                offset += 1;
            }
        }
        Some(matrix)
    } else {
        None
    };
    InertialComponents {
        mass_matrix: mass_matrix_components(message),
        position,
        orientation,
        fluid_added_mass,
    }
}

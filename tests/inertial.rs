// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2016 Open Source Robotics Foundation
// Copyright (C) 2023 Open Source Robotics Foundation
// Original Utility_TEST.cc inertial/mass-matrix cases and native state oracle.
use gz_msgs::{
    msgs::{Header, Inertial, Pose, Quaternion, Time, Vector3d},
    prost::Message,
    *,
};
fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!("fixtures/inertial-reference.json")).unwrap()
}
fn rotation() -> [f64; 4] {
    let f = fixture();
    std::array::from_fn(|i| f64::from_bits(f["orientation_bits"][i].as_u64().unwrap()))
}
fn mass(invalid: bool) -> MassMatrixComponents {
    if invalid {
        MassMatrixComponents {
            mass: -12.,
            diagonal: [-2., 0., 0.],
            off_diagonal: [-0.1, 2., 3.],
        }
    } else {
        MassMatrixComponents {
            mass: 12.,
            diagonal: [2., 3., 4.],
            off_diagonal: [0.1, 0.2, 0.3],
        }
    }
}
fn upper() -> [f64; 21] {
    [
        0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1., 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8,
        1.9, 2., 2.1,
    ]
}
#[allow(clippy::needless_range_loop)] // Populate the two triangles together.
fn matrix(asymmetric: bool) -> [[f64; 6]; 6] {
    let mut matrix = [[0.; 6]; 6];
    let mut i = 0;
    for row in 0..6 {
        for col in row..6 {
            matrix[row][col] = upper()[i];
            matrix[col][row] = if asymmetric && col != row {
                -99.
            } else {
                upper()[i]
            };
            i += 1;
        }
    }
    matrix
}
fn values(added: bool) -> InertialComponents {
    InertialComponents {
        mass_matrix: mass(false),
        position: [5., 6., 7.],
        orientation: rotation(),
        fluid_added_mass: added.then(|| matrix(false)),
    }
}
fn assert_mass(message: &Inertial) {
    assert_eq!(message.mass, 12.);
    assert_eq!(message.ixx, 2.);
    assert_eq!(message.iyy, 3.);
    assert_eq!(message.izz, 4.);
    assert_eq!(message.ixy, 0.1);
    assert_eq!(message.ixz, 0.2);
    assert_eq!(message.iyz, 0.3);
}
fn assert_domain_mass(value: MassMatrixComponents) {
    assert_eq!(value.mass, 12.);
    assert_eq!(value.diagonal, [2., 3., 4.]);
    assert_eq!(value.off_diagonal, [0.1, 0.2, 0.3]);
}
fn assert_original_pose(position: [f64; 3], orientation: [f64; 4]) {
    for (a, b) in position.into_iter().zip([5., 6., 7.]) {
        assert!((a - b).abs() <= 1e-3);
    }
    for (a, b) in orientation.into_iter().zip(rotation()) {
        assert!((a - b).abs() <= 1e-3);
    }
}
#[test]
fn original_inertial_conversions_and_setter() {
    let value = values(false);
    let message = inertial(&value);
    assert_mass(&message);
    let (position, orientation) = pose_components(message.pose.as_ref().unwrap());
    assert_original_pose(position, orientation);
    assert!(message.fluid_added_mass.is_empty());
    let converted = inertial_components(&message);
    assert_domain_mass(converted.mass_matrix);
    assert_original_pose(converted.position, converted.orientation);
    assert!(converted.fluid_added_mass.is_none());
    let mut assigned = Inertial::default();
    set_inertial(&mut assigned, &value);
    assert_mass(&assigned);
    let (position, orientation) = pose_components(assigned.pose.as_ref().unwrap());
    assert_original_pose(position, orientation);
}
#[test]
fn original_added_mass_conversions() {
    let message = inertial(&values(true));
    assert_mass(&message);
    let (position, orientation) = pose_components(message.pose.as_ref().unwrap());
    assert_original_pose(position, orientation);
    assert_eq!(message.fluid_added_mass.len(), 21);
    for (actual, expected) in message.fluid_added_mass.iter().zip(upper()) {
        assert_eq!(*actual, expected);
    }
    let value = inertial_components(&message);
    assert_domain_mass(value.mass_matrix);
    assert_original_pose(value.position, value.orientation);
    assert!(value.fluid_added_mass.is_some());
    // Native Matrix6::operator== uses scalar tolerance; exact equality is stronger.
    assert_eq!(value.fluid_added_mass.unwrap(), matrix(false));
}
#[test]
fn original_mass_matrix_conversion_and_setter() {
    let message = mass_matrix(mass(false));
    assert_mass(&message);
    let (position, orientation) = pose_components(message.pose.as_ref().unwrap());
    for (actual, expected) in position
        .into_iter()
        .chain(orientation)
        .zip([0., 0., 0., 0., 0., 0., 1.])
    {
        assert!((actual - expected).abs() <= 1e-1);
    }
    let mut assigned = Inertial::default();
    set_mass_matrix(&mut assigned, mass(false));
    assert_mass(&assigned);
    let (position, orientation) = pose_components(assigned.pose.as_ref().unwrap());
    for (actual, expected) in position
        .into_iter()
        .chain(orientation)
        .zip([0., 0., 0., 0., 0., 0., 1.])
    {
        assert!((actual - expected).abs() <= 1e-3);
    }
}
fn header(sec: i64) -> Option<Header> {
    Some(Header {
        stamp: Some(Time { sec, nsec: 0 }),
        ..Default::default()
    })
}
fn seeded() -> Inertial {
    Inertial {
        header: header(11),
        pose: Some(Pose {
            header: header(22),
            name: "retained".into(),
            id: 42,
            position: Some(Vector3d {
                header: header(33),
                x: 8.,
                y: 9.,
                z: 10.,
            }),
            orientation: Some(Quaternion {
                header: header(44),
                x: 0.1,
                y: 0.2,
                z: 0.3,
                w: 0.4,
            }),
        }),
        fluid_added_mass: vec![99.],
        ..Default::default()
    }
}
#[test]
fn native_inertial_wire_and_reverse_state_corpus() {
    let fixture = fixture();
    let rows = fixture["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 24);
    for row in rows {
        let seeded = row["seeded"].as_u64().unwrap() == 1;
        let kind = row["kind"].as_u64().unwrap();
        let repeat = row["repeat"].as_u64().unwrap();
        let mm = mass(row["invalid"].as_u64().unwrap() == 1);
        let mut message = if seeded {
            self::seeded()
        } else {
            Inertial::default()
        };
        if kind == 0 {
            if seeded {
                set_mass_matrix(&mut message, mm);
            } else {
                message = mass_matrix(mm);
            }
        } else {
            let value = InertialComponents {
                mass_matrix: mm,
                position: [5., 6., 7.],
                orientation: rotation(),
                fluid_added_mass: (kind >= 2).then(|| matrix(kind == 3)),
            };
            if seeded {
                set_inertial(&mut message, &value);
            } else {
                message = inertial(&value);
            }
            for _ in 0..repeat {
                set_inertial(&mut message, &value);
            }
        }
        let hex: String = message
            .encode_to_vec()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(hex, row["wire_hex"].as_str().unwrap(), "{row}");
        let value = inertial_components(&message);
        let mm = value.mass_matrix;
        let actual = [
            mm.mass,
            mm.diagonal[0],
            mm.diagonal[1],
            mm.diagonal[2],
            mm.off_diagonal[0],
            mm.off_diagonal[1],
            mm.off_diagonal[2],
        ];
        let expected: Vec<_> = row["mass_bits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap())
            .collect();
        assert_eq!(actual.map(f64::to_bits).as_slice(), expected);
        let expected: Vec<_> = row["pose_bits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap())
            .collect();
        let actual: Vec<_> = value
            .position
            .into_iter()
            .chain(value.orientation)
            .map(f64::to_bits)
            .collect();
        assert_eq!(actual, expected);
        if row["added_bits"].is_null() {
            assert!(value.fluid_added_mass.is_none());
        } else {
            let expected: Vec<_> = row["added_bits"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap())
                .collect();
            let actual: Vec<_> = value
                .fluid_added_mass
                .unwrap()
                .into_iter()
                .flatten()
                .map(f64::to_bits)
                .collect();
            assert_eq!(actual, expected);
        }
    }
}
#[test]
fn defaults_and_incomplete_fluid_data() {
    assert_eq!(
        inertial_components(&Inertial::default()),
        InertialComponents::default()
    );
    let message = Inertial {
        pose: Some(Pose {
            position: Some(vector3([1., 2., 3.])),
            ..Default::default()
        }),
        fluid_added_mass: vec![1.; 20],
        ..Default::default()
    };
    let value = inertial_components(&message);
    assert_eq!(value.position, [1., 2., 3.]);
    assert_eq!(value.orientation, [0., 0., 0., 1.]);
    assert!(value.fluid_added_mass.is_none());
}

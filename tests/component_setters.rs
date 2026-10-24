// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2016 Open Source Robotics Foundation
// Copyright (C) 2023 Open Source Robotics Foundation
// Original Utility_TEST.cc quaternion/pose conversions and five component setters.
use gz_msgs::{
    msgs::{Color, Header, Pose, Quaternion, Time, Vector2d, Vector3d},
    prost::Message,
    *,
};
fn rotations() -> Vec<[f64; 4]> {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/component-reference.json")).unwrap();
    fixture["orientations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| std::array::from_fn(|i| f64::from_bits(row[i].as_u64().unwrap())))
        .collect()
}
#[allow(clippy::excessive_precision)] // Preserve original C++ assertion literals.
fn original_quaternion_values(actual: [f64; 4]) {
    for (actual, expected) in actual.into_iter().zip([
        -0.65328148243818818,
        0.27059805007309856,
        0.65328148243818829,
        0.27059805007309851,
    ]) {
        assert!((actual - expected).abs() <= 1e-6);
    }
}
#[test]
fn original_quaternion_and_pose_conversions_and_assignment() {
    // Euler-to-quaternion computation belongs to the caller's math library. Use
    // the actual pinned native math result, preserving every original assertion.
    let rotation = rotations()[0];
    let message = Quaternion::from(rotation);
    original_quaternion_values([message.x, message.y, message.z, message.w]);
    original_quaternion_values(<[f64; 4]>::from(message));
    let message = pose([1., 2., 3.], rotation);
    let position = message.position.as_ref().unwrap();
    assert_eq!([position.x, position.y, position.z], [1., 2., 3.]);
    let orientation = message.orientation.as_ref().unwrap();
    original_quaternion_values([orientation.x, orientation.y, orientation.z, orientation.w]);
    let (position, orientation) = pose_components(&message);
    assert_eq!(position, [1., 2., 3.]);
    original_quaternion_values(orientation);
    let mut message = Quaternion::default();
    set_quaternion(&mut message, rotation);
    original_quaternion_values([message.x, message.y, message.z, message.w]);
    let mut message = Pose::default();
    set_pose(&mut message, [1., 2., 3.], rotation);
    let position = message.position.unwrap();
    assert_eq!([position.x, position.y, position.z], [1., 2., 3.]);
    let orientation = message.orientation.unwrap();
    original_quaternion_values([orientation.x, orientation.y, orientation.z, orientation.w]);
}
#[test]
fn original_vector_and_color_assignment() {
    let mut message = Vector3d::default();
    set_vector3(&mut message, [1., 2., 3.]);
    assert_eq!([message.x, message.y, message.z], [1., 2., 3.]);
    let mut message = Vector2d::default();
    set_vector2(&mut message, [1., 2.]);
    assert_eq!([message.x, message.y], [1., 2.]);
    let mut message = Color::default();
    set_color(&mut message, [0.1, 0.2, 0.3, 1.]);
    assert_eq!(
        [message.r, message.g, message.b, message.a],
        [0.1, 0.2, 0.3, 1.]
    );
}
#[test]
fn native_full_wire_component_setters_preserve_metadata_and_raw_rotations() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/component-reference.json")).unwrap();
    let rotations = rotations();
    let header = |sec| {
        Some(Header {
            stamp: Some(Time { sec, nsec: 0 }),
            ..Default::default()
        })
    };
    let rows = fixture["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 18);
    for row in rows {
        let seeded = row["seeded"].as_u64().unwrap() == 1;
        let rotation = rotations[row["index"].as_u64().unwrap() as usize];
        let seed = if seeded { header(11) } else { None };
        let bytes = match row["kind"].as_str().unwrap() {
            "v2" => {
                let mut message = Vector2d {
                    header: seed,
                    ..Default::default()
                };
                set_vector2(&mut message, [1., 2.]);
                message.encode_to_vec()
            }
            "v3" => {
                let mut message = Vector3d {
                    header: seed,
                    ..Default::default()
                };
                set_vector3(&mut message, [1., 2., 3.]);
                message.encode_to_vec()
            }
            "color" => {
                let mut message = Color {
                    header: seed,
                    ..Default::default()
                };
                set_color(&mut message, [0.1, 0.2, 0.3, 1.]);
                message.encode_to_vec()
            }
            "quaternion" => {
                let mut message = Quaternion {
                    header: seed,
                    ..Default::default()
                };
                set_quaternion(&mut message, rotation);
                assert_eq!(
                    <[f64; 4]>::from(message.clone()).map(f64::to_bits),
                    rotation.map(f64::to_bits)
                );
                message.encode_to_vec()
            }
            "pose" => {
                let mut message = Pose {
                    header: seed,
                    ..Default::default()
                };
                if seeded {
                    message.name = "retained".into();
                    message.id = 42;
                    message.position = Some(Vector3d {
                        header: header(22),
                        ..Default::default()
                    });
                    message.orientation = Some(Quaternion {
                        header: header(33),
                        ..Default::default()
                    });
                }
                set_pose(&mut message, [1., 2., 3.], rotation);
                let (position, orientation) = pose_components(&message);
                assert_eq!(position, [1., 2., 3.]);
                assert_eq!(orientation.map(f64::to_bits), rotation.map(f64::to_bits));
                message.encode_to_vec()
            }
            other => panic!("unknown native kind {other}"),
        };
        let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hex, row["wire_hex"].as_str().unwrap(), "{row}");
    }
}

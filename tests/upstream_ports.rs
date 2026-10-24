// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Assertion-preserving ports from Gazebo msgs 89c0c3d Utility_TEST.cc/Factory_TEST.cc.
// Copyright (C) Open Source Robotics Foundation; Apache-2.0.
use gz_msgs::*;

#[test]
fn utility_primitive_conversions() {
    macro_rules! check {
        ($message:path, $value:ty, $expected:expr) => {
            let message = <$message>::from($expected);
            assert_eq!(message.data, $expected);
            let value: $value = message.into();
            assert_eq!(value, $expected);
        };
    }
    check!(stringmsg::StringMsg, String, String::from("a string msg"));
    check!(boolean::Boolean, bool, true);
    check!(int32::Int32, i32, -543);
    check!(uint32::UInt32, u32, 123456);
    check!(int64::Int64, i64, i64::MIN);
    check!(uint64::UInt64, u64, u64::MAX);
    check!(double::Double, f64, -33.888);
    check!(float::Float, f32, 0.999);
}

#[test]
fn factory_type_and_new() {
    let names: Vec<_> = files()
        .iter()
        .flat_map(|f| f.messages().map(|m| m.full_name().to_string()))
        .collect();
    assert!(!names.is_empty());
    assert!(names.contains(&"gz.msgs.Vector3d".into()));
    for name in [
        "gz.msgs.Vector3d",
        "gz_msgs.Vector3d",
        ".gz.msgs.Vector3d",
        ".gz_msgs.Vector3d",
    ] {
        assert_eq!(descriptor(name).unwrap().full_name(), "gz.msgs.Vector3d");
    }
    assert!(descriptor("StringMsg").is_none());
    assert!(descriptor("example.msgs.StringMsg").is_none());
    for name in [
        "gz.msgs.SerializedEntityMap",
        "gz.msgs.SerializedStateMap",
        "gz.msgs.SerializedStepMap",
    ] {
        assert!(names.contains(&name.into()));
        assert_eq!(descriptor(name).unwrap().full_name(), name);
    }
}

#[test]
fn utility_time_point() {
    let zero = time_from_nanos(0);
    assert_eq!(zero.sec, 0);
    assert_eq!(zero.nsec, 0);
    assert_eq!(time_to_nanos(&zero), Some(0));
    let time = time_from_nanos(200_000_000_999);
    assert_eq!(time.sec, 200);
    assert_eq!(time.nsec, 999);
}

#[test]
fn python_basic_serialization() {
    use gz_msgs::prost::Message;
    let message = vector3([1., 2., 3.]);
    let bytes = message.encode_to_vec();
    assert!(!bytes.is_empty());
    let mut decoded = vector3d::Vector3d::default();
    assert_ne!(decoded, message);
    decoded.merge(bytes.as_slice()).unwrap();
    assert_eq!(decoded, message);
}

#[test]
fn utility_vector_and_color_conversions() {
    let vector2 = vector2d::Vector2d::from([1., 2.]);
    assert_eq!([vector2.x, vector2.y], [1., 2.]);
    assert_eq!(<[f64; 2]>::from(vector2), [1., 2.]);
    let vector3 = vector3d::Vector3d::from([1., 2., 3.]);
    assert_eq!([vector3.x, vector3.y, vector3.z], [1., 2., 3.]);
    assert_eq!(<[f64; 3]>::from(vector3), [1., 2., 3.]);
    let color = color::Color::from([0.1, 0.2, 0.3, 1.]);
    assert_eq!([color.r, color.g, color.b, color.a], [0.1, 0.2, 0.3, 1.]);
    assert_eq!(<[f32; 4]>::from(color), [0.1, 0.2, 0.3, 1.]);
}

#[test]
fn pose_defaults_distinguish_missing_and_zero_orientation() {
    let pose = Pose {
        position: Some(vector3([1., 2., 2.])),
        ..Default::default()
    };
    let (_, orientation) = pose_components(&pose);
    assert_eq!(orientation, [0., 0., 0., 1.]);
    assert_eq!(
        pose_components(&Pose::default()),
        ([0.; 3], [0., 0., 0., 1.])
    );
    let explicit = gz_msgs::pose([1., 2., 3.], [0.; 4]);
    assert_eq!(pose_components(&explicit), ([1., 2., 3.], [0.; 4]));
    let quaternion = quaternion::Quaternion::from([0.1, 0.2, 0.3, 0.4]);
    assert_eq!(<[f64; 4]>::from(quaternion), [0.1, 0.2, 0.3, 0.4]);
}

#[test]
#[allow(clippy::field_reassign_with_default)] // These preserve the original setter assertions.
fn utility_primitive_assignment() {
    let mut string = stringmsg::StringMsg::default();
    string.data = "a string msg".into();
    assert_eq!(string.data, "a string msg");
    string.data = "a string msg2".into();
    assert_eq!(string.data, "a string msg2");
    macro_rules! assign {
        ($message:path, $value:expr) => {
            let mut message = <$message>::default();
            message.data = $value;
            assert_eq!(message.data, $value);
        };
    }
    assign!(boolean::Boolean, false);
    assign!(int32::Int32, 9976);
    assign!(uint32::UInt32, 0);
    assign!(int64::Int64, i64::MAX);
    assign!(uint64::UInt64, u64::MIN);
    assign!(double::Double, 0.123);
    assign!(float::Float, -367.0);
}

#[test]
fn native_duration_reference_negative_and_boundary_values() {
    use gz_msgs::prost::Message;
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/time-reference.json")).unwrap();
    for row in fixture["encode"].as_array().unwrap() {
        let time = time_from_nanos(row["nanos"].as_i64().unwrap());
        assert_eq!(time.sec, row["sec"].as_i64().unwrap());
        assert_eq!(i64::from(time.nsec), row["nsec"].as_i64().unwrap());
        let hex = time
            .encode_to_vec()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert_eq!(hex, row["wire_hex"].as_str().unwrap());
        assert_eq!(time_to_nanos(&time), row["nanos"].as_i64());
    }
    for row in fixture["decode"].as_array().unwrap() {
        let time = Time {
            sec: row["sec"].as_i64().unwrap(),
            nsec: row["nsec"].as_i64().unwrap() as i32,
        };
        assert_eq!(time_to_nanos(&time), row["nanos"].as_i64());
    }
}

#[test]
fn factory_original_text_construction_and_all_registered_types() {
    use gz_msgs::{
        prost::Message,
        prost_reflect::{DynamicMessage, ReflectMessage},
    };
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/factory-reference.json")).unwrap();
    let expected = vector3([1.0, 2.0, 3.0]);
    let valid = DynamicMessage::parse_text_format(
        descriptor("gz.msgs.Vector3d").unwrap(),
        "x: 1.0, y: 2.0, z: 3.0",
    )
    .unwrap();
    let filled = vector3d::Vector3d::decode(valid.encode_to_vec().as_slice()).unwrap();
    assert_eq!(expected.x, filled.x);
    assert_eq!(expected.y, filled.y);
    assert_eq!(expected.z, filled.z);
    let hex = valid
        .encode_to_vec()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(hex, fixture["valid_wire_hex"].as_str().unwrap());
    assert!(fixture["malformed_is_null"].as_bool().unwrap());
    assert!(DynamicMessage::parse_text_format(
        descriptor("gz.msgs.Vector3d").unwrap(),
        "x: 1.0, y: asdf, z: 3.0"
    )
    .is_err());
    let mut names: Vec<_> = descriptors()
        .all_messages()
        .map(|message| message.full_name().to_owned())
        .collect();
    names.sort();
    let native: Vec<_> = fixture["types"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| name.as_str().unwrap())
        .collect();
    assert!(!names.is_empty());
    assert_eq!(names, native);
    for name in names {
        let message = DynamicMessage::new(descriptor(&name).unwrap());
        assert_eq!(message.descriptor().full_name(), name);
    }
}

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
        assert_eq!(
            descriptor(name)
                .unwrap()
                .new_instance()
                .descriptor_dyn()
                .full_name(),
            "gz.msgs.Vector3d"
        );
    }
    assert!(descriptor("StringMsg").is_none());
    assert!(descriptor("example.msgs.StringMsg").is_none());
    for name in [
        "gz.msgs.SerializedEntityMap",
        "gz.msgs.SerializedStateMap",
        "gz.msgs.SerializedStepMap",
    ] {
        assert!(names.contains(&name.into()));
        assert_eq!(
            descriptor(name)
                .unwrap()
                .new_instance()
                .descriptor_dyn()
                .full_name(),
            name
        );
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
    use gz_msgs::protobuf::Message;
    let message = vector3([1., 2., 3.]);
    let bytes = message.write_to_bytes().unwrap();
    assert!(!bytes.is_empty());
    let mut decoded = vector3d::Vector3d::default();
    assert_ne!(decoded, message);
    decoded.merge_from_bytes(&bytes).unwrap();
    assert_eq!(decoded, message);
}

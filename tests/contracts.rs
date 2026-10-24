use gz_msgs::{
    protobuf::{Message, MessageFull},
    *,
};

#[test]
fn time_contract() {
    for nanos in [i64::MIN, -1_000_000_001, -1, 0, 1, i64::MAX] {
        let time = time_from_nanos(nanos);
        assert_eq!(time_to_nanos(&time), Some(nanos));
    }
    for (sec, nsec) in [(0, -1), (0, 1_000_000_000), (i64::MAX, 0), (i64::MIN, 0)] {
        assert_eq!(
            time_to_nanos(&Time {
                sec,
                nsec,
                ..Default::default()
            }),
            None
        );
    }
}

#[test]
fn all_descriptors_and_dynamic_messages() {
    assert_eq!(files().len(), 216);
    for file in files() {
        for message in file.messages() {
            let found = descriptor(message.full_name()).unwrap();
            assert_eq!(found, message);
            assert!(found.parse_from_bytes(&[]).is_ok());
        }
    }
    assert!(descriptor("gz.msgs.DoesNotExist").is_none());
    assert!(descriptor("Pose").is_none());
    assert!(descriptor("gz.msgs.Discovery.Publisher.ServicePublisher").is_some());
    assert_eq!(Clock::descriptor().full_name(), "gz.msgs.Clock");
    assert_eq!(IMU::descriptor().full_name(), "gz.msgs.IMU");
}

#[test]
fn values_and_presence() {
    let position = vector3([1.0, -2.0, 3.5]);
    assert_eq!([position.x, position.y, position.z], [1.0, -2.0, 3.5]);
    let orientation = quaternion([0.1, 0.2, 0.3, 0.4]);
    assert_eq!(
        [orientation.x, orientation.y, orientation.z, orientation.w],
        [0.1, 0.2, 0.3, 0.4]
    );
    let pose = Pose {
        position: Some(position).into(),
        orientation: Some(orientation).into(),
        ..Default::default()
    };
    assert_eq!(
        Pose::parse_from_bytes(&pose.write_to_bytes().unwrap()).unwrap(),
        pose
    );
    assert!(Pose::default().position.is_none());
    let empty_present = Pose {
        position: Some(vector3d::Vector3d::new()).into(),
        ..Default::default()
    };
    assert_eq!(empty_present.write_to_bytes().unwrap(), [34, 0]);
    assert!(!Pose::parse_from_bytes(&[34, 0]).unwrap().position.is_none());
}

#[test]
fn wire_unknowns_and_malformed() {
    let bytes = [8, 7, 0xa0, 6, 123]; // Time.sec and future field 100.
    let time = Time::parse_from_bytes(&bytes).unwrap();
    assert_eq!(time.sec, 7);
    assert_eq!(time.write_to_bytes().unwrap(), bytes);
    let discovery = discovery::Discovery::parse_from_bytes(&[32, 123]).unwrap();
    assert_eq!(discovery.type_.value(), 123);
    assert_eq!(discovery.write_to_bytes().unwrap(), [32, 123]);
    for bytes in [&[8, 128][..], &[0][..], &[34, 255][..]] {
        assert!(Pose::parse_from_bytes(bytes).is_err());
    }
}

#[test]
fn json_and_text_reflection() {
    let desc = descriptor("gz.msgs.Time").unwrap();
    let value = protobuf_json_mapping::parse_dyn_from_str(
        &desc,
        r#"{"sec":"9223372036854775807","nsec":123}"#,
    )
    .unwrap();
    assert_eq!(
        protobuf_json_mapping::print_to_string(&*value).unwrap(),
        r#"{"sec": "9223372036854775807", "nsec": 123}"#
    );
    assert!(protobuf_json_mapping::parse_dyn_from_str(&desc, r#"{"unknown":1}"#).is_err());
    let text = protobuf::text_format::print_to_string(&*value);
    assert!(text.contains("9223372036854775807"));
}

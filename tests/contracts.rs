use gz_msgs::{prost::Message, prost_reflect::DynamicMessage, *};
#[test]
fn time_contract() {
    for nanos in [i64::MIN, -1_000_000_001, -1, 0, 1, i64::MAX] {
        assert_eq!(time_to_nanos(&time_from_nanos(nanos)), Some(nanos));
    }
    for (sec, nsec) in [
        (9223372036, 854775808),
        (-9223372036, -854775809),
        (i64::MAX, 0),
        (i64::MIN, 0),
    ] {
        assert_eq!(time_to_nanos(&Time { sec, nsec }), None);
    }
}
#[test]
fn all_descriptors_and_dynamic_messages() {
    assert_eq!(files().len(), 216);
    for file in files() {
        for message in file.messages() {
            let found = descriptor(message.full_name()).unwrap();
            assert_eq!(found, message);
            assert!(DynamicMessage::decode(found, [].as_slice()).is_ok());
        }
    }
    assert!(descriptor("gz.msgs.DoesNotExist").is_none());
    assert!(descriptor("Pose").is_none());
    assert!(descriptor("gz.msgs.Discovery.Publisher.ServicePublisher").is_some());
    assert_eq!(
        descriptor("gz.msgs.Clock").unwrap().full_name(),
        "gz.msgs.Clock"
    );
    assert_eq!(
        descriptor("gz.msgs.IMU").unwrap().full_name(),
        "gz.msgs.IMU"
    );
}
#[test]
fn values_and_presence() {
    let position = vector3([1., -2., 3.5]);
    assert_eq!([position.x, position.y, position.z], [1., -2., 3.5]);
    let orientation = quaternion([0.1, 0.2, 0.3, 0.4]);
    assert_eq!(
        [orientation.x, orientation.y, orientation.z, orientation.w],
        [0.1, 0.2, 0.3, 0.4]
    );
    let pose = Pose {
        position: Some(position),
        orientation: Some(orientation),
        ..Default::default()
    };
    assert_eq!(Pose::decode(pose.encode_to_vec().as_slice()).unwrap(), pose);
    assert!(Pose::default().position.is_none());
    let present = Pose {
        position: Some(vector3d::Vector3d::default()),
        ..Default::default()
    };
    assert_eq!(present.encode_to_vec(), [34, 0]);
    assert!(Pose::decode([34, 0].as_slice()).unwrap().position.is_some());
}
#[test]
fn wire_unknowns_and_malformed() {
    let bytes = [8, 7, 160, 6, 123];
    let envelope = WireMessage::<Time>::decode(&bytes).unwrap();
    assert_eq!(envelope.value().sec, 7);
    assert_eq!(envelope.bytes(), bytes);
    // Plain values are not a forward-compatible decode/edit/reencode contract.
    assert_eq!(envelope.value().encode_to_vec(), [8, 7]);
    let nested = [34, 5, 8, 7, 160, 6, 123];
    let envelope = WireMessage::<Clock>::decode(&nested).unwrap();
    assert_eq!(envelope.value().sim.as_ref().unwrap().sec, 7);
    assert_eq!(envelope.bytes(), nested);
    let discovery = discovery::Discovery::decode([32, 123].as_slice()).unwrap();
    assert_eq!(discovery.r#type, 123);
    assert_eq!(discovery.encode_to_vec(), [32, 123]);
    for bytes in [&[8, 128][..], &[0][..], &[34, 255][..]] {
        assert!(WireMessage::<Pose>::decode(bytes).is_err());
    }
}
#[test]
fn json_and_text_reflection() {
    let desc = descriptor("gz.msgs.Time").unwrap();
    let value = DynamicMessage::deserialize(
        desc.clone(),
        &mut serde_json::Deserializer::from_str(r#"{"sec":"9223372036854775807","nsec":123}"#),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(&value).unwrap(),
        serde_json::json!({"sec":"9223372036854775807","nsec":123})
    );
    assert!(DynamicMessage::deserialize(
        desc,
        &mut serde_json::Deserializer::from_str(r#"{"unknown":1}"#)
    )
    .is_err());
    assert!(value.to_text_format().contains("9223372036854775807"));
}

#[test]
fn explicit_dynamic_edits_preserve_unmodified_unknown_fields() {
    let original = [8, 7, 160, 6, 123];
    let mut value =
        DynamicMessage::decode(descriptor("gz.msgs.Time").unwrap(), original.as_slice()).unwrap();
    assert_eq!(value.encode_to_vec(), original);
    value.set_field_by_name("sec", prost_reflect::Value::I64(9));
    assert_eq!(value.encode_to_vec(), [8, 9, 160, 6, 123]);
    let nested = [34, 5, 8, 7, 160, 6, 123];
    let value =
        DynamicMessage::decode(descriptor("gz.msgs.Clock").unwrap(), nested.as_slice()).unwrap();
    assert_eq!(value.encode_to_vec(), nested);
}

#[test]
fn canonical_message_declarations_use_original_source_spans() {
    for file in files() {
        for message in file.messages() {
            let source = message_definition(message.full_name()).unwrap();
            assert!(source.starts_with(&format!("message {} {{", message.name())));
            assert!(source.ends_with('}'));
        }
    }
    let nested = message_definition("gz.msgs.Discovery.Publisher.ServicePublisher").unwrap();
    assert!(nested.contains("string request_type"));
    assert!(message_definition("not.a.Message").is_none());
}

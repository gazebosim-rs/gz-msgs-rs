use gz_msgs::{
    descriptor,
    prost::Message,
    prost_reflect::{DescriptorPool, DynamicMessage, Kind, Value},
    prost_types::*,
};

#[test]
fn generated_and_reflected_float_defaults_preserve_bits() {
    for value in [0.0_f64, -0.0, 1.25, f64::NAN] {
        let message = gz_msgs::double::Double {
            data: value,
            ..Default::default()
        };
        let encoded = message.encode_to_vec();
        assert_eq!(message.encoded_len(), encoded.len());
        let decoded = gz_msgs::double::Double::decode(encoded.as_slice()).unwrap();
        assert_eq!(decoded.data.to_bits(), value.to_bits());
        check_reflection("gz.msgs.Double", &encoded, value.to_bits() == 0);
    }
    for value in [0.0_f32, -0.0, 1.25, f32::NAN] {
        let message = gz_msgs::float::Float {
            data: value,
            ..Default::default()
        };
        let encoded = message.encode_to_vec();
        assert_eq!(message.encoded_len(), encoded.len());
        let decoded = gz_msgs::float::Float::decode(encoded.as_slice()).unwrap();
        assert_eq!(decoded.data.to_bits(), value.to_bits());
        check_reflection("gz.msgs.Float", &encoded, value.to_bits() == 0);
    }
}
fn check_reflection(name: &str, encoded: &[u8], empty: bool) {
    let desc = descriptor(name).unwrap();
    let message = DynamicMessage::decode(desc.clone(), encoded).unwrap();
    assert_eq!(message.encode_to_vec(), encoded);
    assert_eq!(encoded.is_empty(), empty);
    let text = message.to_text_format();
    assert_eq!(
        DynamicMessage::parse_text_format(desc.clone(), &text)
            .unwrap()
            .encode_to_vec(),
        encoded
    );
    let json = gz_msgs::serde_json::to_string(&message).unwrap();
    assert_eq!(
        DynamicMessage::deserialize(
            desc,
            &mut gz_msgs::serde_json::Deserializer::from_str(&json)
        )
        .unwrap()
        .encode_to_vec(),
        encoded
    );
}

#[test]
fn owned_default_predicate_field_kind_and_extension_paths() {
    let extensions = DescriptorPool::from_file_descriptor_set(FileDescriptorSet {
        file: vec![FileDescriptorProto {
            name: Some("runtime-test.proto".into()),
            package: Some("probe".into()),
            syntax: Some("proto2".into()),
            message_type: vec![DescriptorProto {
                name: Some("Defaults".into()),
                extension_range: vec![descriptor_proto::ExtensionRange {
                    start: Some(100),
                    end: Some(536870912),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            extension: [
                field_descriptor_proto::Type::Float,
                field_descriptor_proto::Type::Double,
                field_descriptor_proto::Type::Int32,
            ]
            .into_iter()
            .enumerate()
            .map(|(index, kind)| FieldDescriptorProto {
                name: Some(format!("extension{index}")),
                number: Some(100 + index as i32),
                extendee: Some(".probe.Defaults".into()),
                r#type: Some(kind as i32),
                label: Some(field_descriptor_proto::Label::Optional as i32),
                default_value: Some("0".into()),
                ..Default::default()
            })
            .collect(),
            ..Default::default()
        }],
    })
    .unwrap();
    for (kind, name, values, extension) in [
        (
            Kind::Float,
            "gz.msgs.Float",
            vec![
                (Value::F32(0.), true),
                (Value::F32(-0.), false),
                (Value::F32(1.), false),
                (Value::F32(f32::NAN), false),
            ],
            "probe.extension0",
        ),
        (
            Kind::Double,
            "gz.msgs.Double",
            vec![
                (Value::F64(0.), true),
                (Value::F64(-0.), false),
                (Value::F64(1.), false),
                (Value::F64(f64::NAN), false),
            ],
            "probe.extension1",
        ),
        (
            Kind::Int32,
            "gz.msgs.Int32",
            vec![(Value::I32(0), true), (Value::I32(1), false)],
            "probe.extension2",
        ),
    ] {
        let field = descriptor(name).unwrap().get_field_by_name("data").unwrap();
        let extension = extensions.get_extension_by_name(extension).unwrap();
        for (value, expected) in values {
            assert_eq!(value.is_default(&kind), expected);
            assert_eq!(value.is_default_for_field(&field), expected);
            assert_eq!(value.is_default_for_extension(&extension), expected);
        }
    }
}

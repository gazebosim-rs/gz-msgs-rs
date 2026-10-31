// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use gz_msgs::{
    prost::{bytes::Buf, encoding, Message},
    prost_reflect::{DescriptorPool, DynamicMessage, MessageDescriptor, Value},
    prost_types::{
        field_descriptor_proto::{Label, Type},
        DescriptorProto, EnumDescriptorProto, EnumValueDescriptorProto, FieldDescriptorProto,
        FieldOptions, FileDescriptorProto, FileDescriptorSet,
    },
};

fn descriptor(kind: Type) -> MessageDescriptor {
    DescriptorPool::from_file_descriptor_set(FileDescriptorSet {
        file: vec![FileDescriptorProto {
            name: Some("packed.proto".into()),
            syntax: Some("proto2".into()),
            message_type: vec![DescriptorProto {
                name: Some("Packed".into()),
                field: vec![FieldDescriptorProto {
                    name: Some("data".into()),
                    number: Some(1),
                    label: Some(Label::Repeated as i32),
                    r#type: Some(kind as i32),
                    type_name: (kind == Type::Enum).then(|| ".E".into()),
                    options: Some(FieldOptions {
                        packed: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            enum_type: vec![EnumDescriptorProto {
                name: Some("E".into()),
                value: vec![EnumValueDescriptorProto {
                    name: Some("ZERO".into()),
                    number: Some(0),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }],
    })
    .unwrap()
    .get_message_by_name("Packed")
    .unwrap()
}

macro_rules! packed_control {
    ($test:ident, $kind:ident, $module:ident, $value:ident, $values:expr) => {
        #[test]
        fn $test() {
            let descriptor = descriptor(Type::$kind);
            let values = $values;
            let mut packed = Vec::new();
            encoding::$module::encode_packed(1, &values, &mut packed);
            let mut unpacked = Vec::new();
            encoding::$module::encode_repeated(1, &values, &mut unpacked);
            let mut mixed = packed.clone();
            mixed.extend(&unpacked);
            let mut truncated = packed.clone();
            truncated.pop();
            // Exercise the exact scalar helper and merge-loop error/partial-state
            // contract across contiguous and every two-chunk input boundary.
            for wire in [
                vec![10, 0],
                packed,
                unpacked,
                mixed,
                truncated,
                vec![10, 1, 255],
                vec![10, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0],
                vec![10, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 1],
            ] {
                let mut contiguous = DynamicMessage::new(descriptor.clone());
                contiguous.set_field_by_name("data", Value::List(vec![Value::$value(values[0])]));
                let contiguous_result =
                    contiguous.merge(wire.as_slice()).map_err(|e| e.to_string());
                for split in 0..=wire.len() {
                    let mut expected = vec![values[0]];
                    let mut source = wire[..split].chain(&wire[split..]);
                    let expected_result: Result<(), gz_msgs::prost::DecodeError> = (|| {
                        while source.has_remaining() {
                            let (number, wire_type) = encoding::decode_key(&mut source)?;
                            assert_eq!(number, 1);
                            encoding::$module::merge_repeated(
                                wire_type,
                                &mut expected,
                                &mut source,
                                Default::default(),
                            )?;
                        }
                        Ok(())
                    })();
                    let mut message = DynamicMessage::new(descriptor.clone());
                    message.set_field_by_name("data", Value::List(vec![Value::$value(values[0])]));
                    let mut input = wire[..split].chain(&wire[split..]);
                    let actual_result = message.merge(&mut input).map_err(|e| e.to_string());
                    assert_eq!(
                        actual_result,
                        expected_result.map_err(|e| e.to_string()),
                        "wire {wire:?}, split {split}"
                    );
                    let mut expected_wire = Vec::new();
                    encoding::$module::encode_packed(1, &expected, &mut expected_wire);
                    if split == 0 {
                        assert_eq!(contiguous_result, actual_result);
                        assert_eq!(contiguous.encode_to_vec(), expected_wire);
                    }
                    assert_eq!(message.encode_to_vec(), expected_wire);
                    assert_eq!(
                        input.copy_to_bytes(input.remaining()),
                        source.copy_to_bytes(source.remaining())
                    );
                    assert_eq!(message.clone().encode_to_vec(), expected_wire);
                }
            }
        }
    };
}

packed_control!(
    double,
    Double,
    double,
    F64,
    vec![
        -0.0,
        f64::from_bits(0x7ff0000000000001),
        f64::INFINITY,
        f64::MIN_POSITIVE,
        1.25
    ]
);
packed_control!(
    float,
    Float,
    float,
    F32,
    vec![
        -0.0,
        f32::from_bits(0x7f800001),
        f32::NEG_INFINITY,
        f32::MIN_POSITIVE,
        1.25
    ]
);
packed_control!(
    int32,
    Int32,
    int32,
    I32,
    vec![i32::MIN, 0, 127, 128, i32::MAX]
);
packed_control!(
    int64,
    Int64,
    int64,
    I64,
    vec![i64::MIN, 0, 127, 128, i64::MAX]
);
packed_control!(uint32, Uint32, uint32, U32, vec![0, 127, 128, u32::MAX]);
packed_control!(uint64, Uint64, uint64, U64, vec![0, 127, 128, u64::MAX]);
packed_control!(
    sint32,
    Sint32,
    sint32,
    I32,
    vec![i32::MIN, 0, -1, 1, i32::MAX]
);
packed_control!(
    sint64,
    Sint64,
    sint64,
    I64,
    vec![i64::MIN, 0, -1, 1, i64::MAX]
);
packed_control!(fixed32, Fixed32, fixed32, U32, vec![0, 1, u32::MAX]);
packed_control!(fixed64, Fixed64, fixed64, U64, vec![0, 1, u64::MAX]);
packed_control!(
    sfixed32,
    Sfixed32,
    sfixed32,
    I32,
    vec![i32::MIN, 0, i32::MAX]
);
packed_control!(
    sfixed64,
    Sfixed64,
    sfixed64,
    I64,
    vec![i64::MIN, 0, i64::MAX]
);
packed_control!(boolean, Bool, bool, Bool, vec![false, true]);
packed_control!(enumeration, Enum, int32, EnumNumber, vec![0, -1, i32::MAX]);

#[test]
fn laser_scan_wire_json_unknown_fields_defaults_and_clone() {
    use gz_msgs::{msgs::LaserScan, serde_json};
    let descriptor = gz_msgs::descriptor("gz.msgs.LaserScan").unwrap();
    let scan = LaserScan {
        frame: "robot/lidar".into(),
        world_pose: Some(gz_msgs::pose([0.125, -0.25, -0.0], [-0.0, 0.0, 0.5, 1.0])),
        count: 1080,
        vertical_angle_min: -0.0,
        ranges: (0..1080).map(|i| 1.0 + (i % 128) as f64 * 0.125).collect(),
        intensities: (0..1080).map(|i| (i % 256) as f64).collect(),
        ..Default::default()
    };
    let wire = scan.encode_to_vec();
    let reflected = DynamicMessage::decode(descriptor.clone(), wire.as_slice()).unwrap();
    assert_eq!(reflected.encode_to_vec(), wire);
    let json = serde_json::to_string(&reflected).unwrap();
    let from_json = DynamicMessage::deserialize(
        descriptor.clone(),
        &mut serde_json::Deserializer::from_str(&json),
    )
    .unwrap();
    assert_eq!(from_json.encode_to_vec(), wire);

    let mut unknown_wire = wire.clone();
    encoding::uint64::encode(100, &u64::MAX, &mut unknown_wire);
    let original = DynamicMessage::decode(descriptor.clone(), unknown_wire.as_slice()).unwrap();
    assert_eq!(original.encode_to_vec(), unknown_wire);
    let mut changed = original.clone();
    changed
        .get_field_by_name_mut("ranges")
        .unwrap()
        .as_list_mut()
        .unwrap()[0] = Value::F64(-0.0);
    let mut expected = scan.clone();
    expected.ranges[0] = -0.0;
    let mut expected_wire = expected.encode_to_vec();
    encoding::uint64::encode(100, &u64::MAX, &mut expected_wire);
    assert_eq!(changed.encode_to_vec(), expected_wire);
    assert_eq!(original.encode_to_vec(), unknown_wire);
    changed.clear();
    assert_eq!(changed.encode_to_vec(), Vec::<u8>::new());
    assert_eq!(changed, DynamicMessage::new(descriptor.clone()));
    assert_eq!(
        DynamicMessage::decode(descriptor, [].as_slice())
            .unwrap()
            .encode_to_vec(),
        LaserScan::default().encode_to_vec()
    );
}

#[test]
fn extension_and_map_values_keep_their_resolved_kind() {
    use gz_msgs::prost_types::descriptor_proto::ExtensionRange;
    let original = descriptor(Type::Double);
    let mut set =
        FileDescriptorSet::decode(original.parent_pool().encode_to_vec().as_slice()).unwrap();
    let message = &mut set.file[0].message_type[0];
    let mut extension = message.field.remove(0);
    extension.name = Some("extra".into());
    extension.number = Some(100);
    extension.extendee = Some(".Packed".into());
    message.extension_range.push(ExtensionRange {
        start: Some(100),
        end: Some(101),
        ..Default::default()
    });
    set.file[0].extension.push(extension);
    let pool = DescriptorPool::from_file_descriptor_set(set).unwrap();
    let mut wire = Vec::new();
    encoding::double::encode_packed(100, &[-0.0, f64::INFINITY, 1.25], &mut wire);
    let message =
        DynamicMessage::decode(pool.get_message_by_name("Packed").unwrap(), wire.as_slice())
            .unwrap();
    assert_eq!(message.encode_to_vec(), wire);
    let extension = pool.get_extension_by_name("extra").unwrap();
    assert_eq!(
        message
            .get_extension(&extension)
            .as_list()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap().to_bits())
            .collect::<Vec<_>>(),
        [-0.0, f64::INFINITY, 1.25].map(f64::to_bits)
    );
    let mut unpacked = Vec::new();
    encoding::double::encode_repeated(100, &[-0.0, f64::INFINITY, 1.25], &mut unpacked);
    for wire in [wire, unpacked, vec![0xa2, 6, 1, 255]] {
        let descriptor = pool.get_message_by_name("Packed").unwrap();
        let mut contiguous = DynamicMessage::new(descriptor.clone());
        let expected_result = contiguous.merge(wire.as_slice()).map_err(|e| e.to_string());
        let expected_wire = contiguous.encode_to_vec();
        for split in 0..=wire.len() {
            let mut message = DynamicMessage::new(descriptor.clone());
            let mut input = wire[..split].chain(&wire[split..]);
            assert_eq!(
                message.merge(&mut input).map_err(|e| e.to_string()),
                expected_result
            );
            assert_eq!(message.encode_to_vec(), expected_wire);
        }
    }
    let mut metadata = gz_msgs::msgs::FuelMetadata::default();
    metadata.annotations.insert("key".into(), "value".into());
    let wire = metadata.encode_to_vec();
    let reflected = DynamicMessage::decode(
        gz_msgs::descriptor("gz.msgs.FuelMetadata").unwrap(),
        wire.as_slice(),
    )
    .unwrap();
    assert_eq!(reflected.encode_to_vec(), wire);
}

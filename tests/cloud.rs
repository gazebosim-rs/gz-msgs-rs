// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use gz_msgs::{
    cloud::*,
    msgs::{
        header,
        point_cloud_packed::{field::DataType, Field},
        Header, PointCloudPacked, Time,
    },
    prost::Message,
};
fn initial() -> PointCloudPacked {
    PointCloudPacked {
        width: 3,
        height: 2,
        row_step: 333,
        is_bigendian: true,
        is_dense: true,
        data: b"kept".to_vec(),
        header: Some(Header {
            stamp: Some(Time { sec: 42, nsec: 17 }),
            data: vec![header::Map {
                key: "old".into(),
                value: vec!["oldValue".into()],
            }],
        }),
        field: vec![Field {
            name: "oldField".into(),
            ..Default::default()
        }],
        ..Default::default()
    }
}
#[test]
fn native_layout_and_size_corpus() {
    let fixture: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/cloud-reference.json")).unwrap();
    assert_eq!(
        fixture["word_bytes"].as_u64().unwrap(),
        std::mem::size_of::<usize>() as u64
    );
    for row in fixture["layouts"].as_array().unwrap() {
        let fields: Vec<_> = row["fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| (f[0].as_str().unwrap(), f[1].as_i64().unwrap() as i32))
            .collect();
        let mut cloud = initial();
        init_point_cloud_packed(
            &mut cloud,
            row["frame"].as_str().unwrap(),
            row["aligned"].as_bool().unwrap(),
            &fields,
        );
        let hex = cloud
            .encode_to_vec()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert_eq!(hex, row["wire_hex"].as_str().unwrap());
    }
    for row in fixture["sizes"].as_array().unwrap() {
        assert_eq!(
            size_of_point_field(row["datatype"].as_i64().unwrap() as i32),
            row["size"].as_i64().unwrap() as i32
        );
    }
}
#[test]
fn original_size_and_initializer_assertions() {
    for (datatype, size) in [
        (DataType::Int8, 1),
        (DataType::Uint8, 1),
        (DataType::Int16, 2),
        (DataType::Uint16, 2),
        (DataType::Int32, 4),
        (DataType::Uint32, 4),
        (DataType::Float32, 4),
        (DataType::Float64, 8),
    ] {
        assert_eq!(size_of_point_field(datatype as i32), size);
    }
    let mut cloud = PointCloudPacked::default();
    let xyzrgb = [
        ("xyz", DataType::Float32 as i32),
        ("rgb", DataType::Float32 as i32),
    ];
    init_point_cloud_packed(&mut cloud, "my_frame", true, &xyzrgb);
    assert_eq!(cloud.field.len(), 4);
    assert_eq!(
        cloud.point_step,
        if std::mem::size_of::<usize>() == 4 {
            16
        } else {
            24
        }
    );
    for (field, (name, offset)) in cloud.field.iter().zip([
        ("x", 0),
        ("y", 4),
        ("z", 8),
        (
            "rgb",
            if std::mem::size_of::<usize>() == 4 {
                12
            } else {
                16
            },
        ),
    ]) {
        assert_eq!(field.name, name);
        assert_eq!(field.offset, offset);
        assert_eq!(field.datatype, DataType::Float32 as i32);
        assert_eq!(field.count, 1);
    }
    init_point_cloud_packed(&mut cloud, "my_frame", false, &xyzrgb);
    assert_eq!(cloud.field.len(), 4);
    assert_eq!(cloud.point_step, 16);
    for (field, (name, offset)) in
        cloud
            .field
            .iter()
            .zip([("x", 0), ("y", 4), ("z", 8), ("rgb", 12)])
    {
        assert_eq!(field.name, name);
        assert_eq!(field.offset, offset);
        assert_eq!(field.datatype, DataType::Float32 as i32);
        assert_eq!(field.count, 1);
    }
    let header = cloud.header.as_ref().unwrap();
    assert_eq!(header.data[0].value[0], "my_frame");
    assert_eq!(header.data[0].key, "frame_id");
    let types = [
        ("x", DataType::Int8 as i32),
        ("y", DataType::Uint8 as i32),
        ("z", DataType::Int16 as i32),
        ("r", DataType::Uint16 as i32),
        ("s", DataType::Int32 as i32),
        ("t", DataType::Uint32 as i32),
        ("u", DataType::Float64 as i32),
    ];
    init_point_cloud_packed(&mut cloud, "my_new_frame", false, &types);
    assert_eq!(cloud.header.as_ref().unwrap().data[0].key, "frame_id");
    assert_eq!(
        cloud.header.as_ref().unwrap().data[0].value[0],
        "my_new_frame"
    );
    assert_eq!(
        cloud.field.iter().map(|f| f.offset).collect::<Vec<_>>(),
        [0, 1, 2, 4, 6, 10, 14]
    );
    assert_eq!(cloud.point_step, 22);
}
#[test]
fn native_scalar_and_color_writes() {
    let fixture: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/cloud-scalar-reference.json"))
            .unwrap();
    assert_eq!(
        fixture["byte_order"],
        if cfg!(target_endian = "little") {
            "little"
        } else {
            "big"
        }
    );
    for row in fixture["rows"].as_array().unwrap() {
        let mut cloud = PointCloudPacked::default();
        let kind = row["kind"].as_i64().unwrap() as i32;
        if row["mode"] == "scalar" {
            init_point_cloud_packed(&mut cloud, "scalar", true, &[("value", kind)]);
            cloud.data.resize(5 * cloud.point_step as usize, 0);
            macro_rules! populate {
                ($ty:ty) => {
                    for i in 0..5 {
                        let value = (i as i32 * 3 - 6) as $ty;
                        assert_eq!(write_native(&mut cloud, "value", i, 0, value), Some(()));
                        assert_eq!(read_native::<$ty>(&cloud, "value", i, 0), Some(value));
                    }
                };
            }
            match kind {
                0 => populate!(i8),
                1 => populate!(u8),
                2 => populate!(i16),
                3 => populate!(u16),
                4 => populate!(i32),
                5 => populate!(u32),
                6 => populate!(f32),
                7 => populate!(f64),
                _ => unreachable!(),
            }
        } else {
            init_point_cloud_packed(&mut cloud, "color", true, &[("xyz", 6), ("rgba", 6)]);
            cloud.is_bigendian = kind != 0;
            cloud.data.resize(5 * cloud.point_step as usize, 0);
            for i in 0..5 {
                for (field, factor) in [("x", 2.0), ("y", 4.0), ("z", 6.0)] {
                    let value = i as f32 * factor;
                    assert_eq!(write_native(&mut cloud, field, i, 0, value), Some(()));
                    assert_eq!(read_native::<f32>(&cloud, field, i, 0), Some(value));
                }
                for (field, factor) in [("r", 10), ("g", 20), ("b", 30), ("a", 40)] {
                    let value = i as u8 * factor;
                    assert_eq!(write_native(&mut cloud, field, i, 0, value), Some(()));
                    assert_eq!(read_native::<u8>(&cloud, field, i, 0), Some(value));
                }
            }
        }
        let hex = cloud
            .encode_to_vec()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert_eq!(hex, row["wire_hex"].as_str().unwrap());
    }
}
#[test]
fn safe_native_byte_addressing() {
    let mut cloud = PointCloudPacked::default();
    for name in ["x", "xyz", "r"] {
        assert_eq!(read_native::<i8>(&cloud, name, 0, 0), None);
    }
    init_point_cloud_packed(&mut cloud, "", false, &[("prefix", 0), ("value", 7)]);
    cloud.data.resize(5 * cloud.point_step as usize, 0);
    // Native iterators permit an unaligned FLOAT64 field to be accessed as f32.
    for i in 0..5 {
        assert_eq!(
            write_native(&mut cloud, "value", i, 0, i as f32 + 6.0),
            Some(())
        );
    }
    for i in 0..5 {
        assert_eq!(
            read_native::<f32>(&cloud, "value", i, 0),
            Some(i as f32 + 6.0)
        );
    }
    assert_eq!(write_native(&mut cloud, "value", 0, 1, 42.0_f32), Some(()));
    assert_eq!(read_native::<f32>(&cloud, "value", 0, 1), Some(42.0));
    // Direct names take precedence over aliases and duplicate fields use the first.
    cloud.field.push(Field {
        name: "r".into(),
        offset: 1,
        ..Default::default()
    });
    cloud.field.push(Field {
        name: "r".into(),
        offset: 3,
        ..Default::default()
    });
    assert_eq!(write_native(&mut cloud, "r", 0, 0, 0x5a_u8), Some(()));
    assert_eq!(cloud.data[1], 0x5a);
    cloud.field.push(Field {
        name: "rgb".into(),
        offset: 2,
        ..Default::default()
    });
    cloud.field.push(Field {
        name: "rgba".into(),
        offset: 3,
        ..Default::default()
    });
    assert_eq!(write_native(&mut cloud, "b", 0, 0, 0x7f_u8), Some(()));
    assert_eq!(cloud.data[2], 0x7f);
    let before = cloud.data.clone();
    for (name, point, element) in [
        ("missing", 0, 0),
        ("value", 5, 0),
        ("value", 0, usize::MAX),
        ("value", usize::MAX, 0),
        ("value", 4, 100),
    ] {
        assert_eq!(read_native::<f64>(&cloud, name, point, element), None);
        assert_eq!(
            write_native(&mut cloud, name, point, element, 1.0_f64),
            None
        );
        assert_eq!(cloud.data, before);
    }
    assert_eq!(read_native::<()>(&cloud, "value", 0, 0), None);
    cloud.point_step = 0;
    assert_eq!(read_native::<u8>(&cloud, "r", 0, 0), None);
    cloud.point_step = 100;
    assert_eq!(read_native::<u8>(&cloud, "r", 0, 0), None);
    cloud.point_step = 1;
    cloud.field[0].offset = u32::MAX;
    assert_eq!(read_native::<u8>(&cloud, "prefix", 0, 0), None);
}
#[test]
fn native_float_bits_and_invalid_color_fields() {
    let mut cloud = PointCloudPacked::default();
    init_point_cloud_packed(&mut cloud, "", false, &[("value", 7)]);
    cloud.data.resize(8, 0);
    for bits in [
        0_u64,
        1_u64 << 63,
        0x7ff8_0000_0000_0042,
        0x7ff0_0000_0000_0000,
    ] {
        write_native(&mut cloud, "value", 0, 0, f64::from_bits(bits)).unwrap();
        assert_eq!(
            read_native::<f64>(&cloud, "value", 0, 0).unwrap().to_bits(),
            bits
        );
    }
    for bits in [0_u32, 1_u32 << 31, 0x7fc0_0042, 0x7f80_0000] {
        write_native(&mut cloud, "value", 0, 0, f32::from_bits(bits)).unwrap();
        assert_eq!(
            read_native::<f32>(&cloud, "value", 0, 0).unwrap().to_bits(),
            bits
        );
    }
    assert_eq!(read_native::<u8>(&cloud, "r", 0, 0), None);
    assert_eq!(read_native::<u8>(&cloud, "value", 0, usize::MAX), None);
    cloud.data.truncate(3);
    cloud.point_step = 1;
    assert_eq!(read_native::<u32>(&cloud, "value", 0, 0), None);
    // Contiguous element indexing deliberately crosses point boundaries.
    cloud.data = vec![1, 2, 3];
    assert_eq!(read_native::<u8>(&cloud, "value", 0, 2), Some(3));
}

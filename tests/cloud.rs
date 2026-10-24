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

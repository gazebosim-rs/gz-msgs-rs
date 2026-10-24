// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2022 Open Source Robotics Foundation
// Original PointCloudPackedUtils_TEST.cc assertions, using safe indexed accesses.
use gz_msgs::{
    cloud::*,
    msgs::{point_cloud_packed::field::DataType, PointCloudPacked},
};

#[test]
fn original_bad_fields() {
    let cloud = PointCloudPacked::default();
    for name in ["x", "xyz", "r"] {
        assert_eq!(read_native::<i8>(&cloud, name, 0, 0), None);
    }
}

fn populated(fields: &[(&str, i32)], aligned: bool, frame: &str) -> PointCloudPacked {
    let mut cloud = PointCloudPacked::default();
    init_point_cloud_packed(&mut cloud, frame, aligned, fields);
    cloud.data.resize(5 * cloud.point_step as usize, 0);
    cloud
}
fn present<T: bytemuck::Pod>(cloud: &PointCloudPacked, name: &str) {
    assert!(read_native::<T>(cloud, name, 0, 0).is_some());
}
fn ended<T: bytemuck::Pod>(cloud: &PointCloudPacked, name: &str, index: usize) {
    assert!(read_native::<T>(cloud, name, index, 0).is_none());
}
#[test]
fn original_multiple_fields() {
    let mut cloud = populated(
        &[
            ("x", DataType::Int8 as i32),
            ("y", DataType::Uint8 as i32),
            ("z", DataType::Int16 as i32),
            ("r", DataType::Uint16 as i32),
            ("s", DataType::Int32 as i32),
            ("t", DataType::Uint32 as i32),
            ("u", DataType::Float64 as i32),
        ],
        false,
        "my_new_frame",
    );
    assert_eq!(cloud.field.len(), 7);
    for (index, name, offset, datatype) in [
        (0, "x", 0, DataType::Int8),
        (1, "y", 1, DataType::Uint8),
        (2, "z", 2, DataType::Int16),
    ] {
        let field = &cloud.field[index];
        assert_eq!(field.name, name);
        assert_eq!(field.offset, offset);
        assert_eq!(field.datatype, datatype as i32);
        assert_eq!(field.count, 1);
    }
    macro_rules! starts {
        () => {
            present::<i8>(&cloud, "x");
            present::<u8>(&cloud, "y");
            present::<i16>(&cloud, "z");
            present::<u16>(&cloud, "r");
            present::<i32>(&cloud, "s");
            present::<u32>(&cloud, "t");
            present::<f32>(&cloud, "u");
        };
    }
    macro_rules! ends {
        ($i:expr) => {
            ended::<i8>(&cloud, "x", $i);
            ended::<u8>(&cloud, "y", $i);
            ended::<i16>(&cloud, "z", $i);
            ended::<u16>(&cloud, "r", $i);
            ended::<i32>(&cloud, "s", $i);
            ended::<u32>(&cloud, "t", $i);
            ended::<f32>(&cloud, "u", $i);
        };
    }
    starts!();
    let total = cloud.data.len();
    let mut i = 0;
    while read_native::<i8>(&cloud, "x", i, 0).is_some() {
        write_native(&mut cloud, "x", i, 0, i as i8).unwrap();
        write_native(&mut cloud, "y", i, 0, (i + 1) as u8).unwrap();
        write_native(&mut cloud, "z", i, 0, (i + 2) as i16).unwrap();
        write_native(&mut cloud, "r", i, 0, (i + 3) as u16).unwrap();
        write_native(&mut cloud, "s", i, 0, (i + 4) as i32).unwrap();
        write_native(&mut cloud, "t", i, 0, (i + 5) as u32).unwrap();
        // The original deliberately accesses declared FLOAT64 as float.
        write_native(&mut cloud, "u", i, 0, i as f32 + 6.).unwrap();
        i += 1;
    }
    assert_eq!(i * cloud.point_step as usize, total);
    ends!(i);
    starts!();
    let mut i = 0;
    while read_native::<i8>(&cloud, "x", i, 0).is_some() {
        assert_eq!(read_native::<i8>(&cloud, "x", i, 0), Some(i as i8));
        assert_eq!(read_native::<u8>(&cloud, "y", i, 0), Some((i + 1) as u8));
        assert_eq!(read_native::<i16>(&cloud, "z", i, 0), Some((i + 2) as i16));
        assert_eq!(read_native::<u16>(&cloud, "r", i, 0), Some((i + 3) as u16));
        assert_eq!(read_native::<i32>(&cloud, "s", i, 0), Some((i + 4) as i32));
        assert_eq!(read_native::<u32>(&cloud, "t", i, 0), Some((i + 5) as u32));
        assert_eq!(read_native::<f32>(&cloud, "u", i, 0), Some(i as f32 + 6.));
        i += 1;
    }
    assert_eq!(i * cloud.point_step as usize, total);
    ends!(i);
}

#[test]
fn original_xyz_rgba() {
    let mut cloud = populated(
        &[
            ("xyz", DataType::Float32 as i32),
            ("rgba", DataType::Float32 as i32),
        ],
        true,
        "my_frame",
    );
    macro_rules! starts {
        () => {
            present::<f32>(&cloud, "x");
            present::<f32>(&cloud, "y");
            present::<f32>(&cloud, "z");
            present::<u8>(&cloud, "r");
            present::<u8>(&cloud, "g");
            present::<u8>(&cloud, "b");
            present::<u8>(&cloud, "a");
        };
    }
    macro_rules! ends {
        ($i:expr) => {
            ended::<f32>(&cloud, "x", $i);
            ended::<f32>(&cloud, "y", $i);
            ended::<f32>(&cloud, "z", $i);
            ended::<u8>(&cloud, "r", $i);
            ended::<u8>(&cloud, "g", $i);
            ended::<u8>(&cloud, "b", $i);
            ended::<u8>(&cloud, "a", $i);
        };
    }
    starts!();
    let total = cloud.data.len();
    let mut i = 0;
    while read_native::<f32>(&cloud, "x", i, 0).is_some() {
        write_native(&mut cloud, "x", i, 0, i as f32 * 2.).unwrap();
        write_native(&mut cloud, "y", i, 0, i as f32 * 4.).unwrap();
        write_native(&mut cloud, "z", i, 0, i as f32 * 6.).unwrap();
        write_native(&mut cloud, "r", i, 0, (i * 10) as u8).unwrap();
        write_native(&mut cloud, "g", i, 0, (i * 20) as u8).unwrap();
        write_native(&mut cloud, "b", i, 0, (i * 30) as u8).unwrap();
        write_native(&mut cloud, "a", i, 0, (i * 40) as u8).unwrap();
        i += 1;
    }
    assert_eq!(i * cloud.point_step as usize, total);
    ends!(i);
    let mut i = 0;
    while read_native::<f32>(&cloud, "x", i, 0).is_some() {
        assert_eq!(read_native::<f32>(&cloud, "x", i, 0), Some(i as f32 * 2.));
        assert_eq!(read_native::<f32>(&cloud, "y", i, 0), Some(i as f32 * 4.));
        assert_eq!(read_native::<f32>(&cloud, "z", i, 0), Some(i as f32 * 6.));
        assert_eq!(read_native::<u8>(&cloud, "r", i, 0), Some((i * 10) as u8));
        assert_eq!(read_native::<u8>(&cloud, "g", i, 0), Some((i * 20) as u8));
        assert_eq!(read_native::<u8>(&cloud, "b", i, 0), Some((i * 30) as u8));
        assert_eq!(read_native::<u8>(&cloud, "a", i, 0), Some((i * 40) as u8));
        i += 1;
    }
    assert_eq!(i * cloud.point_step as usize, total);
    ends!(i);
}

#[test]
fn original_operator_values_with_indexed_api() {
    let mut cloud = populated(&[("a", DataType::Float32 as i32)], false, "my_new_frame");
    assert_eq!(cloud.field[0].name, "a");
    assert_eq!(cloud.field[0].offset, 0);
    assert_eq!(cloud.point_step, 4);
    present::<f32>(&cloud, "a");
    let total = cloud.data.len();
    let mut i = 0;
    while read_native::<f32>(&cloud, "a", i, 0).is_some() {
        let value = i as f32 * 2.;
        write_native(&mut cloud, "a", i, 0, value).unwrap();
        assert_eq!(read_native::<f32>(&cloud, "a", i, 0), Some(value));
        i += 1;
    }
    assert_eq!(i * cloud.point_step as usize, total);
    ended::<f32>(&cloud, "a", i);
    for (element, value) in [0., 2., 4., 6., 8.].into_iter().enumerate() {
        assert_eq!(read_native::<f32>(&cloud, "a", 0, element), Some(value));
    }
    // Copied and advanced point indices replace C++ iterator objects. Object
    // identity/operator-overload assertions remain outside this API's claim.
    let original = 0;
    let copy = original;
    for (element, value) in [0., 2., 4., 6., 8.].into_iter().enumerate() {
        assert_eq!(read_native::<f32>(&cloud, "a", copy, element), Some(value));
    }
    let mut advanced = original + 2;
    for (element, value) in [4., 6., 8.].into_iter().enumerate() {
        assert_eq!(
            read_native::<f32>(&cloud, "a", advanced, element),
            Some(value)
        );
    }
    advanced += 1;
    for (element, value) in [6., 8.].into_iter().enumerate() {
        assert_eq!(
            read_native::<f32>(&cloud, "a", advanced, element),
            Some(value)
        );
    }
}

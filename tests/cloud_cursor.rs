// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2022 Open Source Robotics Foundation
use gz_msgs::{
    cloud::*,
    msgs::{point_cloud_packed::field::DataType, PointCloudPacked},
};

#[test]
fn original_operator_identity_copy_and_advancement() {
    let mut cloud = PointCloudPacked::default();
    init_point_cloud_packed(&mut cloud, "", false, &[("a", DataType::Float32 as i32)]);
    assert_eq!(cloud.field.len(), 1);
    assert_eq!(cloud.field[0].name, "a");
    assert_eq!(cloud.field[0].offset, 0);
    assert_eq!(cloud.point_step, 4);
    cloud.data.resize(5 * cloud.point_step as usize, 0);
    let start = NativeFieldCursor::<f32>::new(&cloud, "a");
    assert_ne!(start, start.end());
    let mut bytes = 0;
    for i in 0..5 {
        write_native(&mut cloud, "a", i, 0, i as f32 * 2.).unwrap();
        assert_eq!(
            NativeFieldCursor::<f32>::new(&cloud, "a").get(i),
            Some(i as f32 * 2.)
        );
        bytes += cloud.point_step as usize;
    }
    assert_eq!(bytes, cloud.data.len());
    let a2 = NativeFieldCursor::<f32>::new(&cloud, "a");
    for (i, expected) in [0., 2., 4., 6., 8.].into_iter().enumerate() {
        assert_eq!(a2.get(i), Some(expected));
    }
    let a3 = a2;
    assert_eq!(a3, a2);
    for (i, expected) in [0., 2., 4., 6., 8.].into_iter().enumerate() {
        assert_eq!(a3.get(i), Some(expected));
    }
    let mut a4 = a2 + 2;
    assert_ne!(a4, a2);
    for (i, expected) in [4., 6., 8.].into_iter().enumerate() {
        assert_eq!(a4.get(i), Some(expected));
    }
    a4 += 1;
    assert_eq!(a4.get(0), Some(6.));
    assert_eq!(a4.get(1), Some(8.));
    let mut current = a2;
    let mut count = 0;
    while current != a2.end() {
        assert_eq!(current.next(), Some(count as f32 * 2.));
        count += 1;
    }
    assert_eq!(count * cloud.point_step as usize, cloud.data.len());
    assert_eq!(current.next(), None);
    assert_eq!(a2.collect::<Vec<_>>(), vec![0., 2., 4., 6., 8.]);
    assert_eq!(a2 + usize::MAX, a2.end());
}

#[test]
fn bounded_missing_malformed_and_borrowed_identity() {
    let mut cloud = PointCloudPacked::default();
    let missing = NativeFieldCursor::<u8>::new(&cloud, "x");
    assert_eq!(missing.get(0), None);
    assert_eq!(missing, missing.end());
    init_point_cloud_packed(
        &mut cloud,
        "my_new_frame",
        false,
        &[
            ("x", DataType::Uint8 as i32),
            ("y", DataType::Float64 as i32),
        ],
    );
    cloud.data = vec![0; 19];
    write_native(&mut cloud, "y", 0, 0, 3.5_f64).unwrap();
    let y = NativeFieldCursor::<f64>::new(&cloud, "y");
    assert_eq!(y.get(0), Some(3.5));
    assert_eq!(y.get(usize::MAX), None);
    assert_eq!(y.end().get(0), None);
    assert_ne!(y, NativeFieldCursor::new(&cloud.clone(), "y"));
    let zst_nonzero_stride = NativeFieldCursor::<[u8; 0]>::new(&cloud, "x");
    assert_eq!(zst_nonzero_stride, zst_nonzero_stride.end());
    assert_eq!(zst_nonzero_stride.get(0), None);
    cloud.field[1].offset = cloud.point_step;
    assert_eq!(
        NativeFieldCursor::<u8>::new(&cloud, "x") + 1,
        NativeFieldCursor::<u8>::new(&cloud, "y")
    );
    cloud.field[1].offset = u32::MAX;
    assert_eq!(NativeFieldCursor::<f64>::new(&cloud, "y").next(), None);
    cloud.point_step = 0;
    let mut zero = NativeFieldCursor::<u8>::new(&cloud, "x");
    assert_eq!(zero.next(), None);
    let zst = NativeFieldCursor::<[u8; 0]>::new(&cloud, "x");
    assert_eq!(zst.get(0), None);
}

#[test]
fn cursors_read_native_scalar_and_color_fixtures() {
    use gz_msgs::runtime::Message;
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/cloud-scalar-reference.json")).unwrap();
    for row in fixture["rows"].as_array().unwrap() {
        let hex = row["wire_hex"].as_str().unwrap();
        let bytes: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        let cloud = PointCloudPacked::decode(bytes.as_slice()).unwrap();
        macro_rules! scalar {
            ($t:ty) => {
                assert_eq!(
                    NativeFieldCursor::<$t>::new(&cloud, "value").collect::<Vec<_>>(),
                    [-6, -3, 0, 3, 6].map(|n| n as $t)
                )
            };
        }
        if row["mode"] == "scalar" {
            match row["kind"].as_u64().unwrap() {
                0 => scalar!(i8),
                1 => scalar!(u8),
                2 => scalar!(i16),
                3 => scalar!(u16),
                4 => scalar!(i32),
                5 => scalar!(u32),
                6 => scalar!(f32),
                7 => scalar!(f64),
                _ => unreachable!(),
            }
        } else {
            for (name, scale) in [("r", 10), ("g", 20), ("b", 30), ("a", 40)] {
                assert_eq!(
                    NativeFieldCursor::<u8>::new(&cloud, name).collect::<Vec<_>>(),
                    [0, 1, 2, 3, 4].map(|i| i * scale)
                );
            }
        }
    }
}

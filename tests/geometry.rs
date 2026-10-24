// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2016 Open Source Robotics Foundation
// Assertion ports: Utility_TEST.cc plane and CovertMathAxisAlignedBox cases.
use gz_msgs::{
    msgs::{AxisAlignedBox, PlaneGeom},
    *,
};

#[test]
fn original_plane_conversion_and_assignment() {
    let message = plane([0., 0., 1.], [123., 456.], 1.);
    let normal = message.normal.as_ref().unwrap();
    assert_eq!([normal.x, normal.y, normal.z], [0., 0., 1.]);
    let size = message.size.as_ref().unwrap();
    assert_eq!([size.x, size.y], [123., 456.]);
    let (normal, size, offset) = plane_components(&message);
    assert_eq!(normal, [0., 0., 1.]);
    assert_eq!(size, [123., 456.]);
    assert!((offset - 1.).abs() <= 1e-6);
    let mut assigned = PlaneGeom::default();
    set_plane(&mut assigned, [0., 0., 1.], [123., 456.], 1.);
    let normal = assigned.normal.unwrap();
    assert_eq!([normal.x, normal.y, normal.z], [0., 0., 1.]);
    let size = assigned.size.unwrap();
    assert_eq!([size.x, size.y], [123., 456.]);
    assert!((assigned.d - 1.).abs() <= 1e-6);
}

#[test]
fn original_axis_aligned_box_conversion_and_assignment() {
    let message = axis_aligned_box([-1., -2., -3.], [12., 15., 20.]);
    let min = message.min_corner.as_ref().unwrap();
    assert_eq!([min.x, min.y, min.z], [-1., -2., -3.]);
    let max = message.max_corner.as_ref().unwrap();
    assert_eq!([max.x, max.y, max.z], [12., 15., 20.]);
    let (min, max) = axis_aligned_box_components(&message);
    assert_eq!(min, [-1., -2., -3.]);
    assert_eq!(max, [12., 15., 20.]);
    let mut assigned = AxisAlignedBox::default();
    set_axis_aligned_box(&mut assigned, min, max);
    let min = assigned.min_corner.as_ref().unwrap();
    assert_eq!([min.x, min.y, min.z], [-1., -2., -3.]);
    let max = assigned.max_corner.as_ref().unwrap();
    assert_eq!([max.x, max.y, max.z], [12., 15., 20.]);
    set_axis_aligned_box(&mut assigned, [-23.4, 56.2, -123.5], [34.1, 92.1, -723.2]);
    let min = assigned.min_corner.as_ref().unwrap();
    assert_eq!([min.x, min.y, min.z], [-23.4, 56.2, -123.5]);
    let max = assigned.max_corner.as_ref().unwrap();
    assert_eq!([max.x, max.y, max.z], [34.1, 92.1, -723.2]);
}

#[test]
fn absent_components_and_setters_preserve_metadata_and_values() {
    use gz_msgs::msgs::{Header, Time, Vector2d, Vector3d};
    let header = || {
        Some(Header {
            stamp: Some(Time { sec: 17, nsec: -3 }),
            ..Default::default()
        })
    };
    assert_eq!(
        plane_components(&PlaneGeom::default()),
        ([0.; 3], [0.; 2], 0.)
    );
    assert_eq!(
        axis_aligned_box_components(&AxisAlignedBox::default()),
        ([0.; 3], [0.; 3])
    );
    let mut message = PlaneGeom {
        header: header(),
        normal: Some(Vector3d {
            header: header(),
            ..Default::default()
        }),
        size: Some(Vector2d {
            header: header(),
            ..Default::default()
        }),
        ..Default::default()
    };
    set_plane(&mut message, [-0., 3., 4.], [-5., 6.], -7.);
    assert_eq!(message.header, header());
    assert_eq!(message.normal.as_ref().unwrap().header, header());
    assert_eq!(message.size.as_ref().unwrap().header, header());
    assert_eq!(plane_components(&message), ([-0., 3., 4.], [-5., 6.], -7.));
    assert_eq!(message.normal.unwrap().x.to_bits(), (-0f64).to_bits());
    let mut message = AxisAlignedBox {
        header: header(),
        min_corner: Some(Vector3d {
            header: header(),
            ..Default::default()
        }),
        max_corner: Some(Vector3d {
            header: header(),
            ..Default::default()
        }),
    };
    set_axis_aligned_box(&mut message, [4., 5., 6.], [1., 2., -0.]);
    assert_eq!(message.header, header());
    assert_eq!(message.min_corner.as_ref().unwrap().header, header());
    assert_eq!(message.max_corner.as_ref().unwrap().header, header());
    assert_eq!(
        axis_aligned_box_components(&message),
        ([4., 5., 6.], [1., 2., -0.])
    );
    assert_eq!(message.max_corner.unwrap().z.to_bits(), (-0f64).to_bits());
}

#[test]
fn native_full_wire_plane_and_box_setters() {
    use gz_msgs::{
        msgs::{Header, Time, Vector2d, Vector3d},
        prost::Message,
    };
    let planes = [
        [0., 0., 1., 123., 456., 1.],
        [-2., 3., 4., -5., 6., -7.],
        [-0., 0., 1., -0., 2., -0.],
        [0.; 6],
    ];
    let boxes = [
        [-1., -2., -3., 12., 15., 20.],
        [-23.4, 56.2, -123.5, 34.1, 92.1, -723.2],
        [0.; 6],
        [-0., 2., 3., 4., -0., 6.],
    ];
    let header = |sec| {
        Some(Header {
            stamp: Some(Time { sec, nsec: 0 }),
            ..Default::default()
        })
    };
    let fixture: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/geometry-reference.json")).unwrap();
    let rows = fixture["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 16);
    for row in rows {
        let index = row["index"].as_u64().unwrap() as usize;
        let seeded = row["seeded"].as_u64().unwrap() == 1;
        let actual = if row["kind"].as_u64().unwrap() == 0 {
            let mut message = PlaneGeom::default();
            if seeded {
                message.header = header(11);
                message.normal = Some(Vector3d {
                    header: header(22),
                    ..Default::default()
                });
                message.size = Some(Vector2d {
                    header: header(33),
                    ..Default::default()
                });
            }
            let [x, y, z, sx, sy, d] = planes[index];
            set_plane(&mut message, [x, y, z], [sx, sy], d);
            let (normal, size, offset) = plane_components(&message);
            assert_eq!(normal.map(f64::to_bits), [x, y, z].map(f64::to_bits));
            assert_eq!(size.map(f64::to_bits), [sx, sy].map(f64::to_bits));
            assert_eq!(offset.to_bits(), d.to_bits());
            message.encode_to_vec()
        } else {
            let mut message = AxisAlignedBox::default();
            if seeded {
                message.header = header(11);
                message.min_corner = Some(Vector3d {
                    header: header(22),
                    ..Default::default()
                });
                message.max_corner = Some(Vector3d {
                    header: header(33),
                    ..Default::default()
                });
            }
            let [x, y, z, mx, my, mz] = boxes[index];
            set_axis_aligned_box(&mut message, [x, y, z], [mx, my, mz]);
            let (min, max) = axis_aligned_box_components(&message);
            assert_eq!(min.map(f64::to_bits), [x, y, z].map(f64::to_bits));
            assert_eq!(max.map(f64::to_bits), [mx, my, mz].map(f64::to_bits));
            message.encode_to_vec()
        };
        let hex: String = actual.iter().map(|byte| format!("{byte:02x}")).collect();
        assert_eq!(hex, row["wire_hex"].as_str().unwrap(), "{row}");
    }
}

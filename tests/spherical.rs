// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2016 Open Source Robotics Foundation
// Original Utility_TEST.cc complete spherical conversion and Set cases.
use gz_msgs::{
    msgs::{Entity, Header, SphericalCoordinates, SphericalCoordinatesType as Coordinate, Time},
    prost::Message,
    *,
};
use std::f64::consts::PI;
fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!("fixtures/spherical-reference.json")).unwrap()
}
fn domain(value: &serde_json::Value) -> SphericalComponents {
    let bits = value["bits"].as_array().unwrap();
    let values: Vec<_> = bits
        .iter()
        .map(|b| f64::from_bits(b.as_u64().unwrap()))
        .collect();
    let surface = match value["surface"].as_i64().unwrap() {
        1 => SphericalSurface::EarthWgs84,
        2 => SphericalSurface::MoonScs,
        10 => SphericalSurface::Custom {
            axis_equatorial: values[4],
            axis_polar: values[5],
        },
        code => SphericalSurface::Unrecognized(code as i32),
    };
    SphericalComponents {
        surface,
        latitude_rad: values[0],
        longitude_rad: values[1],
        elevation: values[2],
        heading_rad: values[3],
    }
}
fn input(index: u64) -> SphericalComponents {
    let fixture = fixture();
    let row = fixture["from_math"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["index"].as_u64() == Some(index))
        .unwrap();
    domain(&row["input"])
}
fn double_eq(actual: f64, expected: f64) {
    // Original EXPECT_DOUBLE_EQ accepts four ULP; original constants are positive.
    assert!(
        actual.to_bits().abs_diff(expected.to_bits()) <= 4,
        "{actual} != {expected}"
    );
}
fn message_values(message: &SphericalCoordinates, model: i32, expected: [f64; 4]) {
    assert_eq!(message.surface_model, model);
    for (actual, expected) in [
        message.latitude_deg,
        message.longitude_deg,
        message.elevation,
        message.heading_deg,
    ]
    .into_iter()
    .zip(expected)
    {
        double_eq(actual, expected);
    }
}
#[test]
fn original_complete_spherical_conversion_assertions() {
    assert_eq!(coordinate_type_from_math(2), Coordinate::Ecef);
    assert_eq!(coordinate_type_from_math(3), Coordinate::Global);
    assert_eq!(coordinate_type_from_math(1), Coordinate::Spherical);
    assert_eq!(coordinate_type_from_math(4), Coordinate::Local);
    assert_eq!(coordinate_type_to_math(Coordinate::Ecef as i32), 2);
    assert_eq!(coordinate_type_to_math(Coordinate::Global as i32), 3);
    assert_eq!(coordinate_type_to_math(Coordinate::Spherical as i32), 1);
    assert_eq!(coordinate_type_to_math(Coordinate::Local as i32), 4);
    for (index, model, surface) in [
        (0, 0, SphericalSurface::EarthWgs84),
        (1, 1, SphericalSurface::MoonScs),
        (
            2,
            2,
            SphericalSurface::Custom {
                axis_equatorial: 12000.,
                axis_polar: 10000.,
            },
        ),
    ] {
        let message = spherical_coordinates(input(index));
        message_values(&message, model, [1.1, 2.2, 3.3, 0.4]);
        if model == 2 {
            double_eq(message.surface_axis_equatorial, 12000.);
            double_eq(message.surface_axis_polar, 10000.);
        }
        let value = spherical_components(&message);
        assert_eq!(value.surface, surface);
        for (actual, expected) in [
            value.latitude_rad * 180. / PI,
            value.longitude_rad * 180. / PI,
            value.elevation,
            value.heading_rad * 180. / PI,
        ]
        .into_iter()
        .zip([1.1, 2.2, 3.3, 0.4])
        {
            double_eq(actual, expected);
        }
        if let SphericalSurface::Custom {
            axis_equatorial,
            axis_polar,
        } = value.surface
        {
            double_eq(axis_equatorial, 12000.);
            double_eq(axis_polar, 10000.);
        }
    }
}
#[test]
fn original_spherical_assignment_assertions() {
    for (index, model, expected) in [
        (0, 0, [1.1, 2.2, 3.3, 0.4]),
        (4, 1, [1.2, 2.3, 3.4, 0.5]),
        (3, 2, [1.9, 2.8, 3.7, 0.6]),
    ] {
        let mut message = SphericalCoordinates::default();
        set_spherical_coordinates(&mut message, input(index));
        message_values(&message, model, expected);
        if model == 2 {
            double_eq(message.surface_axis_equatorial, 12000.);
            double_eq(message.surface_axis_polar, 10000.);
        }
    }
}
#[test]
fn native_spherical_parameters_wire_and_reverse_bits() {
    let fixture = fixture();
    let rows = fixture["from_math"].as_array().unwrap();
    assert_eq!(rows.len(), 16);
    for row in rows {
        let mut message = if row["seeded"].as_u64() == Some(1) {
            SphericalCoordinates {
                header: Some(Header {
                    stamp: Some(Time { sec: 11, nsec: 0 }),
                    ..Default::default()
                }),
                entity: Some(Entity {
                    id: 42,
                    name: "retained".into(),
                    ..Default::default()
                }),
                surface_model: 2,
                surface_axis_equatorial: 91.,
                surface_axis_polar: 92.,
                ..Default::default()
            }
        } else {
            SphericalCoordinates::default()
        };
        set_spherical_coordinates(&mut message, domain(&row["input"]));
        let hex: String = message
            .encode_to_vec()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(hex, row["wire_hex"].as_str().unwrap(), "{row}");
    }
    let rows = fixture["to_math"].as_array().unwrap();
    assert_eq!(rows.len(), 11);
    for row in rows {
        let hex = row["wire_hex"].as_str().unwrap();
        let bytes: Vec<_> = hex
            .as_bytes()
            .chunks_exact(2)
            .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
            .collect();
        let message = SphericalCoordinates::decode(bytes.as_slice()).unwrap();
        let value = spherical_components(&message);
        let expected = domain(&row["output"]);
        assert_eq!(value.surface, expected.surface);
        let actual = [
            value.latitude_rad,
            value.longitude_rad,
            value.elevation,
            value.heading_rad,
        ];
        let expected = [
            expected.latitude_rad,
            expected.longitude_rad,
            expected.elevation,
            expected.heading_rad,
        ];
        assert_eq!(
            actual.map(f64::to_bits),
            expected.map(f64::to_bits),
            "{row}"
        );
        let axes = value.surface.axes().unwrap();
        let expected: Vec<_> = row["output"]["bits"].as_array().unwrap()[4..]
            .iter()
            .map(|b| b.as_u64().unwrap())
            .collect();
        assert_eq!(axes.map(f64::to_bits).as_slice(), expected);
    }
    assert_eq!(
        spherical_components(&SphericalCoordinates::default()),
        SphericalComponents::default()
    );
    assert_eq!(SphericalSurface::Unrecognized(500000).axes(), None);
}

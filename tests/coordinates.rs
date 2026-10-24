// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2016 Open Source Robotics Foundation
// Copyright (C) 2023 Open Source Robotics Foundation
// Original Utility_TEST.cc coordinate type conversion cases.
use gz_msgs::{
    coordinate_type_from_math, coordinate_type_to_math,
    msgs::SphericalCoordinatesType as Coordinate,
};
#[test]
fn original_coordinate_enum_conversions() {
    assert_eq!(coordinate_type_to_math(Coordinate::Ecef as i32), 2);
    assert_eq!(coordinate_type_to_math(Coordinate::Global as i32), 3);
    assert_eq!(coordinate_type_to_math(Coordinate::Spherical as i32), 1);
    assert_eq!(coordinate_type_to_math(Coordinate::Local as i32), 4);
    assert_eq!(coordinate_type_to_math(500000), 4);
    assert_eq!(coordinate_type_from_math(2), Coordinate::Ecef);
    assert_eq!(coordinate_type_from_math(3), Coordinate::Global);
    assert_eq!(coordinate_type_from_math(1), Coordinate::Spherical);
    assert_eq!(coordinate_type_from_math(4), Coordinate::Local);
    assert_eq!(coordinate_type_from_math(500000), Coordinate::Local);
}
#[test]
fn native_coordinate_code_corpus() {
    let fixture: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/coordinate-reference.json")).unwrap();
    let rows = fixture["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 14);
    for row in rows {
        let value = row["input"].as_i64().unwrap() as i32;
        let (output, valid) = match row["direction"].as_str().unwrap() {
            "from_math" => (
                coordinate_type_from_math(value) as i32,
                (1..=4).contains(&value),
            ),
            "to_math" => (coordinate_type_to_math(value), (0..=3).contains(&value)),
            other => panic!("unknown native direction {other}"),
        };
        assert_eq!(i64::from(output), row["output"].as_i64().unwrap());
        // Native diagnostic captured; Rust wording independently source-inspected.
        assert_eq!(
            row["stderr"].as_str().unwrap(),
            if valid {
                ""
            } else {
                "Invalid coordinate type passed\n"
            }
        );
    }
}

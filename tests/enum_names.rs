// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use gz_msgs::enum_names::EnumKind;
#[test]
fn native_enum_spelling_and_fallback_corpus() {
    let fixture: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/enum-reference.json")).unwrap();
    for row in fixture["cases"].as_array().unwrap() {
        let family = match row["family"].as_str().unwrap() {
            "Joint" => EnumKind::Joint,
            "Geometry" => EnumKind::Geometry,
            "Shader" => EnumKind::Shader,
            "PixelFormat" => EnumKind::PixelFormat,
            "Discovery" => EnumKind::Discovery,
            _ => unreachable!(),
        };
        let number = row["number"].as_i64().unwrap() as i32;
        let name = row["name"].as_str().unwrap();
        if row["direction"] == "parse" {
            assert_eq!(family.parse(name), number, "{family:?} {name}");
        } else {
            assert_eq!(family.format(number), name, "{family:?} {number}");
        }
    }
}
#[test]
fn original_enum_roundtrip_and_discovery_bound_assertions() {
    for (kind, values) in [
        (EnumKind::Joint, vec![0, 1, 2, 3, 4, 5, 6, 7, 8]),
        (EnumKind::Geometry, vec![0, 2, 14, 10, 1, 15, 3, 4, 5, 6, 9]),
        (EnumKind::Shader, vec![2, 3, 1, 0]),
        (EnumKind::PixelFormat, (0..=18).collect()),
    ] {
        for value in values {
            assert_eq!(value, kind.parse(&kind.format(value)));
        }
        assert_eq!(kind.parse("bad type"), 0);
        assert_eq!(
            kind.format(100),
            if kind == EnumKind::PixelFormat {
                "UNKNOWN_PIXEL_FORMAT"
            } else {
                "unknown"
            }
        );
    }
    let expected = [
        "UNINITIALIZED",
        "ADVERTISE",
        "SUBSCRIBE",
        "UNADVERTISE",
        "HEARTBEAT",
        "BYE",
        "NEW_CONNECTION",
        "END_CONNECTION",
        "SUBSCRIBERS_REQ",
        "SUBSCRIBERS_REP",
    ];
    for (number, name) in expected.iter().enumerate() {
        assert_eq!(EnumKind::Discovery.format(number as i32), *name);
    }
    let desc = gz_msgs::descriptors()
        .get_enum_by_name("gz.msgs.Discovery.Type")
        .unwrap();
    let values: Vec<_> = desc.values().map(|value| value.number()).collect();
    assert_eq!(values.iter().min(), Some(&0));
    assert_eq!(values.iter().max(), Some(&9));
    assert_eq!(values.len(), 10);
}

#[test]
fn original_image_pixel_format_defaults_and_assignment() {
    use gz_msgs::image::{Image, PixelFormatType};
    let mut image = Image::default();
    assert_eq!(
        image.pixel_format_type,
        PixelFormatType::UnknownPixelFormat as i32
    );
    image.pixel_format_type = PixelFormatType::RgbInt8 as i32;
    assert_eq!(image.pixel_format_type, PixelFormatType::RgbInt8 as i32);
}

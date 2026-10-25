// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use gz_msgs::{prost::Message, CanonicalMessage, CompatibleDecodeError, WireMessage};
use std::{
    error::Error,
    io::Write,
    process::{Command, Stdio},
    sync::Once,
};
fn unhex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn peer(name: &str, wire: &[u8], known: bool) -> std::process::Output {
    static BUILD: Once = Once::new();
    BUILD.call_once(|| {
        assert!(Command::new("python3")
            .arg("scripts/native-wire-peer.py")
            .status()
            .unwrap()
            .success())
    });
    let mut command = Command::new("target/native-wire/peer");
    command.arg(name);
    if known {
        command.arg("known");
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(wire).unwrap();
    child.wait_with_output().unwrap()
}
fn compare<M: CanonicalMessage + PartialEq + std::fmt::Debug>(
    wire: &[u8],
    accepted: bool,
    known: &[u8],
) {
    match WireMessage::<M>::decode_compatible(wire) {
        Ok(decoded) => {
            assert!(accepted);
            assert_eq!(decoded.bytes(), wire);
            assert_eq!(decoded.value(), &M::decode(known).unwrap());
        }
        Err(error) => {
            assert!(!accepted, "{error}");
            assert!(!error.to_string().is_empty());
            match &error {
                CompatibleDecodeError::Protobuf(_) => assert!(error.source().is_some()),
                CompatibleDecodeError::Malformed(_) => assert!(error.source().is_none()),
            }
        }
    }
}
#[test]
fn native_wrong_wire_presence_maps_oneofs_and_framing() {
    let data: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/wire-compatibility.json")).unwrap();
    assert_eq!(data["rows"].as_array().unwrap().len(), 70);
    for row in data["rows"].as_array().unwrap() {
        let name = row["type"].as_str().unwrap();
        let wire = unhex(row["wire_hex"].as_str().unwrap());
        let expected = unhex(row["cpp_known"]["output_hex"].as_str().unwrap());
        let accepted = row["cpp_generated"]["accepted"].as_bool().unwrap();
        for (known, key) in [(false, "cpp_generated"), (true, "cpp_known")] {
            let actual = peer(name, &wire, known);
            assert_eq!(
                actual.status.code(),
                row[key]["exit"].as_i64().map(|v| v as i32),
                "{}",
                row["id"]
            );
            assert_eq!(
                actual.stdout,
                unhex(row[key]["output_hex"].as_str().unwrap()),
                "{}",
                row["id"]
            );
        }
        match name {
            "gz.msgs.Time" => compare::<gz_msgs::Time>(&wire, accepted, &expected),
            "gz.msgs.Pose" => compare::<gz_msgs::Pose>(&wire, accepted, &expected),
            "gz.msgs.Parameter" => compare::<gz_msgs::msgs::Parameter>(&wire, accepted, &expected),
            "gz.msgs.Param" => compare::<gz_msgs::msgs::Param>(&wire, accepted, &expected),
            "gz.msgs.Any" => compare::<gz_msgs::msgs::Any>(&wire, accepted, &expected),
            "gz.msgs.Double_V" => compare::<gz_msgs::msgs::DoubleV>(&wire, accepted, &expected),
            _ => panic!("fixture type"),
        }
    }
}
#[test]
fn strict_path_and_generated_nested_binding_remain_explicit() {
    let wrong = [9, 0, 0, 0, 0, 0, 0, 0, 0];
    assert!(gz_msgs::Time::decode(wrong.as_slice()).is_err());
    assert!(WireMessage::<gz_msgs::Time>::decode(&wrong).is_err());
    assert!(gz_msgs::prost_reflect::DynamicMessage::decode(
        gz_msgs::descriptor("gz.msgs.Time").unwrap(),
        wrong.as_slice()
    )
    .is_err());
    assert_eq!(
        WireMessage::<gz_msgs::Time>::decode_compatible(&wrong)
            .unwrap()
            .value(),
        &gz_msgs::Time::default()
    );
    assert_eq!(
        gz_msgs::msgs::header::Map::PROTOBUF_NAME,
        "gz.msgs.Header.Map"
    );
    assert_eq!(
        gz_msgs::msgs::discovery::publisher::MessagePublisher::PROTOBUF_NAME,
        "gz.msgs.Discovery.Publisher.MessagePublisher"
    );
    for wire in [&[10, 128][..], &[10, 2, 0][..]] {
        let error = WireMessage::<gz_msgs::Pose>::decode_compatible(wire)
            .err()
            .unwrap();
        assert!(!error.to_string().is_empty());
    }
}

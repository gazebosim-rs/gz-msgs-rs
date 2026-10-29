// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use gz_msgs::{
    descriptor_definition, factory::MessageFactory, registry::DescriptorRegistry, serde_json,
};
use std::process::Command;

fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "fixtures/integer-definitions/remaining-594.json"
    ))
    .unwrap()
}

fn unhex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn all_remaining_integer_declarations_match_fresh_native_outputs() {
    let directory = tempfile::tempdir().unwrap();
    let peer = directory.path().join("peer");
    let build = Command::new("c++")
        .args([
            "-std=c++17",
            "-Wall",
            "-Wextra",
            "-Werror",
            "tests/integer_definition_peer.cc",
            "-lprotobuf",
            "-o",
        ])
        .arg(&peer)
        .output()
        .unwrap();
    assert!(build.status.success(), "{build:?}");
    let fixture = fixture();
    let rows = fixture["cases"].as_array().unwrap();
    let original: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/integer-defaults/native.json")).unwrap();
    assert_eq!(rows.len(), 594);
    for row in rows {
        let id = row["input"]["id"].as_u64().unwrap() as usize;
        assert_eq!(row["input"], original["cases"][id]);
        assert_eq!(row["descriptor_hex"], row["input"]["descriptor_hex"]);
        assert_eq!(row["descriptor_sha256"], row["input"]["descriptor_sha256"]);
        let input = directory.path().join("input.desc");
        std::fs::write(&input, unhex(row["descriptor_hex"].as_str().unwrap())).unwrap();
        let output = Command::new(&peer)
            .arg(input)
            .env("LC_ALL", "C")
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(row["status"].as_i64().unwrap() as i32)
        );
        assert_eq!(output.stdout, unhex(row["stdout_hex"].as_str().unwrap()));
        assert_eq!(output.stderr, unhex(row["stderr_hex"].as_str().unwrap()));
    }
}

#[test]
fn remaining_integer_kinds_preserve_native_definitions_and_raw_metadata() {
    let directory = tempfile::tempdir().unwrap();
    let fixture = fixture();
    let mut checked = 0;
    for row in fixture["cases"].as_array().unwrap() {
        if row["status"] != 0 {
            continue;
        }
        let bytes = unhex(row["descriptor_hex"].as_str().unwrap());
        let expected = String::from_utf8(unhex(
            row["observation"]["fields"]["DEFINITION_HEX"]
                .as_str()
                .unwrap(),
        ))
        .unwrap();
        let mut registry = DescriptorRegistry::default();
        assert!(registry.load_bytes(&bytes).unwrap().is_empty());
        let descriptor = registry.pool().get_message_by_name("Default").unwrap();
        let raw = descriptor
            .get_field_by_name("value")
            .unwrap()
            .field_descriptor_proto()
            .default_value
            .clone();
        assert_eq!(
            registry.definition("Default").unwrap(),
            Some(expected.clone())
        );
        assert_eq!(descriptor_definition(&descriptor).unwrap(), expected);
        assert_eq!(
            descriptor
                .get_field_by_name("value")
                .unwrap()
                .field_descriptor_proto()
                .default_value,
            raw
        );
        let input = directory.path().join("input.desc");
        std::fs::write(&input, &bytes).unwrap();
        let factory = MessageFactory::with_paths(input.as_os_str());
        assert!(factory.take_initial_errors().is_empty());
        assert_eq!(
            factory.registry_definition("Default").unwrap(),
            Some(expected.clone())
        );
        let output = Command::new(env!("CARGO_BIN_EXE_gz-msgs"))
            .args(["info", "Default"])
            .env("GZ_DESCRIPTOR_PATH", input)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        assert_eq!(
            output.stdout,
            format!("Name: Default\nFile: defaults.proto\n\n{expected}\n").as_bytes()
        );
        checked += 1;
    }
    assert_eq!(checked, 384);
}

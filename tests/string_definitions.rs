// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use gz_msgs::{
    descriptor_definition, factory::MessageFactory, registry::DescriptorRegistry, serde_json,
    DefinitionError,
};
use std::process::Command;

fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!("fixtures/string-definitions.json")).unwrap()
}
fn unhex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn string_declaration_observations_reproduce_native_outputs() {
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
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 54);
    for row in fixture["cases"].as_array().unwrap() {
        let input = directory.path().join("input.desc");
        std::fs::write(&input, unhex(row["descriptor_hex"].as_str().unwrap())).unwrap();
        let output = Command::new(&peer)
            .arg(input)
            .env("LC_ALL", "C")
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(row["status"].as_i64().unwrap() as i32),
            "{}",
            row["input"]
        );
        assert_eq!(
            output.stdout,
            unhex(row["stdout_hex"].as_str().unwrap()),
            "{}",
            row["input"]
        );
        assert_eq!(
            output.stderr,
            unhex(row["stderr_hex"].as_str().unwrap()),
            "{}",
            row["input"]
        );
    }
}

#[test]
fn ordinary_string_defaults_match_native_without_expanding_invalid_shapes() {
    let directory = tempfile::tempdir().unwrap();
    let fixture = fixture();
    let mut supported = 0;
    for row in fixture["cases"].as_array().unwrap() {
        let id = row["input"]["id"].as_str().unwrap();
        let bytes = unhex(row["descriptor_hex"].as_str().unwrap());
        let mut registry = DescriptorRegistry::default();
        if id == "invalid-utf8" {
            assert!(registry.load_bytes(&bytes).is_err());
            assert!(registry.pool().get_message_by_name("Default").is_none());
            continue;
        }
        let errors = registry.load_bytes(&bytes).unwrap();
        assert!(errors.is_empty(), "{id}: {errors:?}");
        if !row["supported"].as_bool().unwrap() {
            let error = match id {
                "proto3" => "proto3 default or required field",
                "unknown-field-option" => "unknown options",
                _ => "proto2 default",
            };
            assert_eq!(
                registry.definition("Default"),
                Err(DefinitionError::Unsupported(error)),
                "{id}"
            );
            continue;
        }
        let expected =
            String::from_utf8(unhex(row["fields"]["DEFINITION_HEX"].as_str().unwrap())).unwrap();
        let descriptor = registry.pool().get_message_by_name("Default").unwrap();
        let raw = descriptor
            .get_field_by_name("value")
            .unwrap()
            .field_descriptor_proto()
            .clone();
        assert_eq!(
            registry.definition("Default").unwrap(),
            Some(expected.clone()),
            "{id}"
        );
        if row["input"]["layout"] != "bare" {
            assert_eq!(
                descriptor_definition(&descriptor).unwrap(),
                expected,
                "{id}"
            );
        }
        assert_eq!(
            descriptor
                .get_field_by_name("value")
                .unwrap()
                .field_descriptor_proto(),
            &raw
        );
        let input = directory.path().join("input.desc");
        std::fs::write(&input, &bytes).unwrap();
        let factory = MessageFactory::with_paths(input.as_os_str());
        assert!(factory.take_initial_errors().is_empty());
        assert_eq!(
            factory.registry_definition("Default").unwrap(),
            Some(expected.clone()),
            "{id}"
        );
        let output = Command::new(env!("CARGO_BIN_EXE_gz-msgs"))
            .args(["info", "Default"])
            .env("GZ_DESCRIPTOR_PATH", input)
            .output()
            .unwrap();
        assert!(output.status.success(), "{id}: {output:?}");
        assert!(output.stderr.is_empty(), "{id}: {output:?}");
        assert_eq!(
            output.stdout,
            format!("Name: Default\nFile: defaults.proto\n\n{expected}\n").as_bytes(),
            "{id}"
        );
        supported += 1;
    }
    assert_eq!(supported, 49);
}

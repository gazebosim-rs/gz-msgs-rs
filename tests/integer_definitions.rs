// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use gz_msgs::{
    descriptor_definition,
    factory::MessageFactory,
    prost::Message,
    prost_reflect::{DescriptorPool, Value},
    prost_types::FileDescriptorSet,
    registry::DescriptorRegistry,
    serde_json, DefinitionError,
};
use std::{path::PathBuf, process::Command};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "gz-integer-definitions-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn native(&self) -> PathBuf {
        let path = self.0.join("peer");
        let output = Command::new("c++")
            .args([
                "-std=c++17",
                "-Wall",
                "-Wextra",
                "-Werror",
                "tests/integer_definition_peer.cc",
                "-lprotobuf",
                "-o",
            ])
            .arg(&path)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        path
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn unhex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    text.as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn observations() -> serde_json::Value {
    serde_json::from_str(include_str!("fixtures/integer-definitions/native-76.json")).unwrap()
}
fn expected(row: &serde_json::Value) -> String {
    String::from_utf8(unhex(
        row["observation"]["fields"]["DEFINITION_HEX"]
            .as_str()
            .unwrap(),
    ))
    .unwrap()
}
#[test]
fn all_472_native_declarations_reproduce_exact_captured_outputs() {
    let directory = Directory::new();
    let peer = directory.native();
    let a = observations();
    let b: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/integer-definitions/existing-396.json"
    ))
    .unwrap();
    assert_eq!(a["cases"].as_array().unwrap().len(), 76);
    assert_eq!(b["cases"].as_array().unwrap().len(), 396);
    for row in a["cases"]
        .as_array()
        .unwrap()
        .iter()
        .chain(b["cases"].as_array().unwrap())
    {
        let bytes = unhex(row["descriptor_hex"].as_str().unwrap());
        let input = directory.0.join("input.desc");
        std::fs::write(&input, &bytes).unwrap();
        let output = Command::new(&peer)
            .arg(&input)
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
        assert_eq!(std::fs::read(input).unwrap(), bytes);
    }
}
#[test]
fn ordinary_proto2_integer_declarations_use_effective_values_and_native_option_order() {
    let fixture = observations();
    let directory = Directory::new();
    let mut checked = 0;
    for row in fixture["cases"].as_array().unwrap() {
        let id = row["input"]["id"].as_str().unwrap();
        // The other native rows characterize explicit remaining boundaries.
        if !(id.starts_with("integer-") || id == "ordinary-required" || id == "no-explicit-default")
        {
            continue;
        }
        let bytes = unhex(row["descriptor_hex"].as_str().unwrap());
        let input = directory.0.join("input.desc");
        std::fs::write(&input, &bytes).unwrap();
        let mut registry = DescriptorRegistry::default();
        assert!(registry.load_bytes(&bytes).unwrap().is_empty());
        let descriptor = registry.pool().get_message_by_name("Default").unwrap();
        let field = descriptor.get_field_by_name("value").unwrap();
        let raw_before = field.field_descriptor_proto().clone();
        let file_before = descriptor.parent_file().encode_to_vec();
        let native = expected(row);
        assert_eq!(
            registry.definition("Default").unwrap(),
            Some(native.clone()),
            "{id}"
        );
        if row["input"]["layout"] == "json-name-and-deprecated" {
            assert_eq!(descriptor_definition(&descriptor).unwrap(), native, "{id}");
        }
        let factory = MessageFactory::with_paths(input.as_os_str());
        assert!(factory.take_initial_errors().is_empty());
        assert_eq!(
            factory.registry_definition("Default").unwrap(),
            Some(native.clone()),
            "{id}"
        );
        let output = Command::new(env!("CARGO_BIN_EXE_gz-msgs"))
            .args(["info", "Default"])
            .env("GZ_DESCRIPTOR_PATH", &input)
            .output()
            .unwrap();
        assert!(output.status.success(), "{id}: {output:?}");
        assert!(output.stderr.is_empty(), "{id}: {output:?}");
        assert_eq!(
            output.stdout,
            format!("Name: Default\nFile: defaults.proto\n\n{native}\n").as_bytes(),
            "{id}"
        );
        assert_eq!(field.field_descriptor_proto(), &raw_before);
        assert_eq!(descriptor.parent_file().encode_to_vec(), file_before);
        let mut reflected_file = FileDescriptorSet::decode(bytes.as_slice())
            .unwrap()
            .file
            .remove(0);
        let original_field = &mut reflected_file.message_type[0].field[0];
        assert_eq!(raw_before.default_value, original_field.default_value);
        // The unchanged reflection loader synthesizes omitted json_name fields.
        // Registry formatting retains the source omission separately; this is
        // not a claim that reflection reproduces the entire raw input descriptor.
        if original_field.json_name.is_none() {
            original_field.json_name = Some("value".into());
        }
        assert_eq!(file_before, reflected_file.encode_to_vec());
        assert_eq!(std::fs::read(input).unwrap(), bytes);
        checked += 1;
    }
    assert_eq!(checked, 66);
}
#[test]
fn prior_boolean_default_control_matches_its_unchanged_native_definition() {
    let fixture = observations();
    let row = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["input"]["id"] == "bool-default")
        .unwrap();
    let bytes = unhex(row["descriptor_hex"].as_str().unwrap());
    let mut registry = DescriptorRegistry::default();
    assert!(registry.load_bytes(&bytes).unwrap().is_empty());
    let descriptor = registry.pool().get_message_by_name("Default").unwrap();
    let raw = descriptor
        .get_field_by_name("value")
        .unwrap()
        .field_descriptor_proto()
        .clone();
    assert_eq!(registry.definition("Default").unwrap(), Some(expected(row)));
    assert_eq!(
        descriptor
            .get_field_by_name("value")
            .unwrap()
            .field_descriptor_proto(),
        &raw
    );
}

#[test]
fn prior_string_default_control_matches_its_unchanged_native_definition() {
    let fixture = observations();
    let row = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["input"]["id"] == "string-default")
        .unwrap();
    let bytes = unhex(row["descriptor_hex"].as_str().unwrap());
    let mut registry = DescriptorRegistry::default();
    assert!(registry.load_bytes(&bytes).unwrap().is_empty());
    let descriptor = registry.pool().get_message_by_name("Default").unwrap();
    let raw = descriptor
        .get_field_by_name("value")
        .unwrap()
        .field_descriptor_proto()
        .clone();
    assert_eq!(registry.definition("Default").unwrap(), Some(expected(row)));
    assert_eq!(
        descriptor
            .get_field_by_name("value")
            .unwrap()
            .field_descriptor_proto(),
        &raw
    );
}

#[test]
fn unsupported_shapes_and_kinds_stay_explicit_and_standard_pool_semantics_stay_distinct() {
    let fixture = observations();
    for id in [
        "optional-oneof",
        "float-default",
        "unknown-field-option",
        "proto3-default",
    ] {
        let row = fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["input"]["id"] == id)
            .unwrap();
        let bytes = unhex(row["descriptor_hex"].as_str().unwrap());
        let mut registry = DescriptorRegistry::default();
        assert!(registry.load_bytes(&bytes).unwrap().is_empty(), "{id}");
        let expected = if id == "proto3-default" {
            "proto3 default or required field"
        } else if id == "unknown-field-option" {
            "unknown options"
        } else {
            "proto2 default"
        };
        assert_eq!(
            registry.definition("Default"),
            Err(DefinitionError::Unsupported(expected)),
            "{id}"
        );
    }
    // Ordinary DescriptorPool keeps its existing decimal interpretation; the
    // formatter uses that supplied effective value, never re-parses raw text.
    let row = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["input"]["id"] == "integer-07")
        .unwrap();
    let bytes = unhex(row["descriptor_hex"].as_str().unwrap());
    let pool = DescriptorPool::from_file_descriptor_set(
        FileDescriptorSet::decode(bytes.as_slice()).unwrap(),
    )
    .unwrap();
    let descriptor = pool.get_message_by_name("Default").unwrap();
    assert_eq!(
        descriptor
            .get_field_by_name("value")
            .unwrap()
            .default_value(),
        Value::I32(10)
    );
    assert_eq!(
        descriptor_definition(&descriptor).unwrap(),
        "message Default {\n  optional int32 value = 1 [default = 10, json_name = \"customValue\", deprecated = true];\n}\n"
    );
    assert_eq!(
        descriptor
            .get_field_by_name("value")
            .unwrap()
            .field_descriptor_proto()
            .default_value
            .as_deref(),
        Some("010")
    );
}

#[test]
fn original_enum_default_observation_preserves_selected_name() {
    let fixture = observations();
    let row = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["input"]["id"] == "enum-default")
        .unwrap();
    let bytes = unhex(row["descriptor_hex"].as_str().unwrap());
    let mut registry = DescriptorRegistry::default();
    assert!(registry.load_bytes(&bytes).unwrap().is_empty());
    let descriptor = registry.pool().get_message_by_name("Default").unwrap();
    let raw = descriptor
        .get_field_by_name("value")
        .unwrap()
        .field_descriptor_proto()
        .clone();
    assert_eq!(registry.definition("Default").unwrap(), Some(expected(row)));
    assert_eq!(
        descriptor
            .get_field_by_name("value")
            .unwrap()
            .field_descriptor_proto(),
        &raw
    );
}

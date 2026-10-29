// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use gz_msgs::{
    descriptor_definition, factory::MessageFactory, registry::DescriptorRegistry, serde_json,
    DefinitionError,
};
use std::process::Command;

fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!("fixtures/floating-definitions.json")).unwrap()
}
fn unhex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn floating_declaration_observations_reproduce_native_outputs() {
    let directory = tempfile::tempdir().unwrap();
    let peer = directory.path().join("peer");
    let build = Command::new("c++")
        .args([
            "-std=c++17",
            "-Wall",
            "-Wextra",
            "-Werror",
            "tests/floating_definition_peer.cc",
            "-lprotobuf",
            "-o",
        ])
        .arg(&peer)
        .output()
        .unwrap();
    assert!(build.status.success(), "{build:?}");
    let corpus = include_str!("fixtures/floating-format.txt");
    let input = directory.path().join("bits.txt");
    let lines: Vec<_> = corpus
        .lines()
        .map(|line| {
            let fields: Vec<_> = line.split_whitespace().collect();
            assert_eq!(fields.len(), 3);
            format!("{} {}\n", fields[0], fields[1])
        })
        .collect();
    assert_eq!(lines.len(), 21397);
    std::fs::write(&input, lines.concat()).unwrap();
    let output = Command::new(&peer)
        .arg("--bits")
        .stdin(std::fs::File::open(input).unwrap())
        .env("LC_ALL", "C")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout, corpus.as_bytes());
    let fixture = fixture();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 280);
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
fn floating_defaults_preserve_effective_values_and_native_dispositions() {
    let directory = tempfile::tempdir().unwrap();
    let fixture = fixture();
    let mut supported = 0;
    for row in fixture["cases"].as_array().unwrap() {
        let id = row["input"]["id"].as_str().unwrap();
        let bytes = unhex(row["descriptor_hex"].as_str().unwrap());
        let mut registry = DescriptorRegistry::default();
        let disposition = row["rust_disposition"].as_str().unwrap();
        if disposition == "DecodeError" {
            assert!(registry.load_bytes(&bytes).is_err(), "{id}");
            assert!(
                registry.pool().get_message_by_name("Default").is_none(),
                "{id}"
            );
            continue;
        }
        let errors = registry.load_bytes(&bytes).unwrap();
        if disposition == "DescriptorError" {
            assert!(!errors.is_empty(), "{id}");
            assert!(
                registry.pool().get_message_by_name("Default").is_none(),
                "{id}"
            );
            continue;
        }
        assert!(errors.is_empty(), "{id}: {errors:?}");
        if disposition == "Unsupported" {
            let error = match row["input"]["shape"].as_str().unwrap() {
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
        let expected = row["rust_definition"].as_str().unwrap().to_owned();
        let descriptor = registry.pool().get_message_by_name("Default").unwrap();
        let field = descriptor.get_field_by_name("value").unwrap();
        let raw = field.field_descriptor_proto().clone();
        let actual = field.default_value();
        let bits = match actual {
            gz_msgs::prost_reflect::Value::F32(value) => {
                assert_eq!(row["input"]["kind"], "f32");
                u64::from(value.to_bits())
            }
            gz_msgs::prost_reflect::Value::F64(value) => {
                assert_eq!(row["input"]["kind"], "f64");
                value.to_bits()
            }
            _ => panic!("not floating: {id}"),
        };
        assert_eq!(bits.to_string(), row["rust_bits"].as_str().unwrap(), "{id}");
        let key = if row["input"]["kind"] == "f32" {
            "EFFECTIVE_F32_BITS"
        } else {
            "EFFECTIVE_F64_BITS"
        };
        if disposition == "native-exact" {
            assert_eq!(
                bits.to_string(),
                row["fields"][key].as_str().unwrap(),
                "{id}"
            );
            assert_eq!(
                expected.as_bytes(),
                unhex(row["fields"]["DEFINITION_HEX"].as_str().unwrap()),
                "{id}"
            );
        } else {
            assert_eq!(disposition, "reflected-difference");
            assert!(id.starts_with("f32-midpoint-0-") || id.starts_with("f32-midpoint-2-"));
            assert_ne!(
                bits.to_string(),
                row["fields"][key].as_str().unwrap(),
                "{id}"
            );
        }
        assert_eq!(
            raw.default_value.as_deref(),
            row["input"]["default"].as_str(),
            "{id}"
        );
        if row["input"]["layout"] != "bare" {
            // Safe admitted inputs also exercise the standard pool without registry preflight.
            let standard =
                gz_msgs::prost_reflect::DescriptorPool::decode(bytes.as_slice()).unwrap();
            let standard = standard.get_message_by_name("Default").unwrap();
            assert_eq!(descriptor_definition(&standard).unwrap(), expected, "{id}");
            let standard_bits = match standard.get_field_by_name("value").unwrap().default_value() {
                gz_msgs::prost_reflect::Value::F32(value) => u64::from(value.to_bits()),
                gz_msgs::prost_reflect::Value::F64(value) => value.to_bits(),
                _ => panic!("not floating: {id}"),
            };
            assert_eq!(standard_bits, bits, "{id}");
        }
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
    assert_eq!(supported, 252);
}

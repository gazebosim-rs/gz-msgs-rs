// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use gz_msgs::{
    descriptor_definition, factory::MessageFactory, prost::Message, registry::DescriptorRegistry,
    serde_json, DefinitionError,
};
use std::{collections::BTreeMap, process::Command};

fn unhex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn body(row: &serde_json::Value) -> String {
    row["native"]["stdout"]
        .as_str()
        .unwrap()
        .lines()
        .find_map(|line| {
            let (name, hex) = line.split_once(' ')?;
            (name == "opts.Root").then(|| String::from_utf8(unhex(hex)).unwrap())
        })
        .unwrap()
}
#[test]
fn native_field_options_keep_presence_shapes_order_and_source_metadata() {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/field-options.json")).unwrap();
    let rows = data["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 354);
    let named: BTreeMap<_, _> = rows
        .iter()
        .filter(|r| r["json_explicit"] == true)
        .map(|r| (r["family"].as_str().unwrap(), r))
        .collect();
    let directory = tempfile::tempdir().unwrap();
    let peer = directory.path().join("peer");
    let built = Command::new("c++")
        .args([
            "-std=c++17",
            "-Wall",
            "-Wextra",
            "-Werror",
            "tests/definition_peer.cc",
            "-lprotobuf",
            "-o",
        ])
        .arg(&peer)
        .output()
        .unwrap();
    assert!(built.status.success(), "{built:?}");
    let mut supported = 0;
    let mut native_accepted = 0;
    for row in rows {
        let input = directory.path().join("input.desc");
        let bytes = unhex(row["descriptor_hex"].as_str().unwrap());
        std::fs::write(&input, &bytes).unwrap();
        let actual = Command::new(&peer).arg(&input).output().unwrap();
        assert_eq!(
            actual.status.code(),
            row["native"]["exit"].as_i64().map(|v| v as i32),
            "{}",
            row["id"]
        );
        assert_eq!(
            actual.stdout,
            row["native"]["stdout"].as_str().unwrap().as_bytes(),
            "{}",
            row["id"]
        );
        assert_eq!(
            actual.stderr,
            row["native"]["stderr"].as_str().unwrap().as_bytes(),
            "{}",
            row["id"]
        );
        native_accepted += usize::from(actual.status.success());
        let mut registry = DescriptorRegistry::default();
        assert!(
            registry.load_bytes(&bytes).unwrap().is_empty(),
            "{}",
            row["id"]
        );
        let descriptor = registry.pool().get_message_by_name("opts.Root").unwrap();
        let raw_before = descriptor.descriptor_proto().clone();
        let field = descriptor.fields().next().unwrap();
        let options_before = field.options().encode_to_vec();
        let factory = MessageFactory::with_paths(input.as_os_str());
        assert!(factory.take_initial_errors().is_empty());
        let cli = Command::new(env!("CARGO_BIN_EXE_gz-msgs"))
            .args(["--info", "opts.Root"])
            .env("GZ_DESCRIPTOR_PATH", &input)
            .output()
            .unwrap();
        if row["expected"] == "native" {
            supported += 1;
            let expected = body(row);
            assert_eq!(
                registry.definition("opts.Root").unwrap(),
                Some(expected.clone()),
                "{}",
                row["id"]
            );
            assert_eq!(
                factory.registry_definition("opts.Root").unwrap(),
                Some(expected.clone()),
                "{}",
                row["id"]
            );
            // The direct descriptor API has computed JSON names, not registry source omissions.
            let direct = body(named[row["family"].as_str().unwrap()]);
            assert_eq!(
                descriptor_definition(&descriptor).unwrap(),
                direct,
                "{}",
                row["id"]
            );
            assert!(cli.status.success(), "{} {cli:?}", row["id"]);
            assert_eq!(
                cli.stdout,
                row["native_cli"]["stdout"].as_str().unwrap().as_bytes(),
                "{}",
                row["id"]
            );
            assert_eq!(
                cli.stderr,
                row["native_cli"]["stderr"].as_str().unwrap().as_bytes(),
                "{}",
                row["id"]
            );
        } else {
            let reason = match row["expected"].as_str().unwrap() {
                "required" => "proto3 default or required field",
                "option" => "option",
                "lazy" => "lazy option on non-message field",
                _ => panic!("fixture disposition"),
            };
            let expected = DefinitionError::Unsupported(reason);
            assert_eq!(
                registry.definition("opts.Root"),
                Err(expected),
                "{}",
                row["id"]
            );
            assert_eq!(
                descriptor_definition(&descriptor),
                Err(expected),
                "{}",
                row["id"]
            );
            assert_eq!(
                factory.registry_definition("opts.Root"),
                Err(expected),
                "{}",
                row["id"]
            );
            assert!(!cli.status.success(), "{}", row["id"]);
            assert!(cli.stdout.is_empty(), "{}", row["id"]);
            let stderr = String::from_utf8(cli.stderr).unwrap();
            assert_eq!(stderr, format!("Error: {expected:?}\n"), "{}", row["id"]);
        }
        assert_eq!(descriptor.descriptor_proto(), &raw_before, "{}", row["id"]);
        assert_eq!(
            field.options().encode_to_vec(),
            options_before,
            "{}",
            row["id"]
        );
    }
    assert_eq!(supported, 272);
    assert_eq!(native_accepted, 275);
}

// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use gz_msgs::{
    descriptor_definition, factory::MessageFactory, registry::DescriptorRegistry, serde_json,
    DefinitionError,
};
use prost::Message;
use std::process::Command;
fn unhex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn check(rows: &[serde_json::Value]) -> usize {
    let dir = tempfile::tempdir().unwrap();
    let peer = dir.path().join("peer");
    assert!(Command::new("c++")
        .args([
            "-std=c++17",
            "-Wall",
            "-Wextra",
            "-Werror",
            "tests/definition_peer.cc",
            "-lprotobuf",
            "-o"
        ])
        .arg(&peer)
        .status()
        .unwrap()
        .success());
    let mut supported = 0;
    for row in rows {
        let bytes = unhex(row["descriptor_hex"].as_str().unwrap());
        let input = dir.path().join("input.desc");
        std::fs::write(&input, &bytes).unwrap();
        let native = Command::new(&peer).arg(&input).output().unwrap();
        assert_eq!(
            native.status.code().unwrap(),
            row["native"]["exit"].as_i64().unwrap() as i32,
            "{}",
            row["id"]
        );
        assert_eq!(
            native.stdout,
            row["native"]["stdout"].as_str().unwrap().as_bytes()
        );
        assert_eq!(
            native.stderr,
            row["native"]["stderr"].as_str().unwrap().as_bytes()
        );
        let mut registry = DescriptorRegistry::default();
        assert!(registry.load_bytes(&bytes).unwrap().is_empty(), "{row}");
        let descriptor = registry.pool().get_message_by_name("probe.Root").unwrap();
        let file_before = descriptor.parent_file().encode_to_vec();
        let owner = if row["layout"] == "nested" {
            descriptor.child_messages().next().unwrap()
        } else {
            descriptor.clone()
        };
        let mut extensions: Vec<_> = owner.child_extensions().collect();
        extensions.sort_by_key(|e| e.path().last().copied());
        assert_eq!(extensions.len(), row["values"].as_array().unwrap().len());
        let options_before: Vec<_> = extensions
            .iter()
            .map(|e| e.options().encode_to_vec())
            .collect();
        for (e, value) in extensions.iter().zip(row["values"].as_array().unwrap()) {
            let expected = value.as_bool();
            assert_eq!(
                e.field_descriptor_proto()
                    .options
                    .as_ref()
                    .and_then(|o| o.deprecated),
                expected
            );
            assert_eq!(
                e.options().has_field_by_name("deprecated"),
                expected.is_some()
            );
            assert_eq!(
                e.options()
                    .get_field_by_name("deprecated")
                    .unwrap()
                    .as_bool(),
                Some(expected.unwrap_or(false))
            );
        }
        assert_eq!(
            descriptor_definition(&descriptor),
            Err(DefinitionError::Unsupported("extension declarations"))
        );
        let factory = MessageFactory::with_paths(input.as_os_str());
        assert!(factory.take_initial_errors().is_empty());
        let cli = Command::new(env!("CARGO_BIN_EXE_gz-msgs"))
            .args(["--info", "probe.Root"])
            .env("GZ_DESCRIPTOR_PATH", &input)
            .output()
            .unwrap();
        if matches!(
            row["mode"].as_str().unwrap(),
            "deprecated" | "mixed" | "historical-false"
        ) {
            supported += 1;
            let body = row["native"]["stdout"]
                .as_str()
                .unwrap()
                .lines()
                .find_map(|line| {
                    let (name, hex) = line.split_once(' ')?;
                    (name == "probe.Root").then(|| String::from_utf8(unhex(hex)).unwrap())
                })
                .unwrap();
            assert_eq!(registry.definition("probe.Root").unwrap().unwrap(), body);
            assert_eq!(
                factory.registry_definition("probe.Root").unwrap().unwrap(),
                body
            );
            assert_eq!(
                cli.status.code().unwrap(),
                row["native_cli"]["exit"].as_i64().unwrap() as i32
            );
            assert_eq!(
                cli.stdout,
                row["native_cli"]["stdout"].as_str().unwrap().as_bytes()
            );
            assert_eq!(
                cli.stderr,
                row["native_cli"]["stderr"].as_str().unwrap().as_bytes()
            );
        } else {
            let reason = match row["mode"].as_str().unwrap() {
                "other-option" => "option",
                "unknown-option" => "unknown options",
                "custom-option" => "custom options",
                _ => "extension declarations",
            };
            assert_eq!(
                registry.definition("probe.Root"),
                Err(DefinitionError::Unsupported(reason)),
                "{row}"
            );
            assert_eq!(
                factory.registry_definition("probe.Root"),
                Err(DefinitionError::Unsupported(reason))
            );
            assert!(!cli.status.success());
            assert!(cli.stdout.is_empty());
            assert!(String::from_utf8(cli.stderr).unwrap().contains(reason));
        }
        assert_eq!(descriptor.parent_file().encode_to_vec(), file_before);
        assert_eq!(
            extensions
                .iter()
                .map(|e| e.options().encode_to_vec())
                .collect::<Vec<_>>(),
            options_before
        );
    }
    supported
}
#[test]
fn declared_deprecated_extension_flags_match_native() {
    let d: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/extension-deprecated.json")).unwrap();
    assert_eq!(d["rows"].as_array().unwrap().len(), 224);
    let old: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/nested-extension-declarations.json")).unwrap();
    let new: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/extension-deprecated.json")).unwrap();
    let old = old["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["mode"] == "deprecated")
        .unwrap();
    let new = new["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["mode"] == "historical-false")
        .unwrap();
    for key in ["descriptor_hex", "native", "native_cli"] {
        assert_eq!(old[key], new[key], "{key}");
    }
    assert_eq!(new["values"], serde_json::json!([false, false, false]));
    assert_eq!(check(d["rows"].as_array().unwrap()), 209);
}

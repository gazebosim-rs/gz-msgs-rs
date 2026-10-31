// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use gz_msgs::{
    descriptor_definition, factory::MessageFactory, registry::DescriptorRegistry, serde_json,
    DefinitionError,
};
use std::process::Command;
fn unhex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
#[test]
fn prepared_source_aware_nested_declarations_match_native() {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/nested-extension-declarations.json")).unwrap();
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
    assert_eq!(data["rows"].as_array().unwrap().len(), 125);
    let (mut supported, mut native_admitted, mut registry_rejected) = (0, 0, 0);
    for row in data["rows"].as_array().unwrap() {
        let bytes = unhex(row["descriptor_hex"].as_str().unwrap());
        let input = dir.path().join("input.desc");
        std::fs::write(&input, &bytes).unwrap();
        let native = Command::new(&peer).arg(&input).output().unwrap();
        assert_eq!(
            native.status.code().unwrap(),
            row["native"]["exit"].as_i64().unwrap() as i32
        );
        assert_eq!(
            native.stdout,
            row["native"]["stdout"].as_str().unwrap().as_bytes()
        );
        assert_eq!(
            native.stderr,
            row["native"]["stderr"].as_str().unwrap().as_bytes()
        );
        native_admitted += usize::from(native.status.success());
        let mut registry = DescriptorRegistry::default();
        let errors = registry.load_bytes(&bytes).unwrap();
        if !errors.is_empty() {
            registry_rejected += 1;
            assert_eq!(row["mode"], "oneof");
            assert_ne!(row["native"]["exit"], 0);
            continue;
        }
        let d = registry.pool().get_message_by_name("probe.Root").unwrap();
        let before = d.parent_file().encode_to_vec();
        if row["mode"] == "deprecated" {
            assert_eq!(d.child_extensions().len(), 3);
            for extension in d.child_extensions() {
                assert_eq!(
                    extension
                        .field_descriptor_proto()
                        .options
                        .as_ref()
                        .unwrap()
                        .deprecated,
                    Some(false)
                );
                assert!(extension.options().has_field_by_name("deprecated"));
                assert_eq!(
                    extension
                        .options()
                        .get_field_by_name("deprecated")
                        .unwrap()
                        .as_bool(),
                    Some(false)
                );
            }
        }
        assert_eq!(
            descriptor_definition(&d),
            Err(DefinitionError::Unsupported("extension declarations"))
        );
        let result = registry.definition("probe.Root");
        if (matches!(
            row["mode"].as_str().unwrap(),
            "ordinary" | "empty-options" | "digit-name" | "deprecated"
        ) && row["label"] != 2)
            || row["id"] == 108
        {
            supported += 1;
            let expected = row["native"]["stdout"]
                .as_str()
                .unwrap()
                .lines()
                .find_map(|line| {
                    let (name, hex) = line.split_once(' ')?;
                    (name == "probe.Root").then(|| String::from_utf8(unhex(hex)).unwrap())
                })
                .unwrap();
            assert_eq!(result.unwrap().unwrap(), expected);
            let factory = MessageFactory::with_paths(input.as_os_str());
            assert!(factory.take_initial_errors().is_empty());
            assert_eq!(
                factory.registry_definition("probe.Root").unwrap().unwrap(),
                expected
            );
            let cli = Command::new(env!("CARGO_BIN_EXE_gz-msgs"))
                .args(["--info", "probe.Root"])
                .env("GZ_DESCRIPTOR_PATH", &input)
                .output()
                .unwrap();
            assert!(cli.status.success(), "{cli:?}");
            assert_eq!(
                cli.stdout,
                row["native_cli"]["stdout"].as_str().unwrap().as_bytes()
            );
            assert!(cli.stderr.is_empty());
        } else {
            let reason = match row["mode"].as_str().unwrap() {
                "unknown-option" => "unknown options",
                "custom-option" => "custom options",
                _ => "extension declarations",
            };
            assert_eq!(result, Err(DefinitionError::Unsupported(reason)), "{row}");
        }
        assert_eq!(d.parent_file().encode_to_vec(), before);
    }
    assert_eq!((supported, native_admitted), (110, 115));
    assert_eq!(registry_rejected, 1);
}

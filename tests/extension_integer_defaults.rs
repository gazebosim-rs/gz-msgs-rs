// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use gz_msgs::{
    descriptor_definition, factory::MessageFactory, prost_reflect::Kind,
    registry::DescriptorRegistry, serde_json, DefinitionError,
};
use std::process::Command;

fn unhex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn data() -> serde_json::Value {
    serde_json::from_str(include_str!("fixtures/extension-integer-defaults.json")).unwrap()
}

fn native_definition(row: &serde_json::Value) -> String {
    row["native"]["stdout"]
        .as_str()
        .unwrap()
        .lines()
        .find_map(|line| {
            let hex = line.strip_prefix("probe.Root ")?;
            Some(String::from_utf8(unhex(hex)).unwrap())
        })
        .unwrap()
}

#[test]
fn prepared_exact_and_new_source_aware_integer_defaults_match_native_and_keep_guards() {
    let data = data();
    let rows = data["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 601);
    let directory = tempfile::tempdir().unwrap();
    let peer = directory.path().join("peer");
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
    let path = directory.path().join("input.desc");
    let (mut supported, mut unsupported, mut rejected) = (0, 0, 0);
    let mut native_accepted_loader_rejections = 0;
    for row in rows {
        let id = row["id"].as_str().unwrap();
        if let Some(previous) = row["previous_fixture"].as_str() {
            let old: serde_json::Value = serde_json::from_slice(
                &std::fs::read(std::path::Path::new("tests/fixtures").join(previous)).unwrap(),
            )
            .unwrap();
            let old = old["rows"]
                .as_array()
                .unwrap()
                .iter()
                .find(|value| value["id"] == row["previous_id"])
                .unwrap();
            assert_eq!(row["descriptor_hex"], old["descriptor_hex"], "{id}");
            assert_eq!(row["native"], old["native"], "{id}");
            assert_eq!(row["native_cli"], old["native_cli"], "{id}");
        }
        let bytes = unhex(row["descriptor_hex"].as_str().unwrap());
        std::fs::write(&path, &bytes).unwrap();
        let native = Command::new(&peer).arg(&path).output().unwrap();
        assert_eq!(
            native.status.code(),
            row["native"]["exit"].as_i64().map(|n| n as i32),
            "{id}"
        );
        assert_eq!(
            native.stdout,
            row["native"]["stdout"].as_str().unwrap().as_bytes(),
            "{id}"
        );
        assert_eq!(
            native.stderr,
            row["native"]["stderr"].as_str().unwrap().as_bytes(),
            "{id}"
        );
        let mut registry = DescriptorRegistry::default();
        let errors = registry.load_bytes(&bytes).unwrap();
        if row["policy"] == "loader-reject" {
            rejected += 1;
            assert!(!errors.is_empty(), "{id}");
            native_accepted_loader_rejections += usize::from(native.status.success());
            assert!(registry.pool().get_message_by_name("probe.Root").is_none());
            // The retained missing-message CLI contract reports a diagnostic
            // with status zero; native schema-validation prose remains a gap.
            for option in ["--info", "-i", "info"] {
                let cli = Command::new(env!("CARGO_BIN_EXE_gz-msgs"))
                    .args([option, "probe.Root"])
                    .env("GZ_DESCRIPTOR_PATH", &path)
                    .output()
                    .unwrap();
                assert_eq!(cli.status.code(), Some(0), "{id}/{option}");
                assert!(cli.stdout.is_empty(), "{id}/{option}");
                assert_eq!(
                    cli.stderr, b"Unable to create message of type[probe.Root]\n",
                    "{id}/{option}"
                );
            }
            continue;
        }
        assert!(errors.is_empty(), "{id}: {errors:?}");
        let descriptor = registry.pool().get_message_by_name("probe.Root").unwrap();
        let before = descriptor.parent_file().encode_to_vec();
        // Standalone descriptors still lack the raw JSON-name presence boundary.
        assert_eq!(
            descriptor_definition(&descriptor),
            Err(DefinitionError::Unsupported("extension declarations")),
            "{id}"
        );
        let factory = MessageFactory::with_paths(path.as_os_str());
        assert!(factory.take_initial_errors().is_empty(), "{id}");
        if row["policy"] == "supported" {
            supported += 1;
            assert!(native.status.success(), "{id}");
            let expected = native_definition(row);
            assert_eq!(
                registry.definition("probe.Root").unwrap().unwrap(),
                expected,
                "{id}"
            );
            assert_eq!(
                factory.registry_definition("probe.Root").unwrap().unwrap(),
                expected,
                "{id}"
            );
            let owner = if row["layout"] == "nested" {
                descriptor
                    .child_messages()
                    .find(|child| child.name() == "Child")
                    .unwrap()
            } else {
                descriptor.clone()
            };
            for extension in owner.child_extensions() {
                if let Some(raw) = &extension.field_descriptor_proto().default_value {
                    let value = match extension.kind() {
                        Kind::Int32 | Kind::Sint32 | Kind::Sfixed32 => {
                            extension.default_value().as_i32().unwrap().to_string()
                        }
                        Kind::Int64 | Kind::Sint64 | Kind::Sfixed64 => {
                            extension.default_value().as_i64().unwrap().to_string()
                        }
                        Kind::Uint32 | Kind::Fixed32 => {
                            extension.default_value().as_u32().unwrap().to_string()
                        }
                        Kind::Uint64 | Kind::Fixed64 => {
                            extension.default_value().as_u64().unwrap().to_string()
                        }
                        _ => panic!("unqualified default kind"),
                    };
                    assert_eq!(&value, raw, "{id}");
                }
            }
            for option in ["--info", "-i", "info"] {
                let cli = Command::new(env!("CARGO_BIN_EXE_gz-msgs"))
                    .args([option, "probe.Root"])
                    .env("GZ_DESCRIPTOR_PATH", &path)
                    .output()
                    .unwrap();
                assert_eq!(
                    cli.status.code(),
                    Some(row["native_cli"]["exit"].as_i64().unwrap() as i32),
                    "{id}/{option}"
                );
                assert_eq!(
                    cli.stdout,
                    row["native_cli"]["stdout"].as_str().unwrap().as_bytes(),
                    "{id}"
                );
                assert_eq!(
                    cli.stderr,
                    row["native_cli"]["stderr"].as_str().unwrap().as_bytes(),
                    "{id}/{option}"
                );
            }
        } else {
            unsupported += 1;
            let reason = match row["reason"].as_str().unwrap() {
                "extension declarations" => "extension declarations",
                "unknown options" => "unknown options",
                "custom options" => "custom options",
                "packed extension shape" => "packed extension shape",
                "jstype option on non-64-bit integer field" => {
                    "jstype option on non-64-bit integer field"
                }
                _ => panic!("unqualified error policy"),
            };
            assert_eq!(
                registry.definition("probe.Root"),
                Err(DefinitionError::Unsupported(reason)),
                "{id}"
            );
            assert_eq!(
                factory.registry_definition("probe.Root"),
                Err(DefinitionError::Unsupported(reason)),
                "{id}"
            );
            for option in ["--info", "-i", "info"] {
                let cli = Command::new(env!("CARGO_BIN_EXE_gz-msgs"))
                    .args([option, "probe.Root"])
                    .env("GZ_DESCRIPTOR_PATH", &path)
                    .output()
                    .unwrap();
                assert_eq!(cli.status.code(), Some(1), "{id}/{option}");
                assert!(
                    cli.stdout.is_empty(),
                    "{id}/{option}: partial definition exposed"
                );
                assert_eq!(
                    cli.stderr,
                    format!("Error: Unsupported({reason:?})\n").as_bytes(),
                    "{id}/{option}"
                );
            }
        }
        assert_eq!(descriptor.parent_file().encode_to_vec(), before, "{id}");
    }
    assert_eq!((supported, unsupported, rejected), (283, 165, 153));
    // Preserve the extension loader's narrower radix/range/whitespace policy.
    assert_eq!(native_accepted_loader_rejections, 97);
}

#[test]
fn integer_extension_default_snapshots_preserve_presence_duplicates_clones_and_retry() {
    let data = data();
    let row = |id: &str| {
        data["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == id)
            .unwrap()
    };
    let mut registry = DescriptorRegistry::default();
    let expected = native_definition(row("4-ordinary-3"));
    {
        let bytes = unhex(row("4-ordinary-3")["descriptor_hex"].as_str().unwrap());
        assert!(registry.load_bytes(&bytes).unwrap().is_empty());
    }
    let cloned = registry.clone();
    let descriptor = registry.pool().get_message_by_name("probe.Root").unwrap();
    let before = descriptor.parent_file().encode_to_vec();
    let other = unhex(row("4-ordinary-0")["descriptor_hex"].as_str().unwrap());
    assert!(registry.load_bytes(&other).unwrap().is_empty());
    assert_eq!(
        registry.definition("probe.Root").unwrap().unwrap(),
        expected
    );
    assert_eq!(cloned.definition("probe.Root").unwrap().unwrap(), expected);
    assert_eq!(descriptor.parent_file().encode_to_vec(), before);
    for extension in descriptor.child_extensions() {
        assert_eq!(
            extension.field_descriptor_proto().default_value.as_deref(),
            Some("3")
        );
    }
    let mut retry = DescriptorRegistry::default();
    let invalid = unhex(row("4-loader-'0x2a'")["descriptor_hex"].as_str().unwrap());
    assert!(!retry.load_bytes(&invalid).unwrap().is_empty());
    assert!(retry.load_bytes(&other).unwrap().is_empty());
    assert_eq!(
        retry.definition("probe.Root").unwrap().unwrap(),
        native_definition(row("4-ordinary-0"))
    );
    let omitted = row("4-ordinary-omitted");
    assert!(!native_definition(omitted).contains("default ="));
    assert!(native_definition(row("4-ordinary-0")).contains("default = 0"));
}

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
#[test]
fn declared_enum_extension_options_match_native_without_metadata_changes() {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/extension-enum-options.json")).unwrap();
    let rows = data["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 2259);
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
    let mut admitted = 0;
    for row in rows {
        let bytes = unhex(row["descriptor_hex"].as_str().unwrap());
        let input = dir.path().join("input.desc");
        std::fs::write(&input, &bytes).unwrap();
        let native = Command::new(&peer).arg(&input).output().unwrap();
        assert_eq!(
            native.status.code().unwrap(),
            row["native"]["exit"].as_i64().unwrap() as i32,
            "{row}"
        );
        assert_eq!(
            native.stdout,
            row["native"]["stdout"].as_str().unwrap().as_bytes()
        );
        assert_eq!(
            native.stderr,
            row["native"]["stderr"].as_str().unwrap().as_bytes()
        );
        admitted += usize::from(native.status.success());
        let mut registry = DescriptorRegistry::default();
        assert!(registry.load_bytes(&bytes).unwrap().is_empty(), "{row}");
        let root = registry.pool().get_message_by_name("probe.Root").unwrap();
        let before = root.parent_file().encode_to_vec();
        let owner = if row["layout"] == "nested" {
            root.child_messages().next().unwrap()
        } else {
            root.clone()
        };
        let mut extensions: Vec<_> = owner.child_extensions().collect();
        extensions.sort_by_key(|e| e.path().last().copied());
        let metadata: Vec<_> = extensions
            .iter()
            .map(|e| e.options().encode_to_vec())
            .collect();
        assert_eq!(extensions.len(), row["profiles"].as_array().unwrap().len());
        for (e, profile) in extensions.iter().zip(row["profiles"].as_array().unwrap()) {
            for name in ["ctype", "jstype"] {
                let expected = profile[name].as_i64().map(|n| n as i32);
                let typed = e.field_descriptor_proto().options.as_ref().and_then(|o| {
                    if name == "ctype" {
                        o.ctype
                    } else {
                        o.jstype
                    }
                });
                assert_eq!(typed, expected, "{row}");
                assert_eq!(e.options().has_field_by_name(name), expected.is_some());
                assert_eq!(
                    e.options()
                        .get_field_by_name(name)
                        .unwrap()
                        .as_enum_number(),
                    Some(expected.unwrap_or(0))
                );
            }
            for name in ["packed", "deprecated"] {
                if matches!(row["mode"].as_str().unwrap(), "matrix" | "mixed") {
                    let expected = profile[name].as_bool();
                    let typed = e.field_descriptor_proto().options.as_ref().and_then(|o| {
                        if name == "packed" {
                            o.packed
                        } else {
                            o.deprecated
                        }
                    });
                    assert_eq!(typed, expected);
                    assert_eq!(e.options().has_field_by_name(name), expected.is_some());
                    assert_eq!(
                        e.options().get_field_by_name(name).unwrap().as_bool(),
                        Some(expected.unwrap_or(false))
                    );
                }
            }
        }
        assert_eq!(
            descriptor_definition(&root),
            Err(DefinitionError::Unsupported("extension declarations"))
        );
        let ordinary = matches!(row["mode"].as_str().unwrap(), "matrix" | "mixed");
        let profiles = row["profiles"].as_array().unwrap();
        let unknown = profiles.iter().any(|p| {
            ["ctype", "jstype"]
                .iter()
                .any(|n| p[n].as_i64().is_some_and(|v| !(0..=2).contains(&v)))
        });
        let packed = profiles.iter().any(|p| p["packed"] == true)
            && (row["label"] != 3 || matches!(row["kind"].as_u64().unwrap(), 9 | 11 | 12));
        let jstype = profiles
            .iter()
            .any(|p| p["jstype"].as_i64().is_some_and(|v| v != 0))
            && !matches!(row["kind"].as_u64().unwrap(), 3 | 4 | 6 | 16 | 18);
        let reason = if ordinary {
            if unknown {
                Some("option")
            } else if packed {
                Some("packed extension shape")
            } else if jstype {
                Some("jstype option on non-64-bit integer field")
            } else {
                None
            }
        } else {
            Some(match row["mode"].as_str().unwrap() {
                "unknown-option" => "unknown options",
                "custom-option" => "custom options",
                "unhandled-option" => "option",
                _ => "extension declarations",
            })
        };
        let factory = MessageFactory::with_paths(input.as_os_str());
        assert!(factory.take_initial_errors().is_empty());
        let cli = Command::new(env!("CARGO_BIN_EXE_gz-msgs"))
            .args(["--info", "probe.Root"])
            .env("GZ_DESCRIPTOR_PATH", &input)
            .output()
            .unwrap();
        if let Some(reason) = reason {
            let error = DefinitionError::Unsupported(reason);
            assert_eq!(registry.definition("probe.Root"), Err(error), "{row}");
            assert_eq!(factory.registry_definition("probe.Root"), Err(error));
            assert!(!cli.status.success());
            assert!(cli.stdout.is_empty());
            assert_eq!(
                String::from_utf8(cli.stderr).unwrap(),
                format!("Error: {error:?}\n")
            );
        } else {
            supported += 1;
            assert!(native.status.success(), "{row}");
            let body = row["native"]["stdout"]
                .as_str()
                .unwrap()
                .lines()
                .find_map(|line| {
                    let (name, hex) = line.split_once(' ')?;
                    (name == "probe.Root").then(|| String::from_utf8(unhex(hex)).unwrap())
                })
                .unwrap();
            assert_eq!(
                registry.definition("probe.Root").unwrap(),
                Some(body.clone())
            );
            assert_eq!(
                factory.registry_definition("probe.Root").unwrap(),
                Some(body)
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
        }
        assert_eq!(root.parent_file().encode_to_vec(), before);
        assert_eq!(
            extensions
                .iter()
                .map(|e| e.options().encode_to_vec())
                .collect::<Vec<_>>(),
            metadata
        );
    }
    assert_eq!(supported, 1065);
    assert_eq!(admitted, 1888);
}

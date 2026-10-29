// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use gz_msgs::{
    descriptor_definition, factory::MessageFactory, registry::DescriptorRegistry, serde_json,
    DefinitionError,
};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

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
            (name == "ranges.Root").then(|| String::from_utf8(unhex(hex)).unwrap())
        })
        .unwrap()
}
fn native_peer(directory: &Path) -> PathBuf {
    let peer = directory.join("peer");
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
    peer
}
fn check_cases(rows: &[serde_json::Value]) -> (usize, usize, usize) {
    let directory = tempfile::tempdir().unwrap();
    let peer = native_peer(directory.path());
    let mut supported = 0;
    let mut native_accepted = 0;
    let mut loader_rejected = 0;
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
        let errors = registry.load_bytes(&bytes).unwrap();
        if !errors.is_empty() {
            loader_rejected += 1;
            if row["expected"] != "registry" {
                assert_eq!(row["expected"], "invalid extension range");
                assert_eq!(row["layout"], "field");
                assert!(
                    errors.iter().all(|error| {
                        let text = error.to_string();
                        text.contains("field number '5' conflicts with extension range")
                            || text.contains("field number '5' conflicts with reserved range")
                    }),
                    "{} {errors:?}",
                    row["id"]
                );
            }
            assert!(registry.pool().get_message_by_name("ranges.Root").is_none());
            continue;
        }
        assert!(errors.is_empty(), "{} {errors:?}", row["id"]);
        let descriptor = registry.pool().get_message_by_name("ranges.Root").unwrap();
        let raw_before = descriptor.descriptor_proto().clone();
        let options_before = descriptor.parent_file().encode_to_vec();
        let factory = MessageFactory::with_paths(input.as_os_str());
        assert!(factory.take_initial_errors().is_empty());
        let cli = Command::new(env!("CARGO_BIN_EXE_gz-msgs"))
            .args(["--info", "ranges.Root"])
            .env("GZ_DESCRIPTOR_PATH", &input)
            .output()
            .unwrap();
        if row["expected"] == "native" {
            supported += 1;
            let expected = body(row);
            assert_eq!(
                registry.definition("ranges.Root").unwrap(),
                Some(expected.clone()),
                "{}",
                row["id"]
            );
            assert_eq!(
                factory.registry_definition("ranges.Root").unwrap(),
                Some(expected.clone()),
                "{}",
                row["id"]
            );
            assert_eq!(
                descriptor_definition(&descriptor).unwrap(),
                expected,
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
            let expected = DefinitionError::Unsupported(match row["expected"].as_str().unwrap() {
                "invalid extension range" => "invalid extension range",
                "unknown options" => "unknown options",
                other => panic!("unexpected disposition {other}"),
            });
            assert_eq!(
                registry.definition("ranges.Root"),
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
                factory.registry_definition("ranges.Root"),
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
            descriptor.parent_file().encode_to_vec(),
            options_before,
            "{}",
            row["id"]
        );
    }
    (supported, native_accepted, loader_rejected)
}

#[test]
fn ordinary_extension_ranges_match_native_or_explicit_validation_boundaries() {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/extension-ranges.json")).unwrap();
    let rows = data["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 123);
    assert_eq!(check_cases(rows), (34, 37, 11));
}

#[test]
fn exact_historical_extension_range_input_retains_full_native_positives() {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/definitions/cli.json")).unwrap();
    let row = data["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| {
            row["args"]
                .as_array()
                .is_some_and(|args| args.iter().any(|arg| arg == "definition.legacy.Legacy"))
        })
        .unwrap();
    let input = Path::new("tests/fixtures/definitions/legacy.desc");
    let directory = tempfile::tempdir().unwrap();
    let peer = native_peer(directory.path());
    let actual = Command::new(peer).arg(input).output().unwrap();
    assert!(actual.status.success(), "{actual:?}");
    assert!(actual.stderr.is_empty());
    let output = String::from_utf8(actual.stdout).unwrap();
    let definition = output
        .lines()
        .find_map(|line| {
            let (name, hex) = line.split_once(' ')?;
            (name == "definition.legacy.Legacy").then(|| String::from_utf8(unhex(hex)).unwrap())
        })
        .unwrap();
    let mut registry = DescriptorRegistry::default();
    assert!(registry.load_file(input).unwrap().is_empty());
    let descriptor = registry
        .pool()
        .get_message_by_name("definition.legacy.Legacy")
        .unwrap();
    assert_eq!(descriptor_definition(&descriptor).unwrap(), definition);
    assert_eq!(
        registry.definition("definition.legacy.Legacy").unwrap(),
        Some(definition.clone())
    );
    let factory = MessageFactory::with_paths(input.as_os_str());
    assert!(factory.take_initial_errors().is_empty());
    assert_eq!(
        factory
            .registry_definition("definition.legacy.Legacy")
            .unwrap(),
        Some(definition)
    );
    let cli = Command::new(env!("CARGO_BIN_EXE_gz-msgs"))
        .args(["--info", "definition.legacy.Legacy"])
        .env("GZ_DESCRIPTOR_PATH", input)
        .output()
        .unwrap();
    assert_eq!(
        cli.status.code(),
        row["status"].as_i64().map(|value| value as i32)
    );
    assert_eq!(cli.stdout, row["stdout"].as_str().unwrap().as_bytes());
    assert_eq!(cli.stderr, row["stderr"].as_str().unwrap().as_bytes());
}

// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use gz_msgs::{
    factory::MessageFactory,
    prost::Message,
    prost_reflect::Value,
    prost_types::{
        field_descriptor_proto::{Label, Type},
        DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet,
    },
    registry::DescriptorRegistry,
};
use std::{path::PathBuf, process::Command};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "gz-byte-defaults-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn field(default: &str) -> FieldDescriptorProto {
    FieldDescriptorProto {
        name: Some("value".into()),
        number: Some(1),
        r#type: Some(Type::Bytes as i32),
        label: Some(Label::Optional as i32),
        default_value: Some(default.into()),
        ..Default::default()
    }
}
fn file(default: &str) -> FileDescriptorProto {
    FileDescriptorProto {
        name: Some("bad-default.proto".into()),
        syntax: Some("proto2".into()),
        message_type: vec![DescriptorProto {
            name: Some("Value".into()),
            field: vec![field(default)],
            ..Default::default()
        }],
        ..Default::default()
    }
}
fn encode(files: Vec<FileDescriptorProto>) -> Vec<u8> {
    FileDescriptorSet { file: files }.encode_to_vec()
}
fn unhex(text: &str) -> Vec<u8> {
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
fn default_bytes(registry: &DescriptorRegistry) -> Vec<u8> {
    let descriptor = registry.pool().get_message_by_name("Value").unwrap();
    let Value::Bytes(bytes) = descriptor
        .get_field_by_name("value")
        .unwrap()
        .default_value()
    else {
        panic!("bytes field")
    };
    bytes.to_vec()
}

#[test]
fn native_defaults_retain_exact_inputs_and_explicit_rust_dispositions() {
    let directory = Directory::new();
    let peer = directory.0.join("peer");
    let build = Command::new("c++")
        .args([
            "-std=c++17",
            "-Wall",
            "-Wextra",
            "-Werror",
            "tests/byte_defaults_peer.cc",
            "-lprotobuf",
            "-o",
        ])
        .arg(&peer)
        .output()
        .unwrap();
    assert!(build.status.success(), "{build:?}");
    let fixture: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/byte-defaults/native.json")).unwrap();
    for row in fixture["cases"].as_array().unwrap() {
        let default = row["default"].as_str().unwrap();
        let bytes = unhex(row["descriptor_hex"].as_str().unwrap());
        assert_eq!(bytes, encode(vec![file(default)]));
        let path = directory.0.join("input.desc");
        std::fs::write(&path, &bytes).unwrap();
        let native = Command::new(&peer).arg(&path).output().unwrap();
        assert_eq!(
            native.status.code(),
            row["native_default"]["status"].as_i64().map(|v| v as i32)
        );
        assert_eq!(
            native.stdout,
            unhex(row["native_default"]["bytes_hex"].as_str().unwrap())
        );
        assert!(native.stderr.is_empty());
        let mut registry = DescriptorRegistry::default();
        if row["rust_disposition"] == "DecodeError" {
            let error: gz_msgs::prost::DecodeError = registry.load_bytes(&bytes).unwrap_err();
            assert!(error.to_string().contains("byte default"));
            assert!(registry.pool().get_message_by_name("Value").is_none());
        } else {
            assert!(registry.load_bytes(&bytes).unwrap().is_empty());
            assert_eq!(default_bytes(&registry), native.stdout);
        }
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}

#[test]
fn ordinary_valid_and_invalid_escape_dispositions_stay_with_the_loader() {
    for (default, expected) in [
        ("", vec![]),
        ("é", "é".as_bytes().to_vec()),
        (r"\0\07\3777", vec![0, 7, 255, b'7']),
        (r"\x00\Xff", vec![0, 255]),
        (
            r#"\a\b\f\n\r\t\v\\\?\'\""#,
            vec![7, 8, 12, 10, 13, 9, 11, 92, 63, 39, 34],
        ),
        (r"\\777", br"\777".to_vec()),
        (r"\x31é", "1é".as_bytes().to_vec()),
    ] {
        let mut registry = DescriptorRegistry::default();
        assert!(
            registry
                .load_bytes(&encode(vec![file(default)]))
                .unwrap()
                .is_empty(),
            "{default:?}"
        );
        assert_eq!(default_bytes(&registry), expected, "{default:?}");
        assert_eq!(
            registry
                .pool()
                .get_message_by_name("Value")
                .unwrap()
                .get_field_by_name("value")
                .unwrap()
                .field_descriptor_proto()
                .default_value
                .as_deref(),
            Some(default)
        );
    }
    // The dependency stops at these errors before reaching the unsafe suffix.
    for default in ["\\", r"\x", r"\x1", r"\xzz\777", r"\q\777", r"\xé\777"] {
        let mut registry = DescriptorRegistry::default();
        assert_eq!(
            registry
                .load_bytes(&encode(vec![file(default)]))
                .unwrap()
                .len(),
            1,
            "{default:?}"
        );
    }
    let mut string = file(r"\777\x€");
    string.message_type[0].field[0].r#type = Some(Type::String as i32);
    assert!(DescriptorRegistry::default()
        .load_bytes(&encode(vec![string]))
        .unwrap()
        .is_empty());
    let mut no_default = file("");
    no_default.message_type[0].field[0].default_value = None;
    assert!(DescriptorRegistry::default()
        .load_bytes(&encode(vec![no_default]))
        .unwrap()
        .is_empty());
}

#[test]
fn nested_extensions_duplicates_partial_progress_and_retry() {
    for placement in 0..3 {
        let mut candidate = file("valid");
        match placement {
            0 => candidate.extension.push(field(r"\777")),
            1 => candidate.message_type[0].extension.push(field(r"\777")),
            _ => candidate.message_type[0].nested_type.push(DescriptorProto {
                name: Some("Nested".into()),
                field: vec![field(r"\x€")],
                ..Default::default()
            }),
        }
        assert!(DescriptorRegistry::default()
            .load_bytes(&encode(vec![candidate]))
            .is_err());
    }
    let mut registry = DescriptorRegistry::default();
    let valid = file("valid");
    let mut unsafe_duplicate = file(r"\777");
    unsafe_duplicate.message_type[0].name = Some("Replacement".into());
    assert!(registry
        .load_bytes(&encode(vec![valid.clone(), unsafe_duplicate]))
        .unwrap()
        .is_empty());
    assert_eq!(default_bytes(&registry), b"valid");
    assert!(registry.pool().get_message_by_name("Replacement").is_none());
    let snapshot = registry.clone();
    let mut failed = file(r"\x1é");
    failed.name = Some("later.proto".into());
    failed.message_type[0].name = Some("Later".into());
    let mut fresh = DescriptorRegistry::default();
    assert!(fresh
        .load_bytes(&encode(vec![valid, failed.clone()]))
        .is_err());
    assert_eq!(default_bytes(&fresh), b"valid");
    assert!(fresh.pool().get_message_by_name("Later").is_none());
    failed.message_type[0].field[0].default_value = Some(r"\377".into());
    assert!(fresh.load_bytes(&encode(vec![failed])).unwrap().is_empty());
    assert!(fresh.pool().get_message_by_name("Later").is_some());
    assert!(snapshot.pool().get_message_by_name("Later").is_none());
}

#[test]
fn file_factory_and_cli_paths_report_or_skip_bad_files_without_poisoning() {
    let directory = Directory::new();
    for default in [r"\400", r"\777", r"\x€", r"\x1é"] {
        let path = directory.0.join("bad.desc");
        let bytes = encode(vec![file(default)]);
        std::fs::write(&path, &bytes).unwrap();
        assert_eq!(
            DescriptorRegistry::default()
                .load_file(&path)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::InvalidData
        );
        let factory = MessageFactory::with_paths(path.as_os_str());
        assert!(factory.new_message("Value").is_none());
        let errors = factory.take_initial_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].0, path);
        assert_eq!(errors[0].1.kind(), std::io::ErrorKind::InvalidData);
        assert!(factory.new_message("gz.msgs.Time").is_some());
        assert!(!factory.types().contains(&"Value".to_owned()));
        for args in [vec!["list"], vec!["info", "Value", "gz.msgs.Time"]] {
            let output = Command::new(env!("CARGO_BIN_EXE_gz-msgs"))
                .args(&args)
                .env("GZ_DESCRIPTOR_PATH", &path)
                .output()
                .unwrap();
            assert!(output.status.success(), "{output:?}");
            if args[0] == "list" {
                assert!(output.stderr.is_empty());
                assert!(!String::from_utf8(output.stdout)
                    .unwrap()
                    .lines()
                    .any(|line| line == "Value"));
            } else {
                assert_eq!(output.stderr, b"Unable to create message of type[Value]\n");
                assert!(String::from_utf8(output.stdout)
                    .unwrap()
                    .contains("Name: gz.msgs.Time"));
            }
        }
        // An explicit retry succeeds in the same factory, proving its lock survives.
        std::fs::write(&path, encode(vec![file("fixed")])).unwrap();
        assert!(factory.load_paths(path.as_os_str()).is_empty());
        assert!(factory.new_message("Value").is_some());
        assert_eq!(bytes, encode(vec![file(default)]));
    }
}

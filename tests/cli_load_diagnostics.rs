// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#![cfg(target_os = "linux")]
use gz_msgs::{
    factory::MessageFactory, prost::Message, prost_types as d, registry::LoadDiagnostic,
};
use std::{ffi::OsString, io, os::unix::ffi::OsStringExt, path::PathBuf, process::Command};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "gz-cli-load-errors-{}-{}",
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
fn unhex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let value = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(value, 16).unwrap()
        })
        .collect()
}

#[test]
fn native_cli_file_diagnostics_keep_bytes_order_exits_and_lazy_shortcuts() {
    let fixture: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/cli-load-diagnostics.json")).unwrap();
    let directory = Directory::new();
    for (name, bytes) in fixture["inputs_hex"].as_object().unwrap() {
        std::fs::write(directory.0.join(name), unhex(bytes.as_str().unwrap())).unwrap();
    }
    std::fs::create_dir(directory.0.join("directory.desc")).unwrap();
    std::os::unix::fs::symlink("/proc/self/mem", directory.0.join("read-error.desc")).unwrap();
    // The existing CLI keeps Rust-owned help prose; only lazy behavior is compared here.
    let help = Command::new(env!("CARGO_BIN_EXE_gz-msgs"))
        .arg("--help")
        .env_remove("GZ_DESCRIPTOR_PATH")
        .env("GZ_MSGS_INSTALL_PREFIX", directory.0.join("no-prefix"))
        .output()
        .unwrap();
    assert!(help.status.success() && help.stderr.is_empty());
    let rows = fixture["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 80);
    let mut complete = 0;
    let mut known_differences = 0;
    let mut wire_differences = 0;
    for row in rows {
        let case = row["case"].as_str().unwrap();
        let label = row["command"].as_str().unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_gz-msgs"));
        for value in row["args_hex"].as_array().unwrap() {
            command.arg(OsString::from_vec(unhex(value.as_str().unwrap())));
        }
        let output = command
            .current_dir(&directory.0)
            .env(
                "GZ_DESCRIPTOR_PATH",
                OsString::from_vec(unhex(row["descriptor_path_hex"].as_str().unwrap())),
            )
            .env("GZ_MSGS_INSTALL_PREFIX", directory.0.join("no-prefix"))
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(row["exit"].as_i64().unwrap() as i32),
            "{case}/{label}"
        );
        let native_stdout = unhex(row["stdout_hex"].as_str().unwrap());
        if label == "list" {
            // Existing CLI contract: Rust sorts names; native list order is unstable.
            let names = |bytes: &[u8]| {
                std::str::from_utf8(bytes)
                    .unwrap()
                    .lines()
                    .map(str::to_owned)
                    .collect::<std::collections::BTreeSet<_>>()
            };
            let mut expected = names(&native_stdout);
            if case == "invalid-name" {
                // Existing loader admission difference, outside console parity.
                assert!(expected.insert("errors.bad-name".into()));
            }
            assert_eq!(names(&output.stdout), expected, "{case}/{label}");
            let lines: Vec<_> = std::str::from_utf8(&output.stdout)
                .unwrap()
                .lines()
                .collect();
            assert!(lines.windows(2).all(|pair| pair[0] < pair[1]));
        } else if label == "help" {
            assert_eq!(output.stdout, help.stdout, "{case}/{label}");
        } else {
            assert_eq!(output.stdout, native_stdout, "{case}/{label}");
        }
        if case.starts_with("wrong-wire-") && ["list", "missing-info"].contains(&label) {
            // Preserve the existing strict descriptor-wire decoder without claiming native parity.
            let native = unhex(row["stderr_hex"].as_str().unwrap());
            let missing = if label == "missing-info" {
                b"Unable to create message of type[errors.Missing]\n".as_slice()
            } else {
                b""
            };
            assert_eq!(native, missing, "{case}/{label}");
            let mut expected = b"DynamicFactory(): Unable to parse descriptor set from [".to_vec();
            expected.extend(unhex(row["descriptor_path_hex"].as_str().unwrap()));
            expected.extend_from_slice(b"]\n");
            expected.extend_from_slice(missing);
            assert_eq!(output.stderr, expected, "{case}/{label}");
            wire_differences += 1;
        } else if ["invalid-name", "invalid-field", "missing-import"].contains(&case)
            && ["list", "missing-info"].contains(&label)
        {
            // Full libprotobuf schema-validation console text is a retained gap.
            let native = unhex(row["stderr_hex"].as_str().unwrap());
            assert!(native.starts_with(b"[libprotobuf ERROR "));
            assert!(native
                .windows(b"Unable to place descriptors".len())
                .any(|bytes| bytes == b"Unable to place descriptors"));
            let expected = if label == "missing-info" {
                b"Unable to create message of type[errors.Missing]\n".as_slice()
            } else {
                b""
            };
            assert_eq!(output.stderr, expected, "{case}/{label}");
            known_differences += 1;
        } else {
            assert_eq!(
                output.stderr,
                unhex(row["stderr_hex"].as_str().unwrap()),
                "{case}/{label}"
            );
            complete += 1;
        }
    }
    assert_eq!((complete, known_differences, wire_differences), (68, 6, 6));
}

#[test]
fn structured_initial_errors_retain_stage_and_original_errors_without_lazy_io() {
    let directory = Directory::new();
    let missing = directory.0.join("missing.desc");
    let malformed = directory.0.join("malformed.desc");
    std::fs::write(&malformed, [0x0a, 0x05, b'x']).unwrap();
    let schema = directory.0.join("schema.desc");
    let guarded = directory.0.join("guarded.desc");
    let file = |name: &str, default: Option<&str>| {
        d::FileDescriptorSet {
            file: vec![d::FileDescriptorProto {
                name: Some(name.into()),
                package: Some("errors".into()),
                syntax: Some("proto2".into()),
                message_type: vec![d::DescriptorProto {
                    name: Some(
                        if default.is_some() {
                            "Guarded"
                        } else {
                            "Invalid"
                        }
                        .into(),
                    ),
                    field: default
                        .map(|value| {
                            vec![d::FieldDescriptorProto {
                                name: Some("value".into()),
                                number: Some(1),
                                label: Some(d::field_descriptor_proto::Label::Optional as i32),
                                r#type: Some(d::field_descriptor_proto::Type::Bytes as i32),
                                default_value: Some(value.into()),
                                ..Default::default()
                            }]
                        })
                        .unwrap_or_else(|| {
                            vec![d::FieldDescriptorProto {
                                name: Some("value".into()),
                                number: Some(0),
                                label: Some(1),
                                r#type: Some(5),
                                ..Default::default()
                            }]
                        }),
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
        .encode_to_vec()
    };
    std::fs::write(&schema, file("schema.proto", None)).unwrap();
    std::fs::write(&guarded, file("guarded.proto", Some("\\777"))).unwrap();
    let factory = MessageFactory::with_paths(
        std::env::join_paths([&missing, &malformed, &schema, &guarded]).unwrap(),
    );
    assert!(factory.take_initial_errors_detailed().is_empty());
    assert!(factory.take_initial_errors().is_empty());
    factory.types();
    let errors = factory.take_initial_errors_detailed();
    assert_eq!(errors.len(), 4);
    assert_eq!(
        errors
            .iter()
            .map(|error| error.diagnostic)
            .collect::<Vec<_>>(),
        [
            Some(LoadDiagnostic::Open),
            Some(LoadDiagnostic::Parse),
            None,
            None
        ]
    );
    assert_eq!(errors[0].error.kind(), io::ErrorKind::NotFound);
    assert!(errors[1]
        .error
        .get_ref()
        .unwrap()
        .is::<gz_msgs::prost::DecodeError>());
    assert!(errors[2]
        .error
        .get_ref()
        .unwrap()
        .is::<gz_msgs::prost_reflect::DescriptorError>());
    assert!(errors[3]
        .error
        .get_ref()
        .unwrap()
        .is::<gz_msgs::prost::DecodeError>());
    assert!(factory.take_initial_errors().is_empty());
    assert!(factory.take_initial_errors_detailed().is_empty());
    assert_eq!(factory.load_paths(missing.as_os_str()).len(), 1);
}

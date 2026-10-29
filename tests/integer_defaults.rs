// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use gz_msgs::{
    factory::MessageFactory,
    prost::Message,
    prost_reflect::{DescriptorPool, DynamicMessage, Value},
    prost_types::{
        field_descriptor_proto::{Label, Type},
        DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet,
    },
    registry::DescriptorRegistry,
    serde_json,
};
use std::{path::PathBuf, process::Command};
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "gz-integer-defaults-{}-{}",
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
        let result = Command::new("c++")
            .args([
                "-std=c++17",
                "-Wall",
                "-Wextra",
                "-Werror",
                "tests/integer_defaults_peer.cc",
                "-lprotobuf",
                "-o",
            ])
            .arg(&path)
            .output()
            .unwrap();
        assert!(result.status.success(), "{result:?}");
        path
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!("fixtures/integer-defaults/native.json")).unwrap()
}
fn native_integer_definition(row: &serde_json::Value) -> String {
    static OBSERVED: std::sync::OnceLock<serde_json::Value> = std::sync::OnceLock::new();
    let observed = OBSERVED.get_or_init(|| {
        let mut observed: serde_json::Value = serde_json::from_str(include_str!(
            "fixtures/integer-definitions/existing-396.json"
        ))
        .unwrap();
        let remaining: serde_json::Value = serde_json::from_str(include_str!(
            "fixtures/integer-definitions/remaining-594.json"
        ))
        .unwrap();
        observed["cases"]
            .as_array_mut()
            .unwrap()
            .extend(remaining["cases"].as_array().unwrap().iter().cloned());
        observed
    });
    let cases = observed["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 990);
    let native = cases
        .iter()
        .find(|case| case["input"]["id"] == row["id"])
        .unwrap();
    assert_eq!(&native["input"], row);
    assert_eq!(native["descriptor_hex"], row["descriptor_hex"]);
    assert!(native["observation"]["admitted"].as_bool().unwrap());
    String::from_utf8(unhex(
        native["observation"]["fields"]["DEFINITION_HEX"]
            .as_str()
            .unwrap(),
    ))
    .unwrap()
}
fn unhex(text: &str) -> Vec<u8> {
    text.as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn number(value: Value) -> String {
    match value {
        Value::I32(v) => v.to_string(),
        Value::I64(v) => v.to_string(),
        Value::U32(v) => v.to_string(),
        Value::U64(v) => v.to_string(),
        other => panic!("expected integer, got {other:?}"),
    }
}
fn file(kind: Type, default: &str) -> FileDescriptorProto {
    FileDescriptorProto {
        name: Some("defaults.proto".into()),
        syntax: Some("proto2".into()),
        message_type: vec![DescriptorProto {
            name: Some("Default".into()),
            field: vec![FieldDescriptorProto {
                name: Some("value".into()),
                number: Some(1),
                label: Some(Label::Optional as i32),
                r#type: Some(kind as i32),
                default_value: Some(default.into()),
                json_name: Some("value".into()),
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    }
}
fn encode(files: Vec<FileDescriptorProto>) -> Vec<u8> {
    FileDescriptorSet { file: files }.encode_to_vec()
}

#[test]
fn exact_native_integer_corpus_matches_effective_values_wire_and_raw_spelling() {
    let directory = Directory::new();
    let peer = directory.native();
    let fixture = fixture();
    let platform = Command::new(&peer).arg("--platform").output().unwrap();
    assert!(platform.status.success());
    assert_eq!(
        String::from_utf8(platform.stdout).unwrap().trim(),
        fixture["platform"].as_str().unwrap()
    );
    let rows = fixture["cases"].as_array().unwrap();
    assert_eq!(rows.len(), 990);
    let study: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/integer-defaults/study-190.json")).unwrap();
    let legacy: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/integer-defaults/legacy-190.json")).unwrap();
    for row in rows {
        let default = row["input"].as_str().unwrap();
        let bytes = unhex(row["descriptor_hex"].as_str().unwrap());
        let kind = Type::try_from(row["kind"].as_i64().unwrap() as i32).unwrap();
        assert_eq!(bytes, encode(vec![file(kind, default)]));
        let id = row["id"].as_u64().unwrap() as usize;
        if id < 190 {
            assert_eq!(row["input"], study[id]["input"]);
            assert_eq!(row["kind"], study[id]["kind"]);
            assert_eq!(row["descriptor_sha256"], study[id]["descriptor_sha256"]);
            assert_eq!(row["status"], study[id]["status"]);
            assert_eq!(row["input"], legacy[id]["input"]);
            assert_eq!(row["kind"], legacy[id]["kind"]);
            match DescriptorPool::from_file_descriptor_set(
                FileDescriptorSet::decode(bytes.as_slice()).unwrap(),
            ) {
                Ok(pool) => {
                    assert_eq!(legacy[id]["accepted"], true);
                    let value = pool
                        .get_message_by_name("Default")
                        .unwrap()
                        .get_field_by_name("value")
                        .unwrap()
                        .default_value();
                    assert_eq!(number(value), legacy[id]["value"].as_str().unwrap());
                }
                Err(error) => {
                    assert_eq!(legacy[id]["accepted"], false);
                    assert_eq!(error.to_string(), legacy[id]["error"].as_str().unwrap());
                }
            }
        }
        let path = directory.0.join("input.desc");
        std::fs::write(&path, &bytes).unwrap();
        let native = Command::new(&peer)
            .arg(&path)
            .env("LC_ALL", "C")
            .output()
            .unwrap();
        assert_eq!(
            native.status.code(),
            Some(row["status"].as_i64().unwrap() as i32),
            "{row}"
        );
        assert_eq!(
            native.stdout,
            unhex(row["stdout_hex"].as_str().unwrap()),
            "{row}"
        );
        assert_eq!(
            native.stderr.is_empty(),
            row["stderr_hex"].as_str().unwrap().is_empty()
        );
        let mut registry = DescriptorRegistry::default();
        let errors = registry.load_bytes(&bytes).unwrap();
        if native.status.success() {
            assert!(errors.is_empty(), "{row}: {errors:?}");
            let descriptor = registry.pool().get_message_by_name("Default").unwrap();
            let field = descriptor.get_field_by_name("value").unwrap();
            let stdout = String::from_utf8(native.stdout).unwrap();
            let (expected_value, wire) = stdout.trim_end_matches('\n').split_once('\n').unwrap();
            assert_eq!(number(field.default_value()), expected_value, "{row}");
            assert_eq!(
                field.field_descriptor_proto().default_value.as_deref(),
                Some(default)
            );
            assert_eq!(
                descriptor.parent_file().encode_to_vec(),
                FileDescriptorSet::decode(bytes.as_slice()).unwrap().file[0].encode_to_vec()
            );
            let exported =
                FileDescriptorProto::decode(descriptor.parent_file().encode_to_vec().as_slice())
                    .unwrap();
            assert_eq!(
                exported.message_type[0].field[0].default_value.as_deref(),
                Some(default)
            );
            let mut message = DynamicMessage::decode(descriptor.clone(), &[][..]).unwrap();
            assert!(!message.has_field(&field));
            assert!(message.encode_to_vec().is_empty());
            assert_eq!(
                number(message.get_field(&field).into_owned()),
                expected_value
            );
            message.set_field(&field, field.default_value());
            assert!(message.has_field(&field));
            assert_eq!(message.encode_to_vec(), unhex(wire), "{row}");
            assert_eq!(
                DynamicMessage::decode(descriptor, unhex(wire).as_slice()).unwrap(),
                message
            );
            assert_eq!(
                registry.definition("Default").unwrap(),
                Some(native_integer_definition(row))
            );
        } else {
            assert_eq!(errors.len(), 1, "{row}");
            let _: &gz_msgs::prost_reflect::DescriptorError = &errors[0];
            assert!(registry.pool().get_message_by_name("Default").is_none());
        }
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}

#[test]
fn factory_empty_decode_paths_observe_native_integer_defaults_and_errors() {
    let directory = Directory::new();
    for row in fixture()["cases"].as_array().unwrap().iter().filter(|row| {
        [
            "010",
            "0x2a",
            "2147483648",
            "-1",
            "18446744073709551616",
            "\0junk",
            "09",
        ]
        .contains(&row["input"].as_str().unwrap())
    }) {
        let path = directory.0.join("input.desc");
        std::fs::write(&path, unhex(row["descriptor_hex"].as_str().unwrap())).unwrap();
        let factory = MessageFactory::with_paths(path.as_os_str());
        if row["status"] == 0 {
            let message = factory.new_message("Default").unwrap();
            let expected = String::from_utf8(unhex(row["stdout_hex"].as_str().unwrap())).unwrap();
            let expected = expected.lines().next().unwrap();
            assert_eq!(
                number(message.get_field_by_name("value").unwrap().into_owned()),
                expected
            );
            assert_eq!(factory.new_text("Default", "").unwrap().unwrap(), message);
            assert!(factory.types().contains(&"Default".to_owned()));
            assert!(factory.take_initial_errors().is_empty());
            assert_eq!(
                factory.registry_definition("Default").unwrap(),
                Some(native_integer_definition(row))
            );
        } else {
            assert!(factory.new_message("Default").is_none());
            let errors = factory.take_initial_errors();
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].1.kind(), std::io::ErrorKind::InvalidData);
        }
        assert!(factory.new_message("gz.msgs.Time").is_some());
    }
    for (default, success, expected_stderr) in [
        ("0x2a", true, ""),
        ("09", true, "Unable to create message of type[Default]"),
    ] {
        let path = directory.0.join("cli.desc");
        std::fs::write(&path, encode(vec![file(Type::Int32, default)])).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_gz-msgs"))
            .args(["info", "Default"])
            .env("GZ_DESCRIPTOR_PATH", &path)
            .output()
            .unwrap();
        assert_eq!(output.status.success(), success, "{output:?}");
        assert!(std::str::from_utf8(&output.stderr)
            .unwrap()
            .contains(expected_stderr));
        if default == "0x2a" {
            let row = fixture()["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["kind"] == 5 && row["input"] == default)
                .unwrap()
                .clone();
            let native = native_integer_definition(&row);
            assert!(output.stderr.is_empty());
            assert_eq!(
                output.stdout,
                format!("Name: Default\nFile: defaults.proto\n\n{native}\n").as_bytes()
            );
        } else {
            assert!(output.stdout.is_empty());
        }
    }
}

#[test]
fn opt_in_keeps_standard_pool_extensions_proto3_and_other_scalar_rules() {
    let mut standard = DescriptorPool::new();
    standard
        .add_file_descriptor_proto(file(Type::Int32, "010"))
        .unwrap();
    assert_eq!(
        standard
            .get_message_by_name("Default")
            .unwrap()
            .get_field_by_name("value")
            .unwrap()
            .default_value(),
        Value::I32(10)
    );
    assert!(DescriptorPool::new()
        .add_file_descriptor_proto(file(Type::Int32, "0x2a"))
        .is_err());
    assert!(DescriptorPool::new()
        .decode_file_descriptor_proto_with_integer_defaults(&[255][..])
        .is_err());
    let mut proto3 = file(Type::Int32, "010");
    proto3.syntax = Some("proto3".into());
    let mut registry = DescriptorRegistry::default();
    assert!(registry
        .load_bytes(&encode(vec![proto3]))
        .unwrap()
        .is_empty());
    assert_eq!(
        registry
            .pool()
            .get_message_by_name("Default")
            .unwrap()
            .get_field_by_name("value")
            .unwrap()
            .default_value(),
        Value::I32(10)
    );
    for (kind, default, expected) in [
        (Type::Bool, "true", Value::Bool(true)),
        (Type::Float, "1.25", Value::F32(1.25)),
        (Type::Double, "1.25", Value::F64(1.25)),
        (Type::String, "é", Value::String("é".into())),
        (Type::Bytes, r"\377", Value::Bytes(vec![255].into())),
    ] {
        let mut registry = DescriptorRegistry::default();
        assert!(registry
            .load_bytes(&encode(vec![file(kind, default)]))
            .unwrap()
            .is_empty());
        assert_eq!(
            registry
                .pool()
                .get_message_by_name("Default")
                .unwrap()
                .get_field_by_name("value")
                .unwrap()
                .default_value(),
            expected
        );
    }
    let mut extension = FileDescriptorProto {
        name: Some("extension.proto".into()),
        package: Some("ext".into()),
        syntax: Some("proto2".into()),
        message_type: vec![DescriptorProto {
            name: Some("Base".into()),
            extension_range: vec![gz_msgs::prost_types::descriptor_proto::ExtensionRange {
                start: Some(100),
                end: Some(200),
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut raw = file(Type::Int32, "010")
        .message_type
        .remove(0)
        .field
        .remove(0);
    raw.name = Some("value".into());
    raw.number = Some(101);
    raw.extendee = Some(".ext.Base".into());
    extension.extension.push(raw);
    let mut registry = DescriptorRegistry::default();
    assert!(registry
        .load_bytes(&encode(vec![extension.clone()]))
        .unwrap()
        .is_empty());
    let value = registry.pool().get_extension_by_name("ext.value").unwrap();
    assert_eq!(value.default_value(), Value::I32(10));
    assert_eq!(
        value.field_descriptor_proto().default_value.as_deref(),
        Some("010")
    );
    extension.message_type.clear();
    extension.dependency = vec!["extension.proto".into()];
    extension.name = Some("hex-extension.proto".into());
    extension.extension[0].name = Some("other".into());
    extension.extension[0].number = Some(102);
    extension.extension[0].default_value = Some("0x2a".into());
    assert_eq!(
        registry.load_bytes(&encode(vec![extension])).unwrap().len(),
        1
    );
}

#[test]
fn ordered_errors_duplicates_snapshots_and_import_retry_keep_raw_defaults() {
    let first = file(Type::Int32, "010");
    let mut invalid = file(Type::Int32, "09");
    invalid.name = Some("invalid.proto".into());
    invalid.message_type[0].name = Some("Invalid".into());
    let mut later = file(Type::Uint64, "-1");
    later.name = Some("later.proto".into());
    later.message_type[0].name = Some("Later".into());
    let mut registry = DescriptorRegistry::default();
    assert!(registry
        .load_bytes(&encode(vec![first.clone()]))
        .unwrap()
        .is_empty());
    let snapshot = registry.clone();
    assert_eq!(
        registry
            .load_bytes(&encode(vec![invalid, later]))
            .unwrap()
            .len(),
        1
    );
    assert!(registry.pool().get_message_by_name("Later").is_some());
    assert!(snapshot.pool().get_message_by_name("Later").is_none());
    let mut duplicate = first.clone();
    duplicate.message_type[0].field[0].default_value = Some("09".into());
    assert!(registry
        .load_bytes(&encode(vec![duplicate]))
        .unwrap()
        .is_empty());
    let field = registry
        .pool()
        .get_message_by_name("Default")
        .unwrap()
        .get_field_by_name("value")
        .unwrap();
    assert_eq!(field.default_value(), Value::I32(8));
    assert_eq!(
        field.field_descriptor_proto().default_value.as_deref(),
        Some("010")
    );
    let mut dependent = first;
    dependent.name = Some("dependent.proto".into());
    dependent.dependency = vec!["dependency.proto".into()];
    dependent.message_type[0].name = Some("Dependent".into());
    assert_eq!(
        registry
            .load_bytes(&encode(vec![dependent.clone()]))
            .unwrap()
            .len(),
        1
    );
    assert!(registry.pool().get_message_by_name("Dependent").is_none());
    let dependency = FileDescriptorProto {
        name: Some("dependency.proto".into()),
        syntax: Some("proto2".into()),
        ..Default::default()
    };
    assert!(registry
        .load_bytes(&encode(vec![dependency, dependent]))
        .unwrap()
        .is_empty());
    let field = registry
        .pool()
        .get_message_by_name("Dependent")
        .unwrap()
        .get_field_by_name("value")
        .unwrap();
    assert_eq!(field.default_value(), Value::I32(8));
    assert_eq!(
        field.field_descriptor_proto().default_value.as_deref(),
        Some("010")
    );
}

#[test]
fn native_shape_controls_keep_rejected_labels_on_the_legacy_path() {
    let directory = Directory::new();
    let peer = directory.native();
    let rows: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/integer-defaults/shapes.json")).unwrap();
    for row in rows.as_array().unwrap() {
        let bytes = unhex(row["descriptor_hex"].as_str().unwrap());
        let path = directory.0.join("shape.desc");
        std::fs::write(&path, &bytes).unwrap();
        let native = Command::new(&peer)
            .arg(&path)
            .env("LC_ALL", "C")
            .output()
            .unwrap();
        assert_eq!(
            native.status.code(),
            Some(row["status"].as_i64().unwrap() as i32)
        );
        assert_eq!(native.stdout, unhex(row["stdout_hex"].as_str().unwrap()));
        let set = FileDescriptorSet::decode(bytes.as_slice()).unwrap();
        let mut registry = DescriptorRegistry::default();
        let errors = registry.load_bytes(&bytes).unwrap();
        if native.status.success() {
            assert!(errors.is_empty());
            let descriptor = registry.pool().get_message_by_name("Default").unwrap();
            let field = descriptor.get_field_by_name("value").unwrap();
            let output = String::from_utf8(native.stdout).unwrap();
            let mut parts = output.lines();
            assert_eq!(number(field.default_value()), parts.next().unwrap());
            assert_eq!(
                field.field_descriptor_proto().default_value.as_deref(),
                row["literal"].as_str()
            );
            let mut message = DynamicMessage::new(descriptor);
            message.set_field(&field, field.default_value());
            assert_eq!(message.encode_to_vec(), unhex(parts.next().unwrap()));
        } else {
            // Native rejects these shapes. Keep reflection's previous disposition
            // rather than making this integer correction a structural validator.
            match DescriptorPool::from_file_descriptor_set(set) {
                Ok(legacy) => {
                    assert!(errors.is_empty());
                    let legacy_field = legacy
                        .get_message_by_name("Default")
                        .unwrap()
                        .get_field_by_name("value")
                        .unwrap();
                    let field = registry
                        .pool()
                        .get_message_by_name("Default")
                        .unwrap()
                        .get_field_by_name("value")
                        .unwrap();
                    assert_eq!(field.default_value(), legacy_field.default_value());
                    assert_eq!(
                        field.field_descriptor_proto().default_value,
                        legacy_field.field_descriptor_proto().default_value
                    );
                }
                Err(_) => assert_eq!(errors.len(), 1),
            }
        }
    }
}

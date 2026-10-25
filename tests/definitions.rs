// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use gz_msgs::{
    descriptor_definition, prost::Message, registry::DescriptorRegistry, DefinitionError,
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "gz-definitions-{}-{}",
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
        let path = self.0.join("definition-peer");
        let result = Command::new("c++")
            .args([
                "-std=c++17",
                "-Wall",
                "-Wextra",
                "-Werror",
                "tests/definition_peer.cc",
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
fn native_definitions(peer: &Path, descriptor_set: &Path) -> BTreeMap<String, String> {
    let result = Command::new(peer).arg(descriptor_set).output().unwrap();
    assert!(result.status.success(), "{result:?}");
    assert!(result.stderr.is_empty(), "{result:?}");
    String::from_utf8(result.stdout)
        .unwrap()
        .lines()
        .map(|line| {
            let (name, hex) = line.split_once(' ').unwrap();
            let bytes: Vec<_> = hex
                .as_bytes()
                .chunks_exact(2)
                .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
                .collect();
            (name.to_owned(), String::from_utf8(bytes).unwrap())
        })
        .collect()
}

#[test]
fn descriptor_fixtures_regenerate_from_their_sources() {
    let directory = Directory::new();
    let peer = directory.native();
    for name in ["supported", "unsupported", "legacy", "reserved", "ranges"] {
        let output = directory.0.join(format!("{name}.desc"));
        let result = Command::new("protoc")
            .args([
                "--proto_path=tests/fixtures/definitions",
                "--proto_path=schemas",
                "--include_imports",
            ])
            .arg(format!("--descriptor_set_out={}", output.display()))
            .arg(format!("{name}.proto"))
            .output()
            .unwrap();
        assert!(result.status.success(), "{result:?}");
        assert_eq!(
            std::fs::read(output).unwrap(),
            std::fs::read(format!("tests/fixtures/definitions/{name}.desc")).unwrap()
        );
    }
    let output = directory.0.join("unknown.desc");
    assert!(Command::new(peer)
        .arg("tests/fixtures/definitions/supported.desc")
        .arg(&output)
        .status()
        .unwrap()
        .success());
    assert_eq!(
        std::fs::read(output).unwrap(),
        include_bytes!("fixtures/definitions/unknown-options.desc")
    );
}

#[test]
fn every_canonical_dynamic_descriptor_matches_native_definition() {
    let directory = Directory::new();
    let peer = directory.native();
    let set = gz_msgs::prost_types::FileDescriptorSet {
        file: gz_msgs::descriptors()
            .file_descriptor_protos()
            .cloned()
            .collect(),
    };
    let input = directory.0.join("canonical.desc");
    std::fs::write(&input, set.encode_to_vec()).unwrap();
    let expected = native_definitions(&peer, &input);
    assert_eq!(expected.len(), 271);
    for (name, definition) in expected {
        let descriptor = gz_msgs::descriptor(&name).unwrap();
        assert_eq!(
            descriptor_definition(&descriptor).unwrap(),
            definition,
            "{name}"
        );
    }
    // Generated C++ and dynamic descriptors differ in explicit JSON-name metadata.
    assert_ne!(
        descriptor_definition(&gz_msgs::descriptor("gz.msgs.Time").unwrap()).unwrap(),
        gz_msgs::message_definition("gz.msgs.Time").unwrap()
    );
}

#[test]
fn custom_definitions_are_native_exact_or_explicitly_unsupported() {
    let directory = Directory::new();
    let peer = directory.native();
    for file in ["supported", "unsupported", "legacy", "unknown-options"] {
        let input = PathBuf::from(format!("tests/fixtures/definitions/{file}.desc"));
        let expected = native_definitions(&peer, &input);
        let mut registry = DescriptorRegistry::default();
        assert!(registry.load_file(&input).unwrap().is_empty());
        for (name, expected) in expected
            .into_iter()
            .filter(|(name, _)| name.starts_with("definition."))
        {
            let descriptor = registry.pool().get_message_by_name(&name).unwrap();
            let actual = descriptor_definition(&descriptor);
            let feature = match name.as_str() {
                "definition.unsupported.StandardMessageOption"
                | "definition.unsupported.StandardFieldOption" => Some("option"),
                "definition.unsupported.DeclaresExtension" => Some("extension declarations"),
                "definition.legacy.Legacy" => Some("proto2"),
                name if name.starts_with("definition.unsupported.Custom") => Some("custom options"),
                "definition.example.Surface" if file == "unknown-options" => {
                    Some("unknown options")
                }
                _ => None,
            };
            if let Some(feature) = feature {
                let error = actual.unwrap_err();
                assert_eq!(error, DefinitionError::Unsupported(feature), "{name}");
                assert!(error.to_string().contains(feature));
            } else {
                assert_eq!(actual.unwrap(), expected, "{name}");
            }
        }
    }
}

#[test]
fn programmatic_unhandled_metadata_and_depth_are_not_silently_dropped() {
    use gz_msgs::{
        prost_reflect::DescriptorPool,
        prost_types::{
            DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet,
            OneofDescriptorProto,
        },
    };
    let pool = |message| {
        DescriptorPool::from_file_descriptor_set(FileDescriptorSet {
            file: vec![FileDescriptorProto {
                name: Some("programmatic.proto".into()),
                syntax: Some("proto3".into()),
                message_type: vec![message],
                ..Default::default()
            }],
        })
        .unwrap()
    };
    let directory = Directory::new();
    let peer = directory.native();
    for (label, default_value, diagnostic) in [
        (
            1,
            Some("1".into()),
            "Explicit default values are not allowed in proto3",
        ),
        (2, None, "Required fields are not allowed in proto3"),
    ] {
        let descriptor_pool = pool(DescriptorProto {
            name: Some("InvalidProto3".into()),
            field: vec![FieldDescriptorProto {
                name: Some("value".into()),
                number: Some(1),
                r#type: Some(5),
                label: Some(label),
                default_value,
                ..Default::default()
            }],
            ..Default::default()
        });
        assert_eq!(
            descriptor_definition(
                &descriptor_pool
                    .get_message_by_name("InvalidProto3")
                    .unwrap()
            ),
            Err(DefinitionError::Unsupported(
                "proto3 default or required field"
            ))
        );
        // Reflection accepts these raw descriptors; native validation rejects them.
        let input = directory.0.join("invalid.desc");
        std::fs::write(
            &input,
            FileDescriptorSet {
                file: descriptor_pool.file_descriptor_protos().cloned().collect(),
            }
            .encode_to_vec(),
        )
        .unwrap();
        let native = Command::new(&peer).arg(input).output().unwrap();
        assert_eq!(native.status.code(), Some(5));
        assert!(native.stdout.is_empty());
        assert!(String::from_utf8(native.stderr)
            .unwrap()
            .contains(diagnostic));
    }
    let empty_oneof = pool(DescriptorProto {
        name: Some("EmptyOneof".into()),
        oneof_decl: vec![OneofDescriptorProto {
            name: Some("empty".into()),
            ..Default::default()
        }],
        ..Default::default()
    });
    assert_eq!(
        descriptor_definition(&empty_oneof.get_message_by_name("EmptyOneof").unwrap()),
        Err(DefinitionError::Unsupported("empty oneof"))
    );
    let mut child = DescriptorProto {
        name: Some("Leaf".into()),
        ..Default::default()
    };
    for index in (0..100).rev() {
        child = DescriptorProto {
            name: Some(format!("Level{index}")),
            nested_type: vec![child],
            ..Default::default()
        };
    }
    let deep = pool(child);
    assert_eq!(
        descriptor_definition(&deep.get_message_by_name("Level0").unwrap()),
        Err(DefinitionError::Unsupported("definition nesting limit"))
    );
}

#[test]
fn external_cli_info_preserves_canonical_output_and_continuation() {
    // Native CLI replay has its separate pinned reference script. This test also
    // runs in CI without a Gazebo C++ installation, using checked complete outputs.
    let fixture: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/definitions/cli.json")).unwrap();
    for row in fixture["cases"].as_array().unwrap() {
        let args: Vec<_> = row["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a.as_str().unwrap())
            .collect();
        let result = Command::new(env!("CARGO_BIN_EXE_gz-msgs"))
            .args(&args)
            .env("GZ_DESCRIPTOR_PATH", row["descriptor"].as_str().unwrap())
            .output()
            .unwrap();
        if let Some(feature) = row.get("unsupported") {
            assert!(!result.status.success());
            assert!(result.stdout.is_empty());
            assert!(String::from_utf8(result.stderr)
                .unwrap()
                .contains(feature.as_str().unwrap()));
        } else {
            assert_eq!(
                result.status.code().unwrap() as i64,
                row["status"].as_i64().unwrap()
            );
            assert_eq!(
                result.stdout,
                row["stdout"].as_str().unwrap().as_bytes(),
                "{args:?}"
            );
            assert_eq!(
                result.stderr,
                row["stderr"].as_str().unwrap().as_bytes(),
                "{args:?}"
            );
        }
    }
}

#[test]
fn reserved_ranges_names_and_json_escaping_match_native() {
    let directory = Directory::new();
    let peer = directory.native();
    for file in ["reserved", "ranges"] {
        let input = PathBuf::from(format!("tests/fixtures/definitions/{file}.desc"));
        let mut registry = DescriptorRegistry::default();
        assert!(registry.load_file(&input).unwrap().is_empty());
        for (name, expected) in native_definitions(&peer, &input) {
            let descriptor = registry.pool().get_message_by_name(&name).unwrap();
            assert_eq!(
                descriptor_definition(&descriptor).unwrap(),
                expected,
                "{name}"
            );
        }
    }
}

#[test]
fn raw_reserved_ranges_preserve_native_acceptance_and_layout() {
    use gz_msgs::{
        prost_reflect::DescriptorPool,
        prost_types::{
            descriptor_proto::ReservedRange, DescriptorProto, FileDescriptorProto,
            FileDescriptorSet,
        },
    };
    let directory = Directory::new();
    let peer = directory.native();
    for (start, end) in [(i32::MAX, i32::MAX), (1, i32::MIN)] {
        let pool = DescriptorPool::from_file_descriptor_set(FileDescriptorSet {
            file: vec![FileDescriptorProto {
                name: Some("overflow.proto".into()),
                syntax: Some("proto3".into()),
                message_type: vec![DescriptorProto {
                    name: Some("Overflow".into()),
                    reserved_range: vec![ReservedRange {
                        start: Some(start),
                        end: Some(end),
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        })
        .unwrap();
        // Avoid reproducing native signed-overflow behavior as a compatibility target.
        assert_eq!(
            descriptor_definition(&pool.get_message_by_name("Overflow").unwrap()),
            Err(DefinitionError::Unsupported(
                "reserved range boundary overflow"
            ))
        );
    }
    for (start, end, status) in [
        (-1, 2, 5),
        (0, 1, 5),
        (1, -1, 0),
        (1, 0, 0),
        (2, 2, 0),
        (5, 4, 0),
        (1, 536870913, 0),
        (536870912, 536870913, 0),
        (1, i32::MAX, 0),
    ] {
        let set = FileDescriptorSet {
            file: vec![FileDescriptorProto {
                name: Some("invalid.proto".into()),
                syntax: Some("proto3".into()),
                message_type: vec![DescriptorProto {
                    name: Some("Invalid".into()),
                    reserved_range: vec![ReservedRange {
                        start: Some(start),
                        end: Some(end),
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let input = directory.0.join("range.desc");
        std::fs::write(&input, set.encode_to_vec()).unwrap();
        let pool = DescriptorPool::from_file_descriptor_set(set).unwrap();
        let native = Command::new(&peer).arg(&input).output().unwrap();
        assert_eq!(
            native.status.code(),
            Some(status),
            "{start}/{end}: {native:?}"
        );
        let result = descriptor_definition(&pool.get_message_by_name("Invalid").unwrap());
        if status == 0 {
            assert!(native.stderr.is_empty());
            let expected = native_definitions(&peer, &input);
            assert_eq!(result.unwrap(), expected["Invalid"]);
        } else {
            assert!(native.stdout.is_empty());
            assert!(!native.stderr.is_empty());
            assert_eq!(
                result,
                Err(DefinitionError::Unsupported("invalid reserved range"))
            );
        }
    }
}

#[test]
fn malformed_reflection_map_entries_do_not_panic_or_lose_metadata() {
    use gz_msgs::{
        prost_reflect::DescriptorPool,
        prost_types::{
            DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet,
            MessageOptions,
        },
    };
    let directory = Directory::new();
    let peer = directory.native();
    for case in [
        "valid",
        "empty",
        "key_only",
        "value_only",
        "extra",
        "name",
        "float_key",
        "repeated_value",
        "default_key",
    ] {
        let mut fields: Vec<_> = [("key", 1), ("value", 2)]
            .into_iter()
            .map(|(name, number)| FieldDescriptorProto {
                name: Some(name.into()),
                number: Some(number),
                r#type: Some(5),
                label: Some(1),
                ..Default::default()
            })
            .collect();
        match case {
            "empty" => fields.clear(),
            "key_only" => {
                fields.pop();
            }
            "value_only" => {
                fields.remove(0);
            }
            "extra" => {
                let mut field = fields[0].clone();
                field.name = Some("extra".into());
                field.number = Some(3);
                fields.push(field);
            }
            "name" => fields[0].name = Some("wrong".into()),
            "float_key" => fields[0].r#type = Some(2),
            "repeated_value" => fields[1].label = Some(3),
            "default_key" => fields[0].default_value = Some("1".into()),
            "valid" => {}
            _ => unreachable!(),
        }
        let set = FileDescriptorSet {
            file: vec![FileDescriptorProto {
                name: Some("map.proto".into()),
                syntax: Some("proto3".into()),
                message_type: vec![DescriptorProto {
                    name: Some("Container".into()),
                    nested_type: vec![DescriptorProto {
                        name: Some("ValuesEntry".into()),
                        options: Some(MessageOptions {
                            map_entry: Some(true),
                            ..Default::default()
                        }),
                        field: fields,
                        ..Default::default()
                    }],
                    field: vec![FieldDescriptorProto {
                        name: Some("values".into()),
                        number: Some(1),
                        r#type: Some(11),
                        label: Some(3),
                        type_name: Some(".Container.ValuesEntry".into()),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let input = directory.0.join("map.desc");
        let pool = DescriptorPool::from_file_descriptor_set(set).unwrap();
        // Compare the same reflection descriptor metadata, including synthesized JSON names.
        std::fs::write(
            &input,
            FileDescriptorSet {
                file: pool.file_descriptor_protos().cloned().collect(),
            }
            .encode_to_vec(),
        )
        .unwrap();
        let native = Command::new(&peer).arg(&input).output().unwrap();
        for name in ["Container", "Container.ValuesEntry"] {
            let result = descriptor_definition(&pool.get_message_by_name(name).unwrap());
            if case == "valid" {
                assert!(result.is_ok());
            } else {
                assert_eq!(
                    result,
                    Err(DefinitionError::Unsupported("malformed map entry")),
                    "{case}/{name}"
                );
            }
        }
        if case == "valid" {
            assert!(native.status.success(), "{native:?}");
            for (name, expected) in native_definitions(&peer, &input) {
                assert_eq!(
                    descriptor_definition(&pool.get_message_by_name(&name).unwrap()).unwrap(),
                    expected
                );
            }
        } else {
            assert_eq!(native.status.code(), Some(5), "{case}: {native:?}");
            assert!(native.stdout.is_empty());
            assert!(!native.stderr.is_empty());
        }
    }
}

#[test]
fn synthesized_json_names_are_an_explicit_source_provenance_gap() {
    use gz_msgs::{
        prost_reflect::DescriptorPool,
        prost_types::{
            DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet,
        },
    };
    let directory = Directory::new();
    let peer = directory.native();
    let set = FileDescriptorSet {
        file: vec![FileDescriptorProto {
            name: Some("omitted-name.proto".into()),
            syntax: Some("proto3".into()),
            message_type: vec![DescriptorProto {
                name: Some("Omitted".into()),
                field: vec![FieldDescriptorProto {
                    name: Some("value".into()),
                    number: Some(1),
                    r#type: Some(5),
                    label: Some(1),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }],
    };
    let input = directory.0.join("omitted.desc");
    std::fs::write(&input, set.encode_to_vec()).unwrap();
    let native = native_definitions(&peer, &input);
    let pool = DescriptorPool::from_file_descriptor_set(set.clone()).unwrap();
    let reflected = descriptor_definition(&pool.get_message_by_name("Omitted").unwrap()).unwrap();
    assert_eq!(
        native["Omitted"],
        "message Omitted {\n  int32 value = 1;\n}\n"
    );
    assert_eq!(
        reflected,
        "message Omitted {\n  int32 value = 1 [json_name = \"value\"];\n}\n"
    );
    assert_ne!(reflected, native["Omitted"]);
    let mut registry = DescriptorRegistry::default();
    assert!(registry
        .load_bytes(&set.encode_to_vec())
        .unwrap()
        .is_empty());
    assert_eq!(
        registry.definition("Omitted").unwrap(),
        Some(native["Omitted"].clone())
    );
    let factory = gz_msgs::factory::MessageFactory::with_paths(input.as_os_str());
    assert_eq!(
        factory.registry_definition("Omitted").unwrap(),
        Some(native["Omitted"].clone())
    );
    let output = Command::new(env!("CARGO_BIN_EXE_gz-msgs"))
        .args(["--info", "Omitted"])
        .env("GZ_DESCRIPTOR_PATH", &input)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!(
            "Name: Omitted\nFile: omitted-name.proto\n\n{}\n",
            native["Omitted"]
        )
    );
}

#[test]
fn registry_definitions_keep_source_presence_across_duplicates_failures_and_snapshots() {
    use gz_msgs::prost_types::{
        DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet,
    };
    let directory = Directory::new();
    let peer = directory.native();
    let file = |explicit: bool| FileDescriptorProto {
        name: Some("presence.proto".into()),
        package: Some("presence".into()),
        syntax: Some("proto3".into()),
        message_type: vec![DescriptorProto {
            name: Some("Outer".into()),
            nested_type: vec![DescriptorProto {
                name: Some("Inner".into()),
                field: [None, Some(""), Some("third")]
                    .into_iter()
                    .enumerate()
                    .map(|(i, json)| FieldDescriptorProto {
                        name: Some(["first", "second", "third"][i].into()),
                        number: Some(i as i32 + 1),
                        r#type: Some(5),
                        label: Some(1),
                        json_name: if explicit {
                            Some(format!("explicit{i}"))
                        } else {
                            json.map(str::to_owned)
                        },
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
    let encode = |file| FileDescriptorSet { file: vec![file] }.encode_to_vec();
    for explicit in [false, true] {
        let original = file(explicit);
        let input = directory.0.join("presence.desc");
        std::fs::write(&input, encode(original.clone())).unwrap();
        let native = native_definitions(&peer, &input);
        let mut registry = DescriptorRegistry::default();
        assert!(
            registry.load_bytes(&encode(original)).unwrap().is_empty(),
            "explicit={explicit}"
        );
        let snapshot = registry.clone();
        assert!(registry
            .load_bytes(&encode(file(!explicit)))
            .unwrap()
            .is_empty());
        for (name, body) in native {
            assert_eq!(registry.definition(&name).unwrap(), Some(body.clone()));
            assert_eq!(snapshot.definition(&name).unwrap(), Some(body));
        }
        assert_eq!(registry.definition("absent").unwrap(), None);
    }
    let mut registry = DescriptorRegistry::default();
    let mut failed = file(false);
    failed.dependency.push("missing.proto".into());
    assert_eq!(registry.load_bytes(&encode(failed)).unwrap().len(), 1);
    assert_eq!(registry.definition("presence.Outer").unwrap(), None);
    let snapshot = registry.clone();
    let accepted = file(true);
    let input = directory.0.join("accepted.desc");
    std::fs::write(&input, encode(accepted.clone())).unwrap();
    assert!(registry.load_bytes(&encode(accepted)).unwrap().is_empty());
    for (name, body) in native_definitions(&peer, &input) {
        assert_eq!(registry.definition(&name).unwrap(), Some(body));
        assert_eq!(snapshot.definition(&name).unwrap(), None);
    }
}

#[test]
fn registry_definition_does_not_execute_registered_constructors() {
    assert_eq!(
        DescriptorRegistry::default()
            .definition("gz.msgs.Clock")
            .unwrap()
            .as_deref(),
        gz_msgs::message_definition("gz.msgs.Clock")
    );
    let factory = gz_msgs::factory::MessageFactory::with_paths("");
    factory.register("gz.msgs.Clock", || {
        panic!("definition executed constructor")
    });
    factory.register("custom", || panic!("definition executed constructor"));
    for name in [
        "gz.msgs.Clock",
        "gz_msgs.Clock",
        ".gz.msgs.Clock",
        ".gz_msgs.Clock",
    ] {
        assert_eq!(
            factory.registry_definition(name).unwrap().as_deref(),
            gz_msgs::message_definition("gz.msgs.Clock")
        );
    }
    for name in ["custom", "absent", ".custom"] {
        assert_eq!(factory.registry_definition(name).unwrap(), None);
    }
}

#[test]
fn source_definitions_preserve_omissions_in_maps_oneofs_and_optional_fields() {
    use gz_msgs::prost_types::{DescriptorProto, FileDescriptorSet};
    fn omit(message: &mut DescriptorProto) {
        for field in &mut message.field {
            field.json_name = None;
        }
        for nested in &mut message.nested_type {
            omit(nested);
        }
    }
    let directory = Directory::new();
    let peer = directory.native();
    let mut set =
        FileDescriptorSet::decode(include_bytes!("fixtures/definitions/supported.desc").as_slice())
            .unwrap();
    for file in &mut set.file {
        for message in &mut file.message_type {
            omit(message);
        }
    }
    let bytes = set.encode_to_vec();
    let input = directory.0.join("omissions.desc");
    std::fs::write(&input, &bytes).unwrap();
    let mut registry = DescriptorRegistry::default();
    assert!(registry.load_bytes(&bytes).unwrap().is_empty());
    for (name, body) in native_definitions(&peer, &input) {
        assert_eq!(registry.definition(&name).unwrap(), Some(body), "{name}");
    }
}

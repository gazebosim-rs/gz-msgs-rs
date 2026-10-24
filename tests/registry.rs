// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Original retry assertion source: Copyright (C) 2026 Open Source Robotics Foundation.
use gz_msgs::{
    prost::Message,
    prost_reflect::{DynamicMessage, ReflectMessage, Value},
    registry::DescriptorRegistry,
};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
fn directory() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = PathBuf::from(format!(
        "target/registry-tests/{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}
fn fixture() -> (Vec<u8>, Vec<u8>) {
    let json: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/registry-reference.json")).unwrap();
    for name in ["initial_missing", "dependency_loaded", "retry_succeeded"] {
        assert_eq!(json[name], true);
    }
    let decode = |name: &str| {
        json[name]
            .as_str()
            .unwrap()
            .as_bytes()
            .chunks_exact(2)
            .map(|byte| u8::from_str_radix(std::str::from_utf8(byte).unwrap(), 16).unwrap())
            .collect()
    };
    (decode("a_hex"), decode("b_hex"))
}
#[test]
fn original_missing_import_retry_assertions() {
    let (a, b) = fixture();
    let dir = directory();
    let a_path = dir.join("a.desc");
    let b_path = dir.join("b.desc");
    std::fs::write(&a_path, &a).unwrap();
    std::fs::write(&b_path, &b).unwrap();
    let mut registry = DescriptorRegistry::default();
    assert_eq!(registry.load_file(&a_path).unwrap().len(), 1);
    assert!(registry.pool().get_message_by_name("a.msgs.Foo").is_none());
    assert!(registry.load_file(&b_path).unwrap().is_empty());
    let bar = DynamicMessage::new(registry.pool().get_message_by_name("b.msgs.Bar").unwrap());
    assert_eq!(bar.descriptor().full_name(), "b.msgs.Bar");
    assert!(registry.load_file(&a_path).unwrap().is_empty());
    let foo = registry.pool().get_message_by_name("a.msgs.Foo").unwrap();
    let message = DynamicMessage::parse_text_format(foo, "bar { x: 42 }").unwrap();
    assert_eq!(message.encode_to_vec(), [10, 2, 8, 42]);
}
#[test]
fn ordered_partial_loads_duplicates_errors_and_owned_snapshots() {
    let (a, b) = fixture();
    let mut mixed = a.clone();
    mixed.extend(&b);
    let mut registry = DescriptorRegistry::default();
    assert_eq!(registry.load_bytes(&mixed).unwrap().len(), 1);
    assert!(registry.pool().get_message_by_name("a.msgs.Foo").is_none());
    let bar = registry.pool().get_message_by_name("b.msgs.Bar").unwrap();
    let snapshot = bar.parent_pool().all_messages().len();
    assert!(registry.load_bytes(&mixed).unwrap().is_empty());
    assert_eq!(bar.parent_pool().all_messages().len(), snapshot);
    assert_eq!(registry.pool().all_messages().len(), snapshot + 1);
    let count = registry.pool().all_messages().len();
    assert!(registry.load_bytes(&mixed).unwrap().is_empty());
    assert_eq!(registry.pool().all_messages().len(), count);
    assert!(registry.load_bytes(&[]).unwrap().is_empty());
    assert!(registry.load_bytes(&[0xff]).is_err());
    let dir = directory();
    assert!(registry.load_file(dir.join("missing.desc")).is_err());
    assert!(registry.load_file(&dir).is_err());
    let bad = dir.join("bad.desc");
    std::fs::write(&bad, [0xff]).unwrap();
    assert_eq!(
        registry.load_file(bad).unwrap_err().kind(),
        std::io::ErrorKind::InvalidData
    );
    let clone = registry.clone();
    assert_eq!(clone.pool().all_messages().len(), count);
    drop(registry);
    assert!(DynamicMessage::new(bar).encode_to_vec().is_empty());
}
#[test]
fn custom_descriptor_options_survive_reflective_loading() {
    let dir = directory();
    std::fs::write(dir.join("options.proto"),r#"syntax="proto2"; package ext; import "google/protobuf/descriptor.proto"; extend google.protobuf.MessageOptions { optional string note=50001; } message Tagged { option (note)="retained"; }"#).unwrap();
    assert!(std::process::Command::new("protoc")
        .arg(format!("-I{}", dir.display()))
        .arg("-I/usr/include")
        .arg("--include_imports")
        .arg(format!(
            "--descriptor_set_out={}",
            dir.join("options.desc").display()
        ))
        .arg(dir.join("options.proto"))
        .status()
        .unwrap()
        .success());
    let mut registry = DescriptorRegistry::default();
    assert!(registry
        .load_file(dir.join("options.desc"))
        .unwrap()
        .is_empty());
    let extension = registry.pool().get_extension_by_name("ext.note").unwrap();
    let message = registry.pool().get_message_by_name("ext.Tagged").unwrap();
    assert_eq!(
        message.options().get_extension(&extension).as_ref(),
        &Value::String("retained".into())
    );
}

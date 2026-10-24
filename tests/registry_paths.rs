// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Original assertion source: Copyright (C) 2024 Open Source Robotics Foundation.
use gz_msgs::{
    prost_reflect::{DynamicMessage, ReflectMessage},
    registry::DescriptorRegistry,
};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};
const FIXTURES: &str = "tests/reference/descriptors";
fn directory() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = PathBuf::from(format!(
        "target/registry-paths/{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}
fn message(registry: &DescriptorRegistry, name: &str) -> Option<DynamicMessage> {
    registry
        .pool()
        .get_message_by_name(name)
        .map(DynamicMessage::new)
}
#[test]
fn original_dynamic_factory_path_and_directory_assertions() {
    let fixture: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/registry-paths-reference.json"))
            .unwrap();
    let mut registry = DescriptorRegistry::default();
    let common = [
        "example.msgs.StringMsg",
        "testing.Bytes",
        "testing.FooMessage",
        "testing.BarMessage",
        "testing.BazMessage",
    ];
    let check =
        |registry: &DescriptorRegistry, index: usize, names: [&str; 5], expected: [bool; 5]| {
            let row = &fixture["stages"][index];
            assert_eq!(row["types"], gz_msgs::serde_json::json!(names));
            let actual = names.map(|name| message(registry, name).is_some());
            assert_eq!(actual, expected);
            assert_eq!(row["present"], gz_msgs::serde_json::json!(actual));
        };
    check(&registry, 0, common, [false; 5]);
    let missing = Path::new(FIXTURES).join("does_not_exist.desc");
    assert_eq!(registry.load_paths(missing.as_os_str()).len(), 1);
    check(&registry, 1, common, [false; 5]);
    assert!(registry
        .load_paths(Path::new(FIXTURES).join("testing.invalid"))
        .is_empty());
    check(&registry, 2, common, [false; 5]);
    assert_eq!(
        registry
            .load_paths(Path::new(FIXTURES).join("testing.proto"))
            .len(),
        1
    );
    check(
        &registry,
        3,
        [
            "example.msgs.StringMsg",
            "testing.BarBytes",
            "testing.FooMessage",
            "testing.BarMessage",
            "testing.BazMessage",
        ],
        [false; 5],
    );
    assert!(registry
        .load_paths(Path::new(FIXTURES).join("stringmsg.desc"))
        .is_empty());
    check(&registry, 4, common, [true, false, false, false, false]);
    assert_eq!(registry.load_paths(FIXTURES).len(), 1); // Invalid source text still reports its parse error.
    check(
        &registry,
        5,
        [
            "example.msgs.StringMsg",
            "testing.FooBytes",
            "testing.FooMessage",
            "testing.BarMessage",
            "testing.BazMessage",
        ],
        [true; 5],
    );
}
#[test]
fn original_duplicate_descriptor_path_has_no_database_diagnostic() {
    const PROBE: &str = "GZ_MSGS_DUPLICATE_PATH_PROBE";
    if std::env::var_os(PROBE).is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "original_duplicate_descriptor_path_has_no_database_diagnostic",
            ])
            .env(PROBE, "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!String::from_utf8_lossy(&output.stderr).contains("already exists in database"));
        return;
    }
    let path = Path::new(FIXTURES).join("stringmsg.desc");
    let paths = std::env::join_paths([&path, &path]).unwrap();
    let mut registry = DescriptorRegistry::default();
    assert!(registry.load_paths(paths).is_empty());
    assert_eq!(
        message(&registry, "example.msgs.StringMsg")
            .unwrap()
            .descriptor()
            .full_name(),
        "example.msgs.StringMsg"
    );
}
#[test]
fn directory_priority_substring_recognition_and_error_paths() {
    let fixture: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/registry-reference.json")).unwrap();
    let decode = |key: &str| {
        fixture[key]
            .as_str()
            .unwrap()
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect::<Vec<_>>()
    };
    let dir = directory();
    std::fs::write(dir.join("a.desc"), decode("a_hex")).unwrap();
    std::fs::write(dir.join("gz-msgs.gz_desc"), decode("b_hex")).unwrap();
    std::fs::write(dir.join("ignored.txt"), b"invalid").unwrap();
    let mut registry = DescriptorRegistry::default();
    assert!(registry.load_directory(&dir).unwrap().is_empty());
    assert!(message(&registry, "a.msgs.Foo").is_some());
    assert!(message(&registry, "b.msgs.Bar").is_some());
    let count = registry.pool().all_messages().len();
    assert!(registry.load_directory(dir.clone()).unwrap().is_empty());
    assert_eq!(registry.pool().all_messages().len(), count);
    assert!(registry.load_paths(OsStr::new("")).is_empty());
    assert!(registry.load_directory(dir.join("missing")).is_err());
    assert!(registry.load_directory(dir.join("a.desc")).is_err());
    let orphan = directory().join("a.proto.bin");
    std::fs::write(&orphan, decode("a_hex")).unwrap();
    assert_eq!(DescriptorRegistry::default().load_paths(orphan).len(), 1);
    // Legacy filtering tests the whole path, including a recognized parent name.
    let dotted = directory().join("parent.desc");
    std::fs::create_dir(&dotted).unwrap();
    let name = dotted.join("not_a_descriptor.txt");
    std::fs::write(&name, decode("b_hex")).unwrap();
    let mut registry = DescriptorRegistry::default();
    assert!(registry.load_paths(name).is_empty());
    assert!(message(&registry, "b.msgs.Bar").is_some());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let protected = directory();
        let permissions = std::fs::metadata(&protected).unwrap().permissions();
        std::fs::set_permissions(&protected, std::fs::Permissions::from_mode(0o0)).unwrap();
        let denied = std::fs::read_dir(&protected).is_err();
        let errors = registry.load_paths(&protected);
        std::fs::set_permissions(&protected, permissions).unwrap();
        assert_eq!(errors.is_empty(), !denied);
        let execute_only = directory();
        std::fs::copy(
            Path::new(FIXTURES).join("stringmsg.desc"),
            execute_only.join("gz-msgs.gz_desc"),
        )
        .unwrap();
        let permissions = std::fs::metadata(&execute_only).unwrap().permissions();
        std::fs::set_permissions(&execute_only, std::fs::Permissions::from_mode(0o111)).unwrap();
        let listing_failed = std::fs::read_dir(&execute_only).is_err();
        let mut own_registry = DescriptorRegistry::default();
        let loaded = own_registry.load_directory(&execute_only);
        std::fs::set_permissions(&execute_only, permissions).unwrap();
        let native: gz_msgs::serde_json::Value =
            gz_msgs::serde_json::from_str(include_str!("fixtures/registry-paths-reference.json"))
                .unwrap();
        assert!(native["execute_only_directory"]["own_loaded"]
            .as_bool()
            .unwrap());
        assert_eq!(loaded.is_err(), listing_failed);
        assert!(message(&own_registry, "example.msgs.StringMsg").is_some());
        if listing_failed {
            assert!(native["execute_only_directory"]["listing_failed"]
                .as_bool()
                .unwrap());
        }
        if denied {
            assert_eq!(errors[0].0, protected);
            assert_eq!(errors[0].1.kind(), std::io::ErrorKind::PermissionDenied);
        }
    }
}

// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#![cfg(all(target_os = "linux", target_env = "gnu"))]
use gz_msgs::{
    factory::MessageFactory,
    registry::DescriptorRegistry,
    serde_json::{self, Value},
};
use std::{
    ffi::OsString,
    io,
    os::unix::ffi::OsStringExt,
    path::{Path, PathBuf},
    process::Command,
};
fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/factory-cache.json")).unwrap()
}
fn unhex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|v| u8::from_str_radix(std::str::from_utf8(v).unwrap(), 16).unwrap())
        .collect()
}
fn bytes(data: &Value, name: &str) -> Vec<u8> {
    unhex(data["inputs"][name].as_str().unwrap())
}
fn selected_path(kind: &str) -> PathBuf {
    let raw = || PathBuf::from(OsString::from_vec(b"raw-\xff.desc".to_vec()));
    match kind {
        "relative" => "a.desc".into(),
        "dot" => "./a.desc".into(),
        "parent" => "sub/../a.desc".into(),
        "absolute" => std::env::current_dir().unwrap().join("a.desc"),
        "absolute-dot" => std::env::current_dir().unwrap().join("./a.desc"),
        "absolute-parent" => std::env::current_dir().unwrap().join("sub/../a.desc"),
        "alias" => "alias.desc".into(),
        "absolute-alias" => std::env::current_dir().unwrap().join("alias.desc"),
        "trailing" => "a.desc/".into(),
        "absolute-trailing" => std::env::current_dir().unwrap().join("a.desc/"),
        "trailing-dot" => "a.desc/.".into(),
        "notdir" => "a.desc/../a.desc".into(),
        "missing-parent" => "gone/../a.desc".into(),
        "absolute-missing-parent" => std::env::current_dir().unwrap().join("gone/../a.desc"),
        "nested" => "sub/a.desc".into(),
        "nested-dot" => "./sub/a.desc".into(),
        "nested-absolute" => std::env::current_dir().unwrap().join("sub/a.desc"),
        "nested-absolute-parent" => std::env::current_dir().unwrap().join("gone/../sub/a.desc"),
        "directory-alias" => "dir-alias/a.desc".into(),
        "directory-alias-absolute" => std::env::current_dir().unwrap().join("dir-alias/a.desc"),
        "directory-alias-parent" => "deep-alias/../a.desc".into(),
        "raw-other" => PathBuf::from(OsString::from_vec(b"raw-\xfe.desc".to_vec())),
        "replacement" => "raw-\u{fffd}.desc".into(),
        "raw" => raw(),
        "raw-absolute" => std::env::current_dir().unwrap().join(raw()),
        _ => panic!("unknown fixture path {kind}"),
    }
}
fn state(factory: &MessageFactory) -> Vec<u64> {
    ["audit.A", "audit.B", "audit.C", "audit.Dep"]
        .into_iter()
        .map(|name| u64::from(factory.new_message(name).is_some()))
        .collect()
}
fn check_state(factory: &MessageFactory, row: &Value, step: u64) {
    let states = row["states"].as_array().unwrap();
    let expected = states.iter().find(|s| s[0] == step).unwrap_or_else(|| {
        assert!(["loop", "cached-loop"].contains(&row["action"].as_str().unwrap()));
        assert_eq!(step, 3);
        assert!(row["native"]["exit"].as_i64().unwrap() < 0);
        // Native aborts here. Rust's existing IO error contract preserves prior state.
        &states[0]
    });
    assert_eq!(
        state(factory),
        expected.as_array().unwrap()[1..]
            .iter()
            .map(|v| v.as_u64().unwrap())
            .collect::<Vec<_>>(),
        "{} step{step}",
        row["id"]
    );
}
fn load(factory: &MessageFactory, path: &Path, count: usize) -> Vec<(PathBuf, io::Error)> {
    let errors = factory.load_paths(path);
    assert_eq!(errors.len(), count, "{path:?}: {errors:?}");
    for (actual, _) in &errors {
        assert_eq!(actual.as_os_str(), path.as_os_str());
    }
    errors
}
fn run_case(data: &Value, row: &Value) {
    use std::os::unix::fs::symlink;
    let write = |name: &str, path: &Path| std::fs::write(path, bytes(data, name)).unwrap();
    let action = row["action"].as_str().unwrap();
    std::fs::create_dir("sub").unwrap();
    symlink("a.desc", "alias.desc").unwrap();
    match action {
        "malformed" => std::fs::write("a.desc", [0xff]).unwrap(),
        "partial" => write("partial", Path::new("a.desc")),
        "empty" => write("empty", Path::new("a.desc")),
        "missing" | "dangling" => {}
        _ => write("A", Path::new("a.desc")),
    }
    if ["raw-replace", "raw-delete", "raw-distinct"].contains(&action) {
        write("A", &selected_path("raw"));
    }
    if action == "delete-parent" {
        write("A", Path::new("sub/a.desc"));
    }
    if action == "dangling-directory" {
        symlink("sub", "dir-alias").unwrap();
        write("A", Path::new("sub/a.desc"));
    }
    if action == "symlink-parent" {
        std::fs::create_dir("sub/deep").unwrap();
        symlink("sub/deep", "deep-alias").unwrap();
        write("A", Path::new("sub/a.desc"));
    }
    let factory = MessageFactory::with_paths("");
    let first_error =
        usize::from(["missing", "dangling", "malformed", "partial"].contains(&action));
    let errors = load(
        &factory,
        &selected_path(row["first"].as_str().unwrap()),
        first_error,
    );
    if let Some((_, error)) = errors.first() {
        assert_eq!(
            error.kind(),
            if ["malformed", "partial"].contains(&action) {
                io::ErrorKind::InvalidData
            } else {
                io::ErrorKind::NotFound
            }
        );
    }
    check_state(&factory, row, 1);
    match action {
        "delete" => std::fs::remove_file("a.desc").unwrap(),
        "raw-distinct" => write("B", &selected_path(row["second"].as_str().unwrap())),
        "raw-replace" => write("B", &selected_path("raw")),
        "raw-delete" => std::fs::remove_file(selected_path("raw")).unwrap(),
        "delete-parent" | "dangling-directory" => std::fs::remove_dir_all("sub").unwrap(),
        "symlink-parent" => std::fs::remove_file("sub/a.desc").unwrap(),
        "cached-loop" => {
            std::fs::remove_file("a.desc").unwrap();
            symlink("a.desc", "a.desc").unwrap();
        }
        "loop" => {
            std::fs::remove_file("alias.desc").unwrap();
            symlink("loop.desc", "alias.desc").unwrap();
            symlink("alias.desc", "loop.desc").unwrap();
        }
        "retarget" => {
            write("B", Path::new("b.desc"));
            std::fs::remove_file("alias.desc").unwrap();
            symlink("b.desc", "alias.desc").unwrap();
        }
        "partial" => {
            write("Dep", Path::new("dep.desc"));
            load(&factory, Path::new("dep.desc"), 0);
            check_state(&factory, row, 2);
        }
        "missing" | "dangling" | "malformed" => write("A", Path::new("a.desc")),
        _ => write("B", Path::new("a.desc")),
    }
    let stderr_nonempty = row["native"]["stderr"]
        .as_array()
        .unwrap()
        .iter()
        .any(|part| {
            part.get("case_directory").is_some()
                || part["hex"].as_str().is_some_and(|v| !v.is_empty())
        });
    let final_error = usize::from(
        row["native"]["exit"].as_i64().unwrap() < 0 || first_error == 0 && stderr_nonempty,
    );
    let errors = load(
        &factory,
        &selected_path(row["second"].as_str().unwrap()),
        final_error,
    );
    if let Some((_, error)) = errors.first() {
        assert!(error.raw_os_error().is_some());
    }
    check_state(&factory, row, 3);
}
#[test]
fn native_file_cache_workflows() {
    let data = fixture();
    let rows = data["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 46);
    if let Some(index) = std::env::var_os("GZ_MSGS_CACHE_CASE") {
        run_case(
            &data,
            &rows[index.to_str().unwrap().parse::<usize>().unwrap()],
        );
        return;
    }
    // A separate process gives each relative-path/cwd control an isolated filesystem.
    for (index, row) in rows.iter().enumerate() {
        let dir = tempfile::tempdir().unwrap();
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "native_file_cache_workflows", "--nocapture"])
            .env("GZ_MSGS_CACHE_CASE", index.to_string())
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}: {}\n{}",
            row["id"],
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
#[test]
fn byte_loading_and_cloned_registry_caches_keep_their_own_state() {
    let data = fixture();
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.desc");
    let b = dir.path().join("b.desc");
    std::fs::write(&a, bytes(&data, "A")).unwrap();
    std::fs::write(&b, bytes(&data, "B")).unwrap();
    let mut registry = DescriptorRegistry::default();
    assert!(registry.load_file(&a).unwrap().is_empty());
    let mut clone = registry.clone();
    let before = registry.definition("audit.A").unwrap();
    std::fs::write(&a, bytes(&data, "B")).unwrap();
    assert!(registry.load_file(&a).unwrap().is_empty());
    assert!(registry.pool().get_message_by_name("audit.B").is_none());
    assert_eq!(registry.definition("audit.A").unwrap(), before);
    assert!(registry.load_bytes(&bytes(&data, "B")).unwrap().is_empty());
    assert!(registry.pool().get_message_by_name("audit.B").is_some());
    assert!(clone.pool().get_message_by_name("audit.B").is_none());
    assert!(registry.load_file(&b).unwrap().is_empty());
    assert!(clone.load_file(&b).unwrap().is_empty());
    assert!(clone.pool().get_message_by_name("audit.B").is_some());
    assert!(clone.load_file(&a).unwrap().is_empty());
    assert_eq!(clone.definition("audit.A").unwrap(), before);
}
#[test]
fn initial_and_explicit_failures_keep_order_and_retry() {
    let data = fixture();
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.desc");
    let b = dir.path().join("b.desc");
    std::fs::write(&a, [0xff]).unwrap();
    std::fs::write(&b, [0xff]).unwrap();
    let factory = MessageFactory::with_paths(a.clone());
    let errors = factory.load_paths(&b);
    assert_eq!(errors.len(), 2);
    assert_eq!(errors[0].0, a);
    assert_eq!(errors[1].0, b);
    assert!(errors
        .iter()
        .all(|(_, e)| e.kind() == io::ErrorKind::InvalidData));
    assert!(factory.take_initial_errors().is_empty());
    std::fs::write(&a, bytes(&data, "A")).unwrap();
    std::fs::write(&b, bytes(&data, "B")).unwrap();
    let paths = std::env::join_paths([&a, &b]).unwrap();
    assert!(factory.load_paths(&paths).is_empty());
    assert!(factory.new_message("audit.A").is_some());
    assert!(factory.new_message("audit.B").is_some());
    std::fs::write(&a, [0xff]).unwrap();
    assert!(factory.load_paths(&a).is_empty());
    assert!(factory.new_message("audit.A").is_some());
}

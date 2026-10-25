// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use std::{collections::BTreeMap, path::PathBuf, process::Command};

struct Directory(PathBuf);
impl Drop for Directory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn external_cli_list_matches_complete_native_name_sets() {
    let fixture: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/cli-external-list.json")).unwrap();
    let path = std::env::temp_dir().join(format!(
        "gz-msgs-cli-list-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&path).unwrap();
    let directory = Directory(path);
    let output = directory.0.join("generated.desc");
    let compiled = Command::new("protoc")
        .args([
            "--proto_path=tests/fixtures/cli-external-list",
            "--proto_path=schemas",
            "--include_imports",
        ])
        .arg(format!("--descriptor_set_out={}", output.display()))
        .arg("external.proto")
        .output()
        .unwrap();
    assert!(compiled.status.success(), "{compiled:?}");
    assert_eq!(
        std::fs::read(output).unwrap(),
        include_bytes!("fixtures/cli-external-list/external.desc")
    );
    let mut paths = BTreeMap::new();
    for key in ["testing", "alias", "external"] {
        paths.insert(
            key,
            std::fs::canonicalize(fixture["inputs"][key]["path"].as_str().unwrap()).unwrap(),
        );
    }
    for kind in ["directory", "priority"] {
        let path = directory.0.join(kind);
        std::fs::create_dir(&path).unwrap();
        for key in ["testing", "alias", "external"] {
            let name = if kind == "priority" && key == "external" {
                "gz-msgs.gz_desc".to_owned()
            } else {
                format!("{key}.desc")
            };
            std::fs::copy(&paths[key], path.join(name)).unwrap();
        }
        paths.insert(kind, path);
    }
    let ignored = directory.0.join("ignored.bin");
    std::fs::write(&ignored, "not a descriptor").unwrap();
    std::fs::copy(&ignored, paths["directory"].join("ignored.bin")).unwrap();
    paths.insert("ignored", ignored);
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 11);
    for row in fixture["cases"].as_array().unwrap() {
        for flag in ["--list", "-l", "list"] {
            let mut command = Command::new(env!("CARGO_BIN_EXE_gz-msgs"));
            command.arg(flag).env_remove("GZ_DESCRIPTOR_PATH");
            if row["environment_present"].as_bool().unwrap() {
                command.env(
                    "GZ_DESCRIPTOR_PATH",
                    std::env::join_paths(
                        row["paths"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|key| &paths[key.as_str().unwrap()]),
                    )
                    .unwrap(),
                );
            }
            let result = command.output().unwrap();
            assert_eq!(result.status.code().unwrap(), 0, "{}", row["label"]);
            assert_eq!(result.stderr, row["stderr"].as_str().unwrap().as_bytes());
            let names = String::from_utf8(result.stdout).unwrap();
            let mut names: Vec<_> = names.lines().collect();
            names.sort();
            let mut expected: Vec<_> = fixture["canonical_names"]
                .as_array()
                .unwrap()
                .iter()
                .chain(row["additional_names"].as_array().unwrap())
                .map(|name| name.as_str().unwrap())
                .collect();
            expected.sort();
            assert_eq!(names, expected, "{} {flag}", row["label"]);
        }
    }
}

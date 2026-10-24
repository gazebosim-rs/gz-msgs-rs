// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
fn main() {
    let mut files: Vec<_> = std::fs::read_dir("../../schemas/gz/msgs")
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "proto"))
        .collect();
    files.sort();
    prost_build::Config::new()
        .file_descriptor_set_path(
            std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("messages.pb"),
        )
        .compile_protos(&files, &["../../schemas"])
        .unwrap();
}

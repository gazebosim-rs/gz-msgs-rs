// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#![cfg(unix)]
use gz_msgs::serde_json;
use std::{
    ffi::OsString,
    os::unix::ffi::OsStringExt,
    process::{Command, Output},
};
fn unhex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn run(args: &[OsString], path: Option<&std::path::Path>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_gz-msgs"));
    command
        .args(args)
        .env_remove("GZ_DESCRIPTOR_PATH")
        .env_remove("GZ_MSGS_INSTALL_PREFIX");
    if let Some(path) = path {
        command.env("GZ_DESCRIPTOR_PATH", path);
    }
    command.output().unwrap()
}
#[test]
fn raw_os_arguments_preserve_native_bytes_exits_and_precedence() {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/os-argv.json")).unwrap();
    let rows = data["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 56);
    let dir = tempfile::tempdir().unwrap();
    let path = dir
        .path()
        .join(OsString::from_vec(b"external-\xff.desc".to_vec()));
    std::fs::write(
        &path,
        unhex(data["external_descriptor_hex"].as_str().unwrap()),
    )
    .unwrap();
    for row in rows {
        let args: Vec<_> = row["args_hex"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| OsString::from_vec(unhex(v.as_str().unwrap())))
            .collect();
        let output = run(
            &args,
            row["external_path"]
                .as_bool()
                .unwrap()
                .then_some(path.as_path()),
        );
        let native = &row["native"];
        assert_eq!(
            output.status.code(),
            Some(native["exit"].as_i64().unwrap() as i32),
            "{row}"
        );
        match row["binding"].as_str().unwrap() {
            // Existing package-version and Rust-owned help text bindings remain unchanged.
            binding @ ("help" | "version") => {
                assert_eq!(native, &data["controls"][binding]);
                let control = run(&[OsString::from(format!("--{binding}"))], None);
                assert_eq!(output.stdout, control.stdout, "{row}");
                assert_eq!(output.stderr, control.stderr, "{row}");
            }
            "list" => {
                let native = unhex(native["stdout_hex"].as_str().unwrap());
                let mut expected: Vec<_> = native.split(|b| *b == b'\n').collect();
                let mut actual: Vec<_> = output.stdout.split(|b| *b == b'\n').collect();
                expected.sort_unstable();
                actual.sort_unstable();
                assert_eq!(actual, expected, "{row}");
                assert_eq!(
                    output.stderr,
                    unhex(row["native"]["stderr_hex"].as_str().unwrap())
                );
            }
            "exact" => {
                assert_eq!(
                    output.stdout,
                    unhex(native["stdout_hex"].as_str().unwrap()),
                    "{row}"
                );
                assert_eq!(
                    output.stderr,
                    unhex(native["stderr_hex"].as_str().unwrap()),
                    "{row}"
                );
            }
            _ => panic!("unknown fixture binding"),
        }
    }
}
#[test]
fn rust_format_commands_reject_non_utf8_arguments_without_replacement_or_panic() {
    for command in ["encode-json", "decode-json", "encode-text", "decode-text"] {
        let output = run(
            &[command.into(), OsString::from_vec(b"gz.msgs.\xff".to_vec())],
            None,
        );
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert_eq!(
            output.stderr,
            b"format command arguments must be UTF-8\nRun with --help for more information.\n"
        );
    }
}

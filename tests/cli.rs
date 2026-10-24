// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use std::{
    io::Write,
    process::{Command, Stdio},
};
fn run(args: &[&str], input: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_gz-msgs"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}
#[test]
fn formats_and_commands() {
    let list = run(&["list"], &[]);
    assert!(list.status.success());
    assert!(String::from_utf8(list.stdout)
        .unwrap()
        .lines()
        .any(|line| line == "gz.msgs.Clock"));
    let info = run(&["info", "gz.msgs.Time"], &[]);
    assert!(info.status.success());
    assert!(String::from_utf8(info.stdout).unwrap().contains("sec"));
    for (command, input) in [
        ("encode-json", r#"{"sec":"7","nsec":8}"#),
        ("encode-text", "sec: 7 nsec: 8"),
    ] {
        let encoded = run(&[command, "gz.msgs.Time"], input.as_bytes());
        assert!(
            encoded.status.success(),
            "{}",
            String::from_utf8_lossy(&encoded.stderr)
        );
        assert_eq!(encoded.stdout, [8, 7, 16, 8]);
        let json = run(&["decode-json", "gz.msgs.Time"], &encoded.stdout);
        assert!(json.status.success());
        assert_eq!(
            gz_msgs::serde_json::from_slice::<gz_msgs::serde_json::Value>(&json.stdout).unwrap(),
            gz_msgs::serde_json::json!({"sec":"7","nsec":8})
        );
        let text = run(&["decode-text", "gz.msgs.Time"], &encoded.stdout);
        assert!(text.status.success());
        assert_eq!(String::from_utf8(text.stdout).unwrap(), "sec:7,nsec:8");
    }
    for args in [
        &["nonsense"][..],
        &["nonsense", "gz.msgs.Time"],
        &["info", "missing"],
        &["info", "gz.msgs.PixelFormatType"],
    ] {
        assert!(!run(args, &[]).status.success());
    }
    for command in ["encode-json", "encode-text", "decode-json", "decode-text"] {
        assert!(!run(&[command, "gz.msgs.Time"], b"malformed")
            .status
            .success());
    }
}

#[test]
fn invalid_utf8_and_io_failure() {
    for command in ["encode-json", "encode-text"] {
        assert!(!run(&[command, "gz.msgs.Time"], &[0xff]).status.success());
    }
    let output = Command::new(env!("CARGO_BIN_EXE_gz-msgs"))
        .args(["decode-json", "gz.msgs.Time"])
        .stdin(std::fs::File::open(".").unwrap())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("directory"));
    let output = Command::new(env!("CARGO_BIN_EXE_gz-msgs"))
        .args(["decode-json", "gz.msgs.Time"])
        .stdin(Stdio::null())
        .stdout(
            std::fs::OpenOptions::new()
                .write(true)
                .open("/dev/full")
                .unwrap(),
        )
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("space"));
}

#[test]
fn negative_zero_survives_tooling_formats() {
    for name in ["gz.msgs.Float", "gz.msgs.Double"] {
        let encoded = run(&["encode-text", name], b"data: -0");
        assert!(encoded.status.success());
        assert!(!encoded.stdout.is_empty());
        for (decode, encode) in [
            ("decode-text", "encode-text"),
            ("decode-json", "encode-json"),
        ] {
            let text = run(&[decode, name], &encoded.stdout);
            assert!(text.status.success());
            assert!(String::from_utf8_lossy(&text.stdout).contains("-0"));
            let roundtrip = run(&[encode, name], &text.stdout);
            assert!(roundtrip.status.success());
            assert_eq!(roundtrip.stdout, encoded.stdout);
        }
    }
}

#[test]
fn native_cli_version_help_list_info_and_completion_assertions() {
    for version in ["--version", "-v"] {
        let output = run(&[version], &[]);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout)
            .unwrap()
            .contains(env!("CARGO_PKG_VERSION")));
    }
    for help in [&["--help"][..], &["-h"], &[]] {
        let output = run(help, &[]);
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("list"));
        assert!(text.contains("info"));
    }
    for list in ["--list", "-l"] {
        let output = run(&[list], &[]);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout)
            .unwrap()
            .contains("gz.msgs.WorldControl"));
    }
    for name in ["gz_msgs.WorldControl", "gz.msgs.WorldControl"] {
        let output = run(&["--info", name], &[]);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout)
            .unwrap()
            .contains("message WorldControl {"));
    }
    for info in ["--info", "-i"] {
        let output = run(&[info, "gz.msgs.WorldControl", "gz.msgs.Wrench"], &[]);
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("message WorldControl {"));
        assert!(text.contains("message Wrench {"));
    }
    let completion = Command::new("bash")
        .args([
            "-c",
            "source scripts/msgs.bash_completion.sh; _gz_msgs_flags",
        ])
        .output()
        .unwrap();
    assert!(completion.status.success());
    let flags = String::from_utf8(completion.stdout).unwrap();
    let flags: Vec<_> = flags.split_whitespace().collect();
    assert!(!flags.is_empty());
    let help = String::from_utf8(run(&["--help"], &[]).stdout).unwrap();
    for flag in flags {
        assert!(help.contains(flag));
    }
}

#[test]
fn unchanged_cpp_cli_assertions() {
    assert!(Command::new("python3")
        .args(["scripts/rehost-cli.py", env!("CARGO_BIN_EXE_gz-msgs")])
        .status()
        .unwrap()
        .success());
}

#[test]
fn complete_native_info_output_corpus() {
    let fixture: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("../schemas/cli-info.json")).unwrap();
    let records = fixture.as_object().unwrap();
    let mut args = vec!["--info"];
    let mut expected = String::new();
    for (name, record) in records {
        args.push(name);
        expected.push_str(record["output"].as_str().unwrap());
    }
    let output = run(&args, &[]);
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    for (input, canonical) in [
        ("gz.msgs.WorldControl", "gz.msgs.WorldControl"),
        ("gz_msgs.WorldControl", "gz.msgs.WorldControl"),
        ("gz.msgs.Param.ParamsEntry", "gz.msgs.Param.ParamsEntry"),
        ("gz.msgs.Discovery.Publisher", "gz.msgs.Discovery.Publisher"),
        ("google.protobuf.Any", "google.protobuf.Any"),
    ] {
        let output = run(&["--info", input], &[]);
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            records[canonical]["output"].as_str().unwrap()
        );
    }
    assert_eq!(
        gz_msgs::message_definition("gz.msgs.Param.ParamsEntry"),
        Some("")
    );
    for message in gz_msgs::descriptors().all_messages() {
        assert_eq!(
            gz_msgs::message_info(message.full_name()),
            Some(records[message.full_name()]["output"].as_str().unwrap())
        );
    }
}

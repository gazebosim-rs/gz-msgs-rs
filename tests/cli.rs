// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Original assertion sources:
// Copyright (C) 2018 Open Source Robotics Foundation
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
    for args in [&["nonsense"][..], &["nonsense", "gz.msgs.Time"]] {
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

#[test]
fn complete_native_factory_list() {
    let fixture: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/factory-reference.json")).unwrap();
    let expected = fixture["types"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| name.as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    for command in ["list", "--list", "-l"] {
        let output = run(&[command], &[]);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    }
}

#[test]
fn native_info_errors_continue_with_success_and_exact_diagnostics() {
    let fixture: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/cli-errors-reference.json")).unwrap();
    assert_eq!(fixture["rows"].as_array().unwrap().len(), 9);
    for row in fixture["rows"].as_array().unwrap() {
        let args: Vec<_> = row["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a.as_str().unwrap())
            .collect();
        let output = run(&args, &[]);
        assert_eq!(
            output.status.code(),
            Some(row["exit_code"].as_i64().unwrap() as i32),
            "{row}"
        );
        assert_eq!(
            output.stdout,
            row["stdout"].as_str().unwrap().as_bytes(),
            "{row}"
        );
        assert_eq!(
            output.stderr,
            row["stderr"].as_str().unwrap().as_bytes(),
            "{row}"
        );
    }
    let alias = run(&["info", "missing"], &[]);
    assert!(alias.status.success());
    assert_eq!(alias.stderr, b"Unable to create message of type[missing]\n");
}

#[test]
fn json_requires_exactly_one_complete_document() {
    let fixture: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/json-framing.json")).unwrap();
    let grammar: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/json-grammar.json")).unwrap();
    assert_eq!(grammar["rows"].as_array().unwrap().len(), 15);
    let arrays: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/json-array-grammar.json")).unwrap();
    assert_eq!(arrays["rows"].as_array().unwrap().len(), 14);
    let single_quotes: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/json-single-quote.json")).unwrap();
    assert_eq!(single_quotes["matching"].as_array().unwrap().len(), 21);
    assert_eq!(
        single_quotes["native_permissive_pending"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let raw_controls: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/json-raw-controls.json")).unwrap();
    let escaped_bytes: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/json-escaped-bytes.json")).unwrap();
    let escaped_utf8: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/json-escaped-utf8.json")).unwrap();
    let trailing_dot: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/json-trailing-dot.json")).unwrap();
    let negative_zero: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/json-negative-zero.json")).unwrap();
    let vtff: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/json-vtff-whitespace.json")).unwrap();
    let raw_rows = raw_controls["matching"].as_array().unwrap();
    assert_eq!(raw_rows.len(), 64);
    assert_eq!(raw_controls["rejected"].as_array().unwrap().len(), 10);
    let mut seen = [[false; 32]; 2];
    for row in raw_rows {
        let quote = row["quote"].as_str().unwrap();
        let quote_index = match quote {
            "'" => 0,
            "\"" => 1,
            _ => panic!("unexpected quote {quote}"),
        };
        let byte = usize::try_from(row["byte"].as_u64().unwrap()).unwrap();
        assert!(byte < 32 && !seen[quote_index][byte]);
        seen[quote_index][byte] = true;
        let delimiter = quote.as_bytes()[0];
        assert_eq!(
            row["input_hex"],
            format!("7b2264617461223a{delimiter:02x}61{byte:02x}62{delimiter:02x}7d")
        );
    }
    assert!(seen.into_iter().flatten().all(|value| value));
    let escape_rows = escaped_bytes["matching"].as_array().unwrap();
    assert_eq!(escape_rows.len(), 254);
    let rejected_escapes = escaped_bytes["rejected"].as_array().unwrap();
    assert_eq!(rejected_escapes.len(), 2);
    let mut escaped_seen = [[false; 128]; 2];
    for row in escape_rows.iter().chain(rejected_escapes) {
        let quote = row["quote"].as_str().unwrap();
        let (quote_index, delimiter) = match quote {
            "double" => (0, b'"'),
            "single" => (1, b'\''),
            _ => panic!("unexpected quote {quote}"),
        };
        let byte = usize::try_from(row["escaped_byte"].as_u64().unwrap()).unwrap();
        assert!(byte < 128 && !escaped_seen[quote_index][byte]);
        escaped_seen[quote_index][byte] = true;
        assert_eq!(row["type"], "gz.msgs.StringMsg");
        assert_eq!(
            row["input_hex"],
            format!("7b2264617461223a{delimiter:02x}415c{byte:02x}42{delimiter:02x}7d")
        );
        assert_eq!(
            row["cpp_status"].as_i64().unwrap() == 0,
            byte != b'u' as usize
        );
    }
    assert!(escaped_seen.into_iter().flatten().all(|value| value));
    let utf8_matches = escaped_utf8["matching"].as_array().unwrap();
    let utf8_rejected = escaped_utf8["rejected"].as_array().unwrap();
    assert_eq!((utf8_matches.len(), utf8_rejected.len()), (24, 20));
    let valid_sequences: Vec<_> = escaped_utf8["valid_sequence_hex"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect();
    let mut expected_sequences = valid_sequences.clone();
    expected_sequences.extend(
        escaped_utf8["invalid_sequence_hex"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_owned()),
    );
    expected_sequences.sort();
    expected_sequences.dedup();
    assert_eq!(expected_sequences.len(), 22);
    let mut observed = [Vec::new(), Vec::new()];
    for row in utf8_matches.iter().chain(utf8_rejected) {
        let (quote_index, delimiter) = match row["quote"].as_str().unwrap() {
            "double" => (0, b'"'),
            "single" => (1, b'\''),
            quote => panic!("unexpected quote {quote}"),
        };
        let sequence = row["sequence_hex"].as_str().unwrap();
        assert_eq!(row["type"], "gz.msgs.StringMsg");
        assert_eq!(
            row["input_hex"],
            format!("7b2264617461223a{delimiter:02x}415c{sequence}42{delimiter:02x}7d")
        );
        assert_eq!(
            row["cpp_status"].as_i64().unwrap() == 0,
            valid_sequences.iter().any(|value| value == sequence)
        );
        observed[quote_index].push(sequence.to_owned());
    }
    for actual in &mut observed {
        actual.sort();
        assert_eq!(*actual, expected_sequences);
    }
    let expected_forms = [
        "0.", "-0.", "7.", "-7.", "7.e2", "7.E+2", "7.e-2", "7.5", "7.0", "7.e", "7.e+", "7..",
        "7.  ", "7.\n", "\"7.\"", "\"-7.\"", "\"7.e2\"", "7.f",
    ];
    assert_eq!(
        trailing_dot["scalar_forms"].as_array().unwrap(),
        &expected_forms
            .iter()
            .map(|form| gz_msgs::serde_json::Value::String((*form).to_owned()))
            .collect::<Vec<_>>()
    );
    let scalar_rows = trailing_dot["scalar_rows"].as_array().unwrap();
    assert_eq!(scalar_rows.len(), 36);
    for (index, row) in scalar_rows.iter().enumerate() {
        let type_name = if index < 18 {
            "gz.msgs.Time"
        } else {
            "gz.msgs.Double_V"
        };
        let form = expected_forms[index % 18];
        assert_eq!(row["type"], type_name);
        assert_eq!(row["form"], form);
        let input = if index < 18 {
            format!("{{\"sec\":{form}}}")
        } else {
            format!("{{\"data\":[{form}]}}")
        };
        assert_eq!(
            row["input_hex"],
            input
                .as_bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
    }
    assert_eq!(
        scalar_rows[..18]
            .iter()
            .filter(|row| row["cpp_status"].as_i64() == Some(0))
            .count(),
        9
    );
    assert_eq!(
        scalar_rows[18..]
            .iter()
            .filter(|row| row["cpp_status"].as_i64() == Some(0))
            .count(),
        14
    );
    let composite_rows = trailing_dot["composite_rows"].as_array().unwrap();
    let expected_composites: [(&str, &str, &[u8], bool); 8] = [
        (
            "time_two_fields",
            "gz.msgs.Time",
            br#"{"sec":7.,"nsec":8.}"#,
            true,
        ),
        (
            "double_two_values",
            "gz.msgs.Double_V",
            br#"{"data":[7.,8.]}"#,
            true,
        ),
        (
            "double_trailing_comma",
            "gz.msgs.Double_V",
            br#"{"data":[7.,]}"#,
            true,
        ),
        (
            "time_trailing_comma",
            "gz.msgs.Time",
            br#"{"sec":7.,}"#,
            true,
        ),
        (
            "time_block_comment",
            "gz.msgs.Time",
            br#"{"sec":7. /*x*/}"#,
            false,
        ),
        (
            "time_line_comment",
            "gz.msgs.Time",
            b"{\"sec\":7.//x\n}",
            false,
        ),
        (
            "time_space_before_dot",
            "gz.msgs.Time",
            br#"{"sec":7 .}"#,
            false,
        ),
        (
            "double_space_before_dot",
            "gz.msgs.Double_V",
            br#"{"data":[7 .]}"#,
            false,
        ),
    ];
    assert_eq!(composite_rows.len(), expected_composites.len());
    for (row, (name, type_name, input, accepted)) in composite_rows.iter().zip(expected_composites)
    {
        assert_eq!(row["name"], name);
        assert_eq!(row["type"], type_name);
        assert_eq!(row["cpp_status"].as_i64() == Some(0), accepted);
        assert_eq!(
            row["input_hex"],
            input
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
    }
    let second_dot_rejected = trailing_dot["fraction_or_exponent_rejected"]
        .as_array()
        .unwrap();
    let rejected_forms = [
        "1.2.e3", "1.2.E+3", "1.2.", "1.2. ", "1.2.e-3", "7e2.", "7.e2.", "1e+2.", "1e-2.", "7.0.",
        "7.0.e2", "1.2.e3", "7e2.", "7.0.",
    ];
    assert_eq!(second_dot_rejected.len(), rejected_forms.len());
    for (index, row) in second_dot_rejected.iter().enumerate() {
        let type_name = if index < 11 {
            "gz.msgs.Double_V"
        } else {
            "gz.msgs.Time"
        };
        let form = rejected_forms[index];
        let input = if index < 11 {
            format!("{{\"data\":[{form}]}}")
        } else {
            format!("{{\"sec\":{form}}}")
        };
        assert_eq!(row["type"], type_name);
        assert_eq!(row["form"], form);
        assert_ne!(row["cpp_status"].as_i64(), Some(0));
        assert_eq!(
            row["input_hex"],
            input
                .as_bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
    }
    let numeric_pending = trailing_dot["native_permissive_pending"]
        .as_array()
        .unwrap();
    assert_eq!(numeric_pending.len(), 5);
    for (index, row) in numeric_pending.iter().enumerate() {
        let type_name = if index < 2 {
            "gz.msgs.Time"
        } else {
            "gz.msgs.Double_V"
        };
        let form = match index {
            0 | 2 => "07.",
            1 | 3 => "0x7.",
            4 => "0x7.e3",
            _ => unreachable!(),
        };
        let input = if index < 2 {
            format!("{{\"sec\":{form}}}")
        } else {
            format!("{{\"data\":[{form}]}}")
        };
        assert_eq!(row["type"], type_name);
        assert_eq!(row["form"], form);
        assert_eq!(row["cpp_status"].as_i64(), Some(0));
        assert_eq!(
            row["input_hex"],
            input
                .as_bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
    }
    let negative_forms = [
        "-0", "-0 ", "-0\t", "-0\n", "-0.", "-0.0", "-0e0", "-0E+0", "\"-0\"", "\"-0.0\"", "-00",
        "-0x0", "-0e", "0",
    ];
    assert_eq!(
        negative_zero["scalar_forms"].as_array().unwrap(),
        &negative_forms
            .iter()
            .map(|form| gz_msgs::serde_json::Value::String((*form).to_owned()))
            .collect::<Vec<_>>()
    );
    let negative_scalars = negative_zero["scalar_rows"].as_array().unwrap();
    assert_eq!(negative_scalars.len(), 28);
    for (index, row) in negative_scalars.iter().enumerate() {
        let type_name = if index < 14 {
            "gz.msgs.Double_V"
        } else {
            "gz.msgs.Float_V"
        };
        let form = negative_forms[index % 14];
        assert_eq!(row["type"], type_name);
        assert_eq!(row["form"], form);
        assert_eq!(
            row["input_hex"],
            format!("{{\"data\":[{form}]}}")
                .as_bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
        assert_eq!(
            row["cpp_status"].as_i64() == Some(0),
            !matches!(index % 14, 10..=12)
        );
    }
    assert_eq!(
        negative_scalars[0]["cpp_stdout_hex"],
        "0a080000000000000000"
    );
    assert_eq!(negative_scalars[14]["cpp_stdout_hex"], "120400000000");
    assert_eq!(
        negative_scalars[8]["cpp_stdout_hex"],
        "0a080000000000000080"
    );
    let negative_composites = negative_zero["composite_rows"].as_array().unwrap();
    let expected_negative_composites: [(&str, &str, &str); 16] = [
        ("double_middle", "gz.msgs.Double_V", "{\"data\":[1,-0,2]}"),
        ("float_last", "gz.msgs.Float_V", "{\"data\":[0,-0]}"),
        (
            "double_trailing_comma",
            "gz.msgs.Double_V",
            "{\"data\":[-0,]}",
        ),
        ("double_bare_key", "gz.msgs.Double_V", "{data:[-0]}"),
        ("double_single_key", "gz.msgs.Double_V", "{'data':[-0]}"),
        (
            "double_quoted_value",
            "gz.msgs.Double_V",
            "{\"data\":[\"-0\"]}",
        ),
        ("time_integer", "gz.msgs.Time", "{\"sec\":-0}"),
        (
            "vector_two_fields",
            "gz.msgs.Vector3d",
            "{\"x\":-0,\"y\":-0.0}",
        ),
        ("string_value", "gz.msgs.StringMsg", "{\"data\":\"-0\"}"),
        (
            "double_missing_comma",
            "gz.msgs.Double_V",
            "{\"data\":[-0 -0]}",
        ),
        (
            "double_suffix",
            "gz.msgs.Double_V",
            "{\"data\":[-0]}garbage",
        ),
        (
            "double_comment",
            "gz.msgs.Double_V",
            "{\"data\":[-0/*comment*/]}",
        ),
        (
            "double_bad_exponent",
            "gz.msgs.Double_V",
            "{\"data\":[-0e]}",
        ),
        (
            "double_adjacent_minus",
            "gz.msgs.Double_V",
            "{\"data\":[1-0]}",
        ),
        ("double_two_minus", "gz.msgs.Double_V", "{\"data\":[-0-0]}"),
        ("time_adjacent_minus", "gz.msgs.Time", "{\"sec\":1-0}"),
    ];
    assert_eq!(
        negative_composites.len(),
        expected_negative_composites.len()
    );
    for (row, (name, type_name, input)) in
        negative_composites.iter().zip(expected_negative_composites)
    {
        assert_eq!(row["name"], name);
        assert_eq!(row["type"], type_name);
        assert_eq!(
            row["input_hex"],
            input
                .as_bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
    }
    assert_eq!(
        negative_composites[7]["cpp_stdout_hex"],
        "190000000000000080"
    );
    let quoted_hex_pending = negative_zero["native_permissive_pending"]
        .as_array()
        .unwrap();
    assert_eq!(quoted_hex_pending.len(), 2);
    for (row, type_name) in quoted_hex_pending
        .iter()
        .zip(["gz.msgs.Double_V", "gz.msgs.Float_V"])
    {
        assert_eq!(row["type"], type_name);
        assert_eq!(row["input_hex"], "7b2264617461223a5b22307837225d7d");
        assert_eq!(row["cpp_status"].as_i64(), Some(0));
    }
    let vtff_rows = vtff["rows"].as_array().unwrap();
    let vtff_cases: [(&str, &str, &str, bool); 21] = [
        ("leading", "gz.msgs.Double_V", "{w}{\"data\":[1]}", true),
        ("trailing", "gz.msgs.Double_V", "{\"data\":[1]}{w}", true),
        (
            "after_open",
            "gz.msgs.Double_V",
            "{{{w}\"data\":[1]}}",
            true,
        ),
        (
            "before_colon",
            "gz.msgs.Double_V",
            "{{\"data\"{w}:[1]}}",
            true,
        ),
        (
            "after_colon",
            "gz.msgs.Double_V",
            "{{\"data\":{w}[1]}}",
            true,
        ),
        (
            "after_array_open",
            "gz.msgs.Double_V",
            "{{\"data\":[{w}1]}}",
            true,
        ),
        (
            "after_number",
            "gz.msgs.Double_V",
            "{{\"data\":[1{w}]}}",
            true,
        ),
        (
            "after_comma",
            "gz.msgs.Double_V",
            "{{\"data\":[1,{w}2]}}",
            true,
        ),
        (
            "before_comma",
            "gz.msgs.Double_V",
            "{{\"data\":[1{w},2]}}",
            true,
        ),
        (
            "trailing_comma",
            "gz.msgs.Double_V",
            "{{\"data\":[1,{w}]}}",
            true,
        ),
        ("bare_key", "gz.msgs.Double_V", "{{data{w}:[1]}}", true),
        (
            "negative_zero",
            "gz.msgs.Double_V",
            "{{\"data\":[-0{w}]}}",
            true,
        ),
        (
            "negative_zero_float",
            "gz.msgs.Float_V",
            "{{\"data\":[-0{w}]}}",
            true,
        ),
        (
            "trailing_dot",
            "gz.msgs.Double_V",
            "{{\"data\":[7.{w}]}}",
            true,
        ),
        ("time_integer", "gz.msgs.Time", "{{\"sec\":7{w}}}", true),
        (
            "time_negative_zero",
            "gz.msgs.Time",
            "{{\"sec\":-0{w}}}",
            true,
        ),
        (
            "raw_in_string",
            "gz.msgs.StringMsg",
            "{{\"data\":\"a{w}b\"}}",
            true,
        ),
        (
            "between_digits",
            "gz.msgs.Double_V",
            "{{\"data\":[1{w}2]}}",
            false,
        ),
        (
            "after_minus",
            "gz.msgs.Double_V",
            "{{\"data\":[-{w}0]}}",
            false,
        ),
        (
            "before_suffix",
            "gz.msgs.Double_V",
            "{{\"data\":[1]}}{w}junk",
            false,
        ),
        (
            "comment",
            "gz.msgs.Double_V",
            "{{\"data\":[1{w}/*x*/]}}",
            false,
        ),
    ];
    assert_eq!(vtff_rows.len(), vtff_cases.len() * 2);
    for (index, row) in vtff_rows.iter().enumerate() {
        let (name, type_name, template, accepted) = vtff_cases[index % vtff_cases.len()];
        let whitespace = [b'\x0b', b'\x0c'][index / vtff_cases.len()];
        assert_eq!(row["name"], name);
        assert_eq!(row["type"], type_name);
        assert_eq!(row["whitespace_hex"], format!("{whitespace:02x}"));
        assert_eq!(row["cpp_status"].as_i64() == Some(0), accepted);
        let input = template
            .replace("{{", "{")
            .replace("}}", "}")
            .replace("{w}", &String::from(whitespace as char));
        assert_eq!(
            row["input_hex"],
            input
                .as_bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
        if !accepted {
            assert_eq!(row["cpp_stdout_hex"], "");
        }
    }
    assert_eq!(vtff_rows[11]["cpp_stdout_hex"], "0a080000000000000000");
    assert_eq!(vtff_rows[12]["cpp_stdout_hex"], "120400000000");
    assert_eq!(vtff_rows[16]["cpp_stdout_hex"], "1203610b62");
    assert_eq!(vtff_rows[37]["cpp_stdout_hex"], "1203610c62");
    assert!(Command::new("python3")
        .arg("scripts/native-peer.py")
        .status()
        .unwrap()
        .success());
    let check = |row: &gz_msgs::serde_json::Value, type_name: &str, match_native: bool| {
        let unhex = |s: &str| {
            s.as_bytes()
                .chunks_exact(2)
                .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
                .collect::<Vec<_>>()
        };
        let input = unhex(row["input_hex"].as_str().unwrap());
        let mut child = Command::new("target/native/protobuf-peer")
            .args(["target/native/messages.pb", type_name, "encode-json"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(&input).unwrap();
        let native = child.wait_with_output().unwrap();
        assert_eq!(
            native.status.code().unwrap() as i64,
            row["cpp_status"].as_i64().unwrap()
        );
        assert_eq!(
            native.stdout,
            unhex(row["cpp_stdout_hex"].as_str().unwrap())
        );
        let actual = run(&["encode-json", type_name], &input);
        if match_native {
            assert_eq!(
                actual.status.success(),
                native.status.success(),
                "{input:?}"
            );
            assert_eq!(actual.stdout, native.stdout, "{input:?}");
        } else {
            assert!(native.status.success(), "{input:?}");
            assert!(!actual.status.success(), "{input:?}");
            assert!(actual.stdout.is_empty(), "{input:?}");
        }
    };
    for rows in [
        &fixture["matching"],
        &fixture["native_permissive_grammar_pending"],
        &grammar["rows"],
    ] {
        for row in rows.as_array().unwrap() {
            check(row, "gz.msgs.Time", true);
        }
    }
    for row in arrays["rows"].as_array().unwrap() {
        check(row, row["type"].as_str().unwrap(), true);
    }
    for row in single_quotes["matching"].as_array().unwrap() {
        check(row, row["type"].as_str().unwrap(), true);
    }
    for row in single_quotes["native_permissive_pending"]
        .as_array()
        .unwrap()
    {
        check(row, row["type"].as_str().unwrap(), true);
    }
    for row in raw_rows
        .iter()
        .chain(raw_controls["rejected"].as_array().unwrap())
    {
        check(row, "gz.msgs.StringMsg", true);
    }
    for row in escape_rows.iter().chain(rejected_escapes) {
        check(row, "gz.msgs.StringMsg", true);
    }
    for row in utf8_matches.iter().chain(utf8_rejected) {
        check(row, "gz.msgs.StringMsg", true);
    }
    for row in scalar_rows.iter().chain(composite_rows) {
        check(row, row["type"].as_str().unwrap(), true);
    }
    for row in second_dot_rejected {
        check(row, row["type"].as_str().unwrap(), true);
    }
    for row in numeric_pending {
        check(row, row["type"].as_str().unwrap(), false);
    }
    for row in negative_scalars.iter().chain(negative_composites) {
        check(row, row["type"].as_str().unwrap(), true);
    }
    for row in quoted_hex_pending {
        check(row, row["type"].as_str().unwrap(), true);
    }
    for row in vtff_rows {
        check(row, row["type"].as_str().unwrap(), true);
    }
}

#[test]
fn quoted_hex_float_strings_match_native() {
    let fixture: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/json-quoted-hex.json")).unwrap();
    assert_eq!(fixture["native_protobuf"], "3.21.12");
    assert_eq!(fixture["baseline_cases"], 78);
    assert_eq!(fixture["boundary_cases"], 138);
    let rows = fixture["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 216);
    let unhex = |s: &str| {
        s.as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect::<Vec<_>>()
    };
    for row in rows {
        let type_name = row["type"].as_str().unwrap();
        let input = unhex(row["input_hex"].as_str().unwrap());
        let expected = unhex(row["cpp_stdout_hex"].as_str().unwrap());
        let result = run(&["encode-json", type_name], &input);
        assert_eq!(
            result.status.success(),
            row["cpp_status"].as_i64().unwrap() == 0,
            "{type_name}: {}: {}",
            row["form"],
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(result.stdout, expected, "{type_name}: {}", row["form"]);
    }
    let pending = fixture["repeated_data_pending"].as_array().unwrap();
    assert_eq!(pending.len(), 2);
    for row in pending {
        let type_name = row["type"].as_str().unwrap();
        let input = unhex(row["input_hex"].as_str().unwrap());
        assert_eq!(row["cpp_status"], 0);
        let expected = unhex(row["cpp_stdout_hex"].as_str().unwrap());
        assert!(!expected.is_empty());
        let result = run(&["encode-json", type_name], &input);
        assert!(result.status.success(), "{type_name}");
        assert_eq!(result.stdout, expected, "{type_name}");
    }
}

#[test]
fn repeated_float_data_keys_append_in_native_order() {
    let fixture: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/json-repeated-data.json")).unwrap();
    assert_eq!(fixture["native_protobuf"], "3.21.12");
    let rows = fixture["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 66);
    assert_eq!(rows.iter().filter(|row| row["cpp_status"] == 0).count(), 38);
    let unhex = |s: &str| {
        s.as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect::<Vec<_>>()
    };
    for row in rows {
        let type_name = row["type"].as_str().unwrap();
        let label = row["label"].as_str().unwrap();
        let input = unhex(row["input_hex"].as_str().unwrap());
        let expected = unhex(row["cpp_stdout_hex"].as_str().unwrap());
        let result = run(&["encode-json", type_name], &input);
        assert_eq!(
            result.status.success(),
            row["cpp_status"] == 0,
            "{type_name} {label}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(result.stdout, expected, "{type_name} {label}");
    }
}

fn assert_native_option(row: &gz_msgs::serde_json::Value) {
    let args: Vec<_> = row["args"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a.as_str().unwrap())
        .collect();
    let mut variants = vec![args.clone()];
    if args
        .first()
        .is_some_and(|a| matches!(*a, "--info" | "--list"))
    {
        let mut alias = args.clone();
        alias[0] = alias[0].trim_start_matches("--");
        variants.push(alias);
    }
    for args in variants {
        let actual = run(&args, &[]);
        assert_eq!(
            actual.status.code().unwrap() as i64,
            row["status"].as_i64().unwrap(),
            "{args:?}"
        );
        match row["mode"].as_str().unwrap() {
            "version" => {
                assert_eq!(
                    actual.stdout,
                    format!("{}\n", env!("CARGO_PKG_VERSION")).as_bytes()
                );
                assert!(actual.stderr.is_empty());
            }
            "help" => {
                assert!(String::from_utf8(actual.stdout)
                    .unwrap()
                    .contains("--info|-i"));
                assert!(actual.stderr.is_empty());
            }
            "list" => {
                assert_eq!(actual.stderr, row["stderr"].as_str().unwrap().as_bytes());
                let mut actual: Vec<_> = String::from_utf8(actual.stdout)
                    .unwrap()
                    .lines()
                    .map(str::to_owned)
                    .collect();
                let mut expected: Vec<_> = row["stdout"]
                    .as_str()
                    .unwrap()
                    .lines()
                    .map(str::to_owned)
                    .collect();
                actual.sort();
                expected.sort();
                assert_eq!(actual, expected);
            }
            "exact" => {
                assert_eq!(
                    actual.stdout,
                    row["stdout"].as_str().unwrap().as_bytes(),
                    "{args:?}"
                );
                assert_eq!(
                    actual.stderr,
                    row["stderr"].as_str().unwrap().as_bytes(),
                    "{args:?}"
                );
            }
            _ => panic!("fixture mode"),
        }
    }
}

#[test]
fn native_legacy_option_combinations() {
    let fixture: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/cli-options.json")).unwrap();
    assert_eq!(fixture["matching"].as_array().unwrap().len(), 75);
    assert_eq!(
        fixture["formerly_unsupported_flag_assignments"]
            .as_array()
            .unwrap()
            .len(),
        15
    );
    for category in ["matching", "formerly_unsupported_flag_assignments"] {
        for row in fixture[category].as_array().unwrap() {
            assert_native_option(row);
        }
    }
}

#[test]
fn native_flag_value_conversions_and_callback_precedence() {
    let fixture: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/cli-flag-values.json")).unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 171);
    for row in fixture["cases"].as_array().unwrap() {
        assert_native_option(row);
    }
}

#[test]
fn native_info_vector_values_and_required_consumption() {
    let fixture: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/cli-info-vectors.json")).unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 119);
    for row in fixture["cases"].as_array().unwrap() {
        assert_native_option(row);
    }
}

// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use gz_msgs::{
    msgs::DoubleV,
    prost::{bytes::Buf, encoding, Message},
    prost_reflect::{DynamicMessage, Value},
    serde_json, WireMessage,
};
use std::{
    io::Write,
    process::{Command, Stdio},
    sync::Once,
};

fn unhex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn bits(value: &DoubleV) -> Vec<u64> {
    value.data.iter().map(|v| v.to_bits()).collect()
}
fn peer(wire: &[u8], known: bool) -> std::process::Output {
    static BUILD: Once = Once::new();
    BUILD.call_once(|| {
        assert!(Command::new("python3")
            .arg("scripts/native-wire-peer.py")
            .status()
            .unwrap()
            .success())
    });
    let mut command = Command::new("target/native-wire/peer");
    command.arg("gz.msgs.Double_V");
    if known {
        command.arg("known");
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(wire).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn native_packed_float_bits_lengths_and_errors() {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/packed-wire.json")).unwrap();
    let rows = data["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 64);
    for row in rows {
        let wire = unhex(row["wire_hex"].as_str().unwrap());
        for (known, key) in [(false, "cpp_generated"), (true, "cpp_known")] {
            let actual = peer(&wire, known);
            assert_eq!(
                actual.status.code(),
                row[key]["exit"].as_i64().map(|v| v as i32),
                "{}",
                row["id"]
            );
            assert_eq!(
                actual.stdout,
                unhex(row[key]["output_hex"].as_str().unwrap()),
                "{}",
                row["id"]
            );
            assert_eq!(
                actual.stderr,
                row[key]["stderr"].as_str().unwrap().as_bytes(),
                "{}",
                row["id"]
            );
        }
        let strict = row["strict"].as_bool().unwrap();
        let direct = DoubleV::decode(wire.as_slice());
        assert_eq!(direct.is_ok(), strict, "{}", row["id"]);
        let envelope = WireMessage::<DoubleV>::decode(&wire);
        assert_eq!(envelope.is_ok(), strict, "{}", row["id"]);
        let reflected = DynamicMessage::decode(
            gz_msgs::descriptor("gz.msgs.Double_V").unwrap(),
            wire.as_slice(),
        );
        assert_eq!(reflected.is_ok(), strict, "{}", row["id"]);
        let compatible = WireMessage::<DoubleV>::decode_compatible(&wire);
        assert_eq!(
            compatible.is_ok(),
            row["accepted"].as_bool().unwrap(),
            "{}",
            row["id"]
        );
        if let Some(expected) = row["expected_bits"].as_array() {
            let expected: Vec<_> = expected.iter().map(|v| v.as_u64().unwrap()).collect();
            let known = unhex(row["cpp_known"]["output_hex"].as_str().unwrap());
            assert_eq!(
                bits(&DoubleV::decode(known.as_slice()).unwrap()),
                expected,
                "{}",
                row["id"]
            );
            let compatible = compatible.unwrap();
            assert_eq!(compatible.bytes(), wire);
            assert_eq!(bits(compatible.value()), expected, "{}", row["id"]);
            if strict {
                assert_eq!(bits(&direct.unwrap()), expected);
                assert_eq!(bits(envelope.unwrap().value()), expected);
                let reflected = reflected.unwrap();
                let values = reflected.get_field_by_name("data").unwrap();
                let Value::List(values) = values.as_ref() else {
                    panic!("double list")
                };
                let actual: Vec<_> = values
                    .iter()
                    .map(|v| v.as_f64().unwrap().to_bits())
                    .collect();
                assert_eq!(actual, expected, "{}", row["id"]);
            }
        }
    }
}

type State = (Vec<u64>, Option<String>, Vec<u8>);
fn state(mut input: impl Buf) -> State {
    let mut values = vec![-0.0];
    let error = encoding::double::merge_repeated(
        encoding::WireType::LengthDelimited,
        &mut values,
        &mut input,
        encoding::DecodeContext::default(),
    )
    .err()
    .map(|e| e.to_string());
    let tail = input.copy_to_bytes(input.remaining()).to_vec();
    (values.into_iter().map(f64::to_bits).collect(), error, tail)
}
#[test]
fn existing_partial_value_and_chunked_cursor_boundaries() {
    let zero = (-0.0_f64).to_bits();
    let one = 1.25_f64.to_bits();
    let payload = one.to_le_bytes();
    let mut valid = vec![16];
    valid.extend(payload);
    valid.extend(zero.to_le_bytes());
    valid.extend([8, 1]);
    let mut crossing = vec![7];
    crossing.extend(&payload[..7]);
    crossing.extend([8, 1]);
    let mut nine = vec![9];
    nine.extend(payload);
    nine.extend([0, 8, 1, 17, 0, 0, 0, 0, 0, 0, 0, 0]);
    let cases = [
        (vec![0], vec![zero], None, vec![]),
        (valid, vec![zero, one, zero], None, vec![8, 1]),
        (
            vec![8, 1, 2, 3, 4, 5, 6, 7],
            vec![zero],
            Some("buffer underflow"),
            vec![1, 2, 3, 4, 5, 6, 7],
        ),
        // Existing partial writes may include suffix bits before the limit error.
        (
            crossing,
            vec![zero, 0x08f4000000000000],
            Some("delimited length exceeded"),
            vec![1],
        ),
        (
            nine,
            vec![zero, one, 0x11010800],
            Some("delimited length exceeded"),
            vec![0; 4],
        ),
        (
            vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 1],
            vec![zero],
            Some("buffer underflow"),
            vec![],
        ),
    ];
    for (input, values, error, tail) in cases {
        let expected = (
            values,
            error.map(|e| format!("failed to decode Protobuf message: {e}")),
            tail,
        );
        assert_eq!(state(input.as_slice()), expected);
        for split in 0..=input.len() {
            assert_eq!(
                state(input[..split].chain(&input[split..])),
                expected,
                "split {split}"
            );
        }
    }
    let input = [255; 11];
    for split in 0..=input.len() {
        // The current contiguous varint fast path leaves its invalid input unread.
        let remaining = if split == 0 || split == 11 { 11 } else { 1 };
        assert_eq!(
            state(input[..split].chain(&input[split..])),
            (
                vec![zero],
                Some("failed to decode Protobuf message: invalid varint".into()),
                vec![255; remaining]
            )
        );
    }
}

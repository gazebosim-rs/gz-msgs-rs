// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use gz_msgs::{
    msgs::FloatV,
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
fn bits(value: &FloatV) -> Vec<u32> {
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
    command.arg("gz.msgs.Float_V");
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
fn native_packed_fixed32_bits_headers_lengths_and_errors() {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/packed-float-wire.json")).unwrap();
    let rows = data["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 61);
    let mut strict_count = 0;
    let mut compatible_count = 0;
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
        strict_count += usize::from(strict);
        let direct = FloatV::decode(wire.as_slice());
        assert_eq!(direct.is_ok(), strict, "{}", row["id"]);
        let envelope = WireMessage::<FloatV>::decode(&wire);
        assert_eq!(envelope.is_ok(), strict, "{}", row["id"]);
        let reflected = DynamicMessage::decode(
            gz_msgs::descriptor("gz.msgs.Float_V").unwrap(),
            wire.as_slice(),
        );
        assert_eq!(reflected.is_ok(), strict, "{}", row["id"]);
        let compatible = WireMessage::<FloatV>::decode_compatible(&wire);
        assert_eq!(
            compatible.is_ok(),
            row["accepted"].as_bool().unwrap(),
            "{}",
            row["id"]
        );
        if let Some(expected) = row["expected_bits"].as_array() {
            compatible_count += 1;
            let expected: Vec<_> = expected
                .iter()
                .map(|v| u32::try_from(v.as_u64().unwrap()).unwrap())
                .collect();
            let known = unhex(row["cpp_known"]["output_hex"].as_str().unwrap());
            assert_eq!(
                bits(&FloatV::decode(known.as_slice()).unwrap()),
                expected,
                "{}",
                row["id"]
            );
            let compatible = compatible.unwrap();
            assert_eq!(compatible.bytes(), wire);
            assert_eq!(bits(compatible.value()), expected, "{}", row["id"]);
            assert_eq!(compatible.value().encode_to_vec(), known, "{}", row["id"]);
            match row["header"].as_str() {
                Some("stamp7") => {
                    let header = compatible.value().header.as_ref().unwrap();
                    let stamp = header.stamp.as_ref().unwrap();
                    assert_eq!((stamp.sec, stamp.nsec), (7, 0));
                    assert!(header.data.is_empty());
                }
                Some("present" | "unknown") => {
                    let header = compatible.value().header.as_ref().unwrap();
                    assert!(header.stamp.is_none());
                    assert!(header.data.is_empty());
                }
                None => assert!(compatible.value().header.is_none()),
                other => panic!("unexpected header control {other:?}"),
            }
            if strict {
                assert_eq!(bits(&direct.unwrap()), expected);
                assert_eq!(bits(envelope.unwrap().value()), expected);
                let reflected = reflected.unwrap();
                let values = reflected.get_field_by_name("data").unwrap();
                let Value::List(values) = values.as_ref() else {
                    panic!("float list")
                };
                let actual: Vec<_> = values
                    .iter()
                    .map(|v| v.as_f32().unwrap().to_bits())
                    .collect();
                assert_eq!(actual, expected, "{}", row["id"]);
            }
        }
    }
    assert_eq!((strict_count, compatible_count), (28, 31));
}

type State = (Vec<u32>, Option<String>, Vec<u8>);
fn state(mut input: impl Buf) -> State {
    let mut values = vec![-0.0_f32];
    let error = encoding::float::merge_repeated(
        encoding::WireType::LengthDelimited,
        &mut values,
        &mut input,
        encoding::DecodeContext::default(),
    )
    .err()
    .map(|error| error.to_string());
    let tail = input.copy_to_bytes(input.remaining()).to_vec();
    (values.into_iter().map(f32::to_bits).collect(), error, tail)
}
#[test]
fn existing_fixed32_partial_values_and_all_chunk_cursor_boundaries() {
    let zero = (-0.0_f32).to_bits();
    let one = 1.25_f32.to_bits();
    let payload = one.to_le_bytes();
    let mut valid = vec![8];
    valid.extend(payload);
    valid.extend(zero.to_le_bytes());
    valid.extend([8, 1]);
    let mut crossing = vec![3];
    crossing.extend(&payload[..3]);
    crossing.extend([8, 1]);
    let mut five = vec![5];
    five.extend(payload);
    five.extend([0, 8, 1, 21, 0, 0, 0, 0]);
    let cases = [
        (vec![0], vec![zero], None, vec![]),
        (valid, vec![zero, one, zero], None, vec![8, 1]),
        (
            vec![4, 1, 2, 3],
            vec![zero],
            Some("buffer underflow"),
            vec![1, 2, 3],
        ),
        // The existing decoder can consume suffix bytes before reporting a limit error.
        (
            crossing,
            vec![zero, 0x08a00000],
            Some("delimited length exceeded"),
            vec![1],
        ),
        (
            five,
            vec![zero, one, 0x15010800],
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
            error.map(|error| format!("failed to decode Protobuf message: {error}")),
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

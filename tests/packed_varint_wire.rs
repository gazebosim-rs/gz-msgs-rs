// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use gz_msgs::{
    msgs::{Header, Int32V, Int64V, UInt32V, UInt64V},
    prost::{bytes::Buf, encoding},
    prost_reflect::{DynamicMessage, Value},
    serde_json::{self, Value as Json},
    CanonicalMessage, WireMessage,
};
use std::{
    io::Write,
    process::{Command, Stdio},
    sync::Once,
};

trait VectorMessage: CanonicalMessage {
    fn values(&self) -> Vec<Json>;
    fn header(&self) -> Option<&Header>;
}
macro_rules! vector_message {
    ($($ty:ty),*) => {$(impl VectorMessage for $ty {
        fn values(&self) -> Vec<Json> { self.data.iter().map(|v| serde_json::json!(v)).collect() }
        fn header(&self) -> Option<&Header> { self.header.as_ref() }
    })*};
}
vector_message!(Int32V, UInt32V, Int64V, UInt64V);
fn unhex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn peer(row: &Json, known: bool) -> std::process::Output {
    static BUILD: Once = Once::new();
    BUILD.call_once(|| {
        assert!(Command::new("python3")
            .arg("scripts/native-varint-wire-peer.py")
            .status()
            .unwrap()
            .success())
    });
    let mut command = Command::new("target/native-varint-wire/peer");
    command.arg(row["message"].as_str().unwrap());
    if known {
        command.arg("known");
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&unhex(row["wire_hex"].as_str().unwrap()))
        .unwrap();
    child.wait_with_output().unwrap()
}
fn header(value: Option<&Header>, expected: Option<&str>) {
    match expected {
        Some("stamp7") => {
            let h = value.unwrap();
            let stamp = h.stamp.as_ref().unwrap();
            assert_eq!((stamp.sec, stamp.nsec), (7, 0));
            assert!(h.data.is_empty());
        }
        Some("present" | "unknown") => {
            let h = value.unwrap();
            assert!(h.stamp.is_none());
            assert!(h.data.is_empty());
        }
        None => assert!(value.is_none()),
        other => panic!("unexpected header {other:?}"),
    }
}
fn check<T: VectorMessage>(row: &Json) {
    let wire = unhex(row["wire_hex"].as_str().unwrap());
    let strict = row["strict"].as_bool().unwrap();
    let compatible = row["compatible"].as_bool().unwrap();
    let direct = T::decode(wire.as_slice());
    let envelope = WireMessage::<T>::decode(&wire);
    let loose = WireMessage::<T>::decode_compatible(&wire);
    let reflected = DynamicMessage::decode(
        gz_msgs::descriptor(row["message"].as_str().unwrap()).unwrap(),
        wire.as_slice(),
    );
    assert_eq!(direct.is_ok(), strict, "{} direct", row["id"]);
    assert_eq!(envelope.is_ok(), strict, "{} envelope", row["id"]);
    assert_eq!(reflected.is_ok(), strict, "{} reflection", row["id"]);
    assert_eq!(loose.is_ok(), compatible, "{} compatible", row["id"]);
    if !row["accepted"].as_bool().unwrap() {
        return;
    }
    let expected = row["expected_values"].as_array().unwrap();
    let known = unhex(row["cpp_known"]["output_hex"].as_str().unwrap());
    let native = T::decode(known.as_slice()).unwrap();
    assert_eq!(&native.values(), expected, "{} native values", row["id"]);
    header(native.header(), row["header"].as_str());
    if compatible {
        let loose = loose.unwrap();
        assert_eq!(loose.bytes(), wire);
        assert_eq!(
            &loose.value().values(),
            expected,
            "{} compatible values",
            row["id"]
        );
        assert_eq!(
            loose.value().encode_to_vec(),
            known,
            "{} known bytes",
            row["id"]
        );
        header(loose.value().header(), row["header"].as_str());
    }
    if strict {
        let direct = direct.unwrap();
        assert_eq!(&direct.values(), expected);
        assert_eq!(direct.encode_to_vec(), known);
        header(direct.header(), row["header"].as_str());
        assert_eq!(&envelope.unwrap().value().values(), expected);
        let reflected = reflected.unwrap();
        let values = reflected.get_field_by_name("data").unwrap();
        let Value::List(values) = values.as_ref() else {
            panic!("not repeated data");
        };
        let values: Vec<_> = values
            .iter()
            .map(|v| match v {
                Value::I32(v) => serde_json::json!(v),
                Value::U32(v) => serde_json::json!(v),
                Value::I64(v) => serde_json::json!(v),
                Value::U64(v) => serde_json::json!(v),
                _ => panic!("not an integer"),
            })
            .collect();
        assert_eq!(&values, expected);
    }
}
#[test]
fn canonical_integer_vectors_match_native_with_explicit_overflow_boundaries() {
    let fixture: Json =
        serde_json::from_str(include_str!("fixtures/packed-varint-wire.json")).unwrap();
    let rows = fixture["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 208);
    for (key, expected) in [("accepted", 140), ("strict", 112), ("compatible", 124)] {
        assert_eq!(rows.iter().filter(|row| row[key] == true).count(), expected);
    }
    for row in rows {
        for (known, key) in [(false, "cpp_generated"), (true, "cpp_known")] {
            let actual = peer(row, known);
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
        match row["message"].as_str().unwrap() {
            "gz.msgs.Int32_V" => check::<Int32V>(row),
            "gz.msgs.UInt32_V" => check::<UInt32V>(row),
            "gz.msgs.Int64_V" => check::<Int64V>(row),
            "gz.msgs.UInt64_V" => check::<UInt64V>(row),
            other => panic!("unknown vector {other}"),
        }
    }
}

type PartialState = (Vec<u64>, Option<String>, Vec<u8>);
fn state(mut input: impl Buf) -> PartialState {
    let mut values = vec![17];
    let error = encoding::uint64::merge_repeated(
        encoding::WireType::LengthDelimited,
        &mut values,
        &mut input,
        encoding::DecodeContext::default(),
    )
    .err()
    .map(|e| e.to_string());
    let tail = input.copy_to_bytes(input.remaining()).to_vec();
    (values, error, tail)
}
#[test]
fn existing_varint_partial_values_and_fragmented_cursors_remain_explicit() {
    for (input, values, error, tail) in [
        (vec![0], vec![17], None, vec![]),
        (vec![2, 1, 2, 8, 1], vec![17, 1, 2], None, vec![8, 1]),
        (vec![3, 1], vec![17], Some("buffer underflow"), vec![1]),
        // A terminating value may consume suffix bytes before the limit check.
        (
            vec![1, 128, 1],
            vec![17, 128],
            Some("delimited length exceeded"),
            vec![],
        ),
    ] {
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
    let input = [10, 128, 128, 128, 128, 128, 128, 128, 128, 128, 2, 170];
    let invalid = Some("failed to decode Protobuf message: invalid varint".into());
    assert_eq!(
        state(input.as_slice()),
        (vec![17], invalid.clone(), input[1..].to_vec())
    );
    for split in 0..=input.len() {
        let tail = if (2..=10).contains(&split) {
            vec![170]
        } else {
            input[1..].to_vec()
        };
        assert_eq!(
            state(input[..split].chain(&input[split..])),
            (vec![17], invalid.clone(), tail),
            "split {split}"
        );
    }
    let input = [255; 11];
    for split in 0..=input.len() {
        let tail = vec![255; if split == 0 || split == 11 { 11 } else { 1 }];
        assert_eq!(
            state(input[..split].chain(&input[split..])),
            (vec![17], invalid.clone(), tail),
            "split {split}"
        );
    }
}

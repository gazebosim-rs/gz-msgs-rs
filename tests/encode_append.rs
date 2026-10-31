// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use gz_msgs::{encode_append, EncodeAppendError};
use prost::{
    bytes::{Buf, BufMut},
    encoding::{DecodeContext, WireType},
    DecodeError, EncodeError, Message,
};
use prost_types as pb;
use std::{
    error::Error,
    io::Write,
    path::Path,
    process::{Command, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, gz_msgs::runtime::Enumeration)]
#[prost(prost_path = "gz_msgs::runtime")]
#[repr(i32)]
enum EnumValue {
    Zero = 0,
    One = 1,
}

#[derive(Clone, PartialEq, gz_msgs::runtime::Message)]
#[prost(prost_path = "gz_msgs::runtime")]
struct AllPacked {
    #[prost(float, repeated, packed = "true", tag = "1")]
    f32s: Vec<f32>,
    #[prost(double, repeated, packed = "true", tag = "2")]
    f64s: Vec<f64>,
    #[prost(int32, repeated, packed = "true", tag = "3")]
    i32s: Vec<i32>,
    #[prost(int64, repeated, packed = "true", tag = "4")]
    i64s: Vec<i64>,
    #[prost(uint32, repeated, packed = "true", tag = "5")]
    u32s: Vec<u32>,
    #[prost(uint64, repeated, packed = "true", tag = "6")]
    u64s: Vec<u64>,
    #[prost(sint32, repeated, packed = "true", tag = "7")]
    s32s: Vec<i32>,
    #[prost(sint64, repeated, packed = "true", tag = "8")]
    s64s: Vec<i64>,
    #[prost(fixed32, repeated, packed = "true", tag = "9")]
    x32s: Vec<u32>,
    #[prost(fixed64, repeated, packed = "true", tag = "10")]
    x64s: Vec<u64>,
    #[prost(sfixed32, repeated, packed = "true", tag = "11")]
    sx32s: Vec<i32>,
    #[prost(sfixed64, repeated, packed = "true", tag = "12")]
    sx64s: Vec<i64>,
    #[prost(bool, repeated, packed = "true", tag = "13")]
    bools: Vec<bool>,
    #[prost(enumeration = "EnumValue", repeated, packed = "true", tag = "14")]
    enums: Vec<i32>,
}
fn repeated<T: Copy>(values: &[T], length: usize) -> Vec<T> {
    values.iter().copied().cycle().take(length).collect()
}
fn value(length: usize) -> AllPacked {
    let f32s = [
        0.0,
        -0.0,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::from_bits(1),
        f32::MAX,
        f32::from_bits(0x7fc01234),
        f32::from_bits(0xff800001),
    ];
    let f64s = [
        0.0,
        -0.0,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::from_bits(1),
        f64::MAX,
        f64::from_bits(0x7ff8000012345678),
        f64::from_bits(0xfff0000000000001),
    ];
    let i32s = [i32::MIN, -1, 0, 1, 127, 128, i32::MAX];
    let i64s = [i64::MIN, -1, 0, 1, 127, 128, i64::MAX];
    let u32s = [0, 1, 127, 128, u32::MAX];
    let u64s = [0, 1, 127, 128, u64::MAX];
    let bools = [false, true];
    AllPacked {
        f32s: repeated(&f32s, length),
        f64s: repeated(&f64s, length),
        i32s: repeated(&i32s, length),
        i64s: repeated(&i64s, length),
        u32s: repeated(&u32s, length),
        u64s: repeated(&u64s, length),
        s32s: repeated(&i32s, length),
        s64s: repeated(&i64s, length),
        x32s: repeated(&u32s, length),
        x64s: repeated(&u64s, length),
        sx32s: repeated(&i32s, length),
        sx64s: repeated(&i64s, length),
        bools: repeated(&bools, length),
        enums: repeated(&i32s, length),
    }
}
fn descriptor() -> Vec<u8> {
    use pb::field_descriptor_proto::{Label, Type};
    let kinds = [
        Type::Float,
        Type::Double,
        Type::Int32,
        Type::Int64,
        Type::Uint32,
        Type::Uint64,
        Type::Sint32,
        Type::Sint64,
        Type::Fixed32,
        Type::Fixed64,
        Type::Sfixed32,
        Type::Sfixed64,
        Type::Bool,
        Type::Enum,
    ];
    pb::FileDescriptorSet {
        file: vec![pb::FileDescriptorProto {
            name: Some("buffer-control.proto".into()),
            package: Some("probe".into()),
            syntax: Some("proto3".into()),
            message_type: vec![pb::DescriptorProto {
                name: Some("AllPacked".into()),
                field: kinds
                    .into_iter()
                    .enumerate()
                    .map(|(i, kind)| pb::FieldDescriptorProto {
                        name: Some(format!("field{}", i + 1)),
                        number: Some((i + 1) as i32),
                        r#type: Some(kind as i32),
                        label: Some(Label::Repeated as i32),
                        type_name: (kind == Type::Enum).then(|| ".probe.EnumValue".into()),
                        options: Some(pb::FieldOptions {
                            packed: Some(true),
                            ..Default::default()
                        }),
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            }],
            enum_type: vec![pb::EnumDescriptorProto {
                name: Some("EnumValue".into()),
                value: vec![
                    pb::EnumValueDescriptorProto {
                        name: Some("ZERO".into()),
                        number: Some(0),
                        ..Default::default()
                    },
                    pb::EnumValueDescriptorProto {
                        name: Some("ONE".into()),
                        number: Some(1),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
    .encode_to_vec()
}

fn native(peer: &Path, descriptor: &Path, name: &str, wire: &[u8]) -> Vec<u8> {
    let mut child = Command::new(peer)
        .arg(descriptor)
        .args([name, "wire"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(wire).unwrap();
    let result = child.wait_with_output().unwrap();
    assert_eq!(result.status.code(), Some(0));
    assert!(result.stderr.is_empty());
    result.stdout
}

#[test]
fn packed_widths_preserve_native_bits_prefix_and_reused_capacity() {
    let directory = tempfile::tempdir().unwrap();
    let peer = directory.path().join("peer");
    assert!(Command::new("c++")
        .args([
            "-std=c++17",
            "-O2",
            "tests/protobuf_peer.cc",
            "-lprotobuf",
            "-o"
        ])
        .arg(&peer)
        .status()
        .unwrap()
        .success());
    let schema = directory.path().join("packed.pb");
    std::fs::write(&schema, descriptor()).unwrap();
    let prefix = [0xde, 0xad, 0xbe, 0xef];
    let mut output = Vec::with_capacity(value(1080).encoded_len() + prefix.len());
    let pointer = output.as_ptr();
    let capacity = output.capacity();
    for length in [
        0, 1, 8, 15, 16, 17, 31, 32, 33, 63, 64, 65, 128, 257, 1080, 1, 0, 1080,
    ] {
        let message = value(length);
        let expected = message.encode_to_vec();
        output.clear();
        output.extend(prefix);
        encode_append(&message, &mut output).unwrap();
        assert_eq!(&output[..prefix.len()], prefix);
        assert_eq!(&output[prefix.len()..], expected);
        assert_eq!(
            native(&peer, &schema, "probe.AllPacked", &output[prefix.len()..]),
            expected
        );
        assert_eq!(output.as_ptr(), pointer);
        assert_eq!(output.capacity(), capacity);
        let decoded = AllPacked::decode(&output[prefix.len()..]).unwrap();
        assert_eq!(
            decoded.f32s.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            message.f32s.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        );
        assert_eq!(
            decoded.f64s.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            message.f64s.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        );
    }
    let canonical = directory.path().join("canonical.pb");
    std::fs::write(
        &canonical,
        include_bytes!(concat!(env!("OUT_DIR"), "/messages.pb")),
    )
    .unwrap();
    for count in [0, 1, 360, 1080] {
        let message = gz_msgs::msgs::LaserScan {
            header: Some(gz_msgs::msgs::Header {
                stamp: Some(gz_msgs::Time { sec: -1, nsec: 42 }),
                ..Default::default()
            }),
            frame: "robot/lidar".into(),
            count,
            ranges: repeated(
                &[
                    -0.0,
                    1.25,
                    f64::INFINITY,
                    f64::from_bits(0x7ff0000000000001),
                ],
                count as usize,
            ),
            intensities: repeated(&[0.0, 128.0], count as usize),
            ..Default::default()
        };
        output.clear();
        output.extend(prefix);
        encode_append(&message, &mut output).unwrap();
        let wire = &output[prefix.len()..];
        assert_eq!(wire, message.encode_to_vec());
        assert_eq!(native(&peer, &canonical, "gz.msgs.LaserScan", wire), wire);
    }
    let any = prost_types::Any::from_msg(&gz_msgs::Time { sec: -1, nsec: 42 }).unwrap();
    output.clear();
    output.extend(prefix);
    encode_append(&any, &mut output).unwrap();
    let wire = &output[prefix.len()..];
    assert_eq!(wire, any.encode_to_vec());
    assert_eq!(native(&peer, &canonical, "google.protobuf.Any", wire), wire);
}

#[test]
fn standard_any_identity_and_consecutive_message_boundaries() {
    fn standard<M: prost::Message>(message: &M, output: &mut Vec<u8>) {
        encode_append(message, output).unwrap();
    }
    let time = gz_msgs::Time { sec: -1, nsec: 42 };
    let any = prost_types::Any::from_msg(&time).unwrap();
    assert_eq!(any.type_url, "type.googleapis.com/gz.msgs.Time");
    assert_eq!(any.to_msg::<gz_msgs::Time>().unwrap(), time);
    let mut output = vec![0xaa];
    let expected = any.encode_to_vec();
    for _ in 0..2 {
        let start = output.len();
        standard(&any, &mut output);
        assert_eq!(&output[start..], expected);
        assert_eq!(prost_types::Any::decode(&output[start..]).unwrap(), any);
    }
    assert_eq!(output, [&[0xaa][..], &expected, &expected].concat());
    output.clear();
    standard(&gz_msgs::Time::default(), &mut output);
    assert!(output.is_empty());
    output.extend([0xaa, 0xbb]);
    let pointer = output.as_ptr();
    standard(&AllPacked::default(), &mut output);
    assert_eq!(output, [0xaa, 0xbb]);
    assert_eq!(output.as_ptr(), pointer);
}

#[test]
fn reflective_unknown_fields_keep_standard_encoding_semantics() {
    let mut wire = gz_msgs::Time { sec: -1, nsec: 42 }.encode_to_vec();
    wire.extend([0xa0, 6, 123]);
    let descriptor = gz_msgs::descriptor("gz.msgs.Time").unwrap();
    let message =
        gz_msgs::prost_reflect::DynamicMessage::decode(descriptor, wire.as_slice()).unwrap();
    assert_eq!(message.unknown_fields().count(), 1);
    let mut output = vec![0xa5];
    encode_append(&message, &mut output).unwrap();
    assert_eq!(&output[1..], message.encode_to_vec());
    assert_eq!(&output[1..], wire);
}

/// A deliberately inconsistent public Message implementation, not a new codec.
struct Reported {
    length: AtomicUsize,
    next_length: usize,
    bytes: &'static [u8],
}
impl Reported {
    fn new(first: usize, next: usize, bytes: &'static [u8]) -> Self {
        Self {
            length: AtomicUsize::new(first),
            next_length: next,
            bytes,
        }
    }
}
impl Message for Reported {
    fn encode_raw(&self, buf: &mut impl BufMut) {
        buf.put_slice(self.bytes);
    }
    fn merge_field(
        &mut self,
        _: u32,
        _: WireType,
        _: &mut impl Buf,
        _: DecodeContext,
    ) -> Result<(), DecodeError> {
        panic!("encoding-only control")
    }
    fn encoded_len(&self) -> usize {
        self.length.swap(self.next_length, Ordering::SeqCst)
    }
    fn clear(&mut self) {
        self.bytes = &[];
    }
}

#[test]
fn returned_length_and_reservation_errors_keep_original_bytes() {
    let mut output = vec![0xa5, 0x5a];
    let original = output.clone();
    let error =
        encode_append(&Reported::new(usize::MAX, usize::MAX, &[]), &mut output).unwrap_err();
    assert!(matches!(error, EncodeAppendError::LengthOverflow));
    assert_eq!(error.to_string(), "encoded output length overflow");
    assert!(error.source().is_none());
    assert!(format!("{error:?}").contains("LengthOverflow"));
    assert_eq!(output, original);

    let mut empty = Vec::new();
    let error = encode_append(&Reported::new(usize::MAX, usize::MAX, &[]), &mut empty).unwrap_err();
    assert!(matches!(error, EncodeAppendError::Reserve(_)));
    assert!(error
        .to_string()
        .starts_with("cannot reserve encoded output:"));
    assert!(error.source().is_some());
    assert!(format!("{error:?}").contains("Reserve"));
    assert!(empty.is_empty());

    // A consistent handwritten implementation obeys the same public trait.
    encode_append(&Reported::new(2, 2, &[8, 1]), &mut output).unwrap();
    assert_eq!(&output[..original.len()], original);
    assert_eq!(
        gz_msgs::Time::decode(&output[original.len()..]).unwrap(),
        gz_msgs::Time { sec: 1, nsec: 0 }
    );
}

#[test]
fn inconsistent_short_capacity_error_restores_logical_length() {
    for prefix in [vec![], vec![0xa5, 0x5a]] {
        let mut output = prefix.clone();
        let error = encode_append(&Reported::new(1, 2, &[8, 1]), &mut output).unwrap_err();
        let EncodeAppendError::Encode(inner) = &error else {
            panic!("encode error required")
        };
        assert_eq!(inner.required_capacity(), 2);
        assert_eq!(inner.remaining(), 1);
        assert!(format!("{error:?}").contains("Encode"));
        assert!(error.source().is_some());
        assert!(error
            .to_string()
            .starts_with("cannot encode appended message:"));
        assert_eq!(output, prefix);
    }
}

#[test]
fn successful_incomplete_custom_encoder_never_exposes_padding() {
    for (first, next, bytes, written) in [
        (4, 4, &[8, 1][..], 2),
        (4, 0, &[][..], 0),
        (4, 2, &[8, 1][..], 2),
    ] {
        let mut output = vec![0xa5, 0x5a];
        let error = encode_append(&Reported::new(first, next, bytes), &mut output).unwrap_err();
        assert!(
            matches!(error, EncodeAppendError::IncompleteWrite { expected: 4, written: n } if n == written)
        );
        assert_eq!(
            error.to_string(),
            format!("message wrote {written} of 4 encoded bytes")
        );
        assert!(error.source().is_none());
        assert!(format!("{error:?}").contains("IncompleteWrite"));
        assert_eq!(output, [0xa5, 0x5a]);
        encode_append(&gz_msgs::Time::default(), &mut output).unwrap();
        assert_eq!(output, [0xa5, 0x5a]);
        let time = gz_msgs::Time { sec: 1, nsec: 2 };
        encode_append(&time, &mut output).unwrap();
        assert_eq!(&output[2..], time.encode_to_vec());
    }
}

struct PartialError;
impl Message for PartialError {
    fn encode_raw(&self, buf: &mut impl BufMut) {
        buf.put_slice(&[8, 1]);
    }
    fn merge_field(
        &mut self,
        _: u32,
        _: WireType,
        _: &mut impl Buf,
        _: DecodeContext,
    ) -> Result<(), DecodeError> {
        panic!("encoding-only control")
    }
    fn encoded_len(&self) -> usize {
        4
    }
    fn clear(&mut self) {}
    fn encode(&self, buf: &mut impl BufMut) -> Result<(), EncodeError> {
        self.encode_raw(buf);
        let mut insufficient: &mut [u8] = &mut [];
        gz_msgs::Time { sec: 1, nsec: 0 }.encode(&mut insufficient)
    }
}

#[test]
fn returned_custom_error_after_partial_write_keeps_prefix_and_allows_retry() {
    let prefix = [0xde, 0xad, 0xbe, 0xef];
    let mut output = prefix.to_vec();
    assert!(matches!(
        encode_append(&PartialError, &mut output),
        Err(EncodeAppendError::Encode(_))
    ));
    assert_eq!(output, prefix);
    let time = gz_msgs::Time { sec: -1, nsec: 42 };
    encode_append(&time, &mut output).unwrap();
    assert_eq!(&output[..prefix.len()], prefix);
    assert_eq!(&output[prefix.len()..], time.encode_to_vec());
}

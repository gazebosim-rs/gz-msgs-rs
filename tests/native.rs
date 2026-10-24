// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use gz_msgs::{prost::Message, *};
use std::{
    io::Write,
    process::{Command, Stdio},
    sync::Once,
};
static BUILD: Once = Once::new();
fn peer(name: &str, mode: &str, bytes: &[u8]) -> std::process::Output {
    BUILD.call_once(|| {
        assert!(Command::new("python3")
            .arg("scripts/native-peer.py")
            .status()
            .unwrap()
            .success())
    });
    let mut child = Command::new("target/native/protobuf-peer")
        .args(["target/native/messages.pb", name, mode])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(bytes).unwrap();
    child.wait_with_output().unwrap()
}
#[test]
fn cpp_to_rust_and_back() {
    for (name, text) in [
        ("gz.msgs.Clock", "system { sec: -12 nsec: 42 } sim { sec: 9223372036854775807 } real {}"),
        ("gz.msgs.Pose", "name: 'body' id: 4294967295 position { x: 1.25 y: -2 z: 3 } orientation { w: 1 }"),
        ("gz.msgs.IMU", "entity_name: 'imu' orientation { w: 1 } angular_velocity { x: 2 } linear_acceleration { z: -9.8 }"),
        ("gz.msgs.Discovery", "version: 11 type: 123 pub { topic: '/clock' scope: ALL msg_pub { msg_type: 'gz.msgs.Clock' } }"),
    ] {
        let cpp = peer(name, "encode", text.as_bytes());
        assert!(cpp.status.success(), "{}", String::from_utf8_lossy(&cpp.stderr));
        let desc = descriptor(name).unwrap();
        let rust = prost_reflect::DynamicMessage::decode(desc,cpp.stdout.as_slice()).unwrap();
        let rust_bytes = rust.encode_to_vec();
        let decoded = peer(name, "decode", &rust_bytes);
        assert!(decoded.status.success());
        let expected = peer(name, "decode", &cpp.stdout);
        assert_eq!(decoded.stdout, expected.stdout);
    }
    let pose = Pose {
        name: "rust".into(),
        id: 37,
        position: Some(vector3([1., 2., 3.])),
        ..Default::default()
    };
    let decoded = peer("gz.msgs.Pose", "decode", &pose.encode_to_vec());
    assert!(decoded.status.success());
    assert_eq!(
        String::from_utf8(decoded.stdout).unwrap(),
        "name: \"rust\"\nid: 37\nposition {\n  x: 1\n  y: 2\n  z: 3\n}\n"
    );
}
#[test]
fn unknown_and_malformed_cross_language() {
    let bytes = [8, 7, 0xa0, 6, 123];
    let cpp = peer("gz.msgs.Time", "roundtrip", &bytes);
    assert!(cpp.status.success());
    let rust = WireMessage::<Time>::decode(cpp.stdout.as_slice())
        .unwrap()
        .bytes()
        .to_vec();
    assert_eq!(rust, bytes);
    let cpp = peer("gz.msgs.Time", "decode", &rust);
    assert_eq!(String::from_utf8(cpp.stdout).unwrap(), "sec: 7\n100: 123\n");
    let bytes = [8, 128];
    assert!(Time::decode(bytes.as_slice()).is_err());
    assert!(!peer("gz.msgs.Time", "roundtrip", &bytes).status.success());
}

#[test]
fn bytes_oneofs_floats_and_duplicate_fields() {
    let image = image::Image {
        width: 1024,
        height: 1024,
        step: 1024,
        data: (0..=255).cycle().take(1024 * 1024).collect(),
        pixel_format_type: 12345,
        ..Default::default()
    };
    let cpp = peer("gz.msgs.Image", "roundtrip", &image.encode_to_vec());
    assert!(cpp.status.success());
    assert_eq!(image::Image::decode(cpp.stdout.as_slice()).unwrap(), image);
    for (name, text) in [
        ("gz.msgs.Any", "type: DOUBLE double_value: nan"),
        (
            "gz.msgs.Any",
            "type: STRING string_value: 'hello \\342\\230\\203'",
        ),
        ("gz.msgs.Pose", "position {} orientation {}"),
        ("gz.msgs.Vector3d", "x: inf y: -inf z: -0"),
    ] {
        let encoded = peer(name, "encode", text.as_bytes());
        assert!(encoded.status.success());
        let rust = prost_reflect::DynamicMessage::decode(
            descriptor(name).unwrap(),
            encoded.stdout.as_slice(),
        )
        .unwrap();
        let bytes = rust.encode_to_vec();
        assert_eq!(
            peer(name, "decode", &bytes).stdout,
            peer(name, "decode", &encoded.stdout).stdout
        );
    }
    let duplicate = [8, 1, 8, 2];
    let cpp = peer("gz.msgs.Time", "roundtrip", &duplicate);
    assert!(cpp.status.success());
    assert_eq!(cpp.stdout, [8, 2]);
    assert_eq!(
        Time::decode(duplicate.as_slice()).unwrap().encode_to_vec(),
        cpp.stdout
    );
}

#[test]
fn protoc_and_rust_schema_descriptors() {
    for file in files() {
        for descriptor in file.messages() {
            let cpp = peer(descriptor.full_name(), "descriptor", &[]);
            assert!(cpp.status.success());
            let cpp = gz_msgs::prost_types::DescriptorProto::decode(cpp.stdout.as_slice()).unwrap();
            assert_eq!(
                &cpp,
                descriptor.descriptor_proto(),
                "{}",
                descriptor.full_name()
            );
        }
    }
}

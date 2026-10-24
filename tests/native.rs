use gz_msgs::{protobuf::Message, *};
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
        let rust = desc.parse_from_bytes(&cpp.stdout).unwrap();
        let rust_bytes = rust.write_to_bytes_dyn().unwrap();
        let decoded = peer(name, "decode", &rust_bytes);
        assert!(decoded.status.success());
        let expected = peer(name, "decode", &cpp.stdout);
        assert_eq!(decoded.stdout, expected.stdout);
    }
    let pose = Pose {
        name: "rust".into(),
        id: 37,
        position: Some(vector3([1., 2., 3.])).into(),
        ..Default::default()
    };
    let decoded = peer("gz.msgs.Pose", "decode", &pose.write_to_bytes().unwrap());
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
    let rust = Time::parse_from_bytes(&cpp.stdout)
        .unwrap()
        .write_to_bytes()
        .unwrap();
    assert_eq!(rust, bytes);
    let cpp = peer("gz.msgs.Time", "decode", &rust);
    assert_eq!(String::from_utf8(cpp.stdout).unwrap(), "sec: 7\n100: 123\n");
    let bytes = [8, 128];
    assert!(Time::parse_from_bytes(&bytes).is_err());
    assert!(!peer("gz.msgs.Time", "roundtrip", &bytes).status.success());
}

//! Nonrelease dependency qualification: failures are reported and return nonzero.
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage};
use std::{
    io::Write,
    process::{Command, Stdio},
};
include!(concat!(env!("OUT_DIR"), "/gz.msgs.rs"));
fn peer(name: &str, mode: &str, bytes: &[u8]) -> Vec<u8> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut process = Command::new(root.join("target/native/protobuf-peer"))
        .arg(root.join("target/native/messages.pb"))
        .args([name, mode])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    process.stdin.take().unwrap().write_all(bytes).unwrap();
    let output = process.wait_with_output().unwrap();
    assert!(output.status.success());
    output.stdout
}
struct WireMessage<M> {
    bytes: Vec<u8>,
    value: M,
}
impl<M: prost::Message + Default> WireMessage<M> {
    fn decode(bytes: &[u8]) -> std::result::Result<Self, prost::DecodeError> {
        Ok(Self {
            bytes: bytes.to_vec(),
            value: M::decode(bytes)?,
        })
    }
    fn value(&self) -> &M {
        &self.value
    }
    fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
fn main() {
    let pool =
        DescriptorPool::decode(include_bytes!(concat!(env!("OUT_DIR"), "/messages.pb")).as_ref())
            .unwrap();
    let mut results = Vec::new();
    let mut check = |name: &str, passes: bool| {
        println!("{}: {}", name, if passes { "PASS" } else { "FAIL" });
        results.push(serde_json::json!({"probe":name,"passed":passes,"required":name!="generated_unknown_fields"}));
    };
    check(
        "canonical_schema_count_plus_google_any",
        pool.files().count() == 217,
    );
    check(
        "dynamic_descriptor_instances",
        pool.all_messages()
            .all(|m| DynamicMessage::decode(m, [].as_slice()).is_ok()),
    );
    check(
        "nested_descriptor_name",
        pool.get_message_by_name("gz.msgs.Discovery.Publisher.ServicePublisher")
            .is_some(),
    );
    let unknown = [8, 7, 0xa0, 6, 123];
    check(
        "generated_unknown_fields",
        Time::decode(unknown.as_slice()).unwrap().encode_to_vec() == unknown,
    );
    let dynamic = DynamicMessage::decode(
        pool.get_message_by_name("gz.msgs.Time").unwrap(),
        unknown.as_slice(),
    )
    .unwrap();
    check("dynamic_unknown_fields", dynamic.encode_to_vec() == unknown);
    check(
        "dynamic_unknown_fields_cpp",
        peer("gz.msgs.Time", "roundtrip", &dynamic.encode_to_vec()) == unknown,
    );
    let envelope = WireMessage::<Time>::decode(&unknown).unwrap();
    check(
        "wire_envelope_unknown_forwarding",
        envelope.value().sec == 7 && envelope.bytes() == unknown,
    );
    let nested = [34, 5, 8, 7, 160, 6, 123];
    let envelope = WireMessage::<Clock>::decode(&nested).unwrap();
    check(
        "wire_envelope_nested_unknown_forwarding",
        envelope.value().sim.as_ref().unwrap().sec == 7 && envelope.bytes() == nested,
    );
    let negative_zero = peer("gz.msgs.Vector3d", "encode", b"x: -0");
    check(
        "generated_signed_zero",
        Vector3d::decode(negative_zero.as_slice())
            .unwrap()
            .encode_to_vec()
            == negative_zero,
    );
    let dynamic = DynamicMessage::decode(
        pool.get_message_by_name("gz.msgs.Vector3d").unwrap(),
        negative_zero.as_slice(),
    )
    .unwrap();
    check(
        "dynamic_signed_zero",
        dynamic.encode_to_vec() == negative_zero,
    );
    check("text_signed_zero", dynamic.to_text_format().contains("-0"));
    check(
        "json_signed_zero",
        serde_json::to_string(&dynamic).unwrap().contains("-0"),
    );
    let negative_float = peer("gz.msgs.Float", "encode", b"data: -0");
    check(
        "generated_f32_signed_zero",
        Float::decode(negative_float.as_slice())
            .unwrap()
            .encode_to_vec()
            == negative_float,
    );
    let dynamic_float = DynamicMessage::decode(
        pool.get_message_by_name("gz.msgs.Float").unwrap(),
        negative_float.as_slice(),
    )
    .unwrap();
    check(
        "dynamic_f32_signed_zero",
        dynamic_float.encode_to_vec() == negative_float,
    );
    check(
        "text_f32_signed_zero",
        dynamic_float.to_text_format().contains("-0"),
    );
    check(
        "json_f32_signed_zero",
        serde_json::to_string(&dynamic_float)
            .unwrap()
            .contains("-0"),
    );
    for (name,text) in [
        ("gz.msgs.Clock","system { sec: -12 nsec: 42 } sim { sec: 9223372036854775807 } real {}"),
        ("gz.msgs.Pose","name: 'body' id: 4294967295 position { x: 1.25 y: -2 z: 3 } orientation { w: 1 }"),
        ("gz.msgs.IMU","entity_name: 'imu' orientation { w: 1 } angular_velocity { x: 2 } linear_acceleration { z: -9.8 }"),
        ("gz.msgs.Discovery","version: 11 type: 123 pub { topic: '/clock' scope: ALL msg_pub { msg_type: 'gz.msgs.Clock' } }"),
    ] {
        let desc=pool.get_message_by_name(name).unwrap();
        let cpp=peer(name,"encode",text.as_bytes());
        let message=DynamicMessage::decode(desc.clone(),cpp.as_slice()).unwrap();
        check(&format!("cpp_binary_{name}"),peer(name,"decode",&message.encode_to_vec())==peer(name,"decode",&cpp));
        let text=message.to_text_format();
        let parsed=DynamicMessage::parse_text_format(desc.clone(),&text).unwrap();
        check(&format!("text_{name}"),peer(name,"decode",&parsed.encode_to_vec())==peer(name,"decode",&cpp));
        let json=serde_json::to_string(&message).unwrap();
        let parsed=DynamicMessage::deserialize(desc,&mut serde_json::Deserializer::from_str(&json)).unwrap();
        check(&format!("json_{name}"),peer(name,"decode",&parsed.encode_to_vec())==peer(name,"decode",&cpp));
    }
    let bad = [8, 128];
    check("malformed_rejected", Time::decode(bad.as_slice()).is_err());
    let descriptor = pool.get_message_by_name("gz.msgs.Discovery").unwrap();
    check(
        "unknown_enum",
        DynamicMessage::decode(descriptor, [32, 123].as_slice())
            .unwrap()
            .encode_to_vec()
            == [32, 123],
    );
    println!("{}", serde_json::to_string_pretty(&results).unwrap());
    if results
        .iter()
        .any(|r| r["required"] == true && r["passed"] == false)
    {
        std::process::exit(1);
    }
}

#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Verify transitive patched behavior from a clean consumer with no Cargo patches."""
import json, os, pathlib, subprocess, tempfile
root=pathlib.Path(__file__).resolve().parents[1]
with tempfile.TemporaryDirectory(prefix='gz-msgs-consumer-') as directory:
    consumer=pathlib.Path(directory)
    (consumer/'src').mkdir()
    (consumer/'Cargo.toml').write_text('[package]\nname="gazebo-message-consumer"\nversion="0.0.0"\nedition="2021"\n[dependencies]\ngz-msgs={path='+json.dumps(str(root))+'}\nprost="=0.14.4"\nprost-types="=0.14.4"\n')
    (consumer/'src/main.rs').write_text('''use prost::Message;
fn main() {
    let value=gz_msgs::vector3([-0.0,0.0,0.0]);
    let bytes=value.encode_to_vec();
    assert_eq!(bytes,[17,0,0,0,0,0,0,0,128]);
    let reflected=gz_msgs::prost_reflect::DynamicMessage::decode(gz_msgs::descriptor("gz.msgs.Vector3d").unwrap(),bytes.as_slice()).unwrap();
    assert_eq!(reflected.encode_to_vec(),bytes);
    assert!(reflected.to_text_format().contains("-0"));
    assert!(gz_msgs::serde_json::to_string(&reflected).unwrap().contains("-0"));
    let clock=gz_msgs::Clock {sim:Some(gz_msgs::time_from_nanos(123)),..Default::default()};
    let any=prost_types::Any::from_msg(&clock).unwrap();
    assert_eq!(any.type_url,"type.googleapis.com/gz.msgs.Clock");
    assert_eq!(any.to_msg::<gz_msgs::Clock>().unwrap(),clock);
    let parameter=gz_msgs::parameter::Parameter {name:"clock".into(),value:Some(any)};
    assert_eq!(gz_msgs::parameter::Parameter::decode(parameter.encode_to_vec().as_slice()).unwrap(),parameter);
    let unknown=[34,5,8,7,160,6,123];
    let wire=gz_msgs::WireMessage::<gz_msgs::Clock>::decode(&unknown).unwrap();
    assert_eq!(wire.bytes(),unknown);
    let mismatched=[18,5,8,1,18,1,255];
    assert!(gz_msgs::parameter::Parameter::decode(mismatched.as_slice()).is_err());
    let compatible=gz_msgs::WireMessage::<gz_msgs::parameter::Parameter>::decode_compatible(&mismatched).unwrap();
    assert_eq!(compatible.bytes(),mismatched);
    assert_eq!(compatible.value().value,Some(prost_types::Any {type_url:String::new(),value:vec![255]}));
}
''')
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(root/'target/downstream')
    subprocess.run([env.get('CARGO','cargo'),'run','--offline','--manifest-path',str(consumer/'Cargo.toml')],env=env,check=True)
print('Clean downstream consumer: signed zero, standard Any traits, nested unknown forwarding and compatible decoding pass without Cargo patches')

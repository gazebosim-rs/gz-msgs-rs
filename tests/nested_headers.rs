// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2017 Open Source Robotics Foundation
// Original assertions: test/integration/headers.cc, Messages.NestedHeaders.
use gz_msgs::{
    msgs::{Contact, Header, JointWrench, Time},
    prost::Message,
};

#[test]
fn original_nested_headers() {
    let mut contact = Contact::default();
    let mut contact_serialized = Vec::new();
    contact.encode(&mut contact_serialized).unwrap();
    assert_eq!(contact_serialized.len(), 0);
    let mut wrench = JointWrench::default();
    let mut wrench_serialized = Vec::new();
    wrench.encode(&mut wrench_serialized).unwrap();
    assert_eq!(wrench_serialized.len(), 0);
    wrench.header = Some(Header {
        stamp: Some(Time { sec: 123, nsec: 0 }),
        ..Default::default()
    });
    wrench_serialized.clear();
    wrench.encode(&mut wrench_serialized).unwrap();
    assert_eq!(wrench_serialized.len(), 6);
    contact.wrench.push(JointWrench {
        header: Some(Header {
            stamp: Some(Time { sec: 123, nsec: 0 }),
            ..Default::default()
        }),
        ..Default::default()
    });
    contact_serialized.clear();
    contact.encode(&mut contact_serialized).unwrap();
    assert_eq!(contact_serialized.len(), 8);
    contact_serialized.clear();
    contact.wrench[0].encode(&mut contact_serialized).unwrap();
    assert_eq!(contact_serialized.len(), wrench_serialized.len());
}

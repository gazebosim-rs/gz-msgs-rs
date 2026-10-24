// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Ports the original 2026 OSRF FactoryLazyTest assertions.
// No test harness/background threads: this process mutates its own environment
// before starting any threads, as required by std's environment safety contract.
use gz_msgs::factory::MessageFactory;
fn main() {
    std::env::remove_var("GZ_DESCRIPTOR_PATH");
    let factory = MessageFactory::default();
    assert!(factory.new_message("gz.msgs.StringMsg").is_some());
    let frozen = MessageFactory::default();
    assert!(frozen.new_message(".example.msgs.StringMsg").is_none());
    std::env::set_var("GZ_DESCRIPTOR_PATH", "tests/reference/descriptors");
    assert_eq!(
        std::env::var("GZ_DESCRIPTOR_PATH").unwrap(),
        "tests/reference/descriptors"
    );
    assert!(factory.new_message("example.msgs.StringMsg").is_some());
    assert!(frozen.new_message("example.msgs.StringMsg").is_none());
    println!("FactoryLazyTest.DescriptorPathReadAtFirstDynamicUse: passed");
}

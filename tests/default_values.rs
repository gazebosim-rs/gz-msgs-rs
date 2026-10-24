// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2016 Open Source Robotics Foundation
// Original Utility_TEST.cc, UtilityTest.Initialization.
use gz_msgs::msgs::{Vector3d, Wrench};

#[test]
fn original_vector_and_nested_wrench_initialization() {
    let message = Vector3d::default();
    assert_eq!(message.x, 0.);
    assert_eq!(message.y, 0.);
    assert_eq!(message.z, 0.);
    let message = Wrench::default();
    // Generated Rust messages represent absent nested fields explicitly. The
    // native getter's default view is the default nested value, without insertion.
    let force = message.force.as_ref().cloned().unwrap_or_default();
    let torque = message.torque.as_ref().cloned().unwrap_or_default();
    assert_eq!(force.x, 0.);
    assert_eq!(force.y, 0.);
    assert_eq!(force.z, 0.);
    assert_eq!(torque.x, 0.);
    assert_eq!(torque.y, 0.);
    assert_eq!(torque.z, 0.);
    assert!(message.force.is_none());
    assert!(message.torque.is_none());
}

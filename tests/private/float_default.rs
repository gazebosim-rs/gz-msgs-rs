// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use super::{double, float};
#[test]
fn native_declaration_policy_matches_recorded_number_bits() {
    let mut count = 0;
    for line in include_str!("../fixtures/floating-format.txt").lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        assert_eq!(fields.len(), 3);
        let bits = u64::from_str_radix(fields[1], 16).unwrap();
        let actual = match fields[0] {
            "f32" => float(f32::from_bits(bits as u32)),
            "f64" => double(f64::from_bits(bits)),
            _ => panic!("unknown width"),
        };
        assert_eq!(actual, fields[2], "{line}");
        count += 1;
    }
    assert_eq!(count, 21397);
}

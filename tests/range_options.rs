// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use gz_msgs::{prost::Message, registry::DescriptorRegistry};

#[test]
fn range_options_preserve_order_absence_unknown_and_custom_metadata() {
    let mut registry = DescriptorRegistry::default();
    registry
        .load_bytes(include_bytes!("fixtures/range-options.desc"))
        .map(|errors| assert!(errors.is_empty(), "{errors:?}"))
        .unwrap();
    let message = registry
        .pool()
        .get_message_by_name("range_options.Root")
        .unwrap();
    let before = message.parent_file().encode_to_vec();
    let options: Vec<_> = message.extension_range_options().collect();
    assert_eq!(options.len(), 4);
    assert!(options[0].encode_to_vec().is_empty());
    assert!(options[1].encode_to_vec().is_empty());
    assert_eq!(options[2].encode_to_vec(), [0x80, 0x7d, 7]);
    assert_eq!(options[2].unknown_fields().count(), 1);
    assert_eq!(options[3].extensions().count(), 1);
    assert_eq!(
        options[3].extensions().next().unwrap().0.full_name(),
        "range_options.custom"
    );
    assert_eq!(
        options[3].extensions().next().unwrap().1.as_bool(),
        Some(true)
    );
    assert!(message.extension_range_options().nth(4).is_none());
    assert!(registry
        .pool()
        .get_message_by_name("google.protobuf.FileDescriptorSet")
        .unwrap()
        .extension_range_options()
        .next()
        .is_none());
    assert_eq!(
        gz_msgs::descriptor_definition(&message),
        Err(gz_msgs::DefinitionError::Unsupported("unknown options"))
    );
    assert_eq!(message.parent_file().encode_to_vec(), before);
    assert_eq!(
        message
            .descriptor_proto()
            .extension_range
            .iter()
            .map(|r| r.start())
            .collect::<Vec<_>>(),
        [100, 110, 120, 130]
    );
}

#[test]
fn unresolved_uninterpreted_range_options_keep_existing_loader_rejection() {
    let mut registry = DescriptorRegistry::default();
    let errors = registry
        .load_bytes(include_bytes!("fixtures/range-options-uninterpreted.desc"))
        .unwrap();
    assert!(!errors.is_empty());
    assert!(registry
        .pool()
        .get_message_by_name("range_options.Root")
        .is_none());
}

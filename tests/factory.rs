// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2016 Open Source Robotics Foundation
// Ports original Factory_TEST.cc and factory_concurrency.cc assertions.
use gz_msgs::{
    factory::MessageFactory,
    prost::Message,
    prost_reflect::{DynamicMessage, ReflectMessage, Value},
};
use std::{
    ffi::OsString,
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
fn string(value: &str) -> DynamicMessage {
    let mut message = DynamicMessage::new(gz_msgs::descriptor("gz.msgs.StringMsg").unwrap());
    message.set_field_by_name("data", Value::String(value.into()));
    message
}
#[test]
fn factory_names_text_and_original_explicit_dynamic_paths() {
    let factory = MessageFactory::with_paths(OsString::new());
    for name in [
        "gz.msgs.Vector3d",
        "gz_msgs.Vector3d",
        ".gz.msgs.Vector3d",
        ".gz_msgs.Vector3d",
    ] {
        assert_eq!(
            factory.new_message(name).unwrap().descriptor().full_name(),
            "gz.msgs.Vector3d"
        );
    }
    assert!(factory.take_initial_errors().is_empty());
    for name in [
        "StringMsg",
        "example.msgs.StringMsg",
        ".google.protobuf.Any",
        "..gz.msgs.StringMsg",
    ] {
        assert!(factory.new_message(name).is_none(), "{name}");
    }
    assert_eq!(factory.types().len(), 271);
    let parsed = factory
        .new_text("gz.msgs.Vector3d", "x: 1.0, y: 2.0, z: 3.0")
        .unwrap()
        .unwrap();
    let value = gz_msgs::msgs::Vector3d::decode(parsed.encode_to_vec().as_slice()).unwrap();
    assert_eq!([value.x, value.y, value.z], [1., 2., 3.]);
    assert!(factory
        .new_text("gz.msgs.Vector3d", "x: 1.0, y: asdf, z: 3.0")
        .is_err());
    assert!(factory
        .new_text("unknown.Type", "invalid")
        .unwrap()
        .is_none());
    let paths = std::env::join_paths([
        Path::new("tests/reference/descriptors"),
        Path::new("tests/reference"),
    ])
    .unwrap();
    // The original explicit DynamicMessageWithEnv test loads these two paths.
    assert_eq!(factory.load_paths(paths).len(), 1); // Native fixture directory contains invalid source text.
    assert!(factory.new_message("example.msgs.StringMsg").is_some());
    assert!(factory.types().contains(&"testing.FooMessage".into()));
    let mut external = factory.new_message("example.msgs.StringMsg").unwrap();
    external.set_field_by_name("data", Value::String("survives".into()));
    drop(factory);
    assert_eq!(
        external.get_field_by_name("data").unwrap().as_str(),
        Some("survives")
    );
    assert!(!external.encode_to_vec().is_empty());
}
#[test]
fn initialization_errors_are_observable_and_drained_once() {
    let missing = "target/does-not-exist/factory.desc";
    let factory = MessageFactory::with_paths(missing);
    assert!(factory.new_message("gz.msgs.StringMsg").is_some());
    assert!(factory.take_initial_errors().is_empty());
    assert!(factory.new_message("unknown.Type").is_none());
    let errors = factory.take_initial_errors();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].0, Path::new(missing));
    assert!(factory.take_initial_errors().is_empty());
    let factory = MessageFactory::with_paths(missing);
    let errors = factory.load_paths("tests/reference/descriptors/stringmsg.desc");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].0, Path::new(missing));
    assert!(factory.new_message("example.msgs.StringMsg").is_some());
    assert!(factory.take_initial_errors().is_empty());
}
#[test]
fn callbacks_replace_reenter_and_values_outlive_factory() {
    let factory = Arc::new(MessageFactory::with_paths(OsString::new()));
    factory.register("custom", || string("first"));
    assert_eq!(
        factory
            .new_message("custom")
            .unwrap()
            .get_field_by_name("data")
            .unwrap()
            .as_str(),
        Some("first")
    );
    factory.register("custom", || string("second"));
    assert_eq!(
        factory
            .new_message("custom")
            .unwrap()
            .get_field_by_name("data")
            .unwrap()
            .as_str(),
        Some("second")
    );
    assert_eq!(
        factory
            .new_text("custom", "")
            .unwrap()
            .unwrap()
            .get_field_by_name("data")
            .unwrap()
            .as_str(),
        Some("")
    );
    let weak = Arc::downgrade(&factory);
    factory.register("reentrant", move || {
        weak.upgrade().unwrap().new_message("custom").unwrap()
    });
    let (send, receive) = std::sync::mpsc::sync_channel(1);
    let other = factory.clone();
    let thread = std::thread::spawn(move || send.send(other.new_message("reentrant")).unwrap());
    let message = receive
        .recv_timeout(Duration::from_secs(5))
        .unwrap()
        .unwrap();
    thread.join().unwrap();
    assert_eq!(
        message.get_field_by_name("data").unwrap().as_str(),
        Some("second")
    );
    factory.register(".registered", || string("dot"));
    assert!(factory.new_message(".registered").is_some());
    let native: gz_msgs::serde_json::Value =
        gz_msgs::serde_json::from_str(include_str!("fixtures/factory-state-reference.json"))
            .unwrap();
    for outcome in [
        "first_constructor",
        "text_clears_constructor_fields",
        "reentrant_created",
        "constructors_listed",
        "literal_dot_constructor",
        "unrecognized_names_rejected",
    ] {
        assert!(native[outcome].as_bool().unwrap());
    }
    let names = factory.types();
    assert_eq!(names.len() as u64, native["types_count"].as_u64().unwrap());
    assert!(names.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(names.contains(&"custom".into()));
    assert!(names.contains(&"reentrant".into()));
    drop(factory);
    assert_eq!(
        message.get_field_by_name("data").unwrap().as_str(),
        Some("second")
    );
}
#[test]
fn original_parallel_use_counts_and_final_construction_assertions() {
    const ITERS: usize = 10000;
    let factory = MessageFactory::with_paths(OsString::new());
    let failures = AtomicUsize::new(0);
    let path = "tests/reference/descriptors/stringmsg.desc";
    assert!(factory.load_paths(path).is_empty());
    assert!(factory.new_message("example.msgs.StringMsg").is_some());
    std::thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(|| {
                for _ in 0..ITERS {
                    if factory.new_message("gz.msgs.StringMsg").is_none() {
                        failures.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });
        }
        for _ in 0..2 {
            scope.spawn(|| {
                for _ in 0..ITERS / 10 {
                    if factory.new_message("example.msgs.StringMsg").is_none() {
                        failures.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });
        }
        scope.spawn(|| {
            for index in 0..ITERS / 10 {
                factory.register(format!("test.factory_concurrency.Msg{index}"), || {
                    string("")
                });
            }
        });
        scope.spawn(|| {
            for _ in 0..ITERS / 100 {
                if factory.types().is_empty() {
                    failures.fetch_add(1, Ordering::Relaxed);
                }
            }
        });
        scope.spawn(|| {
            for _ in 0..20 {
                assert!(factory.load_paths(path).is_empty());
            }
        });
    });
    assert_eq!(failures.load(Ordering::Relaxed), 0);
    for name in [
        "gz.msgs.StringMsg",
        "example.msgs.StringMsg",
        "test.factory_concurrency.Msg0",
    ] {
        assert!(factory.new_message(name).is_some());
    }
}

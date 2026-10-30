// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Isolated single-thread processes keep runtime-environment mutations sound.
#[cfg(all(target_os = "linux", target_env = "gnu"))]
mod qualified {
    use gz_msgs::{
        factory::MessageFactory,
        prost::Message,
        prost_reflect::ReflectMessage,
        prost_types::{DescriptorProto, FileDescriptorProto, FileDescriptorSet},
        serde_json::Value,
    };
    use std::{
        ffi::OsString,
        os::unix::{ffi::OsStringExt, fs::symlink},
        path::PathBuf,
        process::Command,
    };

    fn descriptor(file: &str, message: &str) -> Vec<u8> {
        FileDescriptorSet {
            file: vec![FileDescriptorProto {
                name: Some(file.into()),
                package: Some("audit".into()),
                syntax: Some("proto2".into()),
                message_type: vec![DescriptorProto {
                    name: Some(message.into()),
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
        .encode_to_vec()
    }
    fn names(factory: &MessageFactory) -> Vec<String> {
        factory
            .types()
            .into_iter()
            .filter(|s| s.starts_with("audit.") && s != "audit.Registered")
            .collect()
    }
    fn child(mode: &str) {
        let reference: Value =
            gz_msgs::serde_json::from_str(include_str!("fixtures/factory-install-share.json"))
                .unwrap();
        for (key, file, name) in [
            ("A", "a.proto", "A"),
            ("B_conflict", "a.proto", "B"),
            ("B_other", "b.proto", "B"),
        ] {
            let encoded: String = descriptor(file, name)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect();
            assert_eq!(encoded, reference["inputs"][key].as_str().unwrap());
        }
        let native = reference["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["mode"] == mode);
        assert_eq!(reference["rows"].as_array().unwrap().len(), 16);
        assert_eq!(
            native.is_none(),
            ["unset", "isolated", "isolated-empty"].contains(&mode)
        );
        let root = std::env::current_dir().unwrap();
        let prefix: PathBuf = match mode {
            "empty" => PathBuf::new(),
            "raw" => OsString::from_vec(b"raw-\xff".to_vec()).into(),
            "colon" => "pre:fix".into(),
            "relative" => "prefix".into(),
            "missing-colon" => root.join("missing.desc:gone"),
            "loop" => root.join("loop"),
            _ => root.join("prefix"),
        };
        let share = prefix.join("share/gz/protos");
        std::env::remove_var("GZ_DESCRIPTOR_PATH");
        std::env::set_var("GZ_MSGS_INSTALL_PREFIX", &prefix);
        if mode == "loop" {
            symlink("loop", &prefix).unwrap();
        } else if !["missing", "missing-colon"].contains(&mode) {
            std::fs::create_dir_all(&share).unwrap();
            std::fs::write(share.join("a.desc"), descriptor("a.proto", "A")).unwrap();
        }
        let explicit = root.join("explicit.desc");
        std::fs::write(&explicit, descriptor("a.proto", "B")).unwrap();
        let other = root.join("other");
        std::fs::create_dir_all(other.join("share/gz/protos")).unwrap();
        std::fs::write(
            other.join("share/gz/protos/b.desc"),
            descriptor("b.proto", "B"),
        )
        .unwrap();
        if mode == "unset" {
            std::env::remove_var("GZ_MSGS_INSTALL_PREFIX");
        }
        if mode == "env-first" {
            std::env::set_var("GZ_DESCRIPTOR_PATH", &explicit);
        }
        if ["bad-share", "errors"].contains(&mode) {
            std::fs::write(share.join("a.desc"), [0xff]).unwrap();
        }
        let bad_env = root.join("bad-env.desc");
        let bad_explicit = root.join("bad-explicit.desc");
        if mode == "errors" {
            std::fs::write(&bad_env, [0xff]).unwrap();
            std::fs::write(&bad_explicit, [0xff]).unwrap();
            std::env::set_var("GZ_DESCRIPTOR_PATH", &bad_env);
        }
        let factory = match mode {
            "isolated" => MessageFactory::with_paths(explicit.as_os_str()),
            "isolated-empty" => MessageFactory::with_paths(""),
            _ => MessageFactory::default(),
        };
        assert!(factory.take_initial_errors().is_empty());
        if mode == "generated" {
            assert!(factory.new_message("gz.msgs.StringMsg").is_some());
            std::env::set_var("GZ_MSGS_INSTALL_PREFIX", &other);
        }
        if mode == "registered" {
            factory.register("audit.Registered", || {
                gz_msgs::prost_reflect::DynamicMessage::new(
                    gz_msgs::descriptor("gz.msgs.StringMsg").unwrap(),
                )
            });
            assert!(factory.new_message("audit.Registered").is_some());
            std::env::set_var("GZ_MSGS_INSTALL_PREFIX", &other);
        }
        if mode == "late" {
            std::env::set_var("GZ_MSGS_INSTALL_PREFIX", &other);
        }
        if mode == "explicit-after" {
            assert!(factory.load_paths(&explicit).is_empty());
        }
        if mode == "errors" {
            let errors = factory.load_paths(&bad_explicit);
            assert_eq!(
                errors.iter().map(|(p, _)| p).collect::<Vec<_>>(),
                [&bad_env, &share.join("a.desc"), &bad_explicit]
            );
            assert!(errors
                .iter()
                .all(|(_, e)| e.kind() == std::io::ErrorKind::InvalidData));
        }
        let expected: Vec<&str> = match mode {
            "missing" | "missing-colon" | "unset" | "isolated-empty" | "colon" | "bad-share"
            | "errors" | "loop" => vec![],
            "env-first" | "isolated" | "late" | "generated" | "registered" => vec!["audit.B"],
            _ => vec!["audit.A"],
        };
        let first = names(&factory);
        assert_eq!(first, expected, "{mode}");
        if let Some(native) = native {
            if native["native"]["exit"] == 0 {
                assert_eq!(
                    gz_msgs::serde_json::to_value(&first).unwrap(),
                    native["states"][0],
                    "{mode}"
                );
            } else {
                assert_eq!(mode, "loop");
                assert_eq!(native["native"]["exit"], -6);
                assert!(native["states"].as_array().unwrap().is_empty());
            }
        }
        if mode == "freeze" {
            std::env::set_var("GZ_MSGS_INSTALL_PREFIX", &other);
            assert_eq!(names(&factory), ["audit.A"]);
        }
        let errors = factory.take_initial_errors();
        if mode == "bad-share" {
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].1.kind(), std::io::ErrorKind::InvalidData);
        } else if mode == "loop" {
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].0, share);
        } else {
            assert!(errors.is_empty(), "{mode}: {errors:?}");
        }
        assert!(factory.take_initial_errors().is_empty());
        if mode == "bad-share" {
            std::fs::write(share.join("a.desc"), descriptor("a.proto", "A")).unwrap();
            assert!(factory.load_paths(share.as_os_str()).is_empty());
            assert_eq!(names(&factory), ["audit.A"]);
        }
        if let Some(native) = native.filter(|row| row["native"]["exit"] == 0) {
            assert_eq!(
                gz_msgs::serde_json::to_value(names(&factory)).unwrap(),
                native["states"][1],
                "{mode}"
            );
            for message in native["messages"].as_array().unwrap() {
                let name = message[0].as_str().unwrap();
                let file = factory
                    .new_message(name)
                    .map(|value| value.descriptor().parent_file().name().to_owned())
                    .unwrap_or_else(|| "-".into());
                assert_eq!(file, message[1].as_str().unwrap(), "{mode} {name}");
            }
        }
    }
    pub(super) fn run() {
        if let Some(mode) = std::env::args().nth(1) {
            child(&mode);
            return;
        }
        let cases = [
            "normal",
            "empty",
            "raw",
            "colon",
            "relative",
            "missing",
            "missing-colon",
            "unset",
            "loop",
            "env-first",
            "explicit-after",
            "isolated",
            "isolated-empty",
            "late",
            "freeze",
            "generated",
            "registered",
            "bad-share",
            "errors",
        ];
        for mode in cases {
            let directory = tempfile::tempdir().unwrap();
            let result = Command::new(std::env::current_exe().unwrap())
                .arg(mode)
                .current_dir(directory.path())
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{mode}: {}{}",
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            );
        }
        let reference: Value =
            gz_msgs::serde_json::from_str(include_str!("fixtures/factory-install-share.json"))
                .unwrap();
        let directory = tempfile::tempdir().unwrap();
        let prefix = directory.path().join("prefix");
        let share = prefix.join("share/gz/protos");
        std::fs::create_dir_all(&share).unwrap();
        std::fs::write(share.join("a.desc"), descriptor("a.proto", "A")).unwrap();
        let cli = Command::new(env!("CARGO_BIN_EXE_gz-msgs"))
            .args(["--info", "audit.A"])
            .env_remove("GZ_DESCRIPTOR_PATH")
            .env("GZ_MSGS_INSTALL_PREFIX", &prefix)
            .output()
            .unwrap();
        assert_eq!(
            cli.status.code().map(i64::from),
            reference["cli_positive"]["exit"].as_i64()
        );
        let hex = |data: &[u8]| {
            data.iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        };
        assert_eq!(
            hex(&cli.stdout),
            reference["cli_positive"]["stdout_hex"].as_str().unwrap()
        );
        assert_eq!(
            hex(&cli.stderr),
            reference["cli_positive"]["stderr_hex"].as_str().unwrap()
        );
        println!("19 isolated install-share workflows and complete native CLI positive passed");
    }
}
fn main() {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    qualified::run();
    #[cfg(not(all(target_os = "linux", target_env = "gnu")))]
    panic!("install-share workflow qualification requires Linux GNU");
}

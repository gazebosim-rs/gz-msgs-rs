// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Standalone protobuf inspection tool. Binary bytes only cross stdin/stdout.
use gz_msgs::{
    prost::Message,
    prost_reflect::{DynamicMessage, ReflectMessage},
};
use std::io::{Read, Write};
mod legacy_cli;
const HELP: &str = "gz-msgs --list|-l | --info|-i TYPE... | --version|-v | --help|-h\nCommands: list, info TYPE..., encode-json, decode-json, encode-text, decode-text\nFormat commands take one fully qualified type and use stdin/stdout.";

// Protobuf's JSON parser accepts bare names, single quotes, trailing commas and
// raw control bytes and permissive escape pairs in strings.
// Rewrite only those tokens; serde_json still validates the entire document.
fn push_control_escape(output: &mut Vec<u8>, byte: u8) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    output.extend_from_slice(b"\\u00");
    output.push(HEX[usize::from(byte >> 4)]);
    output.push(HEX[usize::from(byte & 0x0f)]);
}

fn push_proto_escape(output: &mut Vec<u8>, escaped: u8) {
    // Native protobuf consumes unknown slashes and recognizes \v; serde validates UTF-8.
    match escaped {
        b'\x00'..=b'\x1f' => push_control_escape(output, escaped),
        b'v' => push_control_escape(output, b'\x0b'),
        b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' | b'u' => {
            output.extend_from_slice(&[b'\\', escaped]);
        }
        _ => output.push(escaped),
    }
}

fn normalize_proto_json(input: &[u8]) -> Vec<u8> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Frame {
        ObjectKey,
        ObjectValue,
        Array,
    }
    let mut output = Vec::with_capacity(input.len());
    let mut frames = Vec::new();
    let mut index = 0;
    let mut previous = None;
    while index < input.len() {
        let byte = input[index];
        if byte == b'"' {
            output.push(byte);
            index += 1;
            while index < input.len() {
                let current = input[index];
                if current == b'\\' && index + 1 < input.len() {
                    push_proto_escape(&mut output, input[index + 1]);
                    index += 2;
                    continue;
                }
                if current < b' ' {
                    push_control_escape(&mut output, current);
                } else {
                    output.push(current);
                }
                index += 1;
                if current == b'"' {
                    break;
                }
            }
            previous = Some(b'"');
            continue;
        }
        if byte == b'\'' {
            output.push(b'"');
            index += 1;
            while index < input.len() {
                let current = input[index];
                index += 1;
                match current {
                    b'\\' if index < input.len() => {
                        let escaped = input[index];
                        index += 1;
                        if escaped == b'\'' {
                            output.push(b'\'');
                        } else {
                            push_proto_escape(&mut output, escaped);
                        }
                    }
                    b'"' => output.extend_from_slice(br#"\""#),
                    b'\'' => {
                        output.push(b'"');
                        break;
                    }
                    _ if current < b' ' => push_control_escape(&mut output, current),
                    _ => output.push(current),
                }
            }
            previous = Some(b'"');
            continue;
        }
        if frames.last() == Some(&Frame::ObjectKey) && (byte.is_ascii_alphabetic() || byte == b'_')
        {
            let start = index;
            index += 1;
            while index < input.len()
                && (input[index].is_ascii_alphanumeric() || input[index] == b'_')
            {
                index += 1;
            }
            let mut after = index;
            while after < input.len() && input[after].is_ascii_whitespace() {
                after += 1;
            }
            if input.get(after) == Some(&b':') {
                output.push(b'"');
                output.extend_from_slice(&input[start..index]);
                output.push(b'"');
            } else {
                output.extend_from_slice(&input[start..index]);
            }
            previous = Some(input[index - 1]);
            continue;
        }
        // Native parses bare integer -0 as positive zero. Fractions,
        // exponents and quoted values retain their negative zero sign.
        if byte == b'-'
            && matches!(frames.last(), Some(Frame::ObjectValue | Frame::Array))
            && matches!(previous, Some(b':' | b'[' | b','))
            && input.get(index + 1) == Some(&b'0')
            && input.get(index + 2).is_some_and(|next| {
                matches!(*next, b',' | b'}' | b']' | b' ' | b'\t' | b'\r' | b'\n')
            })
        {
            index += 1;
            continue;
        }
        // Native accepts one dot after an integral token, including before an
        // exponent. A second dot after a fraction or exponent stays invalid.
        if byte == b'.'
            && index > 0
            && input[index - 1].is_ascii_digit()
            && input.get(index + 1).is_some_and(|next| {
                matches!(
                    *next,
                    b',' | b'}' | b']' | b'e' | b'E' | b' ' | b'\t' | b'\r' | b'\n'
                )
            })
        {
            let mut start = index;
            while start > 0
                && matches!(
                    input[start - 1],
                    b'0'..=b'9' | b'+' | b'-' | b'.' | b'e' | b'E'
                )
            {
                start -= 1;
            }
            if !input[start..index]
                .iter()
                .any(|part| matches!(*part, b'.' | b'e' | b'E'))
            {
                index += 1;
                continue;
            }
        }
        if byte == b',' && matches!(frames.last(), Some(Frame::ObjectValue | Frame::Array)) {
            let mut after = index + 1;
            while after < input.len() && input[after].is_ascii_whitespace() {
                after += 1;
            }
            let closing = if frames.last() == Some(&Frame::Array) {
                b']'
            } else {
                b'}'
            };
            if input.get(after) == Some(&closing)
                && !matches!(previous, Some(b'{' | b'[' | b',' | b':'))
            {
                index += 1;
                continue;
            }
        }
        match byte {
            b'{' => frames.push(Frame::ObjectKey),
            b'[' => frames.push(Frame::Array),
            b'}' | b']' => {
                frames.pop();
            }
            b':' if frames.last() == Some(&Frame::ObjectKey) => {
                *frames.last_mut().unwrap() = Frame::ObjectValue
            }
            b',' if frames.last() == Some(&Frame::ObjectValue) => {
                *frames.last_mut().unwrap() = Frame::ObjectKey
            }
            _ => {}
        }
        output.push(byte);
        if !byte.is_ascii_whitespace() {
            previous = Some(byte);
        }
        index += 1;
    }
    output
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = match legacy_cli::normalize(std::env::args().skip(1).collect()) {
        Ok(args) => args,
        Err((code, message)) => {
            eprintln!("{message}");
            std::process::exit(i32::from(code));
        }
    };
    if args.is_empty() || args == ["--help"] || args == ["-h"] {
        println!("{HELP}");
        return Ok(());
    }
    if args == ["--version"] || args == ["-v"] {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args == ["list"] || args == ["--list"] || args == ["-l"] {
        let names = gz_msgs::factory::MessageFactory::default().types();
        println!("{}", names.join("\n"));
        return Ok(());
    }
    if args.len() >= 2 && ["info", "--info", "-i"].contains(&args[0].as_str()) {
        let factory = gz_msgs::factory::MessageFactory::default();
        for name in &args[1..] {
            let descriptor = gz_msgs::descriptor(name)
                .filter(|_| {
                    !name.starts_with('.')
                        || name.starts_with(".gz.msgs.")
                        || name.starts_with(".gz_msgs.")
                })
                .or_else(|| {
                    factory
                        .new_message(name)
                        .map(|message| message.descriptor())
                });
            let Some(descriptor) = descriptor else {
                eprintln!("Unable to create message of type[{name}]");
                continue;
            };
            if let Some(text) = gz_msgs::message_info(descriptor.full_name()) {
                print!("{text}");
            } else {
                let text = factory
                    .registry_definition(descriptor.full_name())?
                    .ok_or("external descriptor source unavailable")?;
                print!(
                    "Name: {}\nFile: {}\n\n{text}\n",
                    descriptor.full_name(),
                    descriptor.parent_file().name()
                );
            }
        }
        return Ok(());
    }
    if args.len() != 2 {
        return Err(HELP.into());
    }
    let descriptor = gz_msgs::descriptor(&args[1]).ok_or("unknown message type")?;
    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input)?;
    let output = match args[0].as_str() {
        "encode-json" => {
            let normalized = normalize_proto_json(&input);
            let mut json = gz_msgs::serde_json::Deserializer::from_slice(&normalized);
            let message = DynamicMessage::deserialize(descriptor, &mut json)?;
            json.end()?;
            message.encode_to_vec()
        }
        "decode-json" => {
            gz_msgs::serde_json::to_vec(&DynamicMessage::decode(descriptor, input.as_slice())?)?
        }
        "encode-text" => {
            DynamicMessage::parse_text_format(descriptor, std::str::from_utf8(&input)?)?
                .encode_to_vec()
        }
        "decode-text" => DynamicMessage::decode(descriptor, input.as_slice())?
            .to_text_format()
            .into_bytes(),
        _ => return Err("unknown command".into()),
    };
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(&output)?;
    stdout.flush()?;
    Ok(())
}

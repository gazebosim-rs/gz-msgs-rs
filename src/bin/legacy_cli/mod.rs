// Copyright (C) 2026 Open Source Robotics Foundation
// Copyright (C) 2023 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Native message-command policy over the maintained GNU argument lexer.
use clap_lex::{ArgCursor, RawArgs};
use std::ffi::{OsStr, OsString};
use std::num::IntErrorKind;

pub enum Command {
    Utf8(Vec<String>),
    Info(Vec<Vec<u8>>),
}
type Error = (u8, Vec<u8>);
fn error(code: u8, message: &[u8]) -> Error {
    let mut message = message.to_vec();
    message.extend_from_slice(b"\nRun with --help for more information.");
    (code, message)
}
fn flag(value: &[u8], name: &str) -> Result<bool, Error> {
    let invalid = || {
        let mut message = format!("Could not convert: --{name} = ").into_bytes();
        message.extend_from_slice(value);
        error(104, &message)
    };
    let lower = value.to_ascii_lowercase();
    match lower.as_slice() {
        b"" | b"{}" | b"true" | b"on" | b"yes" | b"enable" | b"t" | b"y" | b"+" => return Ok(true),
        b"false" | b"off" | b"no" | b"disable" | b"f" | b"n" | b"-" => return Ok(false),
        _ => {}
    }
    let start = lower
        .iter()
        .take_while(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\n' | b'\x0b' | b'\x0c'))
        .count();
    let numeric = &lower[start..];
    let (negative, digits) = match numeric.first() {
        Some(b'-') => (true, &numeric[1..]),
        Some(b'+') => (false, &numeric[1..]),
        _ => (false, numeric),
    };
    let (radix, digits) = if digits.starts_with(b"0x")
        && digits.get(2).is_some_and(u8::is_ascii_hexdigit)
    {
        (16, &digits[2..])
    } else if digits.starts_with(b"0b") && digits.get(2).is_some_and(|b| (b'0'..=b'1').contains(b))
    {
        (2, &digits[2..])
    } else if digits.starts_with(b"0") {
        (8, digits)
    } else {
        (10, digits)
    };
    let end = digits
        .iter()
        .copied()
        .take_while(|b| match radix {
            16 => b.is_ascii_hexdigit(),
            8 => (b'0'..=b'7').contains(b),
            2 => (b'0'..=b'1').contains(b),
            _ => b.is_ascii_digit(),
        })
        .count();
    let signed = format!(
        "{}{prefix}",
        if negative { "-" } else { "" },
        prefix = std::str::from_utf8(&digits[..end]).expect("ASCII digit prefix")
    );
    match i64::from_str_radix(&signed, radix) {
        Ok(number) if end == digits.len() => Ok(number > 0),
        // Native CLI11's bool conversion uses the original first character on
        // strtoll overflow, even with leading whitespace or a trailing suffix.
        Err(error)
            if matches!(
                error.kind(),
                IntErrorKind::PosOverflow | IntErrorKind::NegOverflow
            ) =>
        {
            Ok(!value.starts_with(b"-"))
        }
        _ => Err(invalid()),
    }
}
fn add_names(value: &[u8], names: &mut Vec<Vec<u8>>) {
    // The existing ASCII bracket/comma rule does not require UTF-8 names.
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        if let Some(inner) = value.strip_prefix(b"[").and_then(|v| v.strip_suffix(b"]")) {
            pending.extend(inner.split(|b| *b == b',').rev().filter(|v| !v.is_empty()));
        } else {
            names.push(value.to_vec());
        }
    }
}
fn info(
    value: Option<&OsStr>,
    raw: &RawArgs,
    cursor: &mut ArgCursor,
    names: &mut Vec<Vec<u8>>,
) -> Result<(), Error> {
    let before = names.len();
    if let Some(value) = value.filter(|v| !v.is_empty()) {
        add_names(value.as_encoded_bytes(), names);
    }
    while names.len() == before {
        let value = raw
            .next_os(cursor)
            .ok_or_else(|| error(114, b"--info: 1 required TEXT missing"))?;
        add_names(value.as_encoded_bytes(), names);
    }
    Ok(())
}
pub fn normalize(mut args: Vec<OsString>) -> Result<Command, Error> {
    if args.first().and_then(|v| v.to_str()).is_some_and(|v| {
        matches!(
            v,
            "encode-json" | "decode-json" | "encode-text" | "decode-text"
        )
    }) {
        return args
            .into_iter()
            .map(|v| {
                v.into_string()
                    .map_err(|_| error(1, b"format command arguments must be UTF-8"))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Command::Utf8);
    }
    if args
        .first()
        .and_then(|v| v.to_str())
        .is_some_and(|v| matches!(v, "list" | "info"))
    {
        let mut option = OsString::from("--");
        option.push(&args[0]);
        args[0] = option;
    }
    let raw = RawArgs::new(args);
    let mut cursor = raw.cursor();
    let (mut help, mut collecting) = (false, false);
    let (mut version, mut list) = (None, None);
    let mut names = Vec::new();
    let mut extras = Vec::new();
    while let Some(arg) = raw.next(&mut cursor) {
        let value = arg.to_value_os().as_encoded_bytes();
        if collecting && value.starts_with(b"-") && value.get(1).is_some_and(u8::is_ascii_digit) {
            add_names(value, &mut names);
            continue;
        }
        if arg.is_escape() {
            let remaining: Vec<_> = raw
                .remaining(&mut cursor)
                .map(|v| v.as_encoded_bytes().to_vec())
                .collect();
            if !collecting && !remaining.is_empty() {
                extras.push(b"--".to_vec());
            }
            extras.extend(remaining);
            break;
        }
        if let Some((name, value)) = arg.to_long() {
            collecting = false;
            match name {
                Ok("info") => {
                    info(value, &raw, &mut cursor, &mut names)?;
                    collecting = true;
                }
                Ok("help") => help = true,
                Ok("version") => {
                    version = Some(
                        value
                            .unwrap_or(OsStr::new("true"))
                            .as_encoded_bytes()
                            .to_vec(),
                    )
                }
                Ok("list") => {
                    list = Some(
                        value
                            .unwrap_or(OsStr::new("true"))
                            .as_encoded_bytes()
                            .to_vec(),
                    )
                }
                _ => extras.push(arg.to_value_os().as_encoded_bytes().to_vec()),
            }
        } else if let Some(mut shorts) = arg.to_short() {
            collecting = false;
            while let Some(short) = shorts.next_flag() {
                match short {
                    Ok('i') => {
                        info(shorts.next_value_os(), &raw, &mut cursor, &mut names)?;
                        collecting = true;
                    }
                    Ok('h') => help = true,
                    Ok('v') => version = Some(b"true".to_vec()),
                    Ok('l') => list = Some(b"true".to_vec()),
                    other => {
                        let mut tail = b"-".to_vec();
                        match other {
                            Ok(ch) => {
                                let mut bytes = [0; 4];
                                tail.extend_from_slice(ch.encode_utf8(&mut bytes).as_bytes());
                                if let Some(value) = shorts.next_value_os() {
                                    tail.extend_from_slice(value.as_encoded_bytes());
                                }
                            }
                            Err(suffix) => tail.extend_from_slice(suffix.as_encoded_bytes()),
                        }
                        extras.push(tail);
                        break;
                    }
                }
            }
        } else if collecting {
            add_names(value, &mut names);
        } else {
            extras.push(value.to_vec());
        }
    }
    // Callback conversion/precedence remains identical for UTF-8 and OS bytes.
    if let Some(value) = version {
        if flag(&value, "version")? {
            return Ok(Command::Utf8(vec!["--version".into()]));
        }
    }
    let listing = list
        .as_ref()
        .map(|v| flag(v, "list"))
        .transpose()?
        .unwrap_or(false);
    if help {
        return Ok(Command::Utf8(vec!["--help".into()]));
    }
    if list.is_some() && !names.is_empty() {
        return Err(error(108, b"--list excludes --info"));
    }
    if !extras.is_empty() {
        let mut message = if extras.len() == 1 {
            b"The following argument was not expected: ".to_vec()
        } else {
            extras.reverse();
            b"The following arguments were not expected: ".to_vec()
        };
        message.extend_from_slice(&extras.join(&b" "[..]));
        return Err(error(109, &message));
    }
    if !names.is_empty() {
        return Ok(Command::Info(names));
    }
    Ok(Command::Utf8(vec![if listing {
        "--list"
    } else {
        "--help"
    }
    .into()]))
}

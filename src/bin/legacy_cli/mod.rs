// Copyright (C) 2026 Open Source Robotics Foundation
// Copyright (C) 2023 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Native message-command policy over the maintained GNU argument lexer.
use clap_lex::{ArgCursor, RawArgs};
use std::ffi::OsStr;
use std::num::IntErrorKind;

type Error = (u8, String);
fn error(code: u8, message: impl std::fmt::Display) -> Error {
    (
        code,
        format!("{message}\nRun with --help for more information."),
    )
}
fn flag(value: &str, name: &str) -> Result<bool, Error> {
    let invalid = || error(104, format!("Could not convert: --{name} = {value}"));
    let lower = value.to_ascii_lowercase();
    match lower.as_str() {
        "" | "{}" | "true" | "on" | "yes" | "enable" | "t" | "y" | "+" => return Ok(true),
        "false" | "off" | "no" | "disable" | "f" | "n" | "-" => return Ok(false),
        _ => {}
    }
    let numeric = lower.trim_start_matches([' ', '\t', '\r', '\n', '\x0b', '\x0c']);
    let (negative, digits) = match numeric.as_bytes().first() {
        Some(b'-') => (true, &numeric[1..]),
        Some(b'+') => (false, &numeric[1..]),
        _ => (false, numeric),
    };
    let (radix, digits) = if digits.starts_with("0x")
        && digits.as_bytes().get(2).is_some_and(u8::is_ascii_hexdigit)
    {
        (16, &digits[2..])
    } else if digits.starts_with("0b")
        && digits
            .as_bytes()
            .get(2)
            .is_some_and(|b| (b'0'..=b'1').contains(b))
    {
        (2, &digits[2..])
    } else if digits.starts_with('0') {
        (8, digits)
    } else {
        (10, digits)
    };
    let end = digits
        .bytes()
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
        prefix = &digits[..end]
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
            Ok(!value.starts_with('-'))
        }
        _ => Err(invalid()),
    }
}
fn info(
    value: Option<&OsStr>,
    raw: &RawArgs,
    cursor: &mut ArgCursor,
    names: &mut Vec<String>,
) -> Result<(), Error> {
    let value = match value {
        Some(value) if !value.is_empty() => value,
        Some(_) => return Err(error(114, "--info: 1 required TEXT missing")),
        None => raw
            .next_os(cursor)
            .ok_or_else(|| error(114, "--info: 1 required TEXT missing"))?,
    };
    // CLI11 consumes the first required value even if it looks like a flag.
    names.push(value.to_str().expect("String arguments").to_owned());
    Ok(())
}

pub fn normalize(mut args: Vec<String>) -> Result<Vec<String>, Error> {
    if args.first().is_some_and(|a| {
        matches!(
            a.as_str(),
            "encode-json" | "decode-json" | "encode-text" | "decode-text"
        )
    }) {
        return Ok(args);
    }
    if args
        .first()
        .is_some_and(|a| matches!(a.as_str(), "list" | "info"))
    {
        args[0].insert_str(0, "--");
    }
    let raw = RawArgs::new(args);
    let mut cursor = raw.cursor();
    let (mut help, mut collecting) = (false, false);
    let (mut version, mut list) = (None, None);
    let mut names = Vec::new();
    let mut extras = Vec::new();
    while let Some(arg) = raw.next(&mut cursor) {
        let value = arg.to_value().expect("String arguments");
        // CLI11 treats a digit-prefixed negative token as another string value,
        // including suffixes such as -1x; this is not numeric conversion.
        if collecting
            && value.starts_with('-')
            && value.as_bytes().get(1).is_some_and(u8::is_ascii_digit)
        {
            names.push(value.to_owned());
            continue;
        }
        if arg.is_escape() {
            let remaining: Vec<_> = raw
                .remaining(&mut cursor)
                .map(|v| v.to_str().expect("String arguments").to_owned())
                .collect();
            if !collecting && !remaining.is_empty() {
                extras.push("--".into());
            }
            extras.extend(remaining);
            break;
        }
        if let Some((name, value)) = arg.to_long() {
            collecting = false;
            match name.expect("String arguments") {
                "info" => {
                    info(value, &raw, &mut cursor, &mut names)?;
                    collecting = true;
                }
                "help" => help = true,
                "version" => {
                    version = Some(
                        value
                            .unwrap_or(OsStr::new("true"))
                            .to_str()
                            .expect("String arguments")
                            .to_owned(),
                    )
                }
                "list" => {
                    list = Some(
                        value
                            .unwrap_or(OsStr::new("true"))
                            .to_str()
                            .expect("String arguments")
                            .to_owned(),
                    )
                }
                _ => extras.push(arg.to_value().expect("String arguments").to_owned()),
            }
        } else if let Some(mut shorts) = arg.to_short() {
            collecting = false;
            while let Some(short) = shorts.next_flag() {
                match short.expect("String arguments") {
                    'i' => {
                        info(shorts.next_value_os(), &raw, &mut cursor, &mut names)?;
                        collecting = true;
                    }
                    'h' => help = true,
                    'v' => version = Some("true".to_owned()),
                    'l' => list = Some("true".to_owned()),
                    other => {
                        let mut tail = format!("-{other}");
                        if let Some(value) = shorts.next_value_os() {
                            tail.push_str(value.to_str().expect("String arguments"));
                        }
                        extras.push(tail);
                        break;
                    }
                }
            }
        } else if collecting {
            names.push(arg.to_value().expect("String arguments").to_owned());
        } else {
            extras.push(arg.to_value().expect("String arguments").to_owned());
        }
    }
    // Native callbacks convert version first, then list, before help/validation.
    if let Some(value) = version {
        if flag(&value, "version")? {
            return Ok(vec!["--version".into()]);
        }
    }
    let listing = list
        .as_ref()
        .map(|v| flag(v, "list"))
        .transpose()?
        .unwrap_or(false);
    if help {
        return Ok(vec!["--help".into()]);
    }
    if list.is_some() && !names.is_empty() {
        return Err(error(108, "--list excludes --info"));
    }
    if !extras.is_empty() {
        return Err(if extras.len() == 1 {
            error(
                109,
                format!("The following argument was not expected: {}", extras[0]),
            )
        } else {
            extras.reverse();
            error(
                109,
                format!(
                    "The following arguments were not expected: {}",
                    extras.join(" ")
                ),
            )
        });
    }
    if !names.is_empty() {
        names.insert(0, "--info".into());
        return Ok(names);
    }
    Ok(vec![if listing { "--list" } else { "--help" }.into()])
}

// Copyright (C) 2026 Open Source Robotics Foundation
// Copyright (C) 2023 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Native message-command policy over the maintained GNU argument lexer.
use clap_lex::{ArgCursor, RawArgs};
use std::ffi::OsStr;

type Error = (u8, String);
fn error(code: u8, message: impl std::fmt::Display) -> Error {
    (
        code,
        format!("{message}\nRun with --help for more information."),
    )
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
    let (mut help, mut version, mut list, mut collecting) = (false, false, false, false);
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
                "help" | "version" | "list" if value.is_some() => {
                    return Err(error(1, "Legacy flag value assignments are not supported"))
                }
                "help" => help = true,
                "version" => version = true,
                "list" => list = true,
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
                    'v' => version = true,
                    'l' => list = true,
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
    // Native callbacks prioritize version, then help, before exclusions/extras.
    if version {
        return Ok(vec!["--version".into()]);
    }
    if help {
        return Ok(vec!["--help".into()]);
    }
    if list && !names.is_empty() {
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
    Ok(vec![if list { "--list" } else { "--help" }.into()])
}

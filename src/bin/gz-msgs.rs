// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Standalone protobuf inspection tool. Binary bytes only cross stdin/stdout.
use fhex::FromHex;
use gz_msgs::{
    prost::Message,
    prost_reflect::{DynamicMessage, ReflectMessage, Value},
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

fn is_proto_whitespace(byte: u8) -> bool {
    byte.is_ascii_whitespace() || matches!(byte, b'\x0b' | b'\x0c')
}

#[derive(Clone, Copy)]
enum FloatWidth {
    Double,
    Single,
}

fn invalid_proto_quoted_float(value: &str, width: FloatWidth) -> bool {
    if matches!(value, "Infinity" | "-Infinity" | "NaN") {
        return false;
    }
    match value.parse::<f64>() {
        Ok(number) => {
            !number.is_finite()
                || (matches!(width, FloatWidth::Single) && number.abs() > f64::from(f32::MAX))
        }
        // Leave malformed strings to the maintained decoder.
        Err(_) => false,
    }
}

fn proto_quoted_float_whitespace(value: &str, width: FloatWidth) -> Option<Value> {
    // Pinned protobuf rejects literal edge spaces before calling safe_strtod,
    // which accepts the other ASCII C whitespace at a numeric edge.
    let bytes = value.as_bytes();
    if bytes.first() == Some(&b' ') || bytes.last() == Some(&b' ') {
        return None;
    }
    let trimmed =
        value.trim_matches(|c| matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{000b}' | '\u{000c}'));
    if trimmed == value {
        return None;
    }
    // StringToNumber sends quoted hex through the same double conversion as
    // decimal text. Narrow only after that conversion, including midpoint ties.
    let number = trimmed.parse::<f64>().ok().or_else(|| {
        if let Some(Value::F64(number)) = proto_hex_float(trimmed, FloatWidth::Double) {
            Some(number)
        } else {
            None
        }
    })?;
    if !number.is_finite()
        || (matches!(width, FloatWidth::Single) && number.abs() > f64::from(f32::MAX))
    {
        return None;
    }
    Some(match width {
        FloatWidth::Double => Value::F64(number),
        FloatWidth::Single => Value::F32(number as f32),
    })
}

#[derive(Clone, Copy)]
enum ComponentKind {
    Vector3d,
    Quaternion,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ComponentPath {
    Root(&'static str),
    Pose(&'static str, &'static str),
}

fn component_field(key: &str, kind: ComponentKind) -> Option<&'static str> {
    match key {
        "x" => Some("x"),
        "y" => Some("y"),
        "z" => Some("z"),
        "w" if matches!(kind, ComponentKind::Quaternion) => Some("w"),
        _ => None,
    }
}

fn patch_component_quoted(
    output: &mut Vec<u8>,
    start: usize,
    path: ComponentPath,
    invalid: &mut bool,
    patches: &mut Vec<(ComponentPath, Value)>,
) {
    patches.retain(|(previous, _)| *previous != path);
    if let Ok(value) = gz_msgs::serde_json::from_slice::<String>(&output[start..]) {
        *invalid |= invalid_proto_quoted_float(&value, FloatWidth::Double);
        if let Some(number) = proto_hex_float(&value, FloatWidth::Double)
            .or_else(|| proto_quoted_float_whitespace(&value, FloatWidth::Double))
        {
            output.truncate(start);
            output.extend_from_slice(b"\"0\"");
            patches.push((path, number));
        }
    }
}

fn proto_hex_float(value: &str, width: FloatWidth) -> Option<Value> {
    let mut bytes = value.as_bytes();
    if matches!(bytes.first(), Some(b'+' | b'-')) {
        bytes = &bytes[1..];
    }
    if bytes.len() < 3 || bytes[0] != b'0' || !matches!(bytes[1], b'x' | b'X') {
        return None;
    }
    let mut index = 2;
    let mut digits = 0;
    while bytes.get(index).is_some_and(u8::is_ascii_hexdigit) {
        digits += 1;
        index += 1;
    }
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        while bytes.get(index).is_some_and(u8::is_ascii_hexdigit) {
            digits += 1;
            index += 1;
        }
    }
    if digits == 0 {
        return None;
    }
    if matches!(bytes.get(index), Some(b'p' | b'P')) {
        index += 1;
        if matches!(bytes.get(index), Some(b'+' | b'-')) {
            index += 1;
        }
        let exponent_start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if index == exponent_start {
            return None;
        }
    }
    if index != bytes.len() {
        return None;
    }
    // fhex does the IEEE conversion; native rejects overflow rather than encoding Inf.
    match width {
        FloatWidth::Double => f64::from_hex(value)
            .filter(|number| number.is_finite())
            .map(Value::F64),
        FloatWidth::Single => f32::from_hex(value)
            .filter(|number| number.is_finite())
            .map(Value::F32),
    }
}

type UInt64ArrayNumbers = Vec<(usize, u64)>;

fn proto_uint64_hex(value: &[u8]) -> Option<(usize, u64)> {
    const LIMIT: f64 = 18446744073709551616.0;
    if !value.starts_with(b"0x") {
        return None;
    }
    let mut end = 2;
    while value
        .get(end)
        .is_some_and(|byte| byte.is_ascii_digit() || matches!(*byte, b'e' | b'E'))
    {
        end += 1;
    }
    if end == 2 {
        return None;
    }
    // Protobuf's number lexer selects its double conversion for e/E even
    // without a dot. In a lowercase 0x token those bytes are hex digits.
    // A bare hex integer with neither marker stays on the rejected path.
    let has_exponent_marker = value[2..end].iter().any(|byte| matches!(byte, b'e' | b'E'));
    let dotted = value.get(end) == Some(&b'.');
    if !dotted && !has_exponent_marker {
        return None;
    }
    if dotted {
        end += 1;
    }
    // The protobuf number lexer stops at alphabetic hex digits other than
    // e/E even though strtod would accept them.
    while value
        .get(end)
        .is_some_and(|byte| byte.is_ascii_digit() || matches!(*byte, b'e' | b'E'))
    {
        end += 1;
    }
    if value.get(end).is_some_and(|byte| {
        !matches!(
            *byte,
            b',' | b'}' | b']' | b' ' | b'\t' | b'\r' | b'\n' | b'\x0b' | b'\x0c'
        )
    }) {
        return None;
    }
    let number = f64::from_hex(std::str::from_utf8(&value[..end]).ok()?)?;
    if !number.is_finite() || !(0.0..LIMIT).contains(&number) || number.fract() != 0.0 {
        return None;
    }
    Some((end, number as u64))
}

fn proto_unquoted_hex_float(value: &[u8], width: FloatWidth) -> Option<(usize, Value)> {
    // The numeric lexer treats e/E as a double marker; strtod then reads the
    // same byte as a hexadecimal mantissa digit after lowercase 0x.
    let start = usize::from(value.first() == Some(&b'-'));
    if !value.get(start..)?.starts_with(b"0x") {
        return None;
    }
    let mut end = start + 2;
    let mut digits = 0;
    let mut has_e = false;
    while value
        .get(end)
        .is_some_and(|byte| byte.is_ascii_digit() || matches!(*byte, b'e' | b'E'))
    {
        has_e |= matches!(value[end], b'e' | b'E');
        digits += 1;
        end += 1;
    }
    let dotted = value.get(end) == Some(&b'.');
    if dotted {
        end += 1;
        while value
            .get(end)
            .is_some_and(|byte| byte.is_ascii_digit() || matches!(*byte, b'e' | b'E'))
        {
            digits += 1;
            end += 1;
        }
    }
    if digits == 0 || (!dotted && !has_e) {
        return None;
    }
    if value.get(end).is_some_and(|byte| {
        !matches!(
            *byte,
            b',' | b'}' | b']' | b' ' | b'\t' | b'\r' | b'\n' | b'\x0b' | b'\x0c'
        )
    }) {
        return None;
    }
    let token = std::str::from_utf8(&value[..end]).ok()?;
    proto_hex_float(token, width).map(|number| (end, number))
}

fn proto_leading_zero_float(value: &[u8], width: FloatWidth) -> Option<(usize, Value)> {
    // Protobuf selects floating conversion when a numeric token has a dot or
    // exponent. Only that path accepts leading zeroes; bare integers do not.
    let start = usize::from(value.first() == Some(&b'-'));
    if value.get(start) != Some(&b'0') || !value.get(start + 1).is_some_and(u8::is_ascii_digit) {
        return None;
    }
    let mut end = start + 2;
    while value.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
    }
    let mut dotted = false;
    if value.get(end) == Some(&b'.') {
        dotted = true;
        end += 1;
        while value.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
    }
    let mut has_exponent = false;
    if matches!(value.get(end), Some(b'e' | b'E')) {
        has_exponent = true;
        end += 1;
        if matches!(value.get(end), Some(b'+' | b'-')) {
            end += 1;
        }
        let digits = end;
        while value.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
        if end == digits {
            return None;
        }
    }
    if !dotted && !has_exponent {
        return None;
    }
    if value.get(end).is_some_and(|byte| {
        !matches!(
            *byte,
            b',' | b'}' | b']' | b' ' | b'\t' | b'\r' | b'\n' | b'\x0b' | b'\x0c'
        )
    }) {
        return None;
    }
    let token = std::str::from_utf8(&value[..end]).ok()?;
    // The native double conversion and Rust's float parser agree at decimal
    // rounding boundaries where serde_json can differ by one ULP.
    let number: f64 = token.parse().ok()?;
    if !number.is_finite() {
        return None;
    }
    match width {
        FloatWidth::Double => Some((end, Value::F64(number))),
        FloatWidth::Single if (number as f32).is_finite() => Some((end, Value::F32(number as f32))),
        FloatWidth::Single => None,
    }
}

fn proto_unquoted_float_max_round(value: &[u8]) -> Option<(usize, Value)> {
    // DataPiece::DoubleToFloat accepts a finite JSON number just above f32::MAX
    // when half of it rounds to half the maximum and doubling rounds back to
    // f32::MAX. Quoted numbers take the separate StringToNumber path.
    let end = value
        .iter()
        .position(|byte| {
            matches!(
                *byte,
                b',' | b'}' | b']' | b' ' | b'\t' | b'\r' | b'\n' | b'\x0b' | b'\x0c'
            )
        })
        .unwrap_or(value.len());
    let token = std::str::from_utf8(&value[..end]).ok()?;
    let _: gz_msgs::serde_json::Number = gz_msgs::serde_json::from_str(token).ok()?;
    let number: f64 = token.parse().ok()?;
    if !number.is_finite() || number.abs() < f64::from(f32::MAX) {
        return None;
    }
    let half = number * 0.5;
    if half.abs() >= f64::from(f32::MAX) {
        return None;
    }
    let half_after = half as f32;
    if half_after.abs() > f32::MAX * 0.5 {
        return None;
    }
    Some((end, Value::F32(half_after + half_after)))
}

fn proto_unquoted_double_round(value: &[u8]) -> Option<(usize, Value)> {
    // Protobuf's decimal double conversion and Rust's float parser agree on
    // these measured boundaries. The maintained JSON parser can differ by one
    // ULP, so replace only valid number tokens where their bits differ.
    let end = value
        .iter()
        .position(|byte| {
            matches!(
                *byte,
                b',' | b'}' | b']' | b' ' | b'\t' | b'\r' | b'\n' | b'\x0b' | b'\x0c'
            )
        })
        .unwrap_or(value.len());
    let token = std::str::from_utf8(&value[..end]).ok()?;
    if !token.bytes().any(|byte| matches!(byte, b'.' | b'e' | b'E')) {
        return None;
    }
    let maintained: gz_msgs::serde_json::Number = gz_msgs::serde_json::from_str(token).ok()?;
    let native: f64 = token.parse().ok()?;
    if !native.is_finite() || maintained.as_f64()?.to_bits() == native.to_bits() {
        return None;
    }
    Some((end, Value::F64(native)))
}

#[derive(Clone, Copy)]
enum TimeField {
    Seconds,
    Nanoseconds,
}

fn proto_time_float_integer(value: &[u8], field: TimeField) -> Option<(usize, i64)> {
    // The pinned JSON lexer renders dotted or exponent tokens as double;
    // DataPiece then requires an exact signed integer conversion.
    let (length, Value::F64(number)) = proto_leading_zero_float(value, FloatWidth::Double)
        .or_else(|| proto_unquoted_hex_float(value, FloatWidth::Double))?
    else {
        return None;
    };
    if number.fract() != 0.0 {
        return None;
    }
    let converted = match field {
        TimeField::Seconds if (-9223372036854775808.0..9223372036854775808.0).contains(&number) => {
            number as i64
        }
        TimeField::Nanoseconds if (i32::MIN as f64..=i32::MAX as f64).contains(&number) => {
            (number as i32) as i64
        }
        _ => return None,
    };
    Some((length, converted))
}

fn proto_uint64_array(
    value: &[u8],
) -> Result<Option<UInt64ArrayNumbers>, Box<dyn std::error::Error>> {
    const LIMIT: f64 = 18446744073709551616.0;
    let Ok(items) =
        gz_msgs::serde_json::from_slice::<Vec<Box<gz_msgs::serde_json::value::RawValue>>>(value)
    else {
        return Ok(None);
    };
    let mut numbers = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let token = item.get().trim();
        if !token.as_bytes().first().is_some_and(u8::is_ascii_digit) && !token.starts_with('-') {
            continue;
        }
        let number = if token.bytes().any(|byte| matches!(byte, b'.' | b'e' | b'E')) {
            // Native parses decimal/exponent forms as double before checking
            // integrality and range. Retain the raw token: serde_json's numeric
            // conversion can round differently near large integer boundaries.
            let parsed: f64 = token.parse()?;
            if !parsed.is_finite() || !(0.0..LIMIT).contains(&parsed) || parsed.fract() != 0.0 {
                return Err("invalid native uint64 decimal value".into());
            }
            parsed as u64
        } else {
            token.parse::<u64>()?
        };
        numbers.push((index, number));
    }
    Ok(Some(numbers))
}

fn flatten_repeated_json_values(
    mut pending: Vec<Box<gz_msgs::serde_json::value::RawValue>>,
) -> Result<Vec<Box<gz_msgs::serde_json::value::RawValue>>, Box<dyn std::error::Error>> {
    pending.reverse();
    let mut leaves = Vec::new();
    while let Some(token) = pending.pop() {
        if token.get() == "null" {
            continue;
        }
        if token.get().starts_with('[') {
            let items: Vec<Box<gz_msgs::serde_json::value::RawValue>> =
                gz_msgs::serde_json::from_str(token.get())?;
            pending.extend(items.into_iter().rev());
        } else {
            leaves.push(token);
        }
    }
    Ok(leaves)
}

struct NormalizedProtoJson {
    bytes: Vec<u8>,
    invalid_quoted_float: bool,
    patches: Vec<(usize, usize, Value)>,
    scalar_patches: Vec<(usize, Value)>,
    array_ranges: Vec<(usize, usize)>,
    data_ranges: Vec<(usize, usize)>,
    component_patches: Vec<(ComponentPath, Value)>,
    pose_parent_ranges: Vec<(&'static str, usize, usize)>,
}

// Keep duplicate keys and raw numeric tokens while the maintained JSON parser
// validates object framing. A Value map would discard the earlier fragments.
struct JsonMembers(Vec<(String, Box<gz_msgs::serde_json::value::RawValue>)>);

impl<'de> serde::Deserialize<'de> for JsonMembers {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct MembersVisitor;
        impl<'de> serde::de::Visitor<'de> for MembersVisitor {
            type Value = JsonMembers;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a message object")
            }

            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                mut map: M,
            ) -> Result<Self::Value, M::Error> {
                let mut members = Vec::new();
                while let Some(entry) = map.next_entry()? {
                    members.push(entry);
                }
                Ok(JsonMembers(members))
            }
        }
        deserializer.deserialize_map(MembersVisitor)
    }
}

fn json_members_bytes(members: &JsonMembers) -> Vec<u8> {
    let mut output = vec![b'{'];
    for (index, (key, value)) in members.0.iter().enumerate() {
        if index != 0 {
            output.push(b',');
        }
        // String serialization into a Vec cannot fail.
        gz_msgs::serde_json::to_writer(&mut output, key).unwrap();
        output.push(b':');
        output.extend_from_slice(value.get().as_bytes());
    }
    output.push(b'}');
    output
}

// Ordinary Header.data and inner value arrays append in native protobuf JSON.
// Keep earlier arrays for validation; only replace the last occurrence.
fn append_header_array_members(members: &mut JsonMembers, field: &str) -> Option<()> {
    let indices: Vec<_> = members
        .0
        .iter()
        .enumerate()
        .filter_map(|(index, (key, _))| (key == field).then_some(index))
        .collect();
    if indices.len() < 2 {
        return None;
    }
    let last = *indices.last().unwrap();
    let mut combined = Vec::<Box<gz_msgs::serde_json::value::RawValue>>::new();
    let mut array_seen = false;
    for index in indices {
        let raw = members.0[index].1.get();
        if raw == "null" {
            continue;
        }
        let array: Vec<Box<gz_msgs::serde_json::value::RawValue>> =
            gz_msgs::serde_json::from_str(raw).ok()?;
        combined.extend(array);
        array_seen = true;
    }
    if !array_seen {
        return None;
    }
    members.0[last].1 = gz_msgs::serde_json::value::RawValue::from_string(
        gz_msgs::serde_json::to_string(&combined).ok()?,
    )
    .ok()?;
    Some(())
}

fn merge_header_data_values(
    raw: &gz_msgs::serde_json::value::RawValue,
) -> Option<Box<gz_msgs::serde_json::value::RawValue>> {
    let mut entries: Vec<Box<gz_msgs::serde_json::value::RawValue>> =
        gz_msgs::serde_json::from_str(raw.get()).ok()?;
    for entry in &mut entries {
        let mut members: JsonMembers = gz_msgs::serde_json::from_str(entry.get()).ok()?;
        append_header_array_members(&mut members, "value");
        *entry = gz_msgs::serde_json::value::RawValue::from_string(
            String::from_utf8(json_members_bytes(&members)).ok()?,
        )
        .ok()?;
    }
    gz_msgs::serde_json::value::RawValue::from_string(
        gz_msgs::serde_json::to_string(&entries).ok()?,
    )
    .ok()
}

fn append_header_fields(header: &mut JsonMembers) {
    for (key, value) in &mut header.0 {
        if key != "data" {
            continue;
        }
        if let Some(merged) = merge_header_data_values(value) {
            *value = merged;
        }
    }
    append_header_array_members(header, "data");
}

// Header fields on Vector3d/Quaternion roots and Pose components are qualified.
// Retain earlier occurrences so invalid earlier fields still reach reflection.
fn merge_header_objects(input: &[u8], path: &[&str]) -> Option<Vec<u8>> {
    let (field, rest) = path.split_first()?;
    let mut members: JsonMembers = gz_msgs::serde_json::from_slice(input).ok()?;
    let matching: Vec<_> = members
        .0
        .iter()
        .enumerate()
        .filter_map(|(index, (key, _))| (key == field).then_some(index))
        .collect();
    let last = *matching.last()?;
    let mut combined = JsonMembers(Vec::new());
    let mut object_seen = false;
    for index in matching {
        let value = members.0[index].1.get();
        if value == "null" {
            continue;
        }
        let object: JsonMembers = gz_msgs::serde_json::from_str(value).ok()?;
        combined.0.extend(object.0);
        object_seen = true;
    }
    if !object_seen {
        return None;
    }
    if *field == "header" {
        append_header_fields(&mut combined);
    }
    let raw = json_members_bytes(&combined);
    let raw = merge_header_objects(&raw, rest).unwrap_or(raw);
    members.0[last].1 =
        gz_msgs::serde_json::value::RawValue::from_string(String::from_utf8(raw).ok()?).ok()?;
    Some(json_members_bytes(&members))
}

fn normalize_header_value_members(members: &mut JsonMembers) -> Option<bool> {
    let mut changed = false;
    for (name, value) in &mut members.0 {
        if name != "value" {
            continue;
        }
        if gz_msgs::serde_json::from_str::<String>(value.get()).is_ok() {
            *value = gz_msgs::serde_json::value::RawValue::from_string(
                gz_msgs::serde_json::to_string(&[&**value]).ok()?,
            )
            .ok()?;
            changed = true;
        } else if let Ok(values) = gz_msgs::serde_json::from_str::<
            Vec<Box<gz_msgs::serde_json::value::RawValue>>,
        >(value.get())
        {
            if !values
                .iter()
                .any(|item| item.get() == "null" || item.get().starts_with('['))
            {
                continue;
            }
            // Flatten list structure only. Invalid nonnull leaves still
            // reach the maintained string decoder, including duplicates.
            let leaves = flatten_repeated_json_values(values).ok()?;
            *value = gz_msgs::serde_json::value::RawValue::from_string(
                gz_msgs::serde_json::to_string(&leaves).ok()?,
            )
            .ok()?;
            changed = true;
        }
    }
    Some(changed)
}

fn normalize_header_entries(input: &str) -> Option<Box<gz_msgs::serde_json::value::RawValue>> {
    let mut entries: Vec<Box<gz_msgs::serde_json::value::RawValue>> =
        gz_msgs::serde_json::from_str(input).ok()?;
    let mut changed = entries
        .iter()
        .any(|entry| entry.get() == "null" || entry.get().starts_with('['));
    if changed {
        // Preserve leaf order and invalid entries for reflection validation.
        // Earlier duplicate fragments still pass through this same path.
        entries = flatten_repeated_json_values(entries).ok()?;
    }
    for entry in &mut entries {
        let Ok(mut members) = gz_msgs::serde_json::from_str::<JsonMembers>(entry.get()) else {
            continue;
        };
        let entry_changed = normalize_header_value_members(&mut members)?;
        if entry_changed {
            *entry = gz_msgs::serde_json::value::RawValue::from_string(
                String::from_utf8(json_members_bytes(&members)).ok()?,
            )
            .ok()?;
            changed = true;
        }
    }
    if !changed {
        return None;
    }
    gz_msgs::serde_json::value::RawValue::from_string(
        gz_msgs::serde_json::to_string(&entries).ok()?,
    )
    .ok()
}

// Normalize singleton data objects, null data entries and nested value arrays
// without appending any fields.
// Earlier parent/header fragments still need to be parseable for validation;
// the final merged fragment performs the existing array/value append once.
fn normalize_header_data_members(header: &mut JsonMembers) -> Option<bool> {
    let mut changed = false;
    for (name, data) in &mut header.0 {
        if name != "data" {
            continue;
        }
        if gz_msgs::serde_json::from_str::<JsonMembers>(data.get()).is_ok() {
            *data = gz_msgs::serde_json::value::RawValue::from_string(
                gz_msgs::serde_json::to_string(&[&**data]).ok()?,
            )
            .ok()?;
            changed = true;
        }
        if let Some(coerced) = normalize_header_entries(data.get()) {
            *data = coerced;
            changed = true;
        }
    }
    Some(changed)
}

fn normalize_header_data_shapes(input: &[u8]) -> Option<Vec<u8>> {
    let mut parent: JsonMembers = gz_msgs::serde_json::from_slice(input).ok()?;
    let mut changed = false;
    for (name, value) in &mut parent.0 {
        if name != "header" || value.get() == "null" {
            continue;
        }
        let mut header: JsonMembers = gz_msgs::serde_json::from_str(value.get()).ok()?;
        changed |= normalize_header_data_members(&mut header)?;
        *value = gz_msgs::serde_json::value::RawValue::from_string(
            String::from_utf8(json_members_bytes(&header)).ok()?,
        )
        .ok()?;
    }
    changed.then(|| json_members_bytes(&parent))
}

fn normalize_header_root(input: &[u8]) -> Option<Vec<u8>> {
    let mut header: JsonMembers = gz_msgs::serde_json::from_slice(input).ok()?;
    normalize_header_data_members(&mut header)?;
    append_header_fields(&mut header);
    let bytes = json_members_bytes(&header);
    Some(merge_header_objects(&bytes, &["stamp"]).unwrap_or(bytes))
}

fn normalize_header_map_root(input: &[u8]) -> Option<Vec<u8>> {
    let mut members: JsonMembers = gz_msgs::serde_json::from_slice(input).ok()?;
    normalize_header_value_members(&mut members)?;
    append_header_array_members(&mut members, "value");
    Some(json_members_bytes(&members))
}

// Reflection keeps the last duplicate JSON message key; protobuf merges the
// child fields. Leave earlier objects in place so they still get validated.
fn merge_pose_parent_objects(
    normalized: &[u8],
    ranges: &[(&'static str, usize, usize)],
) -> Option<Vec<u8>> {
    let mut replacements = Vec::new();
    for parent in ["position", "orientation"] {
        let occurrences: Vec<_> = ranges
            .iter()
            .filter(|(name, _, _)| *name == parent)
            .collect();
        if occurrences.is_empty() {
            continue;
        }
        let mut members = Vec::new();
        let mut earlier = Vec::new();
        let mut valid = true;
        for (index, (_, start, end)) in occurrences.iter().enumerate() {
            let value = normalized[*start..*end].trim_ascii();
            if value == b"null" {
                continue;
            }
            if !value.starts_with(b"{") || !value.ends_with(b"}") {
                valid = false;
                break;
            }
            let coerced = normalize_header_data_shapes(value);
            let value = coerced.as_deref().unwrap_or(value);
            let inner = value[1..value.len() - 1].trim_ascii();
            if !inner.is_empty() {
                if !members.is_empty() {
                    members.push(b',');
                }
                members.extend_from_slice(inner);
            }
            if index + 1 == occurrences.len() {
                continue;
            }
            if let Some(coerced) = coerced {
                earlier.push((*start, *end, coerced));
            }
        }
        if !valid || members.is_empty() {
            continue;
        }
        let mut merged = Vec::with_capacity(members.len() + 2);
        merged.push(b'{');
        merged.extend_from_slice(&members);
        merged.push(b'}');
        let nested = merge_header_objects(&merged, &["header", "stamp"]);
        if occurrences.len() == 1 && nested.is_none() {
            continue;
        }
        let merged = nested.unwrap_or(merged);
        let (_, start, end) = occurrences.last().unwrap();
        replacements.extend(earlier);
        replacements.push((*start, *end, merged));
    }
    replacements.sort_by_key(|(start, _, _)| *start);
    if replacements.is_empty() {
        return None;
    }
    let mut output = Vec::with_capacity(normalized.len());
    let mut last = 0;
    for (start, end, merged) in replacements {
        output.extend_from_slice(&normalized[last..start]);
        output.extend_from_slice(&merged);
        last = end;
    }
    output.extend_from_slice(&normalized[last..]);
    Some(output)
}

fn normalize_proto_json(
    input: &[u8],
    float_data: Option<FloatWidth>,
    repeated_data: bool,
    uint64_data: bool,
    time_data: bool,
    component_kind: Option<ComponentKind>,
    pose_data: bool,
) -> NormalizedProtoJson {
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
    let mut data_field = false;
    let mut time_field = None;
    let mut component_path = None;
    let mut component_patches = Vec::new();
    let mut pose_parent_pending = None;
    let mut pose_parent_active = None;
    let mut pose_value_start = None;
    let mut pose_parent_ranges = Vec::new();
    let mut float_array_depth = None;
    let mut float_item_index = 0;
    let mut float_patches = Vec::new();
    let mut invalid_quoted_float = false;
    let mut scalar_float_patches = Vec::new();
    let mut float_array_count = 0;
    let mut float_array_start = None;
    let mut float_array_ranges = Vec::new();
    let mut data_value_start = None;
    let mut data_value_ranges = Vec::new();
    while index < input.len() {
        let byte = input[index];
        if byte == b'"' {
            let output_start = output.len();
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
            if frames.len() == 1 && frames.last() == Some(&Frame::ObjectKey) {
                let key = gz_msgs::serde_json::from_slice::<String>(&output[output_start..]).ok();
                data_field = key.as_deref() == Some("data");
                component_path = key
                    .as_deref()
                    .and_then(|key| component_kind.and_then(|kind| component_field(key, kind)))
                    .map(ComponentPath::Root);
                pose_parent_pending = if pose_data {
                    match key.as_deref() {
                        Some("position") => Some(("position", ComponentKind::Vector3d)),
                        Some("orientation") => Some(("orientation", ComponentKind::Quaternion)),
                        _ => None,
                    }
                } else {
                    None
                };
                time_field = if time_data {
                    match key.as_deref() {
                        Some("sec") => Some(TimeField::Seconds),
                        Some("nsec") => Some(TimeField::Nanoseconds),
                        _ => None,
                    }
                } else {
                    None
                };
            } else if frames.len() == 2
                && frames.last() == Some(&Frame::ObjectKey)
                && pose_parent_active.is_some()
            {
                let key = gz_msgs::serde_json::from_slice::<String>(&output[output_start..]).ok();
                component_path = key.as_deref().and_then(|key| {
                    pose_parent_active.and_then(|(parent, kind)| {
                        component_field(key, kind).map(|field| ComponentPath::Pose(parent, field))
                    })
                });
            } else if matches!(
                (component_path, frames.len()),
                (Some(ComponentPath::Root(_)), 1) | (Some(ComponentPath::Pose(_, _)), 2)
            ) && frames.last() == Some(&Frame::ObjectValue)
            {
                patch_component_quoted(
                    &mut output,
                    output_start,
                    component_path.unwrap(),
                    &mut invalid_quoted_float,
                    &mut component_patches,
                );
            } else if float_data.is_some()
                && (float_array_depth == Some(frames.len())
                    || (frames.len() == 1
                        && frames.last() == Some(&Frame::ObjectValue)
                        && data_field))
            {
                if let (Some(width), Ok(value)) = (
                    float_data,
                    gz_msgs::serde_json::from_slice::<String>(&output[output_start..]),
                ) {
                    invalid_quoted_float |= invalid_proto_quoted_float(&value, width);
                    if float_array_depth == Some(frames.len()) {
                        if let Some(number) = proto_hex_float(&value, width) {
                            output.truncate(output_start);
                            output.extend_from_slice(b"\"0\"");
                            float_patches.push((float_array_count - 1, float_item_index, number));
                        } else if let Some(number) = proto_quoted_float_whitespace(&value, width) {
                            output.truncate(output_start);
                            output.extend_from_slice(b"\"0\"");
                            float_patches.push((float_array_count - 1, float_item_index, number));
                        }
                    } else if let Some(number) = proto_quoted_float_whitespace(&value, width) {
                        output.truncate(output_start);
                        output.extend_from_slice(b"\"0\"");
                        scalar_float_patches.push((data_value_ranges.len(), number));
                    }
                }
            }
            previous = Some(b'"');
            continue;
        }
        if byte == b'\'' {
            let output_start = output.len();
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
            if frames.len() == 1 && frames.last() == Some(&Frame::ObjectKey) {
                let key = gz_msgs::serde_json::from_slice::<String>(&output[output_start..]).ok();
                data_field = key.as_deref() == Some("data");
                component_path = key
                    .as_deref()
                    .and_then(|key| component_kind.and_then(|kind| component_field(key, kind)))
                    .map(ComponentPath::Root);
                pose_parent_pending = if pose_data {
                    match key.as_deref() {
                        Some("position") => Some(("position", ComponentKind::Vector3d)),
                        Some("orientation") => Some(("orientation", ComponentKind::Quaternion)),
                        _ => None,
                    }
                } else {
                    None
                };
                time_field = if time_data {
                    match key.as_deref() {
                        Some("sec") => Some(TimeField::Seconds),
                        Some("nsec") => Some(TimeField::Nanoseconds),
                        _ => None,
                    }
                } else {
                    None
                };
            } else if frames.len() == 2
                && frames.last() == Some(&Frame::ObjectKey)
                && pose_parent_active.is_some()
            {
                let key = gz_msgs::serde_json::from_slice::<String>(&output[output_start..]).ok();
                component_path = key.as_deref().and_then(|key| {
                    pose_parent_active.and_then(|(parent, kind)| {
                        component_field(key, kind).map(|field| ComponentPath::Pose(parent, field))
                    })
                });
            } else if matches!(
                (component_path, frames.len()),
                (Some(ComponentPath::Root(_)), 1) | (Some(ComponentPath::Pose(_, _)), 2)
            ) && frames.last() == Some(&Frame::ObjectValue)
            {
                patch_component_quoted(
                    &mut output,
                    output_start,
                    component_path.unwrap(),
                    &mut invalid_quoted_float,
                    &mut component_patches,
                );
            } else if float_data.is_some()
                && (float_array_depth == Some(frames.len())
                    || (frames.len() == 1
                        && frames.last() == Some(&Frame::ObjectValue)
                        && data_field))
            {
                if let (Some(width), Ok(value)) = (
                    float_data,
                    gz_msgs::serde_json::from_slice::<String>(&output[output_start..]),
                ) {
                    invalid_quoted_float |= invalid_proto_quoted_float(&value, width);
                    if float_array_depth == Some(frames.len()) {
                        if let Some(number) = proto_hex_float(&value, width) {
                            output.truncate(output_start);
                            output.extend_from_slice(b"\"0\"");
                            float_patches.push((float_array_count - 1, float_item_index, number));
                        } else if let Some(number) = proto_quoted_float_whitespace(&value, width) {
                            output.truncate(output_start);
                            output.extend_from_slice(b"\"0\"");
                            float_patches.push((float_array_count - 1, float_item_index, number));
                        }
                    } else if let Some(number) = proto_quoted_float_whitespace(&value, width) {
                        output.truncate(output_start);
                        output.extend_from_slice(b"\"0\"");
                        scalar_float_patches.push((data_value_ranges.len(), number));
                    }
                }
            }
            previous = Some(b'"');
            continue;
        }
        // Protobuf treats these C whitespace bytes as separators outside strings.
        if matches!(byte, b'\x0b' | b'\x0c') {
            output.push(b' ');
            index += 1;
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
            while after < input.len() && is_proto_whitespace(input[after]) {
                after += 1;
            }
            if input.get(after) == Some(&b':') {
                output.push(b'"');
                output.extend_from_slice(&input[start..index]);
                output.push(b'"');
                if frames.len() == 1 {
                    data_field = &input[start..index] == b"data";
                    component_path = std::str::from_utf8(&input[start..index])
                        .ok()
                        .and_then(|key| component_kind.and_then(|kind| component_field(key, kind)))
                        .map(ComponentPath::Root);
                    pose_parent_pending = if pose_data {
                        match &input[start..index] {
                            b"position" => Some(("position", ComponentKind::Vector3d)),
                            b"orientation" => Some(("orientation", ComponentKind::Quaternion)),
                            _ => None,
                        }
                    } else {
                        None
                    };
                    time_field = if time_data {
                        match &input[start..index] {
                            b"sec" => Some(TimeField::Seconds),
                            b"nsec" => Some(TimeField::Nanoseconds),
                            _ => None,
                        }
                    } else {
                        None
                    };
                } else if frames.len() == 2 {
                    component_path =
                        std::str::from_utf8(&input[start..index])
                            .ok()
                            .and_then(|key| {
                                pose_parent_active.and_then(|(parent, kind)| {
                                    component_field(key, kind)
                                        .map(|field| ComponentPath::Pose(parent, field))
                                })
                            });
                }
            } else {
                output.extend_from_slice(&input[start..index]);
            }
            previous = Some(input[index - 1]);
            continue;
        }
        if data_value_start.is_some()
            && matches!(previous, Some(b':' | b'[' | b','))
            && (frames.len() == 1 || float_array_depth == Some(frames.len()))
        {
            if let Some((length, number)) =
                float_data.and_then(|width| proto_unquoted_hex_float(&input[index..], width))
            {
                if float_array_depth == Some(frames.len()) {
                    output.extend_from_slice(b"\"0\"");
                    float_patches.push((float_array_count - 1, float_item_index, number));
                } else {
                    output.push(b'"');
                    output.extend_from_slice(&input[index..index + length]);
                    output.push(b'"');
                }
                previous = Some(b'"');
                index += length;
                continue;
            }
            if let Some(width) = float_data {
                let rounded = match width {
                    FloatWidth::Double => proto_unquoted_double_round(&input[index..]),
                    FloatWidth::Single => proto_unquoted_float_max_round(&input[index..]),
                };
                if let Some((length, number)) =
                    proto_leading_zero_float(&input[index..], width).or(rounded)
                {
                    if float_array_depth == Some(frames.len()) {
                        output.extend_from_slice(b"\"0\"");
                        float_patches.push((float_array_count - 1, float_item_index, number));
                    } else {
                        output.extend_from_slice(b"\"0\"");
                        scalar_float_patches.push((data_value_ranges.len(), number));
                    }
                    previous = Some(b'"');
                    index += length;
                    continue;
                }
            }
        }
        if matches!(frames.len(), 1 | 2) && previous == Some(b':') {
            // A scalar null leaves an earlier native component unchanged.
            if let Some(path) = component_path {
                if matches!(
                    (path, frames.len()),
                    (ComponentPath::Root(_), 1) | (ComponentPath::Pose(_, _), 2)
                ) && frames.last() == Some(&Frame::ObjectValue)
                    && !is_proto_whitespace(byte)
                    && !input[index..].starts_with(b"null")
                {
                    component_patches.retain(|(previous, _)| *previous != path);
                }
            }
            if frames.len() == 1 {
                if let Some((length, number)) =
                    time_field.and_then(|field| proto_time_float_integer(&input[index..], field))
                {
                    let decimal = number.to_string();
                    output.extend_from_slice(decimal.as_bytes());
                    previous = decimal.as_bytes().last().copied();
                    index += length;
                    continue;
                }
            }
        }
        if uint64_data && data_value_start.is_some() && matches!(previous, Some(b':' | b'[' | b','))
        {
            if let Some((length, number)) = proto_uint64_hex(&input[index..]) {
                let decimal = number.to_string();
                output.extend_from_slice(decimal.as_bytes());
                previous = decimal.as_bytes().last().copied();
                index += length;
                continue;
            }
        }
        // Native parses bare integer -0 as positive zero. Fractions,
        // exponents and quoted values retain their negative zero sign.
        if byte == b'-'
            && matches!(frames.last(), Some(Frame::ObjectValue | Frame::Array))
            && matches!(previous, Some(b':' | b'[' | b','))
            && input.get(index + 1) == Some(&b'0')
            && input.get(index + 2).is_some_and(|next| {
                matches!(
                    *next,
                    b',' | b'}' | b']' | b' ' | b'\t' | b'\r' | b'\n' | b'\x0b' | b'\x0c'
                )
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
                    b',' | b'}'
                        | b']'
                        | b'e'
                        | b'E'
                        | b' '
                        | b'\t'
                        | b'\r'
                        | b'\n'
                        | b'\x0b'
                        | b'\x0c'
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
            while after < input.len() && is_proto_whitespace(input[after]) {
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
            b'{' => {
                if frames.len() == 1 && frames.last() == Some(&Frame::ObjectValue) {
                    pose_parent_active = pose_parent_pending;
                }
                frames.push(Frame::ObjectKey);
            }
            b'[' => {
                if frames.len() == 1 && data_field && float_data.is_some() {
                    float_array_depth = Some(2);
                    float_item_index = 0;
                    float_array_count += 1;
                    float_array_start = Some(output.len());
                }
                frames.push(Frame::Array);
            }
            b'}' | b']' => {
                if byte == b'}' && frames.len() == 2 && pose_parent_active.is_some() {
                    pose_parent_active = None;
                    component_path = None;
                }
                if byte == b'}' && frames.len() == 1 {
                    if let Some(start) = data_value_start.take() {
                        data_value_ranges.push((start, output.len()));
                    }
                    if let Some((parent, start)) = pose_value_start.take() {
                        pose_parent_ranges.push((parent, start, output.len()));
                    }
                }
                if float_array_depth == Some(frames.len()) {
                    float_array_depth = None;
                    if byte == b']' {
                        if let Some(start) = float_array_start.take() {
                            float_array_ranges.push((start, output.len() + 1));
                        }
                    }
                }
                frames.pop();
            }
            b':' if frames.last() == Some(&Frame::ObjectKey) => {
                if frames.len() == 1 && data_field && repeated_data {
                    data_value_start = Some(output.len() + 1);
                }
                if frames.len() == 1 {
                    pose_value_start =
                        pose_parent_pending.map(|(parent, _)| (parent, output.len() + 1));
                }
                *frames.last_mut().unwrap() = Frame::ObjectValue
            }
            b',' if frames.last() == Some(&Frame::ObjectValue) => {
                if frames.len() == 1 {
                    if let Some(start) = data_value_start.take() {
                        data_value_ranges.push((start, output.len()));
                    }
                    if let Some((parent, start)) = pose_value_start.take() {
                        pose_parent_ranges.push((parent, start, output.len()));
                    }
                }
                *frames.last_mut().unwrap() = Frame::ObjectKey;
                if frames.len() == 1 {
                    data_field = false;
                    time_field = None;
                    component_path = None;
                    pose_parent_pending = None;
                } else if frames.len() == 2 {
                    component_path = None;
                }
            }
            b',' if float_array_depth == Some(frames.len()) => {
                float_item_index += 1;
            }
            _ => {}
        }
        output.push(byte);
        if !byte.is_ascii_whitespace() {
            previous = Some(byte);
        }
        index += 1;
    }
    NormalizedProtoJson {
        bytes: output,
        invalid_quoted_float,
        patches: float_patches,
        scalar_patches: scalar_float_patches,
        array_ranges: float_array_ranges,
        data_ranges: data_value_ranges,
        component_patches,
        pose_parent_ranges,
    }
}
fn print_info(names: Vec<Vec<u8>>) -> Result<(), Box<dyn std::error::Error>> {
    let factory = gz_msgs::factory::MessageFactory::default();
    for bytes in names {
        let descriptor = if let Ok(name) = std::str::from_utf8(&bytes) {
            gz_msgs::descriptor(name)
                .filter(|_| {
                    !name.starts_with('.')
                        || name.starts_with(".gz.msgs.")
                        || name.starts_with(".gz_msgs.")
                })
                .or_else(|| {
                    factory
                        .new_message(name)
                        .map(|message| message.descriptor())
                })
        } else {
            // A miss still initializes the lazy descriptor source, without
            // substituting a different type name or losing argument bytes.
            let _ = factory.types();
            None
        };
        let Some(descriptor) = descriptor else {
            let mut stderr = std::io::stderr().lock();
            stderr.write_all(b"Unable to create message of type[")?;
            stderr.write_all(&bytes)?;
            stderr.write_all(b"]\n")?;
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
    std::io::stdout().flush()?;
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = match legacy_cli::normalize(std::env::args_os().skip(1).collect()) {
        Ok(legacy_cli::Command::Utf8(args)) => args,
        Ok(legacy_cli::Command::Info(names)) => return print_info(names),
        Err((code, message)) => {
            std::io::stderr().write_all(&message)?;
            std::io::stderr().write_all(b"\n")?;
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
    if args.len() != 2 {
        return Err(HELP.into());
    }
    let descriptor = gz_msgs::descriptor(&args[1]).ok_or("unknown message type")?;
    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input)?;
    let output = match args[0].as_str() {
        "encode-json" => {
            let float_data = match descriptor.full_name() {
                "gz.msgs.Double_V" => Some(FloatWidth::Double),
                "gz.msgs.Float_V" => Some(FloatWidth::Single),
                _ => None,
            };
            let repeated_data = float_data.is_some()
                || matches!(
                    descriptor.full_name(),
                    "gz.msgs.Int32_V"
                        | "gz.msgs.UInt32_V"
                        | "gz.msgs.Int64_V"
                        | "gz.msgs.UInt64_V"
                        | "gz.msgs.StringMsg_V"
                );
            let uint64_data = descriptor.full_name() == "gz.msgs.UInt64_V";
            let component_kind = match descriptor.full_name() {
                "gz.msgs.Vector3d" => Some(ComponentKind::Vector3d),
                "gz.msgs.Quaternion" => Some(ComponentKind::Quaternion),
                _ => None,
            };
            let NormalizedProtoJson {
                bytes: normalized,
                invalid_quoted_float,
                patches,
                scalar_patches,
                array_ranges,
                data_ranges,
                component_patches,
                pose_parent_ranges,
            } = normalize_proto_json(
                &input,
                float_data,
                repeated_data,
                uint64_data,
                descriptor.full_name() == "gz.msgs.Time",
                component_kind,
                descriptor.full_name() == "gz.msgs.Pose",
            );
            if invalid_quoted_float {
                return Err("invalid quoted float data".into());
            }
            let normalized = if descriptor.full_name() == "gz.msgs.Pose" {
                merge_pose_parent_objects(&normalized, &pose_parent_ranges).unwrap_or(normalized)
            } else if component_kind.is_some() {
                let shaped = normalize_header_data_shapes(&normalized).unwrap_or(normalized);
                merge_header_objects(&shaped, &["header", "stamp"]).unwrap_or(shaped)
            } else if descriptor.full_name() == "gz.msgs.Header" {
                normalize_header_root(&normalized).unwrap_or(normalized)
            } else if descriptor.full_name() == "gz.msgs.Header.Map" {
                normalize_header_map_root(&normalized).unwrap_or(normalized)
            } else {
                normalized
            };
            let single_scalar = data_ranges.len() == 1 && {
                let (start, end) = data_ranges[0];
                let value = normalized[start..end].trim_ascii();
                value.first() != Some(&b'[') && value != b"null"
            };
            if repeated_data
                && (data_ranges.len() > 1
                    || single_scalar
                    || (uint64_data && !data_ranges.is_empty()))
            {
                // The maintained decoder keeps only the last repeated key.
                // Decode each native repeated-field occurrence independently,
                // then let the decoder validate all other fields unchanged.
                let mut combined = Vec::new();
                let mut array_index = 0;
                for (ordinal, (start, end)) in data_ranges.iter().copied().enumerate() {
                    let value = normalized[start..end].trim_ascii();
                    if value.is_empty() {
                        return Err("empty repeated data value".into());
                    }
                    if uint64_data {
                        let raw = gz_msgs::serde_json::from_slice(value)?;
                        let leaves = flatten_repeated_json_values(vec![raw])?;
                        let array = gz_msgs::serde_json::to_vec(&leaves)?;
                        let numbers =
                            proto_uint64_array(&array)?.ok_or("invalid uint64 data array")?;
                        let mut document = b"{\"data\":".to_vec();
                        document.extend_from_slice(&array);
                        document.push(b'}');
                        let mut json = gz_msgs::serde_json::Deserializer::from_slice(&document);
                        let mut fragment =
                            DynamicMessage::deserialize(descriptor.clone(), &mut json)?;
                        json.end()?;
                        let Some(Value::List(values)) = fragment.get_field_by_name_mut("data")
                        else {
                            return Err("uint64 data field was not a list".into());
                        };
                        for (index, number) in numbers {
                            *values
                                .get_mut(index)
                                .ok_or("uint64 data index was absent")? = Value::U64(number);
                        }
                        combined.append(values);
                        continue;
                    }
                    let mut document = b"{\"data\":".to_vec();
                    let scalar_hex = if value.first() == Some(&b'[') || value == b"null" {
                        document.extend_from_slice(value);
                        None
                    } else {
                        document.push(b'[');
                        let number = scalar_patches
                            .iter()
                            .find(|(index, _)| *index == ordinal)
                            .map(|(_, number)| number.clone())
                            .or_else(|| {
                                float_data.and_then(|width| {
                                    gz_msgs::serde_json::from_slice::<String>(value)
                                        .ok()
                                        .and_then(|text| proto_hex_float(&text, width))
                                })
                            });
                        if number.is_some() {
                            document.extend_from_slice(b"\"0\"");
                        } else {
                            document.extend_from_slice(value);
                        }
                        document.push(b']');
                        number
                    };
                    document.push(b'}');
                    let mut json = gz_msgs::serde_json::Deserializer::from_slice(&document);
                    let mut fragment = DynamicMessage::deserialize(descriptor.clone(), &mut json)?;
                    json.end()?;
                    let values = match fragment.get_field_by_name_mut("data") {
                        Some(Value::List(values)) => values,
                        _ => return Err("repeated data field was not a list".into()),
                    };
                    if value.first() == Some(&b'[') {
                        for (_, index, number) in patches
                            .iter()
                            .filter(|(ordinal, _, _)| *ordinal == array_index)
                        {
                            *values
                                .get_mut(*index)
                                .ok_or("quoted hex data index was absent")? = number.clone();
                        }
                        if float_data.is_some() {
                            array_index += 1;
                        }
                    } else if let Some(number) = scalar_hex {
                        *values.first_mut().ok_or("quoted hex scalar was absent")? = number;
                    }
                    // The maintained decoder accepts extra spellings for
                    // nonfinite strings; native accepts only these three.
                    let parsed: gz_msgs::serde_json::Value =
                        gz_msgs::serde_json::from_slice(&document)?;
                    for (index, number) in values.iter().enumerate() {
                        let nonfinite = match number {
                            Value::F64(value) => !value.is_finite(),
                            Value::F32(value) => !value.is_finite(),
                            _ => false,
                        };
                        if nonfinite
                            && !matches!(
                                parsed["data"][index].as_str(),
                                Some("NaN" | "Infinity" | "-Infinity")
                            )
                        {
                            return Err("invalid nonfinite data spelling".into());
                        }
                    }
                    combined.append(values);
                }
                if array_index != array_ranges.len() {
                    return Err("repeated data array mismatch".into());
                }
                let mut sanitized = Vec::with_capacity(normalized.len());
                let mut last = 0;
                for (start, end) in data_ranges {
                    sanitized.extend_from_slice(&normalized[last..start]);
                    sanitized.extend_from_slice(b"[]");
                    last = end;
                }
                sanitized.extend_from_slice(&normalized[last..]);
                let mut json = gz_msgs::serde_json::Deserializer::from_slice(&sanitized);
                let mut message = DynamicMessage::deserialize(descriptor, &mut json)?;
                json.end()?;
                match message.get_field_by_name_mut("data") {
                    Some(Value::List(values)) => *values = combined,
                    _ => return Err("repeated data field was not a list".into()),
                }
                message.encode_to_vec()
            } else {
                let mut json = gz_msgs::serde_json::Deserializer::from_slice(&normalized);
                let mut message = DynamicMessage::deserialize(descriptor, &mut json)?;
                json.end()?;
                if !patches.is_empty() {
                    let values = match message.get_field_by_name_mut("data") {
                        Some(Value::List(values)) => values,
                        _ => return Err("quoted hex data field was not a float list".into()),
                    };
                    for (_, index, number) in patches {
                        *values
                            .get_mut(index)
                            .ok_or("quoted hex data index was absent")? = number;
                    }
                }
                for (path, number) in component_patches {
                    match path {
                        ComponentPath::Root(field) => {
                            *message
                                .get_field_by_name_mut(field)
                                .ok_or("quoted component field was absent")? = number;
                        }
                        ComponentPath::Pose(parent, field) => {
                            let child = match message.get_field_by_name_mut(parent) {
                                Some(Value::Message(child)) => child,
                                _ => return Err("quoted Pose parent was absent".into()),
                            };
                            *child
                                .get_field_by_name_mut(field)
                                .ok_or("quoted Pose component was absent")? = number;
                        }
                    }
                }
                message.encode_to_vec()
            }
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

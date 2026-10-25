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
    if end == 2 || value.get(end) != Some(&b'.') {
        return None;
    }
    end += 1;
    // The protobuf number lexer stops at a-f/A-F even though strtod would
    // accept them as hex digits. Only e/E remain in the numeric token.
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

fn flatten_uint64_data(value: &[u8]) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
    let raw: Box<gz_msgs::serde_json::value::RawValue> = gz_msgs::serde_json::from_slice(value)?;
    let mut pending = vec![raw.get().to_owned()];
    let mut leaves = Vec::new();
    while let Some(token) = pending.pop() {
        if token == "null" {
            continue;
        }
        if token.starts_with('[') {
            let items: Vec<Box<gz_msgs::serde_json::value::RawValue>> =
                gz_msgs::serde_json::from_str(&token)?;
            pending.extend(items.into_iter().rev().map(|item| item.get().to_owned()));
        } else {
            leaves.push(token.into_bytes());
        }
    }
    Ok(leaves)
}

struct NormalizedProtoJson {
    bytes: Vec<u8>,
    patches: Vec<(usize, usize, Value)>,
    array_ranges: Vec<(usize, usize)>,
    data_ranges: Vec<(usize, usize)>,
}

fn normalize_proto_json(
    input: &[u8],
    float_data: Option<FloatWidth>,
    repeated_data: bool,
    uint64_data: bool,
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
    let mut float_array_depth = None;
    let mut float_item_index = 0;
    let mut float_patches = Vec::new();
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
                data_field = gz_msgs::serde_json::from_slice::<String>(&output[output_start..])
                    .is_ok_and(|key| key == "data");
            } else if float_array_depth == Some(frames.len()) {
                if let (Some(width), Ok(value)) = (
                    float_data,
                    gz_msgs::serde_json::from_slice::<String>(&output[output_start..]),
                ) {
                    if let Some(number) = proto_hex_float(&value, width) {
                        output.truncate(output_start);
                        output.extend_from_slice(b"\"0\"");
                        float_patches.push((float_array_count - 1, float_item_index, number));
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
                data_field = gz_msgs::serde_json::from_slice::<String>(&output[output_start..])
                    .is_ok_and(|key| key == "data");
            } else if float_array_depth == Some(frames.len()) {
                if let (Some(width), Ok(value)) = (
                    float_data,
                    gz_msgs::serde_json::from_slice::<String>(&output[output_start..]),
                ) {
                    if let Some(number) = proto_hex_float(&value, width) {
                        output.truncate(output_start);
                        output.extend_from_slice(b"\"0\"");
                        float_patches.push((float_array_count - 1, float_item_index, number));
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
                }
            } else {
                output.extend_from_slice(&input[start..index]);
            }
            previous = Some(input[index - 1]);
            continue;
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
            b'{' => frames.push(Frame::ObjectKey),
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
                if byte == b'}' && frames.len() == 1 {
                    if let Some(start) = data_value_start.take() {
                        data_value_ranges.push((start, output.len()));
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
                *frames.last_mut().unwrap() = Frame::ObjectValue
            }
            b',' if frames.last() == Some(&Frame::ObjectValue) => {
                if frames.len() == 1 {
                    if let Some(start) = data_value_start.take() {
                        data_value_ranges.push((start, output.len()));
                    }
                }
                *frames.last_mut().unwrap() = Frame::ObjectKey;
                if frames.len() == 1 {
                    data_field = false;
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
        patches: float_patches,
        array_ranges: float_array_ranges,
        data_ranges: data_value_ranges,
    }
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
            let NormalizedProtoJson {
                bytes: normalized,
                patches,
                array_ranges,
                data_ranges,
            } = normalize_proto_json(&input, float_data, repeated_data, uint64_data);
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
                for (start, end) in data_ranges.iter().copied() {
                    let value = normalized[start..end].trim_ascii();
                    if value.is_empty() {
                        return Err("empty repeated data value".into());
                    }
                    if uint64_data {
                        let leaves = flatten_uint64_data(value)?;
                        let mut array = vec![b'['];
                        for (index, token) in leaves.iter().enumerate() {
                            if index != 0 {
                                array.push(b',');
                            }
                            array.extend_from_slice(token);
                        }
                        array.push(b']');
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
                        let number = float_data.and_then(|width| {
                            gz_msgs::serde_json::from_slice::<String>(value)
                                .ok()
                                .and_then(|text| proto_hex_float(&text, width))
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

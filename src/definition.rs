// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Bounded native declaration formatting over validated reflection descriptors.
use prost_reflect::{
    Cardinality, DynamicMessage, EnumDescriptor, FieldDescriptor, Kind, MessageDescriptor, Syntax,
    Value,
};
use std::{collections::BTreeSet, fmt::Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefinitionError {
    Unsupported(&'static str),
}
impl std::fmt::Display for DefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self::Unsupported(feature) = self;
        write!(f, "unsupported message definition: {feature}")
    }
}
impl std::error::Error for DefinitionError {}

/// Format the qualified common declaration subset using native layout.
///
/// This does not replace the immutable canonical `message_definition` corpus.
/// Unhandled metadata returns Unsupported instead of being silently omitted.
/// The whole definition is produced before return; an error exposes no partial text.
pub fn descriptor_definition(descriptor: &MessageDescriptor) -> Result<String, DefinitionError> {
    definition_with_omissions(descriptor, &BTreeSet::new())
}

pub(crate) fn definition_with_omissions(
    descriptor: &MessageDescriptor,
    omissions: &BTreeSet<String>,
) -> Result<String, DefinitionError> {
    let mut output = String::new();
    message(&mut output, 0, descriptor, omissions)?;
    Ok(output)
}

fn line(output: &mut String, depth: usize, value: impl std::fmt::Display) {
    writeln!(output, "{}{value}", "  ".repeat(depth)).unwrap();
}

fn quoted(value: &str) -> String {
    let mut output = String::from("\"");
    for byte in value.bytes() {
        let escaped = std::ascii::escape_default(byte);
        // Protobuf CEscape uses byte-octal escapes where Rust uses hex.
        if escaped.len() == 4 {
            write!(output, "\\{byte:03o}").unwrap();
        } else {
            output.extend(escaped.map(char::from));
        }
    }
    output.push('"');
    output
}

fn reserved<'a>(
    output: &mut String,
    depth: usize,
    ranges: impl Iterator<Item = Result<(i64, i64), DefinitionError>>,
    names: impl Iterator<Item = &'a str>,
    minimum: i64,
    maximum: i64,
) -> Result<(), DefinitionError> {
    let mut items = Vec::new();
    for range in ranges {
        let (start, end) = range?;
        if start < minimum {
            return Err(DefinitionError::Unsupported("invalid reserved range"));
        }
        items.push(if start == end {
            start.to_string()
        } else if end >= maximum {
            format!("{start} to max")
        } else {
            format!("{start} to {end}")
        });
    }
    if !items.is_empty() {
        line(
            output,
            depth,
            format_args!("reserved {};", items.join(", ")),
        );
    }
    let names: Vec<_> = names.map(quoted).collect();
    if !names.is_empty() {
        line(
            output,
            depth,
            format_args!("reserved {};", names.join(", ")),
        );
    }
    Ok(())
}

fn options(value: DynamicMessage, allowed: &[&str]) -> Result<Vec<String>, DefinitionError> {
    if value.unknown_fields().next().is_some() {
        return Err(DefinitionError::Unsupported("unknown options"));
    }
    if value.extensions().next().is_some() {
        return Err(DefinitionError::Unsupported("custom options"));
    }
    value
        .fields()
        .map(|(field, value)| {
            if !allowed.contains(&field.name()) {
                return Err(DefinitionError::Unsupported("option"));
            }
            let value = value
                .as_bool()
                .ok_or(DefinitionError::Unsupported("option type"))?;
            Ok(format!("{} = {value}", field.name()))
        })
        .collect()
}

fn type_name(kind: Kind) -> String {
    match kind {
        Kind::Message(message) => format!(".{}", message.full_name()),
        Kind::Enum(enumeration) => format!(".{}", enumeration.full_name()),
        scalar => format!("{scalar:?}"),
    }
}

fn map_types(entry: &MessageDescriptor) -> Result<(Kind, Kind), DefinitionError> {
    let invalid = DefinitionError::Unsupported("malformed map entry");
    let key = entry.get_field(1).ok_or(invalid)?;
    let value = entry.get_field(2).ok_or(invalid)?;
    if entry.fields().len() != 2
        || key.name() != "key"
        || value.name() != "value"
        || [key.clone(), value.clone()].iter().any(|field| {
            field.cardinality() != Cardinality::Optional
                || field.containing_oneof().is_some()
                || field.field_descriptor_proto().default_value.is_some()
        })
        || !matches!(
            key.kind(),
            Kind::Int32
                | Kind::Int64
                | Kind::Uint32
                | Kind::Uint64
                | Kind::Sint32
                | Kind::Sint64
                | Kind::Fixed32
                | Kind::Fixed64
                | Kind::Sfixed32
                | Kind::Sfixed64
                | Kind::Bool
                | Kind::String
        )
    {
        return Err(invalid);
    }
    Ok((key.kind(), value.kind()))
}

fn field(
    output: &mut String,
    depth: usize,
    field: &FieldDescriptor,
    omissions: &BTreeSet<String>,
) -> Result<(), DefinitionError> {
    let raw = field.field_descriptor_proto();
    let proto2 = field.parent_file().syntax() == Syntax::Proto2;
    let default = if raw.default_value.is_some() {
        if !proto2 || field.is_list() || field.containing_oneof().is_some() {
            return Err(DefinitionError::Unsupported(if proto2 {
                "proto2 default"
            } else {
                "proto3 default or required field"
            }));
        }
        // Native declarations print the parsed value without changing raw metadata.
        Some(match (field.kind(), field.default_value()) {
            (Kind::Int32 | Kind::Sint32 | Kind::Sfixed32, Value::I32(value)) => value.to_string(),
            (Kind::Int64 | Kind::Sint64 | Kind::Sfixed64, Value::I64(value)) => value.to_string(),
            (Kind::Uint32 | Kind::Fixed32, Value::U32(value)) => value.to_string(),
            (Kind::Uint64 | Kind::Fixed64, Value::U64(value)) => value.to_string(),
            (Kind::Bool, Value::Bool(value)) => value.to_string(),
            (Kind::String, Value::String(value)) => quoted(&value),
            _ => return Err(DefinitionError::Unsupported("proto2 default")),
        })
    } else {
        None
    };
    if field.cardinality() == Cardinality::Required && !proto2 {
        return Err(DefinitionError::Unsupported(
            "proto3 default or required field",
        ));
    }
    if field.is_group() {
        return Err(DefinitionError::Unsupported("groups"));
    }
    if field.containing_oneof().is_some() && field.cardinality() != Cardinality::Optional {
        return Err(DefinitionError::Unsupported("malformed oneof field"));
    }
    let mut field_options = Vec::new();
    if let Some(value) = default {
        field_options.push(format!("default = {value}"));
    }
    if let Some(name) = raw
        .json_name
        .as_ref()
        .filter(|_| !omissions.contains(field.full_name()))
    {
        field_options.push(format!("json_name = {}", quoted(name)));
    }
    field_options.extend(options(field.options(), &["packed", "deprecated"])?);
    let suffix = if field_options.is_empty() {
        String::new()
    } else {
        format!(" [{}]", field_options.join(", "))
    };
    let (label, name) = match field.kind() {
        Kind::Message(entry) if field.is_map() => {
            let (key, value) = map_types(&entry)?;
            ("", format!("map<{}, {}>", type_name(key), type_name(value)))
        }
        kind => (
            if field.is_list() {
                "repeated "
            } else if field.cardinality() == Cardinality::Required {
                "required "
            } else if raw.proto3_optional() || (proto2 && field.containing_oneof().is_none()) {
                "optional "
            } else {
                ""
            },
            type_name(kind),
        ),
    };
    line(
        output,
        depth,
        format_args!(
            "{label}{name} {} = {}{suffix};",
            field.name(),
            field.number()
        ),
    );
    Ok(())
}

fn enumeration(
    output: &mut String,
    depth: usize,
    value: &EnumDescriptor,
) -> Result<(), DefinitionError> {
    line(output, depth, format_args!("enum {} {{", value.name()));
    for option in options(value.options(), &["allow_alias", "deprecated"])? {
        line(output, depth + 1, format_args!("option {option};"));
    }
    let mut values: Vec<_> = value.values().collect();
    values.sort_by_key(|value| value.path().last().copied());
    for value in values {
        let options = options(value.options(), &["deprecated"])?;
        let suffix = if options.is_empty() {
            String::new()
        } else {
            format!(" [{}]", options.join(", "))
        };
        line(
            output,
            depth + 1,
            format_args!("{} = {}{suffix};", value.name(), value.number()),
        );
    }
    reserved(
        output,
        depth + 1,
        value
            .reserved_ranges()
            .map(|range| Ok((i64::from(*range.start()), i64::from(*range.end())))),
        value.reserved_names(),
        i64::from(i32::MIN),
        i64::from(i32::MAX),
    )?;
    line(output, depth, "}");
    Ok(())
}

fn source_fields(fields: impl Iterator<Item = FieldDescriptor>) -> Vec<FieldDescriptor> {
    let mut fields: Vec<_> = fields.collect();
    // Reflection iterates by tag; native declarations retain source index order.
    fields.sort_by_key(|field| field.path().last().copied());
    fields
}

fn message(
    output: &mut String,
    depth: usize,
    value: &MessageDescriptor,
    omissions: &BTreeSet<String>,
) -> Result<(), DefinitionError> {
    if depth >= 100 {
        return Err(DefinitionError::Unsupported("definition nesting limit"));
    }
    if value.is_map_entry() {
        map_types(value)?;
        return Ok(());
    }
    if value.extension_ranges().len() != 0 || value.child_extensions().len() != 0 {
        return Err(DefinitionError::Unsupported("extension declarations"));
    }

    for oneof in value.oneofs() {
        options(oneof.options(), &[])?;
        if oneof.fields().len() == 0 {
            return Err(DefinitionError::Unsupported("empty oneof"));
        }
    }
    line(output, depth, format_args!("message {} {{", value.name()));
    for option in options(value.options(), &["deprecated"])? {
        line(output, depth + 1, format_args!("option {option};"));
    }
    for child in value.child_messages() {
        message(output, depth + 1, &child, omissions)?;
    }
    for child in value.child_enums() {
        enumeration(output, depth + 1, &child)?;
    }
    let mut seen = BTreeSet::new();
    for item in source_fields(value.fields()) {
        if let Some(oneof) = item
            .containing_oneof()
            .filter(|oneof| !oneof.is_synthetic())
        {
            if seen.insert(oneof.name().to_owned()) {
                line(output, depth + 1, format_args!("oneof {} {{", oneof.name()));
                for item in source_fields(oneof.fields()) {
                    field(output, depth + 2, &item, omissions)?;
                }
                line(output, depth + 1, "}");
            }
        } else {
            field(output, depth + 1, &item, omissions)?;
        }
    }
    reserved(
        output,
        depth + 1,
        value.descriptor_proto().reserved_range.iter().map(|range| {
            // Native DebugString performs signed int start+1 and end-1.
            if range.start() == i32::MAX || range.end() == i32::MIN {
                return Err(DefinitionError::Unsupported(
                    "reserved range boundary overflow",
                ));
            }
            Ok((i64::from(range.start()), i64::from(range.end()) - 1))
        }),
        value.reserved_names(),
        1,
        i64::from(crate::prost::encoding::MAX_TAG),
    )?;
    line(output, depth, "}");
    Ok(())
}

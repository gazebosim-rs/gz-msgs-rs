// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Bounded native declaration formatting over validated reflection descriptors.
use prost_reflect::{
    Cardinality, DynamicMessage, EnumDescriptor, FieldDescriptor, Kind, MessageDescriptor, Syntax,
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

/// Format the qualified common proto3 declaration subset using native layout.
///
/// This does not replace the immutable canonical `message_definition` corpus.
/// Unhandled metadata returns Unsupported instead of being silently omitted.
/// The whole definition is produced before return; an error exposes no partial text.
pub fn descriptor_definition(descriptor: &MessageDescriptor) -> Result<String, DefinitionError> {
    if descriptor.parent_file().syntax() != Syntax::Proto3 {
        return Err(DefinitionError::Unsupported("proto2"));
    }
    let mut output = String::new();
    message(&mut output, 0, descriptor)?;
    Ok(output)
}

fn line(output: &mut String, depth: usize, value: impl std::fmt::Display) {
    writeln!(output, "{}{value}", "  ".repeat(depth)).unwrap();
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
) -> Result<(), DefinitionError> {
    let raw = field.field_descriptor_proto();
    if raw.default_value.is_some() || field.cardinality() == Cardinality::Required {
        return Err(DefinitionError::Unsupported(
            "proto3 default or required field",
        ));
    }
    let mut field_options = Vec::new();
    if let Some(name) = &raw.json_name {
        if !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
            return Err(DefinitionError::Unsupported("non-identifier JSON name"));
        }
        field_options.push(format!("json_name = \"{name}\""));
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
            } else if raw.proto3_optional() {
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
    if value.reserved_ranges().len() != 0 || value.reserved_names().len() != 0 {
        return Err(DefinitionError::Unsupported("enum reserved declarations"));
    }
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
    if value.reserved_ranges().len() != 0 || value.reserved_names().len() != 0 {
        return Err(DefinitionError::Unsupported(
            "message reserved declarations",
        ));
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
        message(output, depth + 1, &child)?;
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
                    field(output, depth + 2, &item)?;
                }
                line(output, depth + 1, "}");
            }
        } else {
            field(output, depth + 1, &item)?;
        }
    }
    line(output, depth, "}");
    Ok(())
}

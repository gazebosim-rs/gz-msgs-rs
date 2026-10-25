// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::{
    prost::encoding::{self, DecodeContext, WireType},
    prost_reflect::{Kind, MessageDescriptor},
    WireMessage,
};

pub(crate) mod sealed {
    pub trait Sealed {}
}

/// A generated canonical type bound to its exact bundled descriptor.
/// Implementations are generated from the same schema set as the Rust types.
pub trait CanonicalMessage: sealed::Sealed + prost::Message + Default {
    const PROTOBUF_NAME: &'static str;
}

/// Invalid framing or known-field data in native-compatible decoding.
#[derive(Debug)]
pub enum CompatibleDecodeError {
    Protobuf(prost::DecodeError),
    Malformed(&'static str),
}
impl std::fmt::Display for CompatibleDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Protobuf(error) => error.fmt(f),
            Self::Malformed(reason) => write!(f, "invalid protobuf framing: {reason}"),
        }
    }
}
impl std::error::Error for CompatibleDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Protobuf(error) => Some(error),
            Self::Malformed(_) => None,
        }
    }
}
impl From<prost::DecodeError> for CompatibleDecodeError {
    fn from(value: prost::DecodeError) -> Self {
        Self::Protobuf(value)
    }
}
impl<M: CanonicalMessage> WireMessage<M> {
    /// Decode with native C++ known-tag/wrong-wire handling, retaining original bytes.
    ///
    /// Ordinary inputs use the generated decoder. On failure, a descriptor-bound
    /// pass removes unknown fields only from a temporary typed view. A wrong-wire
    /// occurrence does not set presence, replace a oneof, or erase a known value.
    /// Nested messages (including map entries and google.protobuf.Any fields) are
    /// visited; bytes payloads such as Any.value remain opaque. Forward bytes()
    /// verbatim; encoding value() discards unknowns and may reorder fields.
    pub fn decode_compatible(bytes: &[u8]) -> Result<Self, CompatibleDecodeError> {
        let value = match M::decode(bytes) {
            Ok(value) => value,
            Err(_) => {
                let desc = crate::descriptors()
                    .get_message_by_name(M::PROTOBUF_NAME)
                    .expect("generated canonical binding");
                let known = known_fields(bytes, &desc, 100)?;
                M::decode(known.as_slice())?
            }
        };
        Ok(Self {
            bytes: bytes.to_vec(),
            value,
        })
    }
}

// Share the native default recursion budget across known messages and unknown
// groups. prost's scalar skipping validates framing; its private context cannot
// carry a remaining budget across descriptor recursion.
fn skip(
    input: &mut &[u8],
    tag: u32,
    wire: WireType,
    depth: u32,
) -> Result<(), CompatibleDecodeError> {
    if wire != WireType::StartGroup {
        return Ok(encoding::skip_field(
            wire,
            tag,
            input,
            DecodeContext::default(),
        )?);
    }
    if depth == 0 {
        return Err(CompatibleDecodeError::Malformed("recursion limit"));
    }
    loop {
        let (inner, wire) = encoding::decode_key(input)?;
        if wire == WireType::EndGroup {
            return if tag == inner {
                Ok(())
            } else {
                Err(CompatibleDecodeError::Malformed("mismatched group tag"))
            };
        }
        skip(input, inner, wire, depth - 1)?;
    }
}

fn known_fields(
    mut input: &[u8],
    desc: &MessageDescriptor,
    depth: u32,
) -> Result<Vec<u8>, CompatibleDecodeError> {
    let mut output = Vec::new();
    while !input.is_empty() {
        let original = input;
        let (tag, wire) = encoding::decode_key(&mut input)?;
        let Some(field) = desc.get_field(tag) else {
            skip(&mut input, tag, wire, depth)?;
            continue;
        };
        let kind = field.kind();
        let packable = !matches!(kind, Kind::String | Kind::Bytes | Kind::Message(_));
        if wire != kind.wire_type()
            && !(field.is_list() && packable && wire == WireType::LengthDelimited)
        {
            skip(&mut input, tag, wire, depth)?;
            continue;
        }
        if let Kind::Message(child) = kind {
            if depth == 0 {
                return Err(CompatibleDecodeError::Malformed("recursion limit"));
            }
            let len = encoding::decode_varint(&mut input)?;
            if len > input.len() as u64 {
                return Err(CompatibleDecodeError::Malformed("truncated message"));
            }
            let len = len as usize;
            let filtered = known_fields(&input[..len], &child, depth - 1)?;
            input = &input[len..];
            encoding::encode_key(tag, wire, &mut output);
            encoding::encode_varint(filtered.len() as u64, &mut output);
            output.extend(filtered);
        } else {
            skip(&mut input, tag, wire, depth)?;
            output.extend_from_slice(&original[..original.len() - input.len()]);
        }
    }
    Ok(output)
}

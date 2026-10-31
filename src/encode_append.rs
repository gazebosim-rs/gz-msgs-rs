// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use prost::Message;
use std::{collections::TryReserveError, fmt};

/// A returned append error leaves the original bytes and logical length intact.
#[derive(Debug)]
pub enum EncodeAppendError {
    /// The existing length plus the advertised encoded length exceeds usize.
    LengthOverflow,
    /// Vec could not reserve the advertised additional space.
    Reserve(TryReserveError),
    /// The encoder returned an ordinary prost encoding error.
    Encode(prost::EncodeError),
    /// A custom encoder returned success without consuming the advertised tail.
    IncompleteWrite { expected: usize, written: usize },
}

impl fmt::Display for EncodeAppendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthOverflow => f.write_str("encoded output length overflow"),
            Self::Reserve(error) => write!(f, "cannot reserve encoded output: {error}"),
            Self::Encode(error) => write!(f, "cannot encode appended message: {error}"),
            Self::IncompleteWrite { expected, written } => {
                write!(f, "message wrote {written} of {expected} encoded bytes")
            }
        }
    }
}

impl std::error::Error for EncodeAppendError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Reserve(error) => Some(error),
            Self::Encode(error) => Some(error),
            Self::LengthOverflow | Self::IncompleteWrite { .. } => None,
        }
    }
}

/// Append a standard prost message through an initialized mutable byte slice.
///
/// This is an opt-in alternative to `Message::encode` on a `Vec<u8>`. It keeps
/// the existing prefix, reserves space fallibly, and zero-initializes the tail
/// before encoding. Clear the vector first when reusing it for a new payload.
/// No length delimiter is added. Standard generated/default codecs are unchanged.
///
/// Returned errors restore the original logical length and prefix; capacity may
/// grow. An encoder that returns success without filling its advertised length
/// is rejected rather than exposing zero padding. Panics, aborting allocation
/// failures and arbitrary custom `Message` side effects are not transactional.
/// The length traversal and zero-fill can outweigh any benefit for some messages.
///
/// ```
/// let message = gz_msgs::Time { sec: -1, nsec: 42 };
/// let mut output = vec![0xaa];
/// gz_msgs::encode_append(&message, &mut output)?;
/// assert_eq!(output[0], 0xaa);
/// # Ok::<(), gz_msgs::EncodeAppendError>(())
/// ```
pub fn encode_append<M: Message>(
    message: &M,
    output: &mut Vec<u8>,
) -> Result<(), EncodeAppendError> {
    let prefix = output.len();
    let expected = message.encoded_len();
    let end = prefix
        .checked_add(expected)
        .ok_or(EncodeAppendError::LengthOverflow)?;
    output
        .try_reserve(expected)
        .map_err(EncodeAppendError::Reserve)?;
    output.resize(end, 0);
    let mut remaining = &mut output[prefix..];
    let result = message.encode(&mut remaining);
    let written = expected - remaining.len();
    let error = match result {
        Err(error) => Some(EncodeAppendError::Encode(error)),
        Ok(()) if written != expected => {
            Some(EncodeAppendError::IncompleteWrite { expected, written })
        }
        Ok(()) => None,
    };
    if let Some(error) = error {
        output.truncate(prefix);
        return Err(error);
    }
    Ok(())
}

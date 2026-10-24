// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Instance-owned external descriptors with ordered, retryable loading.
use crate::{
    prost::{DecodeError, Message},
    prost_reflect::{DescriptorError, DescriptorPool, DynamicMessage, ReflectMessage},
    prost_types::FileDescriptorSet,
};
use std::{io, path::Path};

/// A canonical descriptor snapshot extended by caller-supplied descriptor sets.
/// Mutations require an exclusive borrow; existing dynamic values own snapshots.
#[derive(Clone)]
pub struct DescriptorRegistry {
    pool: DescriptorPool,
}
impl Default for DescriptorRegistry {
    fn default() -> Self {
        Self {
            pool: crate::descriptors().clone(),
        }
    }
}
impl DescriptorRegistry {
    pub fn pool(&self) -> &DescriptorPool {
        &self.pool
    }
    /// Read a binary FileDescriptorSet; malformed bytes return InvalidData.
    /// The returned errors describe individual files that failed validation.
    pub fn load_file(&mut self, path: impl AsRef<Path>) -> io::Result<Vec<DescriptorError>> {
        self.load_bytes(&std::fs::read(path)?)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }
    /// Attempt every file in descriptor-set order, retaining successful files.
    /// Missing imports can be supplied and failed files retried on a later call.
    /// Duplicate file names are ignored, matching the native pool; this does not
    /// replace existing schemas. No filesystem path cache or environment is used.
    pub fn load_bytes(&mut self, bytes: &[u8]) -> Result<Vec<DescriptorError>, DecodeError> {
        // Reflect the descriptor set instead of prost-types decoding it: custom
        // options must survive as nested unknown bytes until their schema loads.
        let set = DynamicMessage::decode(FileDescriptorSet::default().descriptor(), bytes)?;
        let files = set
            .get_field_by_name("file")
            .expect("FileDescriptorSet.file");
        let mut errors = Vec::new();
        for file in files.as_list().expect("repeated descriptor files") {
            let bytes = file
                .as_message()
                .expect("FileDescriptorProto")
                .encode_to_vec();
            if let Err(error) = self.pool.decode_file_descriptor_proto(bytes.as_slice()) {
                errors.push(error);
            }
        }
        Ok(errors)
    }
}

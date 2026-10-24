// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Instance-owned external descriptors with ordered, retryable loading.
use crate::{
    prost::{DecodeError, Message},
    prost_reflect::{DescriptorError, DescriptorPool, DynamicMessage, ReflectMessage},
    prost_types::FileDescriptorSet,
};
use std::{
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
};

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
    /// Load a platform-separated list of explicit paths or directories.
    /// Return path-associated IO/descriptor errors without printing diagnostics.
    pub fn load_paths(&mut self, paths: impl AsRef<OsStr>) -> Vec<(PathBuf, io::Error)> {
        self.load_os_paths(paths.as_ref())
    }
    // Share traversal code across path argument types instead of monomorphizing it.
    fn load_os_paths(&mut self, paths: &OsStr) -> Vec<(PathBuf, io::Error)> {
        let mut errors = Vec::new();
        for path in std::env::split_paths(paths) {
            if path.is_dir() {
                match self.load_directory(&path) {
                    Ok(mut found) => errors.append(&mut found),
                    Err(error) => errors.push((path, error)),
                }
            } else {
                self.load_candidate(&path, &mut errors);
            }
        }
        errors
    }
    /// Scan one directory level, prioritizing the native gz-msgs.gz_desc file.
    /// Other entries retain filesystem traversal order; imports are not retried
    /// automatically. Callers may retry after supplying missing dependencies.
    pub fn load_directory(
        &mut self,
        path: impl AsRef<Path>,
    ) -> io::Result<Vec<(PathBuf, io::Error)>> {
        self.load_directory_path(path.as_ref())
    }
    fn load_directory_path(&mut self, path: &Path) -> io::Result<Vec<(PathBuf, io::Error)>> {
        let mut errors = Vec::new();
        let own = path.join("gz-msgs.gz_desc");
        if own.exists() {
            self.load_candidate(&own, &mut errors);
        }
        for entry in std::fs::read_dir(path)? {
            let entry = entry?.path();
            if entry.file_name() != Some(OsStr::new("gz-msgs.gz_desc")) {
                self.load_candidate(&entry, &mut errors);
            }
        }
        Ok(errors)
    }
    fn load_candidate(&mut self, path: &Path, errors: &mut Vec<(PathBuf, io::Error)>) {
        // Native recognition is a substring check on the full path, not a suffix.
        if ![".desc", ".gz_desc", ".proto"]
            .iter()
            .any(|part| path.to_string_lossy().contains(part))
        {
            return;
        }
        match self.load_file(path) {
            Ok(found) => errors.extend(found.into_iter().map(|error| {
                (
                    path.to_owned(),
                    io::Error::new(io::ErrorKind::InvalidData, error),
                )
            })),
            Err(error) => errors.push((path.to_owned(), error)),
        }
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

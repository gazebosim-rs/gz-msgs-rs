// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Instance-owned external descriptors with ordered, retryable loading.
use crate::{
    prost::{DecodeError, Message},
    prost_reflect::{DescriptorError, DescriptorPool, DynamicMessage, ReflectMessage},
    prost_types::{
        field_descriptor_proto::Type, DescriptorProto, FieldDescriptorProto, FileDescriptorProto,
        FileDescriptorSet,
    },
};
use std::{
    collections::BTreeSet,
    ffi::{OsStr, OsString},
    io,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

/// File-level console wording for a descriptor-loading failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadDiagnostic {
    Open,
    Parse,
}

/// A structured loading error with optional native file-diagnostic provenance.
#[derive(Debug)]
pub struct DescriptorLoadError {
    pub path: PathBuf,
    pub error: io::Error,
    pub diagnostic: Option<LoadDiagnostic>,
}

impl DescriptorLoadError {
    pub(crate) fn other(path: PathBuf, error: io::Error) -> Self {
        Self {
            path,
            error,
            diagnostic: None,
        }
    }

    pub(crate) fn into_error(self) -> (PathBuf, io::Error) {
        (self.path, self.error)
    }
}

/// A canonical descriptor snapshot extended by caller-supplied descriptor sets.
/// Mutations require an exclusive borrow; existing dynamic values own snapshots.
#[derive(Clone)]
pub struct DescriptorRegistry {
    pool: DescriptorPool,
    json_omissions: Arc<BTreeSet<String>>,
    loaded_files: Arc<BTreeSet<OsString>>,
}
impl Default for DescriptorRegistry {
    fn default() -> Self {
        Self {
            pool: crate::descriptors().clone(),
            json_omissions: Arc::default(),
            loaded_files: Arc::default(),
        }
    }
}
impl DescriptorRegistry {
    pub fn pool(&self) -> &DescriptorPool {
        &self.pool
    }
    /// Format a registry-owned descriptor with accepted raw JSON-name provenance.
    pub fn definition(&self, name: &str) -> Result<Option<String>, crate::DefinitionError> {
        let Some(descriptor) = self.pool.get_message_by_name(name) else {
            return Ok(None);
        };
        if let Some(body) = crate::message_definition(descriptor.full_name()) {
            return Ok(Some(body.to_owned()));
        }
        crate::definition::definition_with_omissions(&descriptor, &self.json_omissions).map(Some)
    }
    /// Load a platform-separated list of explicit paths or directories.
    /// Return path-associated IO/descriptor errors without printing diagnostics.
    pub fn load_paths(&mut self, paths: impl AsRef<OsStr>) -> Vec<(PathBuf, io::Error)> {
        self.load_paths_detailed(paths.as_ref())
            .into_iter()
            .map(DescriptorLoadError::into_error)
            .collect()
    }
    // Share traversal code across path argument types instead of monomorphizing it.
    pub(crate) fn load_paths_detailed(&mut self, paths: &OsStr) -> Vec<DescriptorLoadError> {
        let mut errors = Vec::new();
        for path in std::env::split_paths(paths) {
            if path.is_dir() {
                match self.load_directory_path(&path) {
                    Ok(mut found) => errors.append(&mut found),
                    Err(error) => errors.push(DescriptorLoadError::other(path, error)),
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
        self.load_directory_path(path.as_ref()).map(|errors| {
            errors
                .into_iter()
                .map(DescriptorLoadError::into_error)
                .collect()
        })
    }
    fn load_directory_path(&mut self, path: &Path) -> io::Result<Vec<DescriptorLoadError>> {
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
    fn load_candidate(&mut self, path: &Path, errors: &mut Vec<DescriptorLoadError>) {
        // Native recognition is a substring check on the full path, not a suffix.
        if ![".desc", ".gz_desc", ".proto"]
            .iter()
            .any(|part| path.to_string_lossy().contains(part))
        {
            return;
        }
        match self.load_file_detailed(path) {
            Ok(found) => errors.extend(found.into_iter().map(|error| {
                DescriptorLoadError::other(
                    path.to_owned(),
                    io::Error::new(io::ErrorKind::InvalidData, error),
                )
            })),
            Err(error) => errors.push(error),
        }
    }
    /// Read a binary FileDescriptorSet; malformed bytes return InvalidData.
    /// Successfully loaded paths are cached by native-compatible canonical key.
    /// IO, parse and individual-file validation failures remain retryable.
    pub fn load_file(&mut self, path: impl AsRef<Path>) -> io::Result<Vec<DescriptorError>> {
        let path = path.as_ref();
        self.load_file_detailed(path)
            .map_err(|failure| failure.error)
    }
    fn load_file_detailed(
        &mut self,
        path: &Path,
    ) -> Result<Vec<DescriptorError>, DescriptorLoadError> {
        let open_error = |error| DescriptorLoadError {
            path: path.to_owned(),
            error,
            diagnostic: Some(LoadDiagnostic::Open),
        };
        let key = descriptor_path_key(path).map_err(open_error)?;
        if self.loaded_files.contains(&key) {
            return Ok(Vec::new());
        }
        let mut file = std::fs::File::open(path).map_err(open_error)?;
        let mut bytes = Vec::new();
        io::Read::read_to_end(&mut file, &mut bytes).map_err(|error| DescriptorLoadError {
            path: path.to_owned(),
            error,
            diagnostic: Some(LoadDiagnostic::Parse),
        })?;
        let errors = self
            .load_bytes_detailed(&bytes)
            .map_err(|(error, diagnostic)| DescriptorLoadError {
                path: path.to_owned(),
                error: io::Error::new(io::ErrorKind::InvalidData, error),
                diagnostic,
            })?;
        if errors.is_empty() {
            Arc::make_mut(&mut self.loaded_files).insert(key);
        }
        Ok(errors)
    }
    /// Attempt every file in descriptor-set order, retaining successful files.
    /// Missing imports can be supplied and failed files retried on a later call.
    /// Duplicate file names are ignored, matching the native pool; this does not
    /// replace existing schemas. No filesystem path cache or environment is used.
    /// Byte defaults that can panic in the pinned loader return DecodeError.
    pub fn load_bytes(&mut self, bytes: &[u8]) -> Result<Vec<DescriptorError>, DecodeError> {
        self.load_bytes_detailed(bytes).map_err(|(error, _)| error)
    }
    fn load_bytes_detailed(
        &mut self,
        bytes: &[u8],
    ) -> Result<Vec<DescriptorError>, (DecodeError, Option<LoadDiagnostic>)> {
        // Reflect the descriptor set instead of prost-types decoding it: custom
        // options must survive as nested unknown bytes until their schema loads.
        let set = DynamicMessage::decode(FileDescriptorSet::default().descriptor(), bytes)
            .map_err(|error| (error, Some(LoadDiagnostic::Parse)))?;
        let files = set
            .get_field_by_name("file")
            .expect("FileDescriptorSet.file");
        let mut errors = Vec::new();
        for file in files.as_list().expect("repeated descriptor files") {
            let bytes = file
                .as_message()
                .expect("FileDescriptorProto")
                .encode_to_vec();
            let raw =
                FileDescriptorProto::decode(bytes.as_slice()).map_err(|error| (error, None))?;
            let existed = self.pool.get_file_by_name(raw.name()).is_some();
            if !existed {
                check_default_fields(&raw.extension).map_err(|error| (error, None))?;
                for message in &raw.message_type {
                    check_message_defaults(message).map_err(|error| (error, None))?;
                }
            }
            match self
                .pool
                .decode_file_descriptor_proto_with_integer_defaults(bytes.as_slice())
            {
                Err(error) => errors.push(error),
                Ok(()) if !existed => {
                    let mut omissions = BTreeSet::new();
                    for message in &raw.message_type {
                        missing_json_names(raw.package(), message, &mut omissions);
                    }
                    Arc::make_mut(&mut self.json_omissions).extend(omissions);
                }
                Ok(()) => {}
            }
        }
        Ok(errors)
    }
}

// Match the pinned native weak-canonical cache policy, including its relative
// missing-prefix behavior. Filesystem resolution stays in std; path-clean owns
// lexical dot/parent normalization. This is an identity key, not an IO sandbox.
fn descriptor_path_key(path: &Path) -> io::Result<OsString> {
    match path.canonicalize() {
        Ok(value) => return Ok(value.into_os_string()),
        Err(error) if missing_path(&error) => {}
        Err(error) => return Err(error),
    }
    let mut prefix = PathBuf::new();
    let mut existing = PathBuf::new();
    let mut resolved = None;
    for component in path.components() {
        prefix.push(component.as_os_str());
        match prefix.canonicalize() {
            Ok(value) => {
                existing = prefix.clone();
                resolved = Some(value);
            }
            Err(error) if missing_path(&error) => break,
            Err(error) => return Err(error),
        }
    }
    let mut key = match resolved {
        Some(value) => path_clean::clean(
            value.join(
                path.strip_prefix(existing)
                    .expect("prefix from the same path"),
            ),
        ),
        None => path_clean::clean(path),
    };
    // Native stores strings, not path objects: a trailing directory separator
    // must not compare equal to a successfully cached ordinary filename.
    let bytes = path.as_os_str().as_encoded_bytes();
    let separator = |byte: u8| std::path::is_separator(char::from(byte));
    let directory_tail = bytes.last().is_some_and(|byte| separator(*byte))
        || (bytes.len() >= 2 && bytes.last() == Some(&b'.') && separator(bytes[bytes.len() - 2]))
        || matches!(path.components().next_back(), Some(Component::ParentDir));
    if directory_tail && key.file_name().is_some() {
        key.push("");
    }
    Ok(key.into_os_string())
}
fn missing_path(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
    )
}

fn check_message_defaults(message: &DescriptorProto) -> Result<(), DecodeError> {
    check_default_fields(&message.field)?;
    check_default_fields(&message.extension)?;
    for child in &message.nested_type {
        check_message_defaults(child)?;
    }
    Ok(())
}

fn check_default_fields(fields: &[FieldDescriptorProto]) -> Result<(), DecodeError> {
    for field in fields {
        if field.r#type == Some(Type::Bytes as i32) {
            if let Some(value) = &field.default_value {
                check_byte_default(value)?;
            }
        }
    }
    Ok(())
}

// Guard only the pinned loader's panicking operations, without normalizing source
// metadata. Stop where that loader returns an ordinary descriptor error, so an
// unreachable later escape does not change the error's public disposition.
// The pinned prost exposes only this deprecated constructor for our existing
// DecodeError return type; keep the allowance local until that API can migrate.
#[allow(deprecated)]
fn check_byte_default(value: &str) -> Result<(), DecodeError> {
    let bytes = value.as_bytes();
    let mut position = 0;
    while position < bytes.len() {
        if bytes[position] != b'\\' {
            position += 1;
            continue;
        }
        position += 1;
        let Some(&escape) = bytes.get(position) else {
            break;
        };
        match escape {
            b'0'..=b'7' => {
                let mut octal = 0u16;
                for _ in 0..3 {
                    match bytes.get(position) {
                        Some(digit @ b'0'..=b'7') => {
                            octal = octal * 8 + u16::from(digit - b'0');
                            position += 1;
                        }
                        _ => break,
                    }
                }
                if octal > u16::from(u8::MAX) {
                    return Err(DecodeError::new("byte default octal escape exceeds 255"));
                }
            }
            b'x' | b'X' => {
                if position + 3 > bytes.len() {
                    break;
                }
                let pair = value.get(position + 1..position + 3).ok_or_else(|| {
                    DecodeError::new("byte default hex escape splits a UTF-8 character")
                })?;
                if u8::from_str_radix(pair, 16).is_err() {
                    break;
                }
                position += 3;
            }
            b'a' | b'b' | b'f' | b'n' | b'r' | b't' | b'v' | b'\\' | b'?' | b'\'' | b'"' => {
                position += 1;
            }
            _ => break,
        }
    }
    Ok(())
}

fn missing_json_names(prefix: &str, message: &DescriptorProto, missing: &mut BTreeSet<String>) {
    let name = if prefix.is_empty() {
        message.name().to_owned()
    } else {
        format!("{prefix}.{}", message.name())
    };
    for field in message.field.iter().chain(&message.extension) {
        if field.json_name.is_none() {
            missing.insert(format!("{name}.{}", field.name()));
        }
    }
    for child in &message.nested_type {
        missing_json_names(&name, child, missing);
    }
}

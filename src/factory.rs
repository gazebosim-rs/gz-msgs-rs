// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Instance-owned message creation with lazy external descriptors.
use crate::{
    prost_reflect::{DynamicMessage, ReflectMessage},
    registry::DescriptorRegistry,
};
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    ffi::{OsStr, OsString},
    io,
    path::PathBuf,
    sync::{Arc, Mutex},
};
type Constructor = Arc<dyn Fn() -> DynamicMessage + Send + Sync>;
#[derive(Default)]
struct State {
    constructors: BTreeMap<String, Constructor>,
    paths: Option<OsString>,
    registry: Option<DescriptorRegistry>,
    initial_errors: Vec<(PathBuf, io::Error)>,
}
impl State {
    fn registry(&mut self) -> &mut DescriptorRegistry {
        if self.registry.is_none() {
            let mut registry = DescriptorRegistry::default();
            let paths = self
                .paths
                .clone()
                .or_else(|| std::env::var_os("GZ_DESCRIPTOR_PATH"))
                .unwrap_or_default();
            self.initial_errors = registry.load_paths(paths);
            self.registry = Some(registry);
        }
        self.registry
            .as_mut()
            .expect("initialized descriptor registry")
    }
}
/// Thread-safe local factory; no worker threads or process-global registration.
/// Generated-type lookup and registration do not read descriptor search paths.
#[derive(Default)]
pub struct MessageFactory {
    state: Mutex<State>,
}
impl MessageFactory {
    /// Use these explicit paths instead of consulting GZ_DESCRIPTOR_PATH.
    pub fn with_paths(paths: impl Into<OsString>) -> Self {
        Self {
            state: Mutex::new(State {
                paths: Some(paths.into()),
                ..Default::default()
            }),
        }
    }
    /// Replace a constructor. Names are stored literally, as in the native API.
    /// Constructors run outside the lock and may reenter the factory.
    pub fn register(
        &self,
        name: impl Into<String>,
        constructor: impl Fn() -> DynamicMessage + Send + Sync + 'static,
    ) {
        let name = name.into();
        let constructor: Constructor = Arc::new(constructor);
        let previous = self
            .state
            .lock()
            .expect("factory state")
            .constructors
            .insert(name, constructor);
        drop(previous);
    }
    pub fn new_message(&self, name: &str) -> Option<DynamicMessage> {
        let name = canonical_name(name);
        let constructor;
        {
            let mut state = self.state.lock().expect("factory state");
            if let Some(found) = state.constructors.get(name.as_ref()) {
                constructor = found.clone();
            } else {
                // Reflection accepts a leading dot; native Factory only accepts
                // the three explicit Gazebo aliases normalized above.
                if name.starts_with('.') {
                    state.registry();
                    return None;
                }
                return crate::descriptors()
                    .get_message_by_name(&name)
                    .or_else(|| state.registry().pool().get_message_by_name(&name))
                    .map(DynamicMessage::new);
            }
        }
        Some(constructor())
    }
    pub fn new_text(
        &self,
        name: &str,
        text: &str,
    ) -> Result<Option<DynamicMessage>, crate::prost_reflect::text_format::ParseError> {
        let Some(message) = self.new_message(name) else {
            return Ok(None);
        };
        DynamicMessage::parse_text_format(message.descriptor(), text).map(Some)
    }
    /// Enumerate the union of canonical, registered and external names, sorted.
    /// Enumeration triggers the first dynamic path lookup, matching native Types.
    pub fn types(&self) -> Vec<String> {
        let mut state = self.state.lock().expect("factory state");
        let mut names: BTreeSet<_> = state
            .registry()
            .pool()
            .all_messages()
            .map(|message| message.full_name().to_owned())
            .collect();
        names.extend(state.constructors.keys().cloned());
        names.into_iter().collect()
    }
    /// Source-aware registry definition; constructor callbacks are not invoked.
    pub fn registry_definition(
        &self,
        name: &str,
    ) -> Result<Option<String>, crate::DefinitionError> {
        let name = canonical_name(name);
        if name.starts_with('.') {
            return Ok(None);
        }
        if let Some(body) = crate::message_definition(&name) {
            return Ok(Some(body.to_owned()));
        }
        let registry = self.state.lock().expect("factory state").registry().clone();
        registry.definition(&name)
    }
    /// Load more paths and return pending initial-path errors plus these errors.
    pub fn load_paths(&self, paths: impl AsRef<OsStr>) -> Vec<(PathBuf, io::Error)> {
        let paths = paths.as_ref();
        let mut state = self.state.lock().expect("factory state");
        let explicit = state.registry().load_paths(paths);
        let mut errors = std::mem::take(&mut state.initial_errors);
        errors.extend(explicit);
        errors
    }
    /// Retrieve initial-path diagnostics without triggering initialization.
    pub fn take_initial_errors(&self) -> Vec<(PathBuf, io::Error)> {
        std::mem::take(&mut self.state.lock().expect("factory state").initial_errors)
    }
}
fn canonical_name(name: &str) -> Cow<'_, str> {
    for prefix in ["gz_msgs.", ".gz.msgs.", ".gz_msgs."] {
        if let Some(local) = name.strip_prefix(prefix) {
            return Cow::Owned(format!("gz.msgs.{local}"));
        }
    }
    Cow::Borrowed(name)
}

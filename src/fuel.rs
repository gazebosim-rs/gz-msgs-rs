// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2023 Open Source Robotics Foundation
//! Fuel model.config conversion; successful outputs replace values explicitly.
mod xml;
use crate::msgs::{
    fuel_metadata::{Contact, Dependency, Model, ResourceType, World},
    FuelMetadata, Version, VersionedName,
};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Xml,
    MissingName,
    MissingSdf,
    InvalidVersion,
    MissingUri,
    NotSdf,
}
fn space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\r' | '\n' | '\u{b}' | '\u{c}')
}
fn trim(s: &str) -> &str {
    s.trim_matches(space)
}
fn integer(s: &str) -> Result<i32, Error> {
    let s = s.trim_start_matches(space);
    let sign = usize::from(s.starts_with(['+', '-']));
    let end = sign + s[sign..].bytes().take_while(u8::is_ascii_digit).count();
    s[..end].parse().map_err(|_| Error::InvalidVersion)
}
// Native SemanticVersion accepts stoi prefixes, abbreviated components and even
// overwrites major for fourth/later numeric tokens. A strict semver parser cannot
// represent that compatibility rule; no general version ordering is implemented.
fn version(s: &str) -> Result<Version, Error> {
    let mut result = Version::default();
    let minus = s.find('-');
    let plus = s.find('+');
    if matches!((plus, minus), (Some(p), Some(m)) if p < m) {
        return Ok(result);
    }
    let end = minus.or(plus).unwrap_or(s.len());
    if let Some(m) = minus {
        result.prerelease = s[m + 1..plus.unwrap_or(s.len())].into();
    }
    if let Some(p) = plus {
        result.build = s[p + 1..].into();
    }
    let numeric = &s[..end];
    if !numeric.is_empty() {
        for (i, token) in numeric
            .strip_suffix('.')
            .unwrap_or(numeric)
            .split('.')
            .enumerate()
        {
            let number = integer(token)?;
            match i {
                1 => result.minor = number,
                2 => result.patch = number,
                _ => result.major = number,
            }
        }
    }
    Ok(result)
}
/// Parse native model/world metadata. C++ integer exceptions and null URI text
/// are explicit errors. Existing caller values are never mutated on failure.
pub fn from_xml(input: &str) -> Result<FuelMetadata, Error> {
    let root = xml::parse(input)?;
    let (top, model) = if let Some(top) = root.child("model") {
        (top, true)
    } else {
        (root.child("world").ok_or(Error::Xml)?, false)
    };
    let name = top
        .child("name")
        .and_then(xml::Node::text)
        .ok_or(Error::MissingName)?;
    let mut meta = FuelMetadata {
        name: trim(name).into(),
        ..FuelMetadata::default()
    };
    if let Some(text) = top.child("version").and_then(xml::Node::text) {
        meta.version = integer(trim(text))?;
    }
    if let Some(text) = top.child("description").and_then(xml::Node::text) {
        meta.description = trim(text).into();
    }
    for node in &top.children {
        match node.name.as_str() {
            "author" => meta.authors.push(Contact {
                name: node
                    .child("name")
                    .and_then(xml::Node::text)
                    .map(trim)
                    .unwrap_or("")
                    .into(),
                email: node
                    .child("email")
                    .and_then(xml::Node::text)
                    .map(trim)
                    .unwrap_or("")
                    .into(),
            }),
            "depend" => {
                if let Some(uri) = node.child("model").and_then(|n| n.child("uri")) {
                    meta.dependencies.push(Dependency {
                        uri: uri.text().ok_or(Error::MissingUri)?.into(),
                    });
                }
            }
            "sdf" => {
                if let (Some(text), Some(raw)) = (node.text(), node.attrs.get("version")) {
                    let ver = version(trim(raw))?;
                    // The native loop never updates maxVer: last positive wins.
                    if ver.major != 0 || ver.minor != 0 || ver.patch != 0 {
                        let file_format = Some(VersionedName {
                            name: "sdf".into(),
                            version: Some(ver),
                        });
                        let file = trim(text).to_owned();
                        meta.resource_type = Some(if model {
                            ResourceType::Model(Model { file, file_format })
                        } else {
                            ResourceType::World(World { file, file_format })
                        });
                    }
                }
            }
            _ => {}
        }
    }
    let file = match &meta.resource_type {
        Some(ResourceType::Model(m)) => &m.file,
        Some(ResourceType::World(w)) => &w.file,
        None => return Err(Error::MissingSdf),
    };
    if file.is_empty() {
        return Err(Error::MissingSdf);
    }
    Ok(meta)
}
/// Emit the exact legacy format, including its unescaped text and major.minor
/// version only. Callers requiring escaped XML must use a separate serializer.
pub fn to_xml(meta: &FuelMetadata) -> Result<String, Error> {
    let (tag, file, format) = match &meta.resource_type {
        Some(ResourceType::Model(m)) => ("model", &m.file, m.file_format.as_ref()),
        Some(ResourceType::World(w)) => ("world", &w.file, w.file_format.as_ref()),
        None => return Err(Error::NotSdf),
    };
    let format = format.filter(|f| f.name == "sdf").ok_or(Error::NotSdf)?;
    let version = format.version.clone().unwrap_or_default();
    let mut out = format!("<?xml version='1.0'?>\n  <{tag}>\n    <sdf version='{}.{}'>{file}</sdf>\n    <name>{}</name>\n    <version>{}</version>\n    <description>{}</description>\n", version.major, version.minor, meta.name, meta.version, meta.description);
    for author in &meta.authors {
        out.push_str(&format!(
            "    <author>\n      <name>{}</name>\n      <email>{}</email>\n    </author>\n",
            author.name, author.email
        ));
    }
    for dep in &meta.dependencies {
        out.push_str(&format!(
            "    <depend>\n      <model>\n        <uri>{}</uri>\n      </model>\n    </depend>\n",
            dep.uri
        ));
    }
    out.push_str(&format!("  </{tag}>\n"));
    Ok(out)
}

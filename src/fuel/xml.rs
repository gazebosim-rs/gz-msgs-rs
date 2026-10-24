// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use super::{space, Error};
use quick_xml::{events::Event, Reader};
use std::collections::BTreeMap;
#[derive(Default)]
pub(super) struct Node {
    pub name: String,
    pub attrs: BTreeMap<String, String>,
    pub children: Vec<Node>,
    text: Option<String>,
    blocked: bool,
    pending: String,
}
impl Node {
    pub fn child(&self, name: &str) -> Option<&Self> {
        self.children.iter().find(|n| n.name == name)
    }
    pub fn text(&self) -> Option<&str> {
        self.text
            .as_deref()
            .map(|s| s.split('\0').next().unwrap_or(""))
    }
    fn append(&mut self, text: &str, reference: bool) {
        if !self.blocked {
            if self.text.is_none() && !reference && text.chars().all(space) {
                self.pending.push_str(text);
            } else {
                self.text
                    .get_or_insert_default()
                    .push_str(&std::mem::take(&mut self.pending));
                self.text.get_or_insert_default().push_str(text);
            }
        }
    }
}
// TinyXML folds either CR/LF or LF/CR in one left-to-right pass; XML 1.0
// normalization cannot express the second rule or overlapping-pair precedence.
fn normalized(raw: &str) -> String {
    let mut input = raw.chars().peekable();
    let mut output = String::with_capacity(raw.len());
    while let Some(ch) = input.next() {
        if matches!(ch, '\r' | '\n') {
            if input
                .peek()
                .is_some_and(|&next| matches!((ch, next), ('\r', '\n') | ('\n', '\r')))
            {
                input.next();
            }
            output.push('\n');
        } else {
            output.push(ch);
        }
    }
    output
}
// Reader::from_str splits this valid UTF-8 input only at ASCII XML delimiters.
fn utf8(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("string-backed XML token is UTF-8")
}
// TinyXML's byte-name grammar permits colons and every non-ASCII byte.
fn name(raw: &[u8]) -> Result<&str, Error> {
    let start = |b: u8| b >= 128 || b.is_ascii_alphabetic() || matches!(b, b':' | b'_');
    if !raw.first().is_some_and(|&b| start(b))
        || !raw
            .iter()
            .skip(1)
            .all(|&b| start(b) || b.is_ascii_digit() || matches!(b, b'.' | b'-'))
    {
        return Err(Error::Xml);
    }
    Ok(utf8(raw))
}
// quick-xml rejects NUL references and malformed references; TinyXML keeps
// malformed spelling and permits NUL in its C-string view, including attributes.
fn reference(raw: &str) -> Result<String, Error> {
    if raw
        .strip_prefix("#x")
        .or_else(|| raw.strip_prefix('#'))
        .is_some_and(|digits| digits.bytes().all(|b| b == b'0'))
    {
        Ok("\0".into())
    } else if let Some(ch) = quick_xml::events::BytesRef::new(raw)
        .resolve_char_ref()
        .unwrap_or(None)
    {
        Ok(ch.to_string())
    } else {
        let numeric = raw
            .strip_prefix("#x")
            .is_some_and(|s| s.bytes().all(|b| b.is_ascii_hexdigit()))
            || raw
                .strip_prefix('#')
                .is_some_and(|s| s.bytes().all(|b| b.is_ascii_digit()));
        if numeric {
            return Err(Error::Xml);
        } // Native invalid UTF-8 has no Rust String value.
        Ok(quick_xml::escape::resolve_predefined_entity(raw)
            .map(str::to_string)
            .unwrap_or_else(|| format!("&{raw};")))
    }
}
fn attribute(raw: &str) -> Result<String, Error> {
    let mut value = String::new();
    for piece in normalized(raw).split_inclusive(';') {
        if let Some((before, entity)) = piece.strip_suffix(';').and_then(|s| s.rsplit_once('&')) {
            value.push_str(before);
            value.push_str(&reference(entity)?);
        } else {
            value.push_str(piece);
        }
    }
    value.truncate(value.find('\0').unwrap_or(value.len()));
    Ok(value)
}
pub(super) fn parse(input: &str) -> Result<Node, Error> {
    let input = input.split('\0').next().unwrap_or("");
    let mut reader = Reader::from_str(input);
    reader.config_mut().expand_empty_elements = true;
    reader.config_mut().allow_dangling_amp = true;
    let mut stack = vec![Node::default()];
    let mut declaration_allowed = true;
    let mut terminal_text = false;
    loop {
        let event = reader.read_event().map_err(|_| Error::Xml)?;
        if matches!(event, Event::Decl(_) | Event::PI(_)) {
            if !declaration_allowed {
                return Err(Error::Xml);
            }
            declaration_allowed = false;
        } else if !matches!(event, Event::Text(_) | Event::Eof) {
            declaration_allowed = false;
        }
        // Captured 496–499-level controls bind the pinned TinyXML2 depth limit.
        if matches!(event, Event::Start(_)) && stack.len() >= 499 {
            return Err(Error::Xml);
        }
        if stack.len() == 1 {
            terminal_text = match &event {
                Event::Text(text) => terminal_text || !utf8(text.as_ref()).chars().all(space),
                Event::GeneralRef(_) => true,
                Event::Eof => terminal_text,
                _ => false,
            };
        }
        let node = stack.last_mut().expect("virtual root is retained");
        match event {
            Event::Start(start) => {
                node.blocked = true;
                let mut next = Node {
                    name: name(start.name().as_ref())?.into(),
                    ..Node::default()
                };
                for attr in start.attributes() {
                    let attr = attr.map_err(|_| Error::Xml)?;
                    let key = name(attr.key.as_ref())?.to_owned();
                    let value = attribute(utf8(&attr.value))?;
                    next.attrs.insert(key, value);
                }
                stack.push(next);
            }
            Event::End(_) => {
                // The reader rejects unmatched ends before emitting this event.
                let finished = stack.pop().expect("checked end has an open element");
                stack
                    .last_mut()
                    .expect("checked end retains virtual root")
                    .children
                    .push(finished);
            }
            Event::Text(text) => node.append(&normalized(utf8(text.as_ref())), false),
            Event::GeneralRef(raw) => node.append(&reference(utf8(raw.as_ref()))?, true),
            Event::CData(text) => {
                if !node.blocked && node.text.is_none() {
                    node.text = Some(normalized(utf8(text.as_ref())));
                }
                node.blocked = true;
            }
            Event::DocType(_) => node.blocked = true,
            Event::Comment(_) => {
                node.blocked |= node.text.is_some();
                node.pending.clear();
            }
            Event::Eof => {
                return if stack.len() == 1 && !terminal_text {
                    stack.pop().ok_or(Error::Xml)
                } else {
                    Err(Error::Xml)
                }
            }
            _ => {}
        }
    }
}

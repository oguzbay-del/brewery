//! A small, safe XML reader (T2.26).
//!
//! The reader builds a tree of elements with these limits and rules:
//! - The input has a size limit ([`Limits::max_bytes`]), a depth limit and a limit for the number
//!   of elements. A "billion laughs" file cannot grow, because entities are not expanded.
//! - A `DOCTYPE` is an error. So no external entity, no external subset and no internal entity
//!   exists. Only the five predefined entities and numeric character references are accepted.
//! - Processing instructions and comments are skipped. Nothing is fetched from a network or a
//!   file.

use quick_xml::{Reader, events::Event};

use crate::{Error, Result};

/// Limits of one parse.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// Largest input, bytes.
    pub max_bytes: usize,
    /// Deepest element nesting.
    pub max_depth: usize,
    /// Most elements.
    pub max_elements: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_bytes: 8 * 1024 * 1024,
            max_depth: 32,
            max_elements: 200_000,
        }
    }
}

/// One element: its name, its text and its child elements.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Element {
    /// Element name (upper case names are kept as they are).
    pub name: String,
    /// The text directly inside the element, trimmed.
    pub text: String,
    /// Child elements in document order.
    pub children: Vec<Element>,
}

impl Element {
    /// The first child with this name.
    #[must_use]
    pub fn child(&self, name: &str) -> Option<&Element> {
        self.children.iter().find(|c| c.name == name)
    }

    /// All children with this name.
    pub fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Element> {
        self.children.iter().filter(move |c| c.name == name)
    }

    /// The trimmed text of a child; `None` when it is missing or empty.
    #[must_use]
    pub fn text_of(&self, name: &str) -> Option<&str> {
        self.child(name)
            .map(|c| c.text.as_str())
            .filter(|t| !t.is_empty())
    }

    /// The number in a child; `None` when it is missing or not a finite number.
    #[must_use]
    pub fn number_of(&self, name: &str) -> Option<f64> {
        self.text_of(name)
            .and_then(|t| t.split_whitespace().next())
            .and_then(|t| t.parse::<f64>().ok())
            .filter(|v| v.is_finite())
    }

    /// A boolean child (`TRUE`/`FALSE`, case blind).
    #[must_use]
    pub fn bool_of(&self, name: &str) -> Option<bool> {
        self.text_of(name).map(|t| t.eq_ignore_ascii_case("true"))
    }
}

fn syntax(e: impl std::fmt::Display) -> Error {
    Error::Xml(e.to_string())
}

/// Parses `input` into the root element.
pub fn parse(input: &[u8], limits: Limits) -> Result<Element> {
    if input.len() > limits.max_bytes {
        return Err(Error::TooLarge);
    }
    let text = std::str::from_utf8(input).map_err(|_| Error::Xml("input is not UTF-8".into()))?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut reader = Reader::from_str(text);
    let mut stack: Vec<Element> = Vec::new();
    let mut root: Option<Element> = None;
    let mut count = 0usize;
    loop {
        match reader.read_event().map_err(syntax)? {
            Event::DocType(_) => return Err(Error::DoctypeNotAllowed),
            Event::Start(e) => {
                count += 1;
                if count > limits.max_elements || stack.len() >= limits.max_depth {
                    return Err(Error::TooComplex);
                }
                if root.is_some() {
                    return Err(Error::Xml("more than one root element".into()));
                }
                stack.push(Element {
                    name: e.name().into_inner().to_owned(),
                    ..Element::default()
                });
            }
            Event::Empty(e) => {
                count += 1;
                if count > limits.max_elements {
                    return Err(Error::TooComplex);
                }
                let el = Element {
                    name: e.name().into_inner().to_owned(),
                    ..Element::default()
                };
                match stack.last_mut() {
                    Some(parent) => parent.children.push(el),
                    None if root.is_none() => root = Some(el),
                    None => return Err(Error::Xml("more than one root element".into())),
                }
            }
            Event::End(_) => {
                let Some(mut el) = stack.pop() else {
                    return Err(Error::Xml("unbalanced end tag".into()));
                };
                let trimmed = el.text.trim().to_owned();
                el.text = trimmed;
                match stack.last_mut() {
                    Some(parent) => parent.children.push(el),
                    None => root = Some(el),
                }
            }
            Event::Text(t) => {
                if let Some(top) = stack.last_mut() {
                    top.text.push_str(&t.xml10_content());
                }
            }
            Event::CData(c) => {
                if let Some(top) = stack.last_mut() {
                    top.text.push_str(&c.xml10_content());
                }
            }
            Event::GeneralRef(r) => {
                let Some(top) = stack.last_mut() else {
                    continue;
                };
                if let Some(ch) = r.resolve_char_ref().map_err(syntax)? {
                    top.text.push(ch);
                    continue;
                }
                let name = r.xml10_content();
                match name.as_ref() {
                    "amp" => top.text.push('&'),
                    "lt" => top.text.push('<'),
                    "gt" => top.text.push('>'),
                    "quot" => top.text.push('"'),
                    "apos" => top.text.push('\''),
                    _ => return Err(Error::EntityNotAllowed),
                }
            }
            Event::Eof => break,
            Event::Decl(_) | Event::PI(_) | Event::Comment(_) => {}
        }
    }
    if !stack.is_empty() {
        return Err(Error::Xml("unclosed element".into()));
    }
    root.ok_or_else(|| Error::Xml("no root element".into()))
}

/// Escapes text for an XML element body or attribute.
#[must_use]
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            // Control characters other than tab, LF and CR are not allowed in XML 1.0.
            c if (c as u32) < 0x20 && !matches!(c, '\t' | '\n' | '\r') => {}
            c => out.push(c),
        }
    }
    out
}

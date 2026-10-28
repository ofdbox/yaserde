//! Generic data structure serialization framework.
//!

use crate::context::{NamespaceFrame, SerializeContext};
use crate::{Error, Result, YaSerialize};
use std::io::{Cursor, Write};
use std::str;
use xml::writer::XmlEvent;
use xml::{EmitterConfig, EventWriter};

/// Serialize XML into a plain String with no formatting (EmitterConfig).
pub fn to_string<T: YaSerialize>(model: &T) -> Result<String> {
  let buf = Cursor::new(Vec::new());
  let cursor = serialize_with_writer(model, buf, &Config::default())?;
  let data = str::from_utf8(cursor.get_ref()).expect("Found invalid UTF-8");
  Ok(data.into())
}

/// Serialize XML into a plain String with control on formatting (via EmitterConfig parameters)
pub fn to_string_with_config<T: YaSerialize>(model: &T, config: &Config) -> Result<String> {
  let buf = Cursor::new(Vec::new());
  let cursor = serialize_with_writer(model, buf, config)?;
  let data = str::from_utf8(cursor.get_ref()).expect("Found invalid UTF-8");
  Ok(data.into())
}

pub fn serialize_with_writer<W: Write, T: YaSerialize>(
  model: &T,
  writer: W,
  config: &Config,
) -> Result<W> {
  let mut serializer = Serializer::new_from_writer(writer, config);
  match YaSerialize::serialize(model, &mut serializer) {
    Ok(()) => Ok(serializer.into_inner()),
    Err(msg) => Err(msg),
  }
}

pub fn to_string_content<T: YaSerialize>(model: &T) -> Result<String> {
  let buf = Cursor::new(Vec::new());
  let cursor = serialize_with_writer_content(model, buf)?;
  let data = str::from_utf8(cursor.get_ref()).expect("Found invalid UTF-8");
  Ok(data.into())
}

pub fn serialize_with_writer_content<W: Write, T: YaSerialize>(model: &T, writer: W) -> Result<W> {
  let mut serializer = Serializer::new_for_inner(writer);
  serializer.set_skip_start_end(true);
  match YaSerialize::serialize(model, &mut serializer) {
    Ok(()) => Ok(serializer.into_inner()),
    Err(msg) => Err(msg),
  }
}

/// Serialize a value as raw, unescaped text to be used as an attribute value.
/// The XML writer escapes attribute values itself, so escaping here would escape twice.
#[doc(hidden)]
pub fn to_string_attribute_value<T: YaSerialize>(model: &T) -> Result<String> {
  let mut config = EmitterConfig::new().write_document_declaration(false);
  config.perform_escaping = false;

  let mut serializer = Serializer::new(EventWriter::new_with_config(Vec::new(), config));
  serializer.set_skip_start_end(true);
  YaSerialize::serialize(model, &mut serializer)?;
  String::from_utf8(serializer.into_inner()).map_err(Error::custom)
}

pub struct Serializer<W: Write> {
  writer: EventWriter<W>,
  context: SerializeContext,
}

impl<W: Write> Serializer<W> {
  pub fn new(writer: EventWriter<W>) -> Self {
    Serializer {
      writer,
      context: SerializeContext::default(),
    }
  }

  pub fn new_from_writer(writer: W, config: &Config) -> Self {
    let mut emitter_config = EmitterConfig::new()
      .cdata_to_characters(false)
      .perform_indent(config.perform_indent)
      .write_document_declaration(config.write_document_declaration);

    if let Some(indent_string_value) = &config.indent_string {
      emitter_config = emitter_config.indent_string(indent_string_value.clone());
    }

    Self::new(EventWriter::new_with_config(writer, emitter_config))
  }

  pub fn new_for_inner(writer: W) -> Self {
    let config = EmitterConfig::new().write_document_declaration(false);

    Self::new(EventWriter::new_with_config(writer, config))
  }

  pub fn into_inner(self) -> W {
    self.writer.into_inner()
  }

  pub fn context(&self) -> &SerializeContext {
    &self.context
  }

  pub fn context_mut(&mut self) -> &mut SerializeContext {
    &mut self.context
  }

  pub fn push_namespaces(&mut self, declarations: Vec<(String, String)>) {
    self
      .context
      .push_namespaces(NamespaceFrame::new(declarations));
  }

  pub fn pop_namespaces(&mut self) {
    self.context.pop_namespaces();
  }

  pub fn prefix_for_uri(&self, uri: &str) -> Option<&str> {
    self.context.prefix_for_uri(uri)
  }

  pub fn element_name(
    &self,
    namespace: Option<&str>,
    local: &str,
    fallback: Option<&str>,
  ) -> String {
    let prefix = namespace
      .and_then(|uri| self.prefix_for_uri(uri))
      .or(fallback);
    match prefix {
      Some(prefix) if !prefix.is_empty() => format!("{prefix}:{local}"),
      _ => local.to_owned(),
    }
  }

  pub fn skip_start_end(&self) -> bool {
    self.context.skip_start_end()
  }

  pub fn set_skip_start_end(&mut self, state: bool) {
    self.context.set_skip_start_end(state);
  }

  pub fn get_start_event_name(&self) -> Option<String> {
    self.context.start_event_name().map(str::to_owned)
  }

  pub fn set_start_event_name(&mut self, name: Option<String>) {
    self.context.set_start_event_name(name);
  }

  pub fn write<'a, E>(&mut self, event: E) -> xml::writer::Result<()>
  where
    E: Into<XmlEvent<'a>>,
  {
    self.writer.write(event)
  }
}

pub struct Config {
  pub perform_indent: bool,
  pub write_document_declaration: bool,
  pub indent_string: Option<String>,
}

impl Default for Config {
  fn default() -> Self {
    Config {
      perform_indent: false,
      write_document_declaration: true,
      indent_string: None,
    }
  }
}

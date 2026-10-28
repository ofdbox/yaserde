//! Generic data structure deserialization framework.
//!

use crate::context::DeserializeContext;
use crate::{Error, Result, YaDeserialize};
use std::io::Read;
use xml::name::OwnedName;
use xml::reader::{EventReader, ParserConfig, XmlEvent};

pub fn from_str<T: YaDeserialize>(s: &str) -> Result<T> {
  from_reader(s.as_bytes())
}

pub fn from_reader<R: Read, T: YaDeserialize>(reader: R) -> Result<T> {
  <T as YaDeserialize>::deserialize(&mut Deserializer::new_from_reader(reader))
}

pub struct Deserializer<R: Read> {
  reader: EventReader<R>,
  context: DeserializeContext,
}

impl<R: Read> Deserializer<R> {
  pub fn new(reader: EventReader<R>) -> Self {
    Deserializer {
      reader,
      context: DeserializeContext::default(),
    }
  }

  pub fn new_from_reader(reader: R) -> Self {
    let config = ParserConfig::new()
      .trim_whitespace(true)
      .whitespace_to_characters(true)
      .cdata_to_characters(true)
      .ignore_comments(true)
      .coalesce_characters(true);

    Self::new(EventReader::new_with_config(reader, config))
  }

  pub fn peek(&mut self) -> Result<&XmlEvent> {
    if self.context.peeked.is_none() {
      self.context.peeked = Some(self.inner_next()?);
    }

    if let Some(ref next) = self.context.peeked {
      Ok(next)
    } else {
      Err(Error::custom("unable to peek next item"))
    }
  }

  pub fn inner_next(&mut self) -> Result<XmlEvent> {
    loop {
      match self.reader.next() {
        Ok(next) => {
          match next {
            XmlEvent::StartDocument { .. }
            | XmlEvent::ProcessingInstruction { .. }
            | XmlEvent::Comment(_) => { /* skip */ }
            other => return Ok(other),
          }
        }
        Err(msg) => {
          return Err(Error::Reader(msg));
        }
      }
    }
  }

  pub fn next_event(&mut self) -> Result<XmlEvent> {
    let next_event = if let Some(peeked) = self.context.peeked.take() {
      peeked
    } else {
      self.inner_next()?
    };
    match next_event {
      XmlEvent::StartElement { .. } => {
        self.context.depth += 1;
      }
      XmlEvent::EndElement { .. } => {
        self.context.depth -= 1;
      }
      _ => {}
    }
    log::debug!("Fetched {:?}, new depth {}", next_event, self.context.depth);
    Ok(next_event)
  }

  pub fn skip_element(&mut self, mut cb: impl FnMut(&XmlEvent)) -> Result<()> {
    let depth = self.context.depth;

    while self.context.depth >= depth {
      cb(&self.next_event()?);
    }

    Ok(())
  }

  pub fn context(&self) -> &DeserializeContext {
    &self.context
  }

  pub fn context_mut(&mut self) -> &mut DeserializeContext {
    &mut self.context
  }

  pub fn depth(&self) -> usize {
    self.context.depth
  }

  pub fn read_inner_value<T, F: FnOnce(&mut Self) -> Result<T>>(&mut self, f: F) -> Result<T> {
    match self.next_event()? {
      XmlEvent::StartElement { name, .. } => {
        let result = f(self)?;
        self.expect_end_element(&name)?;
        Ok(result)
      }
      event => Err(Error::custom(format!(
        "Internal error: expected start element, found {event:?}"
      ))),
    }
  }

  pub fn expect_end_element(&mut self, start_name: &OwnedName) -> Result<()> {
    match self.next_event()? {
      XmlEvent::EndElement { name, .. } if name == *start_name => Ok(()),
      XmlEvent::EndElement { name, .. } => Err(Error::custom(format!(
        "End tag </{}> didn't match the start tag <{}>",
        name.local_name, start_name.local_name
      ))),
      XmlEvent::StartElement { name, .. } => Err(Error::custom(format!(
        "Expected end tag </{}>, found start tag <{}>",
        start_name.local_name, name.local_name
      ))),
      event => Err(Error::custom(format!(
        "Expected end tag </{}>, found {:?}",
        start_name.local_name, event
      ))),
    }
  }
}

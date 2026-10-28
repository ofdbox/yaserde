//! Serialization and deserialization session state.
//!
//! I/O stays in `Serializer` / `Deserializer`; mutable session state lives here.

use xml::reader::XmlEvent;

/// State shared by one serialization run.
#[derive(Debug, Default)]
pub struct SerializeContext {
  pub(crate) skip_start_end: bool,
  pub(crate) start_event_name: Option<String>,
}

impl SerializeContext {
  /// Whether the current value is serialized as inner content.
  pub fn skip_start_end(&self) -> bool {
    self.skip_start_end
  }

  /// Enable or disable inner-content mode.
  pub fn set_skip_start_end(&mut self, state: bool) {
    self.skip_start_end = state;
  }

  /// Element name supplied by the parent serializer.
  pub fn start_event_name(&self) -> Option<&str> {
    self.start_event_name.as_deref()
  }

  /// Set the element name supplied by the parent serializer.
  pub fn set_start_event_name(&mut self, name: Option<String>) {
    self.start_event_name = name;
  }
}

/// State shared by one deserialization run.
#[derive(Debug, Default)]
pub struct DeserializeContext {
  pub(crate) depth: usize,
  pub(crate) peeked: Option<XmlEvent>,
}

impl DeserializeContext {
  /// Current XML nesting depth.
  pub fn depth(&self) -> usize {
    self.depth
  }
}

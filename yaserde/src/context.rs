//! Serialization and deserialization session state.
//!
//! I/O stays in `Serializer` / `Deserializer`; mutable session state lives here.

use xml::reader::XmlEvent;

/// State shared by one serialization run.
#[derive(Debug, Default)]
pub struct SerializeContext {
  pub(crate) skip_start_end: bool,
  pub(crate) start_event_name: Option<String>,
  pub(crate) namespaces: NamespaceStack,
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

  /// Push a namespace frame.
  pub fn push_namespaces(&mut self, frame: NamespaceFrame) {
    self.namespaces.push(frame);
  }

  /// Pop the most recent namespace frame.
  pub fn pop_namespaces(&mut self) {
    self.namespaces.pop();
  }

  /// Resolve the prefix for a namespace URI.
  pub fn prefix_for_uri(&self, uri: &str) -> Option<&str> {
    self.namespaces.prefix_for_uri(uri)
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

/// One namespace scope introduced by an element.
#[derive(Clone, Debug, Default)]
pub struct NamespaceFrame {
  declarations: std::collections::BTreeMap<String, String>,
}

impl NamespaceFrame {
  /// Build a frame from `(prefix, URI)` declarations.
  pub fn new(declarations: impl IntoIterator<Item = (String, String)>) -> Self {
    Self {
      declarations: declarations.into_iter().collect(),
    }
  }

  /// Whether this frame declares no namespaces.
  pub fn is_empty(&self) -> bool {
    self.declarations.is_empty()
  }
}

/// Inherited namespace declarations for serialization.
#[derive(Clone, Debug, Default)]
pub struct NamespaceStack {
  frames: Vec<NamespaceFrame>,
}

impl NamespaceStack {
  /// Push a frame; empty frames are ignored.
  pub fn push(&mut self, frame: NamespaceFrame) {
    if !frame.is_empty() {
      self.frames.push(frame);
    }
  }

  /// Pop the most recent frame.
  pub fn pop(&mut self) {
    self.frames.pop();
  }

  /// Resolve a prefix in the current scope.
  pub fn resolve_prefix(&self, prefix: &str) -> Option<&str> {
    self
      .frames
      .iter()
      .rev()
      .find_map(|frame| frame.declarations.get(prefix).map(String::as_str))
  }

  /// Find the prefix bound to a namespace URI in the current scope.
  pub fn prefix_for_uri(&self, uri: &str) -> Option<&str> {
    self.frames.iter().rev().find_map(|frame| {
      frame
        .declarations
        .iter()
        .find_map(|(prefix, value)| (value == uri).then_some(prefix.as_str()))
    })
  }
}

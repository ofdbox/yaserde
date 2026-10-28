//! # YaSerDe
//!
//! YaSerDe is a framework for ***ser***ializing and ***de***serializing Rust data
//! structures efficiently and generically from and into XML.
//!
//! Enable the `derive` feature to derive [`YaSerialize`] and [`YaDeserialize`]:
//!
//! ```toml
//! [dependencies]
//! yaserde = { version = "0.12", features = ["derive"] }
//! ```
//!
//! # Serialize and deserialize
//!
//! For instance, let's say that one wants to generate an XML file for the
//! [Rust-Embedded community](https://github.com/rust-embedded/). A well known XML
//! file for microcontrollers is called [SVD](https://github.com/rust-embedded/svd/)
//! and part of it can be described with YaSerDe like so:
//!
//! ```rust
//! # #[cfg(feature = "derive")] {
//! use yaserde::{YaDeserialize, YaSerialize};
//!
//! #[derive(Debug, PartialEq, YaSerialize, YaDeserialize)]
//! #[yaserde(rename = "device")]
//! struct Device {
//!   #[yaserde(attribute = true)]
//!   schemaversion: String,
//!   vendor: String,
//!   name: String,
//!   cpu: Cpu,
//! }
//!
//! #[derive(Debug, PartialEq, YaSerialize, YaDeserialize)]
//! struct Cpu {
//!   name: String,
//!   revision: String,
//! }
//!
//! let device = Device {
//!   schemaversion: "1.3".to_string(),
//!   vendor: "STMicroelectronics".to_string(),
//!   name: "STM32F103".to_string(),
//!   cpu: Cpu {
//!     name: "CM3".to_string(),
//!     revision: "r1p1".to_string(),
//!   },
//! };
//!
//! let xml = yaserde::ser::to_string(&device).unwrap();
//! assert_eq!(
//!   xml,
//!   r#"<?xml version="1.0" encoding="UTF-8"?><device schemaversion="1.3"><vendor>STMicroelectronics</vendor><name>STM32F103</name><cpu><name>CM3</name><revision>r1p1</revision></cpu></device>"#
//! );
//!
//! let loaded: Device = yaserde::de::from_str(&xml).unwrap();
//! assert_eq!(loaded, device);
//! # }
//! ```
//!
//! The `#[yaserde(...)]` attributes control the XML representation: `schemaversion` is written
//! as an XML attribute of `<device>`, while the other fields are written as child elements.
//! A field can itself be a struct (here `cpu`), so several structs can be composed to describe
//! the whole XML document. See the `examples` directory for more complete examples.
//!
//! # Pretty printing
//!
//! To get an indented XML output, use [`ser::to_string_with_config`]:
//!
//! ```rust
//! # #[cfg(feature = "derive")] {
//! # use yaserde::YaSerialize;
//! # #[derive(YaSerialize)]
//! # #[yaserde(rename = "device")]
//! # struct Device {
//! #   name: String,
//! # }
//! # let device = Device { name: "STM32F103".to_string() };
//! let config = yaserde::ser::Config {
//!   perform_indent: true,
//!   ..Default::default()
//! };
//!
//! let xml = yaserde::ser::to_string_with_config(&device, &config).unwrap();
//! assert_eq!(
//!   xml,
//!   "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<device>\n  <name>STM32F103</name>\n</device>"
//! );
//! # }
//! ```

#[cfg(feature = "yaserde_derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate yaserde_derive;

#[cfg(feature = "yaserde_derive")]
#[doc(hidden)]
pub use yaserde_derive::*;

use std::io::{Read, Write};
use xml::writer::XmlEvent;

pub mod context;
pub mod de;
mod error;
pub mod primitives;
pub mod ser;

pub use context::{DeserializeContext, SerializeContext};
pub use error::{Error, Result};

/// A **data structure** that can be deserialized from any data format supported by YaSerDe.
pub trait YaDeserialize: Sized {
  fn deserialize<R: Read>(reader: &mut de::Deserializer<R>) -> Result<Self>;
}

/// A **data structure** that can be serialized into any data format supported by YaSerDe.
pub trait YaSerialize: Sized {
  fn serialize<W: Write>(&self, writer: &mut ser::Serializer<W>) -> Result<()>;

  fn serialize_attributes(
    &self,
    attributes: Vec<xml::attribute::OwnedAttribute>,
    namespace: xml::namespace::Namespace,
  ) -> Result<(
    Vec<xml::attribute::OwnedAttribute>,
    xml::namespace::Namespace,
  )>;
}

/// A **visitor** that can be implemented to retrieve information from source file.
pub trait Visitor<'de>: Sized {
  /// The value produced by this visitor.
  type Value;

  fn visit_bool(self, v: &str) -> Result<Self::Value> {
    Err(Error::custom(format!("Unexpected bool {:?}", v)))
  }

  fn visit_i8(self, v: &str) -> Result<Self::Value> {
    Err(Error::custom(format!("Unexpected i8 {:?}", v)))
  }

  fn visit_u8(self, v: &str) -> Result<Self::Value> {
    Err(Error::custom(format!("Unexpected u8 {:?}", v)))
  }

  fn visit_i16(self, v: &str) -> Result<Self::Value> {
    Err(Error::custom(format!("Unexpected i16 {:?}", v)))
  }

  fn visit_u16(self, v: &str) -> Result<Self::Value> {
    Err(Error::custom(format!("Unexpected u16 {:?}", v)))
  }

  fn visit_i32(self, v: &str) -> Result<Self::Value> {
    Err(Error::custom(format!("Unexpected i32 {:?}", v)))
  }

  fn visit_u32(self, v: &str) -> Result<Self::Value> {
    Err(Error::custom(format!("Unexpected u32 {:?}", v)))
  }

  fn visit_i64(self, v: &str) -> Result<Self::Value> {
    Err(Error::custom(format!("Unexpected i64 {:?}", v)))
  }

  fn visit_u64(self, v: &str) -> Result<Self::Value> {
    Err(Error::custom(format!("Unexpected u64 {:?}", v)))
  }

  fn visit_f32(self, v: &str) -> Result<Self::Value> {
    Err(Error::custom(format!("Unexpected f32 {:?}", v)))
  }

  fn visit_f64(self, v: &str) -> Result<Self::Value> {
    Err(Error::custom(format!("Unexpected f64 {:?}", v)))
  }

  fn visit_str(self, v: &str) -> Result<Self::Value> {
    Err(Error::custom(format!("Unexpected str {:?}", v)))
  }
}

macro_rules! serialize_type {
  ($type:ty) => {
    impl YaSerialize for $type {
      fn serialize<W: Write>(&self, writer: &mut ser::Serializer<W>) -> Result<()> {
        let content = format!("{}", self);
        let event = XmlEvent::characters(&content);
        writer.write(event)?;
        Ok(())
      }

      fn serialize_attributes(
        &self,
        attributes: Vec<xml::attribute::OwnedAttribute>,
        namespace: xml::namespace::Namespace,
      ) -> Result<(
        Vec<xml::attribute::OwnedAttribute>,
        xml::namespace::Namespace,
      )> {
        Ok((attributes, namespace))
      }
    }
  };
}

serialize_type!(bool);
serialize_type!(char);

serialize_type!(usize);
serialize_type!(u8);
serialize_type!(u16);
serialize_type!(u32);
serialize_type!(u64);

serialize_type!(isize);
serialize_type!(i8);
serialize_type!(i16);
serialize_type!(i32);
serialize_type!(i64);

serialize_type!(f32);
serialize_type!(f64);

/// Re-export for use in yaserde_derive
#[doc(hidden)]
pub use xml as __xml;

/// Re-export for use in yaserde_derive
#[doc(hidden)]
pub use log as __log;

/// Used in code generated by yaserde_derive for logging
#[macro_export]
#[doc(hidden)]
macro_rules! __derive_debug {
  ($($arg:tt)+) => { ::yaserde::__log::debug!(target: "yaserde_derive", $($arg)+) };
}

/// Used in code generated by yaserde_derive for logging
#[macro_export]
#[doc(hidden)]
macro_rules! __derive_trace {
  ($($arg:tt)+) => { ::yaserde::__log::trace!(target: "yaserde_derive", $($arg)+) };
}

#[test]
fn default_visitor() {
  struct Test;
  impl<'de> Visitor<'de> for Test {
    type Value = u8;
  }

  macro_rules! test_type {
    ($visitor:tt, $message:expr) => {{
      let t = Test {};
      assert_eq!(
        t.$visitor("").map_err(|e| e.to_string()),
        Err($message.to_string())
      );
    }};
  }

  test_type!(visit_bool, "Unexpected bool \"\"");
  test_type!(visit_i8, "Unexpected i8 \"\"");
  test_type!(visit_u8, "Unexpected u8 \"\"");
  test_type!(visit_i16, "Unexpected i16 \"\"");
  test_type!(visit_u16, "Unexpected u16 \"\"");
  test_type!(visit_i32, "Unexpected i32 \"\"");
  test_type!(visit_u32, "Unexpected u32 \"\"");
  test_type!(visit_i64, "Unexpected i64 \"\"");
  test_type!(visit_u64, "Unexpected u64 \"\"");
  test_type!(visit_str, "Unexpected str \"\"");
}

#[doc(hidden)]
#[macro_export]
macro_rules! test_for_type {
  ($type:ty, $value:expr, $content:expr) => {{
    #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
    #[yaserde(rename = "data")]
    pub struct Data {
      item: $type,
    }

    let model = Data { item: $value };

    let content = if let Some(str_value) = $content {
      let str_value: &str = str_value;
      format!("<data><item>{}</item></data>", str_value)
    } else {
      "<data />".to_owned()
    };

    serialize_and_validate!(model, content);
    deserialize_and_validate!(&content, model, Data);
  }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! test_for_attribute_type {
  ($type: ty, $value: expr, $content: expr) => {{
    #[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
    #[yaserde(rename = "data")]
    pub struct Data {
      #[yaserde(attribute = true)]
      item: $type,
    }
    let model = Data { item: $value };

    let content = if let Some(str_value) = $content {
      "<data item=\"".to_string() + str_value + "\" />"
    } else {
      "<data />".to_string()
    };

    serialize_and_validate!(model, content);
    deserialize_and_validate!(&content, model, Data);
  }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! deserialize_and_validate {
  ($content: expr, $model: expr, $struct: tt) => {
    log::debug!("deserialize_and_validate @ {}:{}", file!(), line!());
    let loaded: Result<$struct, String> =
      yaserde::de::from_str($content).map_err(|e: yaserde::Error| e.to_string());
    assert_eq!(loaded, Ok($model));
  };
}

#[doc(hidden)]
#[macro_export]
macro_rules! serialize_and_validate {
  ($model: expr, $content: expr) => {
    log::debug!("serialize_and_validate @ {}:{}", file!(), line!());
    let data: Result<String, String> =
      yaserde::ser::to_string(&$model).map_err(|e: yaserde::Error| e.to_string());

    let content = &format!(r#"<?xml version="1.0" encoding="UTF-8"?>{}"#, $content);
    assert_eq!(
      data,
      Ok(content.split("\n").map(|s| s.trim()).collect::<String>())
    );
  };
}

#[macro_use]
extern crate ofdbox_yaserde_derive;

use ofdbox_yaserde::de::from_str;

fn init() {
  let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn de_no_content() {
  init();

  #[derive(YaDeserialize, PartialEq, Debug)]
  #[yaserde(rename = "book")]
  pub struct Book {
    author: String,
    title: String,
  }

  let content = "";
  let error = from_str::<Book>(content).unwrap_err();
  assert!(matches!(error, ofdbox_yaserde::Error::Reader(_)));
  assert_eq!(
    error.to_string(),
    "1:1 Unexpected end of stream: no root element found"
  );
}

#[test]
fn de_wrong_end_balise() {
  init();

  #[derive(YaDeserialize, PartialEq, Debug)]
  #[yaserde(rename = "book")]
  pub struct Book {
    author: String,
    title: String,
  }

  let content = "<book><author>Antoine de Saint-Exupéry<title>Little prince</title></book>";
  let loaded: Result<Book, String> = from_str(content).map_err(|e| e.to_string());
  assert_eq!(
    loaded,
    Err("author: Expected end tag </author>, found start tag <title>".to_owned())
  );
}

#[test]
fn de_invalid_value() {
  init();

  #[derive(YaDeserialize, PartialEq, Debug)]
  #[yaserde(rename = "data")]
  pub struct Data {
    required: Option<u32>,
  }

  let error = from_str::<Data>("<data><required>abc</required></data>").unwrap_err();
  assert!(matches!(&error, ofdbox_yaserde::Error::Field { name, .. } if name == "required"));
  assert_eq!(error.to_string(), "required: invalid digit found in string");
}

#[test]
fn de_invalid_text_content() {
  init();

  #[derive(YaDeserialize, PartialEq, Debug)]
  #[yaserde(rename = "data")]
  pub struct Data {
    #[yaserde(text = true)]
    value: u32,
  }

  let loaded: Result<Data, String> = from_str("<data>abc</data>").map_err(|e| e.to_string());
  assert_eq!(loaded, Err("invalid digit found in string".to_owned()));
}

#[test]
fn de_invalid_enum_value() {
  init();

  #[derive(YaDeserialize, PartialEq, Debug, Default)]
  #[yaserde(rename = "data")]
  pub enum Data {
    #[default]
    Empty,
    Value(u32),
  }

  #[derive(YaDeserialize, PartialEq, Debug)]
  #[yaserde(rename = "base")]
  pub struct Base {
    data: Data,
  }

  let loaded: Result<Base, String> =
    from_str("<base><data><Value>abc</Value></data></base>").map_err(|e| e.to_string());
  assert_eq!(loaded, Err("invalid digit found in string".to_owned()));
}

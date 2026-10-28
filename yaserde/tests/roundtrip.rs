#[macro_use]
extern crate ofdbox_yaserde_derive;

use ofdbox_yaserde::de::from_str;
use ofdbox_yaserde::ser::to_string;

fn init() {
  let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn empty_string() {
  init();

  #[derive(YaDeserialize, YaSerialize, PartialEq, Debug)]
  #[yaserde(rename = "base")]
  pub struct XmlStruct {
    item: String,
    optional: Option<String>,
  }

  let model = XmlStruct {
    item: String::new(),
    optional: Some(String::new()),
  };

  let content = to_string(&model).unwrap();
  assert_eq!(
    content,
    r#"<?xml version="1.0" encoding="UTF-8"?><base><item></item><optional></optional></base>"#
  );
  assert_eq!(from_str(&content).map_err(|e| e.to_string()), Ok(model));

  let loaded: Result<XmlStruct, String> =
    from_str("<base><item/></base>").map_err(|e| e.to_string());
  assert_eq!(
    loaded,
    Ok(XmlStruct {
      item: String::new(),
      optional: None,
    })
  );
}

#[test]
fn numeric_text_content() {
  init();

  #[derive(YaDeserialize, YaSerialize, PartialEq, Debug)]
  #[yaserde(rename = "base")]
  pub struct XmlStruct {
    #[yaserde(attribute = true)]
    unit: String,
    #[yaserde(text = true)]
    value: u32,
  }

  let model = XmlStruct {
    unit: "ms".to_string(),
    value: 42,
  };

  let content = to_string(&model).unwrap();
  assert_eq!(
    content,
    r#"<?xml version="1.0" encoding="UTF-8"?><base unit="ms">42</base>"#
  );
  assert_eq!(from_str(&content).map_err(|e| e.to_string()), Ok(model));
}

#[test]
fn struct_attribute_escaping() {
  init();

  #[derive(YaDeserialize, YaSerialize, PartialEq, Debug, Default)]
  pub struct Inner {
    #[yaserde(text = true)]
    text: String,
  }

  #[derive(YaDeserialize, YaSerialize, PartialEq, Debug)]
  #[yaserde(rename = "base")]
  pub struct XmlStruct {
    #[yaserde(attribute = true)]
    single: Inner,
    #[yaserde(attribute = true)]
    optional: Option<Inner>,
    #[yaserde(attribute = true)]
    list: Vec<Inner>,
  }

  let model = XmlStruct {
    single: Inner {
      text: "a<b&c".to_string(),
    },
    optional: Some(Inner {
      text: "\"quoted\"".to_string(),
    }),
    list: vec![
      Inner {
        text: "x&y".to_string(),
      },
      Inner {
        text: "z>".to_string(),
      },
    ],
  };

  // Attribute values are escaped exactly once
  let content = to_string(&model).unwrap();
  assert_eq!(
    content,
    r#"<?xml version="1.0" encoding="UTF-8"?><base single="a&lt;b&amp;c" optional="&quot;quoted&quot;" list="x&amp;y z&gt;" />"#
  );
  assert_eq!(from_str(&content).map_err(|e| e.to_string()), Ok(model));
}

#[test]
fn multiple_yaserde_attributes() {
  init();

  #[derive(YaDeserialize, YaSerialize, PartialEq, Debug)]
  #[yaserde(rename = "base")]
  pub struct XmlStruct {
    #[yaserde(attribute = true)]
    #[yaserde(rename = "renamed")]
    item: String,
  }

  let model = XmlStruct {
    item: "value".to_string(),
  };

  let content = to_string(&model).unwrap();
  assert_eq!(
    content,
    r#"<?xml version="1.0" encoding="UTF-8"?><base renamed="value" />"#
  );
  assert_eq!(from_str(&content).map_err(|e| e.to_string()), Ok(model));
}

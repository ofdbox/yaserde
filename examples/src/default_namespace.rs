#![allow(dead_code)]
use ofdbox_yaserde::YaDeserialize;

#[derive(YaDeserialize, Debug, PartialEq)]
#[yaserde(rename = "EBMLSchema", prefix = "a", namespaces = {"a" = "some_namespace"})]
struct Schema {
  #[yaserde(rename = "element", prefix = "a")]
  elements: Vec<Element>,
}

#[derive(YaDeserialize, Debug, PartialEq)]
#[yaserde(rename = "element", prefix = "a", namespaces = {"a" = "some_namespace"})]
struct Element {
  #[yaserde(rename = "documentation", prefix = "a")]
  documentation: Vec<Documentation>,
}

#[derive(YaDeserialize, Debug, PartialEq)]
#[yaserde(rename = "documentation", prefix = "a", namespaces = {"a" = "some_namespace"})]
struct Documentation {
  #[yaserde(text = true)]
  body: String,
}

#[test]
fn nested_element_equality() {
  let parent = r#"
    <EBMLSchema xmlns="some_namespace">
      <element>
        <documentation>Test</documentation>
      </element>
    </EBMLSchema>
  "#;

  let parent: Schema = ofdbox_yaserde::de::from_str(parent).unwrap();

  let child = r#"
    <element xmlns="some_namespace">
      <documentation>Test</documentation>
    </element>
  "#;

  let child: Element = ofdbox_yaserde::de::from_str(child).unwrap();

  println!("{:#?}", child);
  println!("{:#?}", parent);

  assert_eq!(parent.elements[0], child);
}

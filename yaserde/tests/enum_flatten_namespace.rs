use ofdbox_yaserde::{YaDeserialize, YaSerialize};

#[derive(Debug, Default, PartialEq, YaSerialize, YaDeserialize)]
#[yaserde(rename = "root", namespace = "http://www.ofdspec.org/2016", xmlns = { "ns" = "http://www.ofdspec.org/2016" })]
struct Root {
  #[yaserde(rename = "object")]
  object: Obj,
}

#[derive(Debug, PartialEq, YaSerialize, YaDeserialize)]
#[yaserde(rename = "object")]
enum Obj {
  #[yaserde(rename = "PathObject")]
  Path(PathObject),
}

impl Default for Obj {
  fn default() -> Self {
    Self::Path(PathObject::default())
  }
}

#[derive(Debug, Default, PartialEq, YaSerialize, YaDeserialize)]
#[yaserde(rename = "PathObject")]
struct PathObject {
  common: Common,
  #[yaserde(rename = "StrokeColor")]
  stroke_color: Option<Color>,
}

#[derive(Debug, Default, PartialEq, YaSerialize, YaDeserialize)]
struct Common {
  #[yaserde(attribute = true, rename = "ID")]
  id: String,
}

#[derive(Debug, Default, PartialEq, YaSerialize, YaDeserialize)]
#[yaserde(rename = "Color")]
struct Color {
  #[yaserde(flatten = true)]
  kind: ColorKind,
}

#[derive(Debug, Default, PartialEq, YaSerialize, YaDeserialize)]
enum ColorKind {
  #[default]
  #[yaserde(rename = "AxialShd")]
  Axial,
}

#[test]
fn flattened_enum_variant_inherits_namespace() {
  let model = Root {
    object: Obj::Path(PathObject {
      common: Common { id: "1".to_owned() },
      stroke_color: Some(Color {
        kind: ColorKind::Axial,
      }),
    }),
  };
  let xml = ofdbox_yaserde::ser::to_string(&model).unwrap();
  let parsed: Result<Root, _> = ofdbox_yaserde::de::from_str(&xml);
  assert_eq!(parsed.map_err(|e| e.to_string()), Ok(model));
}

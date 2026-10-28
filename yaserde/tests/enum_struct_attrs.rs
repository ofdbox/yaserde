#[macro_use]
extern crate yaserde_derive;

#[derive(Debug, Default, PartialEq, YaSerialize, YaDeserialize)]
#[yaserde(rename = "root", namespace = "http://www.ofdspec.org/2016", xmlns = { "ns" = "http://www.ofdspec.org/2016" })]
struct Root {
  #[yaserde(rename = "color")]
  color: Color,
}

#[derive(Debug, Default, PartialEq, YaSerialize, YaDeserialize)]
struct Color {
  #[yaserde(flatten = true)]
  kind: ColorKind,
}

#[derive(Debug, PartialEq, YaSerialize, YaDeserialize)]
enum ColorKind {
  #[yaserde(rename = "AxialShd")]
  Axial(Axial),
}

impl Default for ColorKind {
  fn default() -> Self {
    Self::Axial(Axial::default())
  }
}

#[derive(Debug, Default, PartialEq, YaSerialize, YaDeserialize)]
#[yaserde(rename = "AxialShd")]
struct Axial {
  #[yaserde(attribute = true, rename = "Value")]
  value: String,
}

#[test]
fn enum_struct_variant_writes_attributes() {
  let model = Root {
    color: Color {
      kind: ColorKind::Axial(Axial {
        value: "red".to_owned(),
      }),
    },
  };
  let xml = yaserde::ser::to_string(&model).unwrap();
  println!("{xml}");
  assert!(xml.contains("Value=\"red\""), "{xml}");
  let parsed: Result<Root, _> = yaserde::de::from_str(&xml);
  assert_eq!(parsed.map_err(|e| e.to_string()), Ok(model));
}

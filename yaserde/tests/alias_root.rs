#[macro_use]
extern crate ofdbox_yaserde_derive;

#[derive(Debug, Default, PartialEq, YaSerialize, YaDeserialize)]
#[yaserde(
    rename = "Color",
    namespace = "http://www.ofdspec.org/2016",
    xmlns = { "ofd" = "http://www.ofdspec.org/2016" },
    namespace_aliases = { "http://www.ofdspec.org" = "http://www.ofdspec.org/2016" }
)]
struct Color {
  #[yaserde(attribute = true)]
  value: String,
}

#[test]
fn root_alias() {
  let parsed: Color =
    ofdbox_yaserde::de::from_str(r#"<ofd:Color xmlns:ofd="http://www.ofdspec.org" value="1"/>"#)
      .unwrap();
  assert_eq!(parsed.value, "1");
}

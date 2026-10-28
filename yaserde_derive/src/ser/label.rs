use crate::common::YaSerdeAttribute;
use proc_macro2::Ident;

pub fn build_label_name(
  label: &Ident,
  field_attrs: &YaSerdeAttribute,
  root_attributes: &YaSerdeAttribute,
) -> String {
  let prefix = if let Some(namespace) = &field_attrs.namespace {
    root_attributes
      .namespaces
      .iter()
      .find_map(|(prefix, uri)| (uri == namespace).then(|| prefix.clone()))
  } else {
    field_attrs.prefix.clone()
  };

  let prefix = if root_attributes.default_namespace == prefix {
    "".to_string()
  } else {
    prefix.map_or("".to_string(), |prefix| prefix + ":")
  };

  let label = field_attrs
    .rename
    .clone()
    .unwrap_or_else(|| label.to_string());

  format!("{}{}", prefix, label)
}

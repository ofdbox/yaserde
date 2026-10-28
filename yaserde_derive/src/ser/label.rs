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

pub fn build_label_expr(
  label: &Ident,
  field_attrs: &YaSerdeAttribute,
  root_attributes: &YaSerdeAttribute,
) -> proc_macro2::TokenStream {
  use quote::quote;

  let local = field_attrs
    .rename
    .clone()
    .unwrap_or_else(|| label.to_string());

  if let Some(namespace) = &field_attrs.namespace {
    let canonical = root_attributes
      .namespace_aliases
      .get(namespace)
      .map_or(namespace, |value| value);
    let prefix = root_attributes
      .namespaces
      .iter()
      .find_map(|(prefix, uri)| (uri == canonical).then(|| prefix.clone()));
    let fallback = if root_attributes.default_namespace == prefix {
      quote!(::std::option::Option::None)
    } else {
      match prefix {
        Some(prefix) => quote!(::std::option::Option::Some(#prefix)),
        None => quote!(::std::option::Option::None),
      }
    };
    return quote!(writer.element_name(
      ::std::option::Option::Some(#canonical),
      #local,
      #fallback,
    ));
  }

  if field_attrs.prefix.is_some() {
    let name = build_label_name(label, field_attrs, root_attributes);
    return quote!((#name).to_string());
  }

  quote!(writer.element_name(
    ::std::option::Option::None,
    #local,
    ::std::option::Option::None,
  ))
}

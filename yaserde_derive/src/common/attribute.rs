use proc_macro2::{Ident, TokenStream};
use quote::quote;
use serde::Deserialize;
use serde_tokenstream::from_tokenstream;
use std::collections::BTreeMap;
use syn::Attribute;

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct YaSerdeAttribute {
  /// Set this field as an XML attribute
  #[serde(default)]
  pub attribute: bool,
  /// Set default callback function
  #[serde(default)]
  pub default: Option<String>,
  /// Set the default namespace
  #[serde(default)]
  pub default_namespace: Option<String>,
  /// Flatten child fields
  #[serde(default)]
  pub flatten: bool,
  /// Namespace URI of the current element.
  #[serde(default)]
  pub namespace: Option<String>,
  /// Preferred namespace declarations keyed by prefix.
  #[serde(default)]
  pub xmlns: BTreeMap<String, String>,
  /// Default namespace URI for this element and its descendants.
  #[serde(default)]
  pub xmlns_default: Option<String>,
  /// Accepted namespace URI aliases, keyed by alias URI.
  #[serde(default)]
  pub namespace_aliases: BTreeMap<String, String>,
  /// Declare all namespaces with prefix/URL
  #[serde(default)]
  pub namespaces: BTreeMap<String, String>,
  /// Set the prefix for the scope
  #[serde(default)]
  pub prefix: Option<String>,
  /// Rename the field/struct/enum name
  #[serde(default)]
  pub rename: Option<String>,
  #[serde(default)]
  pub tag: Option<String>,
  /// Disable the serialization for the field
  #[serde(default)]
  pub skip_serializing: bool,
  /// Disable the serialization for the field based on a condition
  #[serde(default)]
  pub skip_serializing_if: Option<String>,
  /// Set the field as an XML text content
  #[serde(default)]
  pub text: bool,
  /// Set the field as an XML CDATA content
  #[serde(default)]
  pub cdata: bool,
}

impl From<&Vec<Attribute>> for YaSerdeAttribute {
  /// Attributes are checked with `YaSerdeAttribute::parse` before expansion (see `validate`),
  /// so parsing cannot fail here.
  fn from(attributes: &Vec<Attribute>) -> Self {
    YaSerdeAttribute::parse(attributes).unwrap_or_else(|error| panic!("{}", error))
  }
}

impl YaSerdeAttribute {
  /// Parse and merge all `#[yaserde(...)]` attributes of an item.
  pub fn parse(attributes: &[Attribute]) -> syn::Result<Self> {
    let mut result = YaSerdeAttribute::default();

    for attr in attributes
      .iter()
      .filter(|attr| attr.path().is_ident("yaserde"))
    {
      let list = attr.meta.require_list()?;
      let parsed = from_tokenstream::<YaSerdeAttribute>(&list.tokens)?;
      result.merge(parsed).map_err(|key| {
        syn::Error::new_spanned(attr, format!("duplicate yaserde attribute `{}`", key))
      })?;
    }

    result.normalize_xmlns().map_err(|message| {
      let span = attributes
        .iter()
        .find(|attr| attr.path().is_ident("yaserde"))
        .map(syn::spanned::Spanned::span)
        .unwrap_or_else(proc_macro2::Span::call_site);
      syn::Error::new(span, message)
    })?;

    Ok(result)
  }

  fn normalize_xmlns(&mut self) -> Result<(), String> {
    for (prefix, namespace) in std::mem::take(&mut self.xmlns) {
      if prefix.is_empty() {
        return Err("xmlns key must not be empty; use xmlns_default".to_owned());
      }
      match self.namespaces.get(&prefix) {
        Some(existing) if existing != &namespace => {
          return Err(format!(
            "xmlns prefix `{prefix}` is declared with conflicting namespaces"
          ));
        }
        Some(_) => {}
        None => {
          self.namespaces.insert(prefix, namespace);
        }
      }
    }

    if let Some(namespace) = self.xmlns_default.take() {
      let prefix = self
        .namespaces
        .iter()
        .find_map(|(prefix, uri)| (uri == &namespace).then(|| prefix.clone()))
        .unwrap_or_else(|| {
          let mut prefix = "__default".to_owned();
          let mut index = 1;
          while self.namespaces.contains_key(&prefix) {
            prefix = format!("__default_{index}");
            index += 1;
          }
          self.namespaces.insert(prefix.clone(), namespace);
          prefix
        });

      match &self.default_namespace {
        Some(existing) if existing != &prefix => {
          return Err("xmlns_default conflicts with default_namespace".to_owned());
        }
        Some(_) => {}
        None => self.default_namespace = Some(prefix),
      }
    }

    for (alias, canonical) in &self.namespace_aliases {
      if alias.is_empty() || canonical.is_empty() {
        return Err("namespace_aliases must not contain empty URIs".to_owned());
      }
      if alias == canonical {
        return Err(format!(
          "namespace alias `{alias}` must differ from its canonical namespace"
        ));
      }
      if self.namespace_aliases.contains_key(canonical) {
        return Err(format!(
          "namespace alias `{alias}` points to another alias `{canonical}`"
        ));
      }
    }

    Ok(())
  }

  /// Merge `other` into `self`, returning the name of the first option set in both.
  fn merge(&mut self, other: Self) -> Result<(), String> {
    fn merge_bool(target: &mut bool, value: bool, key: &str) -> Result<(), String> {
      if *target && value {
        return Err(key.to_string());
      }
      *target |= value;
      Ok(())
    }

    fn merge_option(
      target: &mut Option<String>,
      value: Option<String>,
      key: &str,
    ) -> Result<(), String> {
      if target.is_some() && value.is_some() {
        return Err(key.to_string());
      }
      if value.is_some() {
        *target = value;
      }
      Ok(())
    }

    merge_bool(&mut self.attribute, other.attribute, "attribute")?;
    merge_option(&mut self.default, other.default, "default")?;
    merge_option(
      &mut self.default_namespace,
      other.default_namespace,
      "default_namespace",
    )?;
    merge_bool(&mut self.flatten, other.flatten, "flatten")?;
    merge_option(&mut self.namespace, other.namespace, "namespace")?;
    merge_option(
      &mut self.xmlns_default,
      other.xmlns_default,
      "xmlns_default",
    )?;
    for (alias, namespace) in other.namespace_aliases {
      if self.namespace_aliases.contains_key(&alias) {
        return Err(format!("namespace_aliases.{}", alias));
      }
      self.namespace_aliases.insert(alias, namespace);
    }
    for (prefix, namespace) in other.xmlns {
      if self.xmlns.contains_key(&prefix) {
        return Err(format!("xmlns.{}", prefix));
      }
      self.xmlns.insert(prefix, namespace);
    }
    for (prefix, namespace) in other.namespaces {
      if self.namespaces.contains_key(&prefix) {
        return Err(format!("namespaces.{}", prefix));
      }
      self.namespaces.insert(prefix, namespace);
    }
    merge_option(&mut self.prefix, other.prefix, "prefix")?;
    merge_option(&mut self.rename, other.rename, "rename")?;
    merge_option(&mut self.tag, other.tag, "tag")?;
    merge_bool(
      &mut self.skip_serializing,
      other.skip_serializing,
      "skip_serializing",
    )?;
    merge_option(
      &mut self.skip_serializing_if,
      other.skip_serializing_if,
      "skip_serializing_if",
    )?;
    merge_bool(&mut self.text, other.text, "text")?;
    merge_bool(&mut self.cdata, other.cdata, "cdata")?;
    Ok(())
  }

  pub fn xml_element_name(&self, ident: &Ident) -> String {
    self.rename.clone().unwrap_or_else(|| ident.to_string())
  }

  pub fn namespace_prefix(&self) -> Option<String> {
    if let Some(namespace) = &self.namespace {
      let canonical = self
        .namespace_aliases
        .get(namespace)
        .map_or(namespace, |value| value);
      return self
        .namespaces
        .iter()
        .find_map(|(prefix, uri)| (uri == canonical).then(|| prefix.clone()));
    }
    self.prefix.clone()
  }

  pub fn prefix_namespace(&self) -> String {
    let prefix = self.namespace_prefix();
    if self.default_namespace == prefix {
      "".to_string()
    } else {
      prefix.map_or("".to_string(), |prefix| prefix + ":")
    }
  }

  pub fn get_namespace_matching(
    &self,
    prefix: &Option<String>,
    expected_namespace: Option<&str>,
    element_namespace: TokenStream,
    element_name: TokenStream,
    take_root_prefix: bool,
  ) -> TokenStream {
    if let Some(expected_namespace) = expected_namespace {
      let canonical = self
        .namespace_aliases
        .get(expected_namespace)
        .map_or(expected_namespace, String::as_str);
      let mut accepted = vec![canonical.to_owned()];
      accepted.extend(
        self
          .namespace_aliases
          .iter()
          .filter(|(_, target)| *target == canonical)
          .map(|(alias, _)| alias.clone()),
      );
      accepted.sort();
      accepted.dedup();
      return quote!(
        match #element_namespace {
          ::std::option::Option::Some(namespace)
            if #(namespace.as_str() == #accepted)||* => {}
          ::std::option::Option::Some(namespace) => {
            let msg = ::std::format!(
              "bad namespace for {}, found {}",
              #element_name,
              namespace.as_str()
            );
            return ::std::result::Result::Err(::ofdbox_yaserde::Error::custom(msg));
          }
          ::std::option::Option::None => {
            let msg = ::std::format!(
              "missing namespace for {}, expected {}",
              #element_name,
              #canonical
            );
            return ::std::result::Result::Err(::ofdbox_yaserde::Error::custom(msg));
          }
        }
      );
    }

    if self.namespaces.is_empty() {
      return quote!(
        match (reader.current_namespace(), #element_namespace) {
          (::std::option::Option::Some(current), ::std::option::Option::Some(actual))
            if current == actual.as_str() => {}
          (::std::option::Option::None, ::std::option::Option::None) => {}
          (_, ::std::option::Option::Some(actual)) => {
            let msg = ::std::format!(
              "bad namespace for {}, found {}",
              #element_name,
              actual.as_str()
            );
            return ::std::result::Result::Err(::ofdbox_yaserde::Error::custom(msg));
          }
          (_, ::std::option::Option::None) => {
            let msg = ::std::format!(
              "missing namespace for {}",
              #element_name
            );
            return ::std::result::Result::Err(::ofdbox_yaserde::Error::custom(msg));
          }
        }
      );
    }

    let configured_prefix = if take_root_prefix {
      self.prefix.clone()
    } else {
      prefix.clone()
    };

    let namespaces_matches: TokenStream = self
      .namespaces
      .iter()
      .filter_map(|(prefix, namespace)| {
        if configured_prefix.as_deref().eq(&Some(prefix)) {
          Some(quote!(#namespace => {}))
        } else {
          None
        }
      })
      .collect();

    quote!(
      if let Some(namespace) = #element_namespace {
        match namespace.as_str() {
          #namespaces_matches
          bad_namespace => {
            let msg =
              ::std::format!("bad namespace for {}, found {}", #element_name, bad_namespace);
            return Err(::ofdbox_yaserde::Error::custom(msg));
          }
        }
      }
    )
  }
}

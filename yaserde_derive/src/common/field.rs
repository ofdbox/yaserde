use crate::common::attribute::YaSerdeAttribute;
use heck::ToUpperCamelCase;
use proc_macro2::Span;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::fmt;
use syn::Type::Path;
use syn::ext::IdentExt;
use syn::spanned::Spanned;

#[derive(Debug)]
pub struct YaSerdeField {
  syn_field: syn::Field,
  attributes: YaSerdeAttribute,
}

impl YaSerdeField {
  pub fn new(syn_field: syn::Field) -> Self {
    let attributes = YaSerdeAttribute::from(&syn_field.attrs);

    YaSerdeField {
      syn_field,
      attributes,
    }
  }

  pub fn is_attribute(&self) -> bool {
    self.attributes.attribute
  }

  pub fn is_text_content(&self) -> bool {
    self.attributes.text
  }

  pub fn is_flatten(&self) -> bool {
    self.attributes.flatten
  }

  pub fn label(&self) -> Option<Ident> {
    self.syn_field.ident.clone()
  }

  pub fn is_skip_serializing(&self) -> bool {
    self.attributes.skip_serializing
  }

  pub fn is_cdata(&self) -> bool {
    self.attributes.cdata
  }

  pub fn get_value_label(&self) -> Option<syn::Ident> {
    self
      .syn_field
      .ident
      .clone()
      .map(|ident| syn::Ident::new(&format!("__{}_value", ident.unraw()), ident.span()))
  }

  pub fn renamed_label_without_namespace(&self) -> String {
    self
      .attributes
      .rename
      .clone()
      .unwrap_or_else(|| self.label().as_ref().unwrap().to_string())
  }

  pub fn renamed_label(&self, root_attributes: &YaSerdeAttribute) -> String {
    let prefix = if let Some(namespace) = &self.attributes.namespace {
      let canonical = root_attributes
        .namespace_aliases
        .get(namespace)
        .map_or(namespace, |value| value);
      root_attributes
        .namespaces
        .iter()
        .find_map(|(prefix, uri)| (uri == canonical).then(|| prefix.clone()))
    } else {
      self.attributes.prefix.clone()
    };

    let prefix = if root_attributes.default_namespace == prefix {
      "".to_string()
    } else {
      prefix.map_or("".to_string(), |prefix| prefix + ":")
    };

    let label = self.renamed_label_without_namespace();

    format!("{}{}", prefix, label)
  }

  /// Runtime expression for the serialized element name.
  pub fn element_label_expr(&self, root_attributes: &YaSerdeAttribute) -> TokenStream {
    let Some(namespace) = &self.attributes.namespace else {
      if self.attributes.prefix.is_none() {
        let local = self.renamed_label_without_namespace();
        return quote!(writer.element_name(
          ::std::option::Option::None,
          #local,
          ::std::option::Option::None,
        ));
      }
      let label = self.renamed_label(root_attributes);
      return quote!((#label).to_string());
    };

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
    let local = self.renamed_label_without_namespace();

    quote!(writer.element_name(
      ::std::option::Option::Some(#canonical),
      #local,
      #fallback,
    ))
  }

  /// Whether this field inherits the parent element namespace.
  pub fn inherits_namespace(&self) -> bool {
    self.attributes.namespace.is_none() && self.attributes.prefix.is_none()
  }

  /// Label for a namespaced XML attribute.
  pub fn attribute_label(&self, root_attributes: &YaSerdeAttribute) -> String {
    let Some(namespace) = &self.attributes.namespace else {
      return self.renamed_label(root_attributes);
    };
    let canonical = root_attributes
      .namespace_aliases
      .get(namespace)
      .map_or(namespace, |value| value);
    let prefix = root_attributes
      .namespaces
      .iter()
      .find_map(|(prefix, uri)| (uri == canonical && !prefix.is_empty()).then(|| prefix.clone()));
    match prefix {
      Some(prefix) => format!("{prefix}:{}", self.renamed_label_without_namespace()),
      None => self.renamed_label_without_namespace(),
    }
  }

  pub fn get_visitor_ident(&self, struct_name: Option<&syn::Path>) -> Ident {
    let label = self.renamed_label_without_namespace();

    let struct_id = struct_name.map_or_else(
      || "".to_string(),
      |struct_name| {
        struct_name
          .segments
          .iter()
          .map(|s| s.ident.to_string())
          .collect()
      },
    );

    let prefix = self
      .attributes
      .prefix
      .clone()
      .map(|p| format!("{}_", p.to_upper_camel_case()))
      .unwrap_or_default();

    let attribute = if self.attributes.attribute {
      "Attribute_"
    } else {
      Default::default()
    };

    Ident::new(
      &format!(
        "__Visitor_{attribute}{}{}_{}",
        prefix,
        label.replace('.', "_").to_upper_camel_case(),
        struct_id
      ),
      self.get_span(),
    )
  }

  /// Types are checked by `common::validate` before expansion, so parsing cannot fail here.
  pub fn get_type(&self) -> Field {
    Field::try_from(&self.syn_field.ty).unwrap_or_else(|error| panic!("{}", error))
  }

  /// Build an error pointing at this field's type.
  pub fn unsupported(&self, message: &str) -> syn::Error {
    syn::Error::new_spanned(&self.syn_field.ty, format!("YaSerDe: {}", message))
  }

  pub fn get_span(&self) -> Span {
    self.syn_field.span()
  }

  pub fn get_default_function(&self) -> Option<Ident> {
    self
      .attributes
      .default
      .as_ref()
      .map(|default| Ident::new(default, self.get_span()))
  }

  pub fn get_skip_serializing_if_function(&self) -> Option<Ident> {
    self
      .attributes
      .skip_serializing_if
      .as_ref()
      .map(|skip_serializing_if| Ident::new(skip_serializing_if, self.get_span()))
  }

  pub fn prefix_namespace(&self, root_attributes: &YaSerdeAttribute) -> String {
    if let Some(namespace) = &self.attributes.namespace {
      return namespace.clone();
    }

    root_attributes
      .namespaces
      .iter()
      .find_map(|(prefix, namespace)| {
        if self.attributes.prefix.as_deref().eq(&Some(prefix)) {
          Some(namespace.clone())
        } else {
          None
        }
      })
      .unwrap_or_default()
  }

  pub fn namespace_values(&self, root_attributes: &YaSerdeAttribute) -> Vec<String> {
    if let Some(namespace) = &self.attributes.namespace {
      let canonical = root_attributes
        .namespace_aliases
        .get(namespace)
        .map_or(namespace, |value| value);
      let mut values = vec![canonical.clone()];
      values.extend(
        root_attributes
          .namespace_aliases
          .iter()
          .filter(|(_, target)| *target == canonical)
          .map(|(alias, _)| alias.clone()),
      );
      values.sort();
      values.dedup();
      return values;
    }

    let mut values = Vec::new();
    for (prefix, namespace) in &root_attributes.namespaces {
      if self.attributes.prefix.as_deref().eq(&Some(prefix.as_str())) {
        values.push(namespace.clone());
      }
    }
    if values.is_empty() {
      values.push(String::new());
    }
    values
  }

  pub fn get_namespace_matching(
    &self,
    root_attributes: &YaSerdeAttribute,
    element_namespace: TokenStream,
    element_name: TokenStream,
  ) -> TokenStream {
    if self.inherits_namespace() {
      return quote!();
    }

    root_attributes.get_namespace_matching(
      &self.attributes.prefix,
      self.attributes.namespace.as_deref(),
      element_namespace,
      element_name,
      false,
    )
  }

  pub fn ser_wrap_default_attribute(
    &self,
    builder: Option<TokenStream>,
    setter: TokenStream,
  ) -> TokenStream {
    let label = self.label();

    let yaserde_inner_definition = builder
      .map(|builder| quote!(let yaserde_inner = #builder;))
      .unwrap_or_default();

    let skip_if = self
      .get_skip_serializing_if_function()
      .map(|skip_if_function| quote!(!self.#skip_if_function(&self.#label)))
      .unwrap_or(quote!(true));

    self
      .get_default_function()
      .map(|default_function| {
        quote! {
          #yaserde_inner_definition
          let struct_start_event =
            if #skip_if && self.#label != #default_function() {
              #setter
            } else {
              struct_start_event
            };
        }
      })
      .unwrap_or(quote! {
        #yaserde_inner_definition
        let struct_start_event = if #skip_if { #setter } else { struct_start_event };
      })
  }
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum Field {
  FieldString,
  FieldBool,
  FieldI8,
  FieldU8,
  FieldI16,
  FieldU16,
  FieldI32,
  FieldU32,
  FieldI64,
  FieldU64,
  FieldF32,
  FieldF64,
  FieldOption { data_type: Box<Field> },
  FieldVec { data_type: Box<Field> },
  FieldStruct { struct_name: syn::Path },
}

impl Field {
  pub fn get_simple_type_visitor(&self) -> Ident {
    format_ident!("visit_{}", self.to_string())
  }
}

impl TryFrom<&syn::Path> for Field {
  type Error = syn::Error;

  fn try_from(path: &syn::Path) -> syn::Result<Self> {
    let Some(segment) = path.segments.last() else {
      return Err(syn::Error::new_spanned(path, "YaSerDe: empty type path"));
    };

    Ok(match segment.ident.to_string().as_str() {
      "String" => Field::FieldString,
      "bool" => Field::FieldBool,
      "i8" => Field::FieldI8,
      "u8" => Field::FieldU8,
      "i16" => Field::FieldI16,
      "u16" => Field::FieldU16,
      "i32" => Field::FieldI32,
      "u32" => Field::FieldU32,
      "i64" => Field::FieldI64,
      "u64" => Field::FieldU64,
      "f32" => Field::FieldF32,
      "f64" => Field::FieldF64,
      "Option" => Field::FieldOption {
        data_type: Box::new(Field::try_from(segment)?),
      },
      "Vec" => Field::FieldVec {
        data_type: Box::new(Field::try_from(segment)?),
      },
      _ => Field::FieldStruct {
        struct_name: path.clone(),
      },
    })
  }
}

impl TryFrom<&syn::Type> for Field {
  type Error = syn::Error;

  fn try_from(ty: &syn::Type) -> syn::Result<Self> {
    let mut inner = ty;
    while let syn::Type::Group(group) = inner {
      inner = &group.elem;
    }
    match inner {
      Path(path) => Field::try_from(&path.path),
      _ => Err(syn::Error::new_spanned(
        ty,
        "YaSerDe does not support this type, only paths like `String`, `Option<T>`, `Vec<T>` or `MyStruct` are supported",
      )),
    }
  }
}

impl TryFrom<&syn::PathSegment> for Field {
  type Error = syn::Error;

  /// Parse the generic argument of `Option<T>` or `Vec<T>`
  fn try_from(path_segment: &syn::PathSegment) -> syn::Result<Self> {
    match &path_segment.arguments {
      syn::PathArguments::AngleBracketed(args) if args.args.len() == 1 => match &args.args[0] {
        syn::GenericArgument::Type(ty) => Field::try_from(ty),
        argument => Err(syn::Error::new_spanned(
          argument,
          "YaSerDe: expected a type argument",
        )),
      },
      _ => Err(syn::Error::new_spanned(
        path_segment,
        format!(
          "YaSerDe: expected `{}<T>` with a single type argument",
          path_segment.ident
        ),
      )),
    }
  }
}

impl From<Field> for proc_macro2::TokenStream {
  fn from(field: Field) -> proc_macro2::TokenStream {
    match field {
      Field::FieldString => quote! { ::std::string::String },
      Field::FieldBool => quote! { bool },
      Field::FieldI8 => quote! { i8 },
      Field::FieldU8 => quote! { u8 },
      Field::FieldI16 => quote! { i16 },
      Field::FieldU16 => quote! { u16 },
      Field::FieldI32 => quote! { i32 },
      Field::FieldU32 => quote! { u32 },
      Field::FieldI64 => quote! { i64 },
      Field::FieldU64 => quote! { u64 },
      Field::FieldF32 => quote! { f32 },
      Field::FieldF64 => quote! { f64 },
      _ => panic!("Not a simple type: {:?}", field),
    }
  }
}

impl From<&Field> for String {
  fn from(field: &Field) -> String {
    match field {
      Field::FieldString => "str".to_string(),
      Field::FieldBool => "bool".to_string(),
      Field::FieldI8 => "i8".to_string(),
      Field::FieldU8 => "u8".to_string(),
      Field::FieldI16 => "i16".to_string(),
      Field::FieldU16 => "u16".to_string(),
      Field::FieldI32 => "i32".to_string(),
      Field::FieldU32 => "u32".to_string(),
      Field::FieldI64 => "i64".to_string(),
      Field::FieldU64 => "u64".to_string(),
      Field::FieldF32 => "f32".to_string(),
      Field::FieldF64 => "f64".to_string(),
      Field::FieldStruct { struct_name } => quote! {#struct_name}.to_string(),
      _ => panic!("Not a simple type: {:?}", field),
    }
  }
}

impl fmt::Display for Field {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let string_representation: String = self.into();
    write!(f, "{}", string_representation)
  }
}

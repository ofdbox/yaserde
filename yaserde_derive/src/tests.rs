use syn::{DeriveInput, parse_quote};

fn deserialize_error(input: DeriveInput) -> String {
  crate::common::validate(&input)
    .and_then(|()| crate::de::expand_derive_deserialize(&input))
    .expect_err("expected a derive error")
    .to_string()
}

fn serialize_error(input: DeriveInput) -> String {
  crate::common::validate(&input)
    .and_then(|()| crate::ser::expand_derive_serialize(&input))
    .expect_err("expected a derive error")
    .to_string()
}

#[test]
fn duplicate_attribute() {
  let input = parse_quote! {
    struct Data {
      #[yaserde(rename = "x")]
      #[yaserde(rename = "y")]
      field: String,
    }
  };
  assert_eq!(
    serialize_error(input),
    "duplicate yaserde attribute `rename`"
  );
}

#[test]
fn attribute_without_arguments() {
  let input = parse_quote! {
    struct Data {
      #[yaserde]
      field: String,
    }
  };
  assert_eq!(
    deserialize_error(input),
    "expected attribute arguments in parentheses: `yaserde(...)`"
  );
}

#[test]
fn unsupported_type() {
  let input = parse_quote! {
    struct Data {
      field: &'static str,
    }
  };
  assert!(deserialize_error(input).starts_with("YaSerDe does not support this type"));
}

#[test]
fn tuple_struct() {
  let input = parse_quote! {
    struct Data(u32);
  };
  assert!(serialize_error(input).starts_with("YaSerDe does not support tuple structs"));
}

#[test]
fn union() {
  let input = parse_quote! {
    union Data {
      field: u32,
    }
  };
  assert_eq!(deserialize_error(input), "YaSerDe does not support unions");
}

#[test]
fn nested_vec() {
  let input: DeriveInput = parse_quote! {
    struct Data {
      field: Vec<Vec<u32>>,
    }
  };
  assert_eq!(
    deserialize_error(input.clone()),
    "YaSerDe: `Option` or `Vec` nested in `Vec` is not supported"
  );
  assert_eq!(
    serialize_error(input),
    "YaSerDe: `Vec` nested in `Vec` is not supported"
  );
}

#[test]
fn nested_option() {
  let input: DeriveInput = parse_quote! {
    struct Data {
      field: Option<Option<u32>>,
    }
  };
  assert_eq!(
    deserialize_error(input.clone()),
    "YaSerDe: `Option` nested in `Option` is not supported"
  );
  assert_eq!(
    serialize_error(input),
    "YaSerDe: `Option` nested in `Option` is not supported"
  );
}

#[test]
fn nested_attribute() {
  let input: DeriveInput = parse_quote! {
    struct Data {
      #[yaserde(attribute = true)]
      field: Vec<Vec<u32>>,
    }
  };
  assert_eq!(
    serialize_error(input),
    "YaSerDe: `Option` or `Vec` nested in `Vec` is not supported for attributes"
  );
}

#[test]
fn flatten_on_vec() {
  let input = parse_quote! {
    struct Data {
      #[yaserde(flatten = true)]
      field: Vec<u32>,
    }
  };
  assert_eq!(
    deserialize_error(input),
    "YaSerDe: `flatten` is only supported on struct fields"
  );
}

#[test]
fn enum_variant_with_multiple_fields() {
  let input = parse_quote! {
    enum Data {
      Variant(u32, u32),
    }
  };
  assert_eq!(
    deserialize_error(input),
    "YaSerDe: enum variants with multiple fields are not supported"
  );
}

#[test]
fn enum_variant_with_nested_type() {
  let input = parse_quote! {
    enum Data {
      Variant(Option<Vec<u32>>),
    }
  };
  assert_eq!(
    deserialize_error(input),
    "YaSerDe: `Option` or `Vec` nested in `Option` or `Vec` is not supported in enum variants"
  );
}

#[test]
fn primitive_on_named_struct() {
  let input = parse_quote! {
    struct Data {
      field: u32,
    }
  };
  assert!(
    crate::primitives::primitive_serde(&input)
      .expect_err("expected a derive error")
      .to_string()
      .starts_with("`PrimitiveYaSerde` can only be derived for a newtype struct")
  );
}

#[test]
fn xmlns_rejects_empty_prefix() {
  let input = parse_quote! {
    #[yaserde(xmlns = { "" = "http://a" })]
    struct Data {
      field: String,
    }
  };
  assert_eq!(
    serialize_error(input),
    "xmlns key must not be empty; use xmlns_default"
  );
}

#[test]
fn xmlns_rejects_conflicting_namespace() {
  let input = parse_quote! {
    #[yaserde(
      namespaces = { "ns" = "http://a" },
      xmlns = { "ns" = "http://b" }
    )]
    struct Data {
      field: String,
    }
  };
  assert_eq!(
    serialize_error(input),
    "xmlns prefix `ns` is declared with conflicting namespaces"
  );
}

#[test]
fn xmlns_default_rejects_conflicting_default_prefix() {
  let input = parse_quote! {
    #[yaserde(
      default_namespace = "ns",
      xmlns_default = "http://a"
    )]
    struct Data {
      field: String,
    }
  };
  assert!(serialize_error(input).contains("xmlns_default conflicts"));
}

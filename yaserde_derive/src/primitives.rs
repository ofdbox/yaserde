// Adds YaSerialize and YaDeserialize implementations for types that support FromStr and Display traits.
// Code originally from `xsd-parser-rs`

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

pub fn primitive_yaserde(input: TokenStream) -> TokenStream {
  let ast = parse_macro_input!(input as DeriveInput);

  let struct_name = &ast.ident;
  let struct_name_literal = &ast.ident.to_string();

  let serde = quote! {
      impl ::ofdbox_yaserde::YaSerialize for #struct_name {
          fn serialize<W: ::std::io::Write>(
              &self,
              writer: &mut ::ofdbox_yaserde::ser::Serializer<W>,
          ) -> ::std::result::Result<(), ::ofdbox_yaserde::Error> {
            ::ofdbox_yaserde::primitives::serialize_primitives(
                  self,
                  #struct_name_literal,
                  writer, |s| s.to_string(),
              )
          }

          fn serialize_attributes(
              &self,
              attributes: ::std::vec::Vec<::ofdbox_yaserde::__xml::attribute::OwnedAttribute>,
              namespace: ::ofdbox_yaserde::__xml::namespace::Namespace,
          ) -> ::std::result::Result<
              (
                  ::std::vec::Vec<::ofdbox_yaserde::__xml::attribute::OwnedAttribute>,
                  ::ofdbox_yaserde::__xml::namespace::Namespace,
              ),
              ::ofdbox_yaserde::Error,
          > {
              Ok((attributes, namespace))
          }
      }

      impl ::ofdbox_yaserde::YaDeserialize for #struct_name {
          fn deserialize<R: ::std::io::Read>(
              reader: &mut ::ofdbox_yaserde::de::Deserializer<R>,
          ) -> ::std::result::Result<Self, ::ofdbox_yaserde::Error> {
              ::ofdbox_yaserde::primitives::deserialize_primitives(
                  reader,
                  |s| <#struct_name as ::std::str::FromStr>::from_str(s).map_err(::ofdbox_yaserde::Error::custom),
              )
          }
      }
  };

  serde.into()
}

pub fn hexbinary_serde(input: TokenStream) -> TokenStream {
  let first = input.clone();
  let DeriveInput { ident, .. } = parse_macro_input!(first);
  // Calculate number digits to determine whether leading zero should be added
  quote!(
    impl std::fmt::Display for #ident {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02X}", self.0)
      }
    }

    impl ::std::str::FromStr for #ident {
      type Err = ::std::string::String;

      fn from_str(s: &::std::primitive::str) -> ::std::result::Result<Self, Self::Err> {
        Self::from_bits(
          s.parse()
              .map_err(|_| String::from("Failed to parse Bitflag integer"))?,
      )
      .ok_or(String::from("Unknown bits were set in Bitflag"))
      }
    }
  )
  .into()
}

pub fn primitive_serde(ast: &DeriveInput) -> syn::Result<TokenStream2> {
  let ident = &ast.ident;
  let fromstr = extract_full_path(ast)?;

  Ok(quote!(
    impl std::fmt::Display for #ident {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
      }
    }

    impl ::std::str::FromStr for #ident {
      type Err = ::std::string::String;

      fn from_str(s: &::std::primitive::str) -> ::std::result::Result<Self, Self::Err> {
        Ok(#ident(#fromstr))
      }
    }
  ))
}

fn extract_full_path(ast: &syn::DeriveInput) -> Result<TokenStream2, syn::Error> {
  if let syn::Data::Struct(data_struct) = &ast.data {
    if let syn::Fields::Unnamed(fields) = &data_struct.fields {
      if let Some(syn::Type::Path(path)) = &fields.unnamed.first().map(|f| &f.ty) {
        return Ok(
          quote! { <#path as ::std::str::FromStr>::from_str(s).map_err(|e| e.to_string())? },
        );
      }
    }
  }

  Err(syn::Error::new_spanned(
    &ast.ident,
    "`PrimitiveYaSerde` can only be derived for a newtype struct, like `struct Value(u32);`",
  ))
}

use crate::common::{Field, YaSerdeAttribute, YaSerdeField};

use crate::ser::{element::*, implement_serializer::implement_serializer};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;
use syn::{DataStruct, Generics};

pub fn serialize(
  data_struct: &DataStruct,
  name: &Ident,
  root: &str,
  root_attributes: &YaSerdeAttribute,
  generics: &Generics,
) -> syn::Result<TokenStream> {
  let append_attributes: TokenStream = data_struct
    .fields
    .iter()
    .map(|field| YaSerdeField::new(field.clone()))
    .filter(|field| field.is_attribute() || field.is_flatten())
    .map(|field| -> syn::Result<TokenStream> {
      let label = field.label();

      Ok(if field.is_attribute() {
        let label_name = field.renamed_label(root_attributes);

        match field.get_type() {
          Field::FieldString
          | Field::FieldBool
          | Field::FieldI8
          | Field::FieldU8
          | Field::FieldI16
          | Field::FieldU16
          | Field::FieldI32
          | Field::FieldU32
          | Field::FieldI64
          | Field::FieldU64
          | Field::FieldF32
          | Field::FieldF64 => field.ser_wrap_default_attribute(
            Some(quote!(self.#label.to_string())),
            quote!({
              struct_start_event.attr(#label_name, &yaserde_inner)
            }),
          ),
          Field::FieldOption { data_type } => match *data_type {
            Field::FieldString => field.ser_wrap_default_attribute(
              None,
              quote!({
                if let ::std::option::Option::Some(ref value) = self.#label {
                  struct_start_event.attr(#label_name, value)
                } else {
                  struct_start_event
                }
              }),
            ),
            Field::FieldBool
            | Field::FieldI8
            | Field::FieldU8
            | Field::FieldI16
            | Field::FieldU16
            | Field::FieldI32
            | Field::FieldU32
            | Field::FieldI64
            | Field::FieldU64
            | Field::FieldF32
            | Field::FieldF64 => field.ser_wrap_default_attribute(
              Some(
                quote!(self.#label.map_or_else(|| ::std::string::String::new(), |v| v.to_string())),
              ),
              quote!({
                if let ::std::option::Option::Some(ref value) = self.#label {
                  struct_start_event.attr(#label_name, &yaserde_inner)
                } else {
                  struct_start_event
                }
              }),
            ),
            Field::FieldVec { data_type } => match *data_type {
              Field::FieldString
              | Field::FieldBool
              | Field::FieldI8
              | Field::FieldU8
              | Field::FieldI16
              | Field::FieldU16
              | Field::FieldI32
              | Field::FieldU32
              | Field::FieldI64
              | Field::FieldU64
              | Field::FieldF32
              | Field::FieldF64 => ser_option_vec_attribute(
                &field,
                &label,
                &label_name,
                quote!(::std::result::Result::<_, ::yaserde::Error>::Ok(
                  item.to_string()
                )),
              ),
              Field::FieldStruct { .. } => ser_option_vec_attribute(
                &field,
                &label,
                &label_name,
                quote!(::yaserde::ser::to_string_attribute_value(item)),
              ),
              _ => {
                return Err(field.unsupported(
                  "`Option` or `Vec` nested in `Option<Vec<_>>` is not supported for attributes",
                ));
              }
            },
            Field::FieldStruct { .. } => field.ser_wrap_default_attribute(
              Some(quote! {
              self.#label
                .as_ref()
                .map_or_else(
                  || ::std::result::Result::Ok(::std::string::String::new()),
                  |v| ::yaserde::ser::to_string_attribute_value(v),
                )?
              }),
              quote!({
                if let ::std::option::Option::Some(ref yaserde_struct) = self.#label {
                  struct_start_event.attr(#label_name, &yaserde_inner)
                } else {
                  struct_start_event
                }
              }),
            ),
            Field::FieldOption { .. } => {
              return Err(
                field.unsupported("`Option` nested in `Option` is not supported for attributes"),
              );
            }
          },
          Field::FieldStruct { .. } => field.ser_wrap_default_attribute(
            Some(quote! { ::yaserde::ser::to_string_attribute_value(&self.#label)? }),
            quote!({
              struct_start_event.attr(#label_name, &yaserde_inner)
            }),
          ),
          Field::FieldVec { data_type } => match *data_type {
            Field::FieldString
            | Field::FieldBool
            | Field::FieldI8
            | Field::FieldU8
            | Field::FieldI16
            | Field::FieldU16
            | Field::FieldI32
            | Field::FieldU32
            | Field::FieldI64
            | Field::FieldU64
            | Field::FieldF32
            | Field::FieldF64 => field.ser_wrap_default_attribute(
              Some(quote! {
                self.#label
                  .iter()
                  .map(|item| item.to_string())
                  .collect::<::std::vec::Vec<_>>()
                  .join(" ")
              }),
              quote!({
                struct_start_event.attr(#label_name, &yaserde_inner)
              }),
            ),
            Field::FieldOption { .. } | Field::FieldVec { .. } => {
              return Err(
                field
                  .unsupported("`Option` or `Vec` nested in `Vec` is not supported for attributes"),
              );
            }
            Field::FieldStruct { .. } => field.ser_wrap_default_attribute(
              Some(quote! {
                self.#label
                  .iter()
                  .map(|item| ::yaserde::ser::to_string_attribute_value(item))
                  .collect::<::std::result::Result<::std::vec::Vec<_>, _>>()?
                  .join(" ")
              }),
              quote!({
                struct_start_event.attr(#label_name, &yaserde_inner)
              }),
            ),
          },
        }
      } else {
        match field.get_type() {
          Field::FieldStruct { .. } => {
            quote!(
              let (attributes, namespace) = self.#label.serialize_attributes(
                ::std::vec![],
                ::yaserde::__xml::namespace::Namespace::empty(),
              )?;
              child_attributes_namespace.extend(&namespace);
              child_attributes.extend(attributes);
            )
          }
          _ => quote!(),
        }
      })
    })
    .collect::<syn::Result<_>>()?;

  let struct_inspector: TokenStream = data_struct
    .fields
    .iter()
    .map(|field| YaSerdeField::new(field.clone()))
    .filter(|field| !field.is_attribute())
    .map(|field| -> syn::Result<Option<TokenStream>> {
      let label = field.label();
      if field.is_text_content() {
        return Ok(match field.get_type() {
          Field::FieldOption { .. } => Some(quote!(
            let s = self.#label.as_deref().unwrap_or_default();
            let data_event = ::yaserde::__xml::writer::XmlEvent::characters(s);
            writer.write(data_event)?;
          )),
          Field::FieldString => Some(quote!(
            let data_event = ::yaserde::__xml::writer::XmlEvent::characters(&self.#label);
            writer.write(data_event)?;
          )),
          _ => Some(quote!(
            let yaserde_value = self.#label.to_string();
            let data_event = ::yaserde::__xml::writer::XmlEvent::characters(&yaserde_value);
            writer.write(data_event)?;
          )),
        });
      }
      let label_name = field.element_label_expr(root_attributes);
      let conditions = condition_generator(&label, &field);

      if field.is_cdata() {
        return Ok(Some(quote! {
            #conditions {
              let yaserde_label_name = #label_name;
              let start_event = ::yaserde::__xml::writer::XmlEvent::start_element(yaserde_label_name.as_str());
              writer.write(start_event)?;
              let data = ::yaserde::__xml::writer::events::XmlEvent::cdata(&self.#label);
              writer.write(data)?;
              let end_event = ::yaserde::__xml::writer::XmlEvent::end_element();
              writer.write(end_event)?;
            }
        }));
      }

      Ok(match field.get_type() {
        Field::FieldString
        | Field::FieldBool
        | Field::FieldI8
        | Field::FieldU8
        | Field::FieldI16
        | Field::FieldU16
        | Field::FieldI32
        | Field::FieldU32
        | Field::FieldI64
        | Field::FieldU64
        | Field::FieldF32
        | Field::FieldF64 => serialize_element(&label, label_name, &conditions),

        Field::FieldOption { data_type } => match *data_type {
          Field::FieldString
          | Field::FieldBool
          | Field::FieldI8
          | Field::FieldU8
          | Field::FieldI16
          | Field::FieldU16
          | Field::FieldI32
          | Field::FieldU32
          | Field::FieldI64
          | Field::FieldU64
          | Field::FieldF32
          | Field::FieldF64 => {
            let item_ident = Ident::new("yaserde_item", field.get_span());
            let inner = enclose_formatted_characters_for_value(&item_ident, label_name);

            Some(quote! {
              #conditions {
                if let Some(ref yaserde_item) = self.#label {
                  #inner
                }
              }
            })
          }
          Field::FieldVec { .. } => {
            // Only use attribute serialization if the field is marked as an attribute
            if field.is_attribute() {
              let item_ident = Ident::new("yaserde_item", field.get_span());
              let inner = enclose_formatted_characters_for_value(&item_ident, label_name);

              Some(quote! {
                #conditions {
                  if let ::std::option::Option::Some(yaserde_items) = &self.#label {
                    for yaserde_item in yaserde_items.iter() {
                      #inner
                    }
                  }
                }
              })
            } else {
              // For non-attribute Option<Vec<T>>, use standard serialization
              Some(quote! {
                #conditions {
                  if let ::std::option::Option::Some(items) = &self.#label {
                    for item in items.iter() {
                      writer.set_start_event_name(::std::option::Option::Some(#label_name.to_string()));
                      writer.set_skip_start_end(false);
                      ::yaserde::YaSerialize::serialize(item, writer)?;
                    }
                  }
                }
              })
            }
          }
          Field::FieldStruct { .. } => Some(if field.is_flatten() {
            quote! {
              #conditions {
                if let ::std::option::Option::Some(item) = &self.#label {
                  writer.set_start_event_name(::std::option::Option::None);
                  writer.set_skip_start_end(true);
                  ::yaserde::YaSerialize::serialize(item, writer)?;
                }
              }
            }
          } else {
            quote! {
              #conditions {
                if let ::std::option::Option::Some(item) = &self.#label {
                  writer.set_start_event_name(::std::option::Option::Some(#label_name.to_string()));
                  writer.set_skip_start_end(false);
                  ::yaserde::YaSerialize::serialize(item, writer)?;
                }
              }
            }
          }),
          Field::FieldOption { .. } => {
            return Err(field.unsupported("`Option` nested in `Option` is not supported"));
          }
        },
        Field::FieldStruct { .. } => {
          let (start_event, skip_start) = if field.is_flatten() {
            (quote!(::std::option::Option::None), true)
          } else {
            (
              quote!(::std::option::Option::Some(#label_name.to_string())),
              false,
            )
          };

          Some(quote! {
            #conditions {
              writer.set_start_event_name(#start_event);
              writer.set_skip_start_end(#skip_start);
              ::yaserde::YaSerialize::serialize(&self.#label, writer)?;
            }
          })
        }
        Field::FieldVec { data_type } => match *data_type {
          Field::FieldString => {
            let item_ident = Ident::new("yaserde_item", field.get_span());
            let inner = enclose_formatted_characters_for_value(&item_ident, label_name);

            Some(quote! {
              #conditions {
                for yaserde_item in &self.#label {
                  #inner
                }
              }
            })
          }
          Field::FieldBool
          | Field::FieldI8
          | Field::FieldU8
          | Field::FieldI16
          | Field::FieldU16
          | Field::FieldI32
          | Field::FieldU32
          | Field::FieldI64
          | Field::FieldU64
          | Field::FieldF32
          | Field::FieldF64 => {
            let item_ident = Ident::new("yaserde_item", field.get_span());
            let inner = enclose_formatted_characters_for_value(&item_ident, label_name);

            Some(quote! {
              #conditions {
                for yaserde_item in &self.#label {
                  #inner
                }
              }
            })
          }
          Field::FieldOption { .. } => Some(quote! {
            #conditions {
              for item in &self.#label {
                if let Some(value) = item {
                  writer.set_start_event_name(None);
                  writer.set_skip_start_end(false);
                  ::yaserde::YaSerialize::serialize(value, writer)?;
                }
              }
            }
          }),
          Field::FieldStruct { .. } => {
            if field.is_flatten() {
              Some(quote! {
                #conditions {
                  for item in &self.#label {
                      writer.set_start_event_name(::std::option::Option::None);
                    writer.set_skip_start_end(true);
                    ::yaserde::YaSerialize::serialize(item, writer)?;
                  }
                }
              })
            } else {
              Some(quote! {
                #conditions {
                  for item in &self.#label {
                    writer.set_start_event_name(::std::option::Option::Some(#label_name.to_string()));
                    writer.set_skip_start_end(false);
                    ::yaserde::YaSerialize::serialize(item, writer)?;
                 }
                }
              })
            }
            /*let (start_event, skip_start) = if field.is_flatten() {
              (quote!(None), true)
            } else {
              (quote!(Some(#label_name.to_string())), false)
            };

            Some(quote! {
              writer.set_start_event_name(#start_event);
              writer.set_skip_start_end(#skip_start);
              ::yaserde::YaSerialize::serialize(&self.#label, writer)?;
            })*/
          }
          Field::FieldVec { .. } => {
            return Err(field.unsupported("`Vec` nested in `Vec` is not supported"));
          }
        },
      })
    })
    .collect::<syn::Result<Vec<_>>>()?
    .into_iter()
    .flatten()
    .collect();

  Ok(implement_serializer(
    name,
    root,
    root_attributes,
    append_attributes,
    struct_inspector,
    generics,
  ))
}

/// Helper function to generate serialization code for `Option<Vec<T>>` attributes
fn ser_option_vec_attribute(
  field: &YaSerdeField,
  label: &Option<Ident>,
  label_name: &str,
  item_serializer: TokenStream,
) -> TokenStream {
  let yaserde_inner_expr = quote! {
    match self.#label.as_ref() {
      ::std::option::Option::Some(yaserde_list) => yaserde_list
        .iter()
        .map(|item| #item_serializer)
        .collect::<::std::result::Result<::std::vec::Vec<_>, ::yaserde::Error>>()?
        .join(" "),
      ::std::option::Option::None => ::std::string::String::new(),
    }
  };

  let attribute_expr = quote!({
    if self.#label.is_some() && !yaserde_inner.is_empty() {
      struct_start_event.attr(#label_name, &yaserde_inner)
    } else if self.#label.is_some() {
      struct_start_event.attr(#label_name, "")
    } else {
      struct_start_event
    }
  });

  field.ser_wrap_default_attribute(Some(yaserde_inner_expr), attribute_expr)
}

use crate::common::YaSerdeAttribute;
use crate::ser::namespace::generate_namespaces_definition;
use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Generics;

pub fn implement_serializer(
  name: &Ident,
  root: &str,
  attributes: &YaSerdeAttribute,
  append_attributes: TokenStream,
  inner_inspector: TokenStream,
  generics: &Generics,
) -> TokenStream {
  let namespaces_definition = generate_namespaces_definition(attributes);
  let flatten = attributes.flatten;

  let namespace_pairs = attributes.namespaces.iter().map(|(prefix, namespace)| {
    let prefix = if attributes.default_namespace.as_deref() == Some(prefix) {
      ""
    } else {
      prefix.as_str()
    };
    quote!((#prefix.to_string(), #namespace.to_string()))
  });
  let namespace_pairs: Vec<_> = namespace_pairs.collect();
  let push_namespaces = if namespace_pairs.is_empty() {
    quote!()
  } else {
    quote!(writer.push_namespaces(::std::vec![#(#namespace_pairs),*]);)
  };
  let pop_namespaces = if namespace_pairs.is_empty() {
    quote!()
  } else {
    quote!(writer.pop_namespaces();)
  };
  let (enter_namespace, restore_namespace) = match &attributes.namespace {
    Some(namespace) => (
      quote! {
        let yaserde_previous_namespace =
          writer.current_namespace().map(::std::string::ToString::to_string);
        writer.set_current_namespace(::std::option::Option::Some(#namespace.to_string()));
      },
      quote!(writer.set_current_namespace(yaserde_previous_namespace);),
    ),
    None => (quote!(), quote!()),
  };

  let namespace_option = match &attributes.namespace {
    Some(namespace) => quote!(::std::option::Option::Some(#namespace)),
    None => quote!(::std::option::Option::None),
  };

  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

  quote! {
    impl #impl_generics ::yaserde::YaSerialize for #name #ty_generics #where_clause {
      #[allow(unused_variables)]
      fn serialize<W: ::std::io::Write>(
        &self,
        writer: &mut ::yaserde::ser::Serializer<W>,
      ) -> ::std::result::Result<(), ::yaserde::Error> {
        let skip = writer.skip_start_end();
        #enter_namespace
        #push_namespaces

        let yaserde_label = match writer.get_start_event_name() {
          ::std::option::Option::Some(name) if !name.contains(':') => writer.element_name(
            #namespace_option,
            name.as_str(),
            ::std::option::Option::None,
          ),
          ::std::option::Option::Some(name) => name.to_string(),
          ::std::option::Option::None => #root.to_string(),
        };

        if !#flatten && !skip {
          let mut child_attributes = ::std::vec![];
          let mut child_attributes_namespace = ::yaserde::__xml::namespace::Namespace::empty();

          let struct_start_event =
            ::yaserde::__xml::writer::XmlEvent::start_element(yaserde_label.as_str()) #namespaces_definition;
          #append_attributes

          let event: ::yaserde::__xml::writer::events::XmlEvent = struct_start_event.into();

          if let ::yaserde::__xml::writer::events::XmlEvent::StartElement {
            name,
            attributes,
            namespace,
          } = event {
            let mut attributes: ::std::vec::Vec<::yaserde::__xml::attribute::OwnedAttribute> =
              attributes.into_owned().to_vec().iter().map(|k| k.to_owned()).collect();
            attributes.extend(child_attributes);

            let all_attributes = attributes.iter().map(|ca| ca.borrow()).collect();

            let mut all_namespaces = namespace.into_owned();
            all_namespaces.extend(&child_attributes_namespace);

            writer.write(::yaserde::__xml::writer::events::XmlEvent::StartElement{
              name,
              attributes: ::std::borrow::Cow::Owned(all_attributes),
              namespace: ::std::borrow::Cow::Owned(all_namespaces)
            })?;
          } else {
            unreachable!()
          }
        }

        #inner_inspector

        if !#flatten && !skip {
          let struct_end_event = ::yaserde::__xml::writer::XmlEvent::end_element();
          writer.write(struct_end_event)?;
        }
        #pop_namespaces
        #restore_namespace

        ::std::result::Result::Ok(())
      }

      fn serialize_attributes(
        &self,
        mut source_attributes: ::std::vec::Vec<::yaserde::__xml::attribute::OwnedAttribute>,
        mut source_namespace: ::yaserde::__xml::namespace::Namespace,
      ) -> ::std::result::Result<
        (::std::vec::Vec<::yaserde::__xml::attribute::OwnedAttribute>, ::yaserde::__xml::namespace::Namespace),
        ::yaserde::Error
      > {
        let mut child_attributes = ::std::vec::Vec::<::yaserde::__xml::attribute::OwnedAttribute>::new();
        let mut child_attributes_namespace = ::yaserde::__xml::namespace::Namespace::empty();

        let struct_start_event =
          ::yaserde::__xml::writer::XmlEvent::start_element("temporary_element_to_generate_attributes")
          #namespaces_definition;

        #append_attributes
        let event: ::yaserde::__xml::writer::events::XmlEvent = struct_start_event.into();

        if let ::yaserde::__xml::writer::events::XmlEvent::StartElement { attributes, namespace, .. } = event {
          source_namespace.extend(&namespace.into_owned());
          source_namespace.extend(&child_attributes_namespace);

          let a: ::std::vec::Vec<::yaserde::__xml::attribute::OwnedAttribute> =
            attributes.into_owned().to_vec().iter().map(|k| k.to_owned()).collect();
          source_attributes.extend(a);
          source_attributes.extend(child_attributes);

          ::std::result::Result::Ok((source_attributes, source_namespace))
        } else {
          unreachable!();
        }
      }
    }
  }
}

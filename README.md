# ofdbox-yaserde

> This project is a fork of [yaserde](https://github.com/luminvent/yaserde), dedicated to the OFDBox project. It is updated only when necessary.

**Yet Another Serializer/Deserializer specialized for XML**

## Goal
This library will support XML de/ser-ializing with all specific features.

## Supported types

- [x] Struct
- [x] Vec<AnyType>
- [x] Enum
- [x] Enum with complex types
- [x] Named enum variants for serialization
- [ ] Named enum variants for deserialization (not supported)
- [x] Option
- [x] String
- [x] bool
- [x] number (u8, u16, u32, u64, i8, i16, i32, i64, f32, f64)

## Attributes

- [x] **attribute**: this field is defined as an attribute
- [x] **cdata**: this field is serialized as XML CDATA
- [x] **default**: defines the default function to init the field
- [x] **default_namespace**: legacy default namespace prefix
- [x] **flatten**: Flatten the contents of the field
- [x] **namespace**: namespace URI of the element
- [x] **xmlns**: prefix-to-URI namespace declarations
- [x] **xmlns_default**: default namespace URI
- [x] **namespace_aliases**: accepted alias URI to canonical URI mappings
- [x] **namespaces**: legacy prefix-to-URI declarations
- [x] **prefix**: legacy prefix for the scope
- [x] **rename**: be able to rename a field
- [x] **skip_serializing**: Exclude this field from the serialized output. [More details...](doc/skip_serializing.md)
- [x] **skip_serializing_if**: Skip the serialisation for this field if the condition is true.  [More details...](doc/skip_serializing.md)
- [x] **tag**: use an XML attribute as the enum discriminator
- [x] **text**: this field match to the text content

## Namespaces

Elements declare a namespace URI; the prefix is chosen by the serializer.

```rust
#[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
#[yaserde(
  rename = "Root",
  namespace = "http://example.com/ns",
  xmlns = {
    "ns" = "http://example.com/ns"
  }
)]
struct Root {
  child: Child,
}

#[derive(Debug, PartialEq, YaDeserialize, YaSerialize)]
struct Child {
  value: String,
}
```

The declared namespace is inherited by nested elements, so child types do not
repeat it:

```xml
<ns:Root xmlns:ns="http://example.com/ns">
  <ns:child>
    <ns:value>ok</ns:value>
  </ns:child>
</ns:Root>
```

Use `xmlns_default` to serialize with a default namespace instead of a prefix:

```rust
#[yaserde(
  namespace = "http://example.com/ns",
  xmlns_default = "http://example.com/ns"
)]
```

Alternate URIs can be accepted on input while output keeps the canonical URI:

```rust
#[yaserde(
  namespace = "http://example.com/ns",
  xmlns = { "ns" = "http://example.com/ns" },
  namespace_aliases = {
    "http://example.com/ns/v1" = "http://example.com/ns"
  }
)]
```

Inherited fields canonicalize input namespaces through these aliases.

## Named enum variants

Named variants can be serialized:

```rust
#[derive(Debug, PartialEq, YaSerialize)]
enum Color {
    #[yaserde(rename = "custom")]
    Custom {
        enabled: String,
        value: u32,
    },
}
```

Deserialization of named variants is not supported because the serialized form
has no variant wrapper or discriminator. Use a tuple variant with a struct
instead:

```rust
#[derive(Debug, PartialEq, YaSerialize, YaDeserialize)]
enum Color {
    Custom(CustomColor),
}

#[derive(Debug, PartialEq, YaSerialize, YaDeserialize)]
struct CustomColor {
    enabled: String,
    value: u32,
}
```

## Custom De/Ser-rializer

Any type can define a custom deserializer and/or serializer.
To implement it, define the implementation of YaDeserialize/YaSerialize:

```rust
use std::io::{Read, Write};
use ofdbox_yaserde::{YaDeserialize, YaSerialize};

impl YaDeserialize for MyType {
  fn deserialize<R: Read>(
    reader: &mut ofdbox_yaserde::de::Deserializer<R>,
  ) -> ofdbox_yaserde::Result<Self> {
    // deserializer code
  }
}

impl YaSerialize for MyType {
  fn serialize<W: Write>(
    &self,
    writer: &mut ofdbox_yaserde::ser::Serializer<W>,
  ) -> ofdbox_yaserde::Result<()> {
    // serializer code
  }

  fn serialize_attributes(
    &self,
    attributes: Vec<ofdbox_yaserde::__xml::attribute::OwnedAttribute>,
    namespace: ofdbox_yaserde::__xml::namespace::Namespace,
  ) -> ofdbox_yaserde::Result<(
    Vec<ofdbox_yaserde::__xml::attribute::OwnedAttribute>,
    ofdbox_yaserde::__xml::namespace::Namespace,
  )> {
    Ok((attributes, namespace))
  }
}
```

`serialize_attributes` is required by `YaSerialize`; return the input
attributes and namespace unchanged when the type contributes no attributes.
The `__xml` module is a re-export of the `xml` crate used by the trait
signatures.

## Errors

Serialization and deserialization return an `ofdbox_yaserde::Error`:

- `Error::Reader`: the XML input is malformed, with its position in the input.
- `Error::Writer`: the XML output could not be written.
- `Error::Message`: the XML does not match the expected structure, or a value could not be converted.
  Create one with `ofdbox_yaserde::Error::custom(message)` in a custom implementation.
- `Error::Field`: an error located in a field, with the field's XML name.

Writer errors convert with `?`, and `String` or `&str` messages convert with `.into()`.

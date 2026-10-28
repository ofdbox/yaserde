# yaserde &emsp; [![Latest Version]][crates.io]

[Latest Version]: https://img.shields.io/crates/v/yaserde.svg
[crates.io]: https://crates.io/crates/yaserde

**Yet Another Serializer/Deserializer specialized for XML**

## Goal
This library will support XML de/ser-ializing with all specific features.

## Supported types

- [x] Struct
- [x] Vec<AnyType>
- [x] Enum
- [x] Enum with complex types
- [x] Option
- [x] String
- [x] bool
- [x] number (u8, i8, u32, i32, f32, f64)

## Attributes

- [x] **attribute**: this field is defined as an attribute
- [x] **default**: defines the default function to init the field
- [x] **flatten**: Flatten the contents of the field
- [x] **namespace**: namespace URI of the element
- [x] **xmlns**: prefix-to-URI namespace declarations
- [x] **xmlns_default**: default namespace URI
- [x] **namespace_aliases**: accepted alias URI to canonical URI mappings
- [x] **namespaces**: legacy prefix-to-URI declarations
- [x] **rename**: be able to rename a field
- [x] **root**: rename the based element. Used only at the XML root.
- [x] **skip_serializing**: Exclude this field from the serialized output. [More details...](doc/skip_serializing.md)
- [x] **skip_serializing_if**: Skip the serialisation for this field if the condition is true.  [More details...](doc/skip_serializing.md)
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

The namespace is inherited from the parent element, so nested fields do not
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

## Custom De/Ser-rializer

Any type can define a custom deserializer and/or serializer.
To implement it, define the implementation of YaDeserialize/YaSerialize

```rust
impl YaDeserialize for MyType {
  fn deserialize<R: Read>(reader: &mut yaserde::de::Deserializer<R>) -> yaserde::Result<Self> {
    // deserializer code
  }
}
```

```rust
impl YaSerialize for MyType {
  fn serialize<W: Write>(&self, writer: &mut yaserde::ser::Serializer<W>) -> yaserde::Result<()> {
    // serializer code
  }
}
```

## Errors

Serialization and deserialization return a `yaserde::Error`:

- `Error::Reader`: the XML input is malformed, with its position in the input.
- `Error::Writer`: the XML output could not be written.
- `Error::Message`: the XML does not match the expected structure, or a value could not be converted.
  Create one with `yaserde::Error::custom(message)` in a custom implementation.
- `Error::Field`: an error located in a field, with the field's XML name.

Writer errors convert with `?`, and `String` or `&str` messages convert with `.into()`.

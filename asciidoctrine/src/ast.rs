use std::{fmt::Debug, marker::PhantomPinned, ops::Deref, pin::Pin, ptr::NonNull};

use serde::{Deserialize, Deserializer, Serialize};

use crate::util::Env;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AST<'a> {
  pub content: &'a str,
  pub elements: Vec<ElementSpan<'a>>,
  pub attributes: Vec<Attribute<'a>>,
}

impl AST<'_> {
  pub fn get_attribute(&self, name: &str) -> Option<&str> {
    for attribute in self.attributes.iter() {
      if &attribute.key == name {
        return match &attribute.value {
          AttributeValue::Ref(value) => Some(value),
          AttributeValue::String(value) => Some(value.as_str()),
        };
      }
    }

    None
  }
}

impl Default for AST<'_> {
  fn default() -> Self {
    Self {
      content: "",
      elements: vec![],
      attributes: vec![],
    }
  }
}

/// The basic element of a document
///
/// This is meant to form a tree of document element.
/// Every element holds references to its source, it
/// subelements and the attributes defined on it.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ElementSpan<'a> {
  // The source document. Could be empty if
  // e.g. it's the same as the source of it's
  // parent
  pub source: Option<String>,
  // A string reference to the source
  pub content: &'a str,
  // TODO Add start and end point
  pub start: usize,
  pub end: usize,
  /// We count the lines for usage in other tools
  pub start_line: usize,
  pub start_col: usize,
  pub end_line: usize,
  pub end_col: usize,

  pub element: Element<'a>,
  /// The subelements of a nodes
  pub children: Vec<ElementSpan<'a>>,
  /// The attributes applying to that node and
  /// all children
  pub positional_attributes: Vec<AttributeValue<'a>>,
  /// The attributes applying to that node and
  /// all children
  pub attributes: Vec<Attribute<'a>>,
}

impl<'a> ElementSpan<'a> {
  pub fn get_attribute(&self, name: &str) -> Option<&str> {
    for attribute in self.attributes.iter() {
      if &attribute.key == name {
        return match &attribute.value {
          AttributeValue::Ref(value) => Some(value),
          AttributeValue::String(value) => Some(value.as_str()),
        };
      }
    }

    None
  }

  pub fn add_offset(self, other: &ElementSpan<'_>) -> Self {
    let mut base = self;

    base.start += other.start;
    base.end += other.start;
    base.start_line += other.start_line - 1;
    base.end_line += other.start_line - 1;
    base.start_col += other.start_col - 1;
    base.end_col += other.start_col - 1;

    base
  }

  pub fn element(self, e: Element<'a>) -> Self {
    let mut base = self;

    base.element = e;
    base
  }

  pub fn error(self, msg: &str) -> Self {
    self.element(Element::Error(msg.to_string()))
  }

  pub fn add_attribute(self, a: Attribute<'a>) -> Self {
    let mut base = self;

    base.attributes.push(a);
    base
  }

  pub fn add_positional_attribute(self, a: AttributeValue<'a>) -> Self {
    let mut base = self;

    base.positional_attributes.push(a);
    base
  }

  pub fn add_child(self, e: ElementSpan<'a>) -> Self {
    let mut base = self;

    base.children.push(e);
    base
  }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Element<'a> {
  Attribute(#[serde(borrow)] Attribute<'a>),
  /// A section of ignored text
  Comment,
  /// A text paragraph
  Paragraph,
  /// A header section
  Title {
    level: u32,
  },
  Table,
  List(ListType),
  Image,
  Anchor,
  /// Holds all blocks with special content and the type
  /// TODO Could be done with ExternalContent and all known
  /// Types here direktly
  TypedBlock {
    kind: BlockType,
  },
  /// Holds content which is not prozessed direktly by
  /// asciidoctrine. It can be anything. Outputs or
  /// postprocessors could use or ignore it at their
  /// will (e.g. videos)
  ExternalContent,
  /// Holds a reference to the include statement
  /// and a document inside
  IncludeElement(IncludeElement<'a>),

  /// The following variants are inline elements nested
  /// inside a conainer element

  /// Element with a special style. The attributes define the kind of style
  Styled,
  /// A chunk of text.
  Text,
  /// An internal reference or link
  XRef,
  /// An external link
  Link,
  /// A list item
  ListItem(u32),
  /// A table row
  TableRow,
  /// A table cell
  TableCell,
  /// A wrong formatted text or block
  Error(String),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum ListType {
  Bullet,
  Number,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum BlockType {
  Comment,
  Passtrough,
  Listing,
  Literal,
  Sidebar,
  Quote,
  Example,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum AttributeValue<'a> {
  String(String),
  Ref(&'a str),
}

impl AttributeValue<'_> {
  pub fn as_str(&self) -> &str {
    match self {
      AttributeValue::Ref(value) => value,
      AttributeValue::String(value) => value.as_str(),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Attribute<'a> {
  pub key: String,
  #[serde(borrow)]
  pub value: AttributeValue<'a>,
}

#[derive(Clone)]
struct UnmovableString {
  pub content: String,
  pub content_ref: NonNull<str>,
  _pin: PhantomPinned,
}

impl PartialEq for UnmovableString {
  fn eq(&self, other: &Self) -> bool {
    self.content == other.content
  }
}

impl Debug for UnmovableString {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "UnmovableString(\"{}\")", self.content)
  }
}

impl UnmovableString {
  fn new(content: String) -> Pin<Box<Self>> {
    let res = Self {
      content,
      content_ref: "".into(),
      _pin: PhantomPinned,
    };
    let mut boxed = Box::new(res);
    boxed.content_ref = NonNull::from(boxed.content.as_str());
    let pin = Box::into_pin(boxed);
    pin
  }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct IncludeElement<'a> {
  #[serde(
    serialize_with = "serialize_unmovable_string",
    deserialize_with = "deserialize_unmovable_string"
  )]
  content: Pin<Box<UnmovableString>>,
  #[serde(borrow)]
  pub inner: AST<'a>,
  #[serde(skip)]
  _pin: PhantomPinned,
}

impl IncludeElement<'_> {
  pub fn from_parser(
    content: String,
    env: &mut Env,
    parser: &dyn for<'a> Fn(&'a str, &mut Env) -> Result<AST<'a>, String>,
  ) -> Result<Self, String> {
    let content = UnmovableString::new(content);
    let content_ref = unsafe { content.content_ref.as_ref() };
    let inner_ast = parser(content_ref, env)?;

    Ok(IncludeElement {
      content,
      inner: inner_ast,
      _pin: PhantomPinned,
    })
  }

  pub fn from_data(content: String, inner: AST<'static>) -> Self {
    IncludeElement {
      content: UnmovableString::new(content),
      inner,
      _pin: PhantomPinned,
    }
  }
}

fn serialize_unmovable_string<S>(
  string: &Pin<Box<UnmovableString>>,
  serializer: S,
) -> Result<S::Ok, S::Error>
where
  S: serde::Serializer,
{
  serializer.serialize_str(&string.content)
}

fn deserialize_unmovable_string<'de, D>(
  deserializer: D,
) -> Result<Pin<Box<UnmovableString>>, D::Error>
where
  D: Deserializer<'de>,
{
  let string = String::deserialize(deserializer)?;
  Ok(UnmovableString::new(string))
}

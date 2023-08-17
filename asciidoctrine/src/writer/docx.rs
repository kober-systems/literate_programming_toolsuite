pub use crate::ast::*;
use crate::{options, Result, AsciidoctrineError};
use std::io;
use docx_rs::*;

pub struct DocxWriter {}

impl DocxWriter {
  pub fn new() -> Self {
    DocxWriter {}
  }
}

impl<T: io::Write + io::Seek> crate::Writer<T> for DocxWriter {
  fn write<'a>(&mut self, ast: AST, _args: &options::Opts, mut out: T) -> Result<()> {
    let doc = ast
      .elements
      .iter()
      .try_fold(Docx::new(), |doc, element| write_doc(element, doc))?;
    doc.build().pack(out).unwrap();

    Ok(())
  }
}

fn paragraph(input: &ElementSpan, out: Paragraph) -> Result<Paragraph> {
  match &input.element {
    Element::Text => {
      let r = Run::new();
      Ok(out.add_run(r.add_text(input.content.to_string())))
    }
    Element::Link => {
      let url = input.get_attribute("url").unwrap_or("");
      let content = match input.positional_attributes.get(0) {
        Some(value) => match value {
          AttributeValue::Ref(value) => value.to_string(),
          AttributeValue::String(value) => value.clone(),
        },
        None => "".to_string(),
      };

      Ok(out.add_hyperlink(Hyperlink::new(url, HyperlinkType::External)))
    }
    _ => Err(AsciidoctrineError::MalformedAst),
  }
}

fn write_doc(input: &ElementSpan, out: Docx) -> Result<Docx> {
  match &input.element {
    Element::Paragraph => {
      let p = input
        .children
        .iter()
        .try_fold(Paragraph::new(), |p, element| paragraph(element, p))?;
      Ok(out.add_paragraph(p))
    }
    Element::Text | Element::Link => Err(AsciidoctrineError::MalformedAst),
    _ => {
      Err(crate::AsciidoctrineError::Childprocess) // TODO Error
    }
  }
}

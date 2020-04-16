pub use crate::ast::*;
use crate::writer::*;
use crate::Result;

pub struct HtmlWriter {}

impl HtmlWriter {
  pub fn new() -> Self {
    HtmlWriter {}
  }
}

impl crate::Writer for HtmlWriter {
  fn write<'a>(&self, ast: AST) -> Result<()> {
    for element in ast.elements.iter() {
      write_html(element);
    }

    Ok(())
  }
}

// TODO Styles etc
// TODO Out buffer
fn write_html(input: &ElementSpan) -> Result<()> {
  match &input.element {
    Element::Title { level: level } => {
      print!("<h{}>{}</h{}>\n", level, input.content, level);
    }
    _ => {
      print!("<NOT-YET-SUPPORTED>{}</NOT-YET-SUPPORTED>", input.content);
    }
  }

  Ok(())
}

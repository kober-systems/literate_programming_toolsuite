pub use crate::ast::*;
use crate::util::Environment;
use crate::{options, Result, AsciidoctrineError};
use std::io;
use tera::{Context, Tera};

pub struct HtmlWriter {
  io: crate::util::Env,
}

impl HtmlWriter {
  pub fn new() -> Self {
    HtmlWriter {
      io: crate::util::Env::Io(crate::util::Io::new()),
    }
  }
}

impl<T: io::Write> crate::Writer<T> for HtmlWriter {
  fn write<'a>(&mut self, ast: AST, args: &options::Opts, mut out: T) -> Result<()> {
    let mut buf = io::BufWriter::new(Vec::new());

    for element in ast.elements.iter() {
      write_html(element, 0, &mut buf)?;
    }
    let bytes = buf.into_inner()?;

    let mut context = Context::new();
    context.insert("lang", "en");
    context.insert("doctitle", "");
    match &args.stylesheet {
      Some(path) => {
        let path = path.to_str().expect("path to stylesheet unreadable");
        let stylesheet = self.io.read_to_string(path)?;
        context.insert("stylesheet", &stylesheet);
      }
      None => {
        context.insert("stylesheet", include_str!("assets/asciidoctor.css"));
      }
    }
    context.insert("body_class", "article toc2 toc-left");
    context.insert("body", std::str::from_utf8(&bytes)?);

    let mut tera = Tera::default();
    tera.autoescape_on(vec![]);
    match &args.template {
      Some(path) => {
        let path = path.to_str().expect("path to template unreadable");
        let template = if path == "-" {
          "{{body}}".to_string()
        } else {
          self.io.read_to_string(path)?
        };
        tera
          .add_raw_template("default.html", &template)
          .expect("couldn't load default template");
      }
      None => {
        tera
          .add_raw_template("default.html", include_str!("assets/template.html"))
          .expect("couldn't load default template");
      }
    }
    out.write_all(tera.render("default.html", &context)?.as_bytes())?;
    out.flush()?;

    Ok(())
  }
}

fn write_html<T: io::Write>(input: &ElementSpan, indent: usize, out: &mut T) -> Result<()> {
  match &input.element {
    Element::Title { level } => {
      let tag = format!("h{}", level);

      if level > &1 {
        let id = match input.get_attribute("anchor") {
          Some(id) => id.to_string(),
          None => {
            let title = input
              .get_attribute("name")
              .ok_or(AsciidoctrineError::MalformedAst)?;
            "_".to_string() + &title.replace(" ", "_").to_lowercase()
          }
        };
        let attrs = format!("id=\"{}\"", id);
        write_attribute_tag(&tag, &attrs, input, indent, out)?;
      } else {
        write_tag(&tag, input, indent, out)?;
      };
      out.write_all(b"\n")?;
    }
    Element::Paragraph => {
      write_tag("p", input, indent, out)?;
      out.write_all(b"\n")?;
    }
    Element::List(list_type) => {
      let list_element = match list_type {
        ListType::Bullet => "ul",
        ListType::Number => "ol",
      };

      let mut current_level = 0;
      for element in input.children.iter() {
        if let Element::ListItem(item_level) = element.element {
          let attrs = match list_type {
            ListType::Bullet => "",
            ListType::Number => {
              if item_level % 2 == 0 {
                "class=\"loweralpha\" type=\"a\""
              } else {
                "class=\"arabic\""
              }
            }
          };

          let item_level = item_level as usize;
          let offset = if current_level > 0 { item_level - 1 } else { 0 };
          if current_level < item_level {
            write_open_attribute_tag_ln(list_element, attrs, indent + current_level + offset, out)?;
            write_open_tag_ln("li", indent + item_level + offset, out)?;
          } else {
            if current_level > item_level {
              let diff = current_level - item_level;
              let offset = (current_level * 2) - 1;
              for i in 0..diff {
                let indent = indent + offset - (2 * i);
                write_close_tag_ln("li", indent, out)?;
                write_close_tag_ln(list_element, indent - 1, out)?;
              }
            }
            let item_level = if item_level > 1 {
              item_level + 1
            } else {
              item_level
            };
            write_close_tag_ln("li", indent + item_level, out)?;
            write_open_tag_ln("li", indent + item_level, out)?;
          }
          write_html(element, indent + item_level + offset, out)?;

          current_level = item_level;
        }
      }
      write_close_tag_ln("li", indent + 1, out)?;
      write_close_tag_ln(list_element, indent, out)?;
    }
    Element::ListItem(_) => {
      for element in input.children.iter() {
        write_html(element, indent + 1, out)?;
      }
    }
    Element::TypedBlock { kind } => {
      if kind == &BlockType::Comment {
        // Comments are not printed in html
        // TODO provide option to print comments
        return Ok(());
      }
      if kind == &BlockType::Example
        && input
          .positional_attributes
          .iter()
          .find(|&attr| attr.as_str().find("%collapsible").is_some())
          .is_some()
      {
        if input
          .positional_attributes
          .iter()
          .find(|&attr| attr.as_str().find("%open").is_some())
          .is_some()
        {
          write_open_tag("details open", indent, out)?;
        } else {
          write_open_tag("details", indent, out)?;
        }

        let title = input.get_attribute("title").unwrap_or("Details");
        out.write_all(&format!("\n  <summary class=\"title\">{}</summary>\n", title).as_bytes())?;

        write_open_tag_ln("div class=\"content\"", indent + 1, out)?;
        write_open_tag_ln("div class=\"paragraph\"", indent + 2, out)?;
        for element in input.children.iter() {
          write_html(element, indent + 3, out)?;
        }
        write_close_tag_ln("div", indent + 2, out)?;
        write_close_tag_ln("div", indent + 1, out)?;
        write_close_tag_ln("details", indent, out)?;

        return Ok(());
      }

      out.write_all(b"<div")?;

      if let Some(id) = input.get_attribute("anchor") {
        out.write_all(&format!(" id=\"{}\" ", id).as_bytes())?;
      };

      let class = match kind {
        BlockType::Listing => "listingblock",
        _ => "unknown-block",
      };
      out.write_all(&format!(" class=\"{}\">\n", class).as_bytes())?;

      if let Some(title) = input.get_attribute("title") {
        out.write_all(&format!("\n  <div class=\"title\">{}</div>\n", title).as_bytes())?;
      };

      if kind == &BlockType::Listing {
        out.write_all(b"  <pre>")?;
      }

      let content = input.get_attribute("content").unwrap_or(input.content);
      out.write_all(escape_text(&content).as_bytes())?;

      if kind == &BlockType::Listing {
        out.write_all(b"</pre>\n")?;
      }
      write_close_tag_ln("div", indent, out)?;
    }
    Element::Image => {
      if let Some(path) = input.get_attribute("path") {
        match input.get_attribute("opts") {
          Some(options) => {
            let content = input.get_attribute("content").unwrap_or("");
            if options == "inline" {
              if path.ends_with(".svg") {
                out.write_all(content.as_bytes())?;
              }
              // TODO
            } else if options == "interactive" {
              // TODO
            } else {
              // TODO
            }
          }
          None => {
            out.write_all(&format!("<img src=\"{}\"></div>\n", path).as_bytes())?;
          }
        }
      }
    }
    Element::Table => {
      write_open_attribute_tag_ln(
        "table",
        "class=\"tableblock frame-all grid-all stretch\"",
        indent,
        out,
      )?;
      write_open_tag_ln("colgroup", indent + 1, out)?;
      write_open_attribute_tag_ln("col", "style=\"width: 50%;\"", indent + 2, out)?;
      write_open_attribute_tag_ln("col", "style=\"width: 50%;\"", indent + 2, out)?;
      write_close_tag_ln("colgroup", indent + 1, out)?;
      write_open_tag_ln("tbody", indent + 1, out)?;
      for table_row in input.children.iter() {
        match &table_row.element {
          Element::TableRow => {
            write_open_tag_ln("tr", indent + 2, out)?;
            for table_cell in table_row.children.iter() {
              match &table_cell.element {
                Element::TableCell => {
                  write_tag("td", table_cell, indent + 3, out)?;
                }
                _ => (),
              }
              out.write_all(b"\n")?;
            }
            write_close_tag_ln("tr", indent + 2, out)?;
          }
          _ => {
            out.write_all(
              &format!(
                "<NOT-YET-SUPPORTED:{:?}>{}</NOT-YET-SUPPORTED:{:?}>\n",
                table_row.element, table_row.content, table_row.element,
              )
              .as_bytes(),
            )?;
          }
        }
      }
      write_close_tag_ln("tbody", indent + 1, out)?;
      write_close_tag_ln("table", indent, out)?;
    }
    _ => {
      out.write_all(
        &format!(
          "<NOT-YET-SUPPORTED:{:?}>{}</NOT-YET-SUPPORTED:{:?}>\n",
          input.element, input.content, input.element,
        )
        .as_bytes(),
      )?;
    }
  }

  Ok(())
}

fn inline<T: io::Write>(input: &ElementSpan, out: &mut T) -> Result<()> {
  match &input.element {
    Element::Text => {
      out.write_all(input.content.as_bytes())?;
    }
    Element::Styled => {
      let style = match input.get_attribute("style").unwrap_or("") {
        "monospaced" => "code",
        style => style,
      };
      write_tag(style, input, 0, out)?;
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

      out.write_all(&format!("<a href=\"{}\">{}</a>", url, content).as_bytes())?;
    }
    Element::XRef => {
      let id = input.get_attribute("id").unwrap_or("");
      let content = input.get_attribute("content").unwrap_or(id.clone());

      out.write_all(&format!("<a href=\"#{}\">{}</a>", id, content).as_bytes())?;
    }
    _ => {
      out.write_all(
        &format!(
          "<NOT-YET-SUPPORTED:{:?}>{}</NOT-YET-SUPPORTED:{:?}>\n",
          input.element, input.content, input.element,
        )
        .as_bytes(),
      )?;
    }
  }

  Ok(())
}

fn table_paragraph<T: io::Write>(input: &ElementSpan, indent: usize, out: &mut T) -> Result<()> {
  match &input.element {
    Element::Paragraph => {
      write_open_tag("p", indent, out)?;
      for child in input.children.iter() {
        inline(child, out)?;
      }
      write_close_tag("p", 0, out)?;
    }
    Element::List(_) => {
      write_html(input, indent, out)?;
    }
    _ => {
      out.write_all(
        &format!(
          "<NOT-YET-SUPPORTED:{:?}>{}</NOT-YET-SUPPORTED:{:?}>\n",
          input.element, input.content, input.element,
        )
        .as_bytes(),
      )?;
    }
  }

  Ok(())
}

// Helper Functions
//----------------------------------------------------

fn escape_text(input: &str) -> String {
  input.replace("<", "&lt;").replace(">", "&gt;")
}

fn write_tag<T: io::Write>(
  tag: &str,
  inner: &ElementSpan,
  indent: usize,
  out: &mut T,
) -> Result<()> {
  write_attribute_tag(tag, "", inner, indent, out)
}

fn write_attribute_tag<T: io::Write>(
  tag: &str,
  attrs: &str,
  inner: &ElementSpan,
  indent: usize,
  out: &mut T,
) -> Result<()> {
  write_open_attribute_tag(tag, attrs, indent, out)?;

  match &inner.element {
    Element::Title { .. } => {
      let title = inner
        .get_attribute("name")
        .ok_or(AsciidoctrineError::MalformedAst)?;
      out.write_all(title.as_bytes())?;
    }
    Element::Paragraph => {
      for element in inner.children.iter() {
        inline(element, out)?;
      }
    }
    Element::Styled => {
      let content = inner.get_attribute("content").unwrap_or("");
      out.write_all(content.as_bytes())?;
    }
    Element::TableCell => {
      let indent = if inner.children.len() == 1 {
        0
      } else {
        indent + 1
      };
      for child in inner.children.iter() {
        if indent != 0 {
          out.write_all(b"\n")?;
        }
        table_paragraph(child, indent, out)?;
      }
      if indent != 0 {
        out.write_all(&b"  ".repeat(indent - 1))?;
      }
    }
    el => write_html(inner, indent + 1, out)?,
  };

  out.write_all(format!("</{}>", tag).as_bytes())?;
  Ok(())
}

fn write_open_tag<T: io::Write>(tag: &str, indent: usize, out: &mut T) -> Result<()> {
  write_open_attribute_tag(tag, "", indent, out)
}

fn write_open_attribute_tag<T: io::Write>(
  tag: &str,
  attrs: &str,
  indent: usize,
  out: &mut T,
) -> Result<()> {
  out.write_all(&b"  ".repeat(indent))?;
  out.write_all(format!("<{}", tag).as_bytes())?;
  if attrs != "" {
    out.write_all(b" ")?;
    out.write_all(attrs.as_bytes())?;
  }
  out.write_all(b">")?;
  Ok(())
}

fn write_close_tag<T: io::Write>(tag: &str, indent: usize, out: &mut T) -> io::Result<()> {
  out.write_all(&b"  ".repeat(indent))?;
  out.write_all(format!("</{}>", tag).as_bytes())
}

fn write_open_tag_ln<T: io::Write>(tag: &str, indent: usize, out: &mut T) -> Result<()> {
  write_open_tag(tag, indent, out)?;
  out.write_all(b"\n")?;
  Ok(())
}

fn write_open_attribute_tag_ln<T: io::Write>(
  tag: &str,
  attrs: &str,
  indent: usize,
  out: &mut T,
) -> Result<()> {
  write_open_attribute_tag(tag, attrs, indent, out)?;
  out.write_all(b"\n")?;
  Ok(())
}

fn write_close_tag_ln<T: io::Write>(tag: &str, indent: usize, out: &mut T) -> io::Result<()> {
  write_close_tag(tag, indent, out)?;
  out.write_all(b"\n")
}

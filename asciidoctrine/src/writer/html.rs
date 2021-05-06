pub use crate::ast::*;
use crate::util::Environment;
use crate::{options, Result};
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
      write_html(element, &mut buf)?;
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
        let path = path.to_str().expect("path to stylesheet unreadable");
        let template = self.io.read_to_string(path)?;
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

fn escape_text(input: &str) -> String {
  input.replace("<", "&lt;").replace(">", "&gt;")
}

// TODO Styles etc
fn write_html<T: io::Write>(input: &ElementSpan, out: &mut T) -> Result<()> {
  match &input.element {
    Element::Title { level } => {
      let title = input.get_attribute("name").unwrap_or("");

      out.write_all(&format!("<h{}", level).as_bytes())?;

      if level > &1 {
        let id = match input.get_attribute("anchor") {
          Some(id) => id.to_string(),
          None => "_".to_string() + &title.replace(" ", "_").to_lowercase(),
        };
        out.write_all(&format!(" id=\"{}\"", id).as_bytes())?;
      };

      out.write_all(&format!(">{}</h{}>\n", title, level).as_bytes())?;
    }
    Element::Paragraph => {
      out.write_all(b"<p>")?;
      for element in input.children.iter() {
        write_html(element, out)?;
      }
      out.write_all(b"</p>\n")?;
    }
    Element::List => {
      let mut level = 1;

      out.write_all(b"<ul>\n")?;
      for element in input.children.iter() {
        if let Element::ListItem(item_level) = element.element {
          if item_level > level {
            out.write_all(b"<ul>\n")?;
            level = item_level;
          } else if item_level < level {
            out.write_all(b"</ul>\n")?;
            level = item_level;
          }
          write_html(element, out)?;
        }
      }
      out.write_all(b"</ul>\n")?;
    }
    Element::ListItem(_level) => {
      out.write_all(b"<li>\n")?;
      for element in input.children.iter() {
        write_html(element, out)?;
      }
      out.write_all(b"</li>\n")?;
    }
    Element::TypedBlock { kind } => {
      if kind == &BlockType::Comment {
        // Comments are not printed in html
        // TODO provide option to print comments
        return Ok(());
      }

      out.write_all(b"<div ")?;

      if let Some(id) = input.get_attribute("anchor") {
        out.write_all(&format!("id=\"{}\" ", id).as_bytes())?;
      };

      let class = match kind {
        BlockType::Listing => "listingblock",
        _ => "unknown-block",
      };
      out.write_all(&format!("class=\"{}\">\n", class).as_bytes())?;

      if let Some(title) = input.get_attribute("title") {
        out.write_all(&format!("<div class=\"title\">{}</div>\n", title).as_bytes())?;
      };

      if kind == &BlockType::Listing {
        out.write_all(b"<pre>")?;
      }

      let content = input.get_attribute("content").unwrap_or(input.content);
      out.write_all(escape_text(&content).as_bytes())?;

      if kind == &BlockType::Listing {
        out.write_all(b"</pre>\n")?;
      }

      out.write_all(b"</div>\n")?;
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
    Element::Text => {
      out.write_all(input.content.as_bytes())?;
    }
    Element::Styled => {
      let style = input.get_attribute("style").unwrap_or("");
      let content = input.get_attribute("content").unwrap_or("");

      if style == "monospaced" {
        out.write_all(b"<code>")?;
      } else if style == "strong" {
        out.write_all(b"<strong>")?;
      }

      out.write_all(content.as_bytes())?;

      if style == "monospaced" {
        out.write_all(b"</code>")?;
      } else if style == "strong" {
        out.write_all(b"</strong>")?;
      };
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

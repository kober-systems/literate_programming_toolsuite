use anyhow::{bail, Context, Result};
use crate::*;
use std::fs;
use std::io::{self, Read, Write};

pub fn cli_template(
  handle_extensions: for<'a> fn(opts: &'a options::Opts, ast: AST<'a>) -> Result<AST<'a>>,
) -> Result<()> {
  simple_logger::init()?;
  let opts = options::from_args();

  let reader: Box<dyn Reader> = match opts.readerfmt {
    options::Reader::Asciidoc => Box::new(AsciidocReader::new()),
    options::Reader::Json => Box::new(JsonReader::new()),
  };

  // read the input
  let input = match &opts.input {
    Some(input) => fs::read_to_string(input).context("Could not read in file")?,
    None => {
      let mut input = String::new();
      io::stdin()
        .read_to_string(&mut input)
        .context("Could not read stdin")?;
      input
    }
  };

  let mut env = util::Env::Io(util::Io::new());
  let ast = reader.parse(&input, &opts, &mut env)?;

  let ast = handle_extensions(&opts, ast)?;

  let output: Box<dyn Write> = match &opts.output {
    Some(output) => Box::new(fs::File::create(output).context("Could not open output file")?),
    None => Box::new(io::stdout()),
  };

  match opts.writerfmt {
    options::Writer::Html5 => HtmlWriter::new().write(ast, &opts, output)?,
    options::Writer::Json => JsonWriter::new().write(ast, &opts, output)?,
    options::Writer::Docx => match &opts.output {
      Some(output) => {
        DocxWriter::new().write(
          ast,
          &opts,
          fs::File::create(output).context("Could not open output file")?,
        )?;
      }
      None => bail!("docx cant only be written to file not to stdout"),
    },
    _ => bail!("not yet supported"),
  };

  Ok(())
}

pub fn cli_no_extensions() -> Result<()> {
  cli_template(no_extensions)
}

fn no_extensions<'a>(_opts: &'a options::Opts, ast: AST<'a>) -> Result<AST<'a>> {
  Ok(ast)
}

use anyhow::{bail, Context, Result};
use crate::*;
use std::fs;
use std::io::{self, Read, Write};

pub fn cli_template(
  handle_extensions: for<'a> fn(
    opts: &'a options::Opts,
    env: &mut util::Env,
    ast: AST<'a>,
  ) -> Result<AST<'a>>,
) -> Result<()> {
  simple_logger::init()?;
  let opts = options::from_args();

  let reader: Box<dyn Reader> = match opts.readerfmt {
    options::Reader::Asciidoc => Box::new(AsciidocReader::new()),
    options::Reader::Markdown => Box::new(MarkdownReader::new()),
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

  let mut env = if opts.dry_run {
    util::Env::FakeOutput(util::FakeOutput::new())
  } else {
    util::Env::Io(util::Io::new())
  };
  let ast = reader.parse(&input, &opts, &mut env)?;

  let ast = handle_extensions(&opts, &mut env, ast)?;

  let output: Box<dyn Write> = if opts.dry_run {
    Box::new(io::sink())
  } else {
    match &opts.output {
      Some(output) => Box::new(fs::File::create(output).context("Could not open output file")?),
      None => Box::new(io::stdout()),
    }
  };

  match opts.writerfmt {
    options::Writer::Html5 => HtmlWriter::new().write(ast, &opts, output)?,
    options::Writer::Json => JsonWriter::new().write(ast, &opts, output)?,
    options::Writer::Asciidoc => AsciidocWriter::new().write(ast, &opts, output)?,
    options::Writer::Docx => match &opts.output {
      Some(output) => {
        DocxWriter::new().write(
          ast,
          &opts,
          fs::File::create(output).context("Could not open output file")?,
        )?;
      }
      None => bail!("docx can only be written to file not to stdout"),
    },
    _ => bail!("not yet supported"),
  };

  if opts.dry_run {
    io::stdout().write_all(serde_json::to_string_pretty(&env.get_cache())?.as_bytes())?;
  }

  Ok(())
}

pub fn cli_no_extensions() -> Result<()> {
  cli_template(no_extensions)
}

fn no_extensions<'a>(
  _opts: &'a options::Opts,
  _env: &mut util::Env,
  ast: AST<'a>,
) -> Result<AST<'a>> {
  Ok(ast)
}

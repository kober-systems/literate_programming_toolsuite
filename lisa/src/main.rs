extern crate asciidoctrine;
extern crate lisa;
extern crate simple_logger;

use anyhow::{bail, Context, Result};
use asciidoctrine::*;
use lisa::*;
use std::fs;
use std::io::{self, BufWriter, Read, Write};

fn main() -> Result<()> {
  simple_logger::init()?;
  let mut opts = options::from_args();

  let reader: Box<dyn Reader> = match opts.readerfmt {
    options::Reader::Asciidoc => Box::new(AsciidocReader::new()),
    options::Reader::Json => Box::new(JsonReader::new()),
  };

  let writer: Box<dyn Writer<BufWriter<Box<dyn Write>>>> = match opts.writerfmt {
    options::Writer::Html5 => Box::new(HtmlWriter::new()),
    options::Writer::Json => Box::new(JsonWriter::new()),
    _ => bail!("not yet supported"),
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

  let mut ast = reader.parse(&input)?;

  // TODO bei diesem Programm gehen wir davon aus,
  // das lisa gewünscht ist.
  // TODO Trotzdem sollte die Extension nur angehängt
  // werden, wenn sie nicht bereits über die Kommandozeile
  // definiert wurde.
  opts.extensions.push("lisa".to_string());

  for extension in opts.extensions.iter() {
    if extension == "lisa" {
      // TODO Lisa ist vordefiniert
      let mut lisa = Lisa::new();
      ast = lisa.transform(ast)?;
    }
    // TODO Ansicht
    else {
      // TODO commandozeilen Programm extension
    }
  }

  // TODO Wenn Erweiterungen in den Kommandozeilenparametern angegeben sind
  // diese in einer Schleife den AST manipulieren lassen
  // TODO Es sollte zwei Arten von Erweiterungen geben:
  // * Die ersten sind Kommandozeilenprogramme, die auf stdin Json bekommen und auf Stdout
  //   wieder Json ausgeben. Diese sollten auf der Kommandozwile parameter übergeben bkommen
  //   können.
  // * Die zweiten sind (lua)-Scripte, die den AST als Struktur übergeben bekommen und wieder
  //   einen AST zurückgeben.

  let output: Box<dyn Write> = match &opts.output {
    Some(output) => Box::new(fs::File::create(output).context("Could not open output file")?),
    None => Box::new(io::stdout()),
  };
  let output = BufWriter::new(output);
  writer.write(ast, output)?;

  Ok(())
}

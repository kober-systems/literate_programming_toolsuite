extern crate asciidoctrine;

use anyhow::{Context, Result};
use asciidoctrine::*;
use std::fs;
use std::io::{self, Read};

fn main() -> Result<()> {
  let opts = options::from_args();

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
  // TODO Den Text parsen
  let ast = asciidoctrine::parse_ast(&input)?;

  // TODO Wenn Erweiterungen in den Kommandozeilenparametern angegeben sind
  // diese in einer Schleife den AST manipulieren lassen
  // TODO Es sollte zwei Arten von Erweiterungen geben:
  // * Die ersten sind Kommandozeilenprogramme, die auf stdin Json bekommen und auf Stdout
  //   wieder Json ausgeben. Diese sollten auf der Kommandozwile parameter übergeben bkommen
  //   können.
  // * Die zweiten sind (lua)-Scripte, die den AST als Struktur übergeben bekommen und wieder
  //   einen AST zurückgeben.

  // TODO Das Ausgabeformat festlegen
  //println!("{:#?}", opts);
  //println!("\n\n{:#?}", ast);

  Ok(())
}

use anyhow::Result;
use asciidoctrine::*;
use lisi::*;

fn run_lisi<'a>(opts: &'a options::Opts, ast: AST<'a>) -> Result<AST<'a>> {
  let env = if opts.dry_run {
    util::Env::FakeOutput(util::FakeOutput::new())
  } else {
    util::Env::Io(util::Io::new())
  };
  let mut lisi = Lisi::from_env(env);
  Ok(lisi.transform(ast)?)
}

fn main() -> Result<()> {
  cli_template::cli_template(run_lisi)
}

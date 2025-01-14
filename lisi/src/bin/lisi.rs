use anyhow::Result;
use asciidoctrine::*;
use lisi::*;

fn run_lisi<'a>(_opts: &'a options::Opts, env: &mut util::Env, ast: AST<'a>) -> Result<AST<'a>> {
  let mut lisi = Lisi::from_env(env);
  Ok(lisi.transform(ast)?)
}

fn main() -> Result<()> {
  cli_template::cli_template(run_lisi)
}

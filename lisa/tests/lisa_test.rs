use anyhow::Result;
use asciidoctrine::{self, *};
use lisa::*;
use pretty_assertions::assert_eq;

#[test]
fn use_snippets() -> Result<()> {
  let content = r#"
We need the testmodule for this project.

[[sample1_required_modules]]
[source, lua]
----
require "testmodule"
----

This is the importing file. We could print out the version.

[source, lua, save]
.sample1.lua
----
<<sample1_required_modules>>

print(testmodule.version)
----
  "#;
  let reader = AsciidocReader::new();
  let ast = reader.parse(content)?;

  let mut lisa = Lisa::from_env(util::Env::Cache(util::Cache::new()));
  let _ast = lisa.transform(ast)?;

  // TODO ast vergleichen

  let mut outputs = lisa.into_cache().unwrap();

  assert_eq!(
    outputs.remove("sample1.lua").unwrap(),
    r#"require "testmodule"

print(testmodule.version)
"#
  );
  assert!(outputs.is_empty());

  Ok(())
}

#[test]
fn handle_snippet_order() -> Result<()> {
  let content = r#"
First we give a short outline of the program. It imports the required modules
and then prints out its version.

[source, lua, save]
.sample2.lua
----
<<sample2_required_modules>>

print(testmodule.version)
----

We need the testmodule for this project.

[[sample2_required_modules]]
[source, lua]
----
require "testmodule"
----
  "#;
  let reader = AsciidocReader::new();
  let ast = reader.parse(content)?;

  let mut lisa = Lisa::from_env(util::Env::Cache(util::Cache::new()));
  let _ast = lisa.transform(ast)?;

  // TODO ast vergleichen

  let mut outputs = lisa.into_cache().unwrap();

  assert_eq!(
    outputs.remove("sample2.lua").unwrap(),
    r#"require "testmodule"

print(testmodule.version)
"#
  );
  assert!(outputs.is_empty());

  Ok(())
}

#[test]
fn use_snippet_multiple_times() -> Result<()> {
  let content = r#"
Lets assume we want to use the following snippet in multiple places.

[[sample3_multiple]]
[source, lua]
----
require "testmodule"
----

Than we could import it in the same snippet multiple times.

[source, lua, save]
.sample3-1.lua
----
<<sample3_multiple>>

print(testmodule.version)

<<sample3_multiple>>
----

And we could even use it again in another snippet.

[source, lua, save]
.sample3-2.lua
----
<<sample3_multiple>>

print(testmodule.version .. "my other snippet")
----

  "#;
  let reader = AsciidocReader::new();
  let ast = reader.parse(content)?;

  let mut lisa = Lisa::from_env(util::Env::Cache(util::Cache::new()));
  let _ast = lisa.transform(ast)?;

  // TODO ast vergleichen

  let mut outputs = lisa.into_cache().unwrap();

  assert_eq!(
    outputs.remove("sample3-1.lua").unwrap(),
    r#"require "testmodule"

print(testmodule.version)

require "testmodule"
"#
  );
  assert_eq!(
    outputs.remove("sample3-2.lua").unwrap(),
    r#"require "testmodule"

print(testmodule.version .. "my other snippet")
"#
  );
  assert!(outputs.is_empty());

  Ok(())
}

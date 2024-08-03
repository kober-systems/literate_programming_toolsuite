#!/usr/bin/env -S deno run --allow-run --ext=ts

const files_modified_before = (await sh("git diff --name-only")).trim().split("\n");

const files_modified_by_lisi = {
  ...await json_sh("lisi --dry-run README.adoc"),
  ...await json_sh("lisi --dry-run asciidoctrine.adoc", "asciidoctrine"),
  ...await json_sh("lisi --dry-run lisi.adoc", "lisi"),
};

var files_in_conflict = [];
for (let path of files_modified_before) {
  if (path in files_modified_by_lisi) {
    files_in_conflict.push(path);
  }
}
if (files_in_conflict.length > 0) {
  console.log("could not build because some changes would be overwritten");
  console.log(files_in_conflict);
  // TODO show a diff?
  Deno.exit(ERR_CONFLICTING_MODIFICATIONS);
}

///////////////////////////////////////////////////////////
// constants
///////////////////////////////////////////////////////////

const ERR_CONFLICTING_MODIFICATIONS = 1;

///////////////////////////////////////////////////////////
// helper functions
///////////////////////////////////////////////////////////

async function sh(cmd: string, cwd?: string): string {
  const [command, ...args] = cmd.split(" ");
  return await run_cmd(command, args, cwd);
}

async function run_cmd(cmd: string, args?: [string], cwd?: string): string {
  const command = new Deno.Command(cmd, {
    args: args,
    cwd: cwd,
  });

  // create subprocess and collect output
  const { code, stdout, stderr } = await command.output();

  return new TextDecoder().decode(stdout);
}

async function json_sh(cmd: string, cwd?: string) {
  const json = JSON.parse(await sh(cmd, cwd));
  if (typeof cwd !== 'undefined') {
    const prefix = cwd.endsWith("/") ? cwd : cwd + "/";
    var out = {};
    for (let [key, value] of Object.entries(json)) {
      out[prefix + key] = value;
    };
    return out;
  }
  return json;
}


#!/usr/bin/env -S deno run --allow-run --allow-read --allow-write=.litstate --ext=ts

const LITERATE_SOURCES = {
  "README.adoc": "",
  "asciidoctrine/asciidoctrine.adoc": "asciidoctrine",
  "lisi/lisi.adoc": "lisi",
};

///////////////////////////////////////////////////////////
// constants
///////////////////////////////////////////////////////////

const ERR_CONFLICTING_MODIFICATIONS = 1;

///////////////////////////////////////////////////////////
// main code
///////////////////////////////////////////////////////////

let state;
try {
  state = await Deno.readTextFile(".litstate");
} catch(e) {
  // if the state is unknown we assume manual changes to
  // the code
  state = "manual code changes";
}

const files_modified_before = (await sh("git diff --name-only")).trim().split("\n");

const files_modified_by_lisi = {
  ...await json_sh("lisi --dry-run README.adoc"),
  ...await json_sh("lisi --dry-run asciidoctrine.adoc", "asciidoctrine"),
  ...await json_sh("lisi --dry-run lisi.adoc", "lisi"),
};

var literate_sources_unchanged = true;
var files_in_conflict = [];
for (let path of files_modified_before) {
  if (path in files_modified_by_lisi) {
    files_in_conflict.push(path);
  }
  if (path in LITERATE_SOURCES) {
    console.log("found", path);
    literate_sources_unchanged = false;
  }
}

if (files_in_conflict.length > 0) {
  if (state != "literate source changes") {
    console.log("could not build because some changes would be overwritten");
    console.log(files_in_conflict);
    // TODO show a diff?
    await Deno.writeTextFile(".litstate", "manual code changes");
    Deno.exit(ERR_CONFLICTING_MODIFICATIONS);
  } else {
    console.log("changing files: ", files_modified_by_lisi);
    await Deno.writeTextFile(".litstate", "literate source changes");
  }
} else {
  if (files_modified_before.length == 0 || literate_sources_unchanged) {
    console.log("everything is in sync");
    await Deno.writeTextFile(".litstate", "sync");
  } else {
    console.log("changing files: ", files_modified_by_lisi);
    await Deno.writeTextFile(".litstate", "literate source changes");
  }
}

console.log("checking done");

///////////////////////////////////////////////////////////
// helper functions
///////////////////////////////////////////////////////////

async function sh(cmd: string, cwd?: string, print_stderr?: boolean): string {
  const [command, ...args] = cmd.split(" ");
  return await run_cmd(command, args, cwd, print_stderr);
}

async function run_cmd(cmd: string, args?: [string], cwd?: string, print_stderr?: boolean): string {
  const command = new Deno.Command(cmd, {
    args: args,
    cwd: cwd,
  });
  if (typeof print_stderr === 'undefined') {
    print_stderr = true;
  }

  // create subprocess and collect output
  const { code, stdout, stderr } = await command.output();
  if (print_stderr) {
    const errlog = await new TextDecoder().decode(stderr);
    if (errlog.length > 0) {
      await console.log(errlog);
    }
  }

  return new TextDecoder().decode(stdout);
}

async function json_sh(cmd: string, cwd?: string, print_stderr?: boolean) {
  const json = JSON.parse(await sh(cmd, cwd, print_stderr));
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


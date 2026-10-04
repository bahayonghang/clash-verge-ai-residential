"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { test } = require("node:test");
const { spawnSync } = require("node:child_process");
const { PUBLIC_FILES, TEMPLATES, ASSETS, INIT_ARGS, bootstrap, parseArgs, entryCommand, runEntry } = require("../scripts/bootstrap-harnesses.js");
const { OVERRIDES, checkContract } = require("../scripts/check-agent-contract.js");

const ROOT = path.resolve(__dirname, "..");
const VERSION = fs.readFileSync(path.join(ROOT, ".trellis/.version"), "utf8").trim();

function write(root, file, bytes) {
  const target = path.join(root, file);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, bytes);
}

function fixture(t) {
  const parent = fs.mkdtempSync(path.join(os.tmpdir(), "trellis-bootstrap-test-"));
  t.after(() => fs.rmSync(parent, { recursive: true, force: true }));
  const root = path.join(parent, "candidate");
  const entry = path.join(parent, "fixed cli.js");
  fs.writeFileSync(entry, "// 合成入口由可注入执行器处理。\n");
  const calls = [];
  const run = (selected, args, cwd) => {
    calls.push({ selected, args: [...args], cwd });
    if (args[0] === "--version") return { status: 0, stdout: VERSION + "\n" };
    assert.deepEqual(args, INIT_ARGS);
    for (const file of ASSETS) write(cwd, file, "fixture\n");
    write(cwd, ".trellis/.template-hashes.json", "{}\n");
    return { status: 0, stdout: "fixture init\n" };
  };
  return { parent, root, entry, calls, run };
}

function existingOverrides(root, selected = OVERRIDES) {
  for (const file of selected) {
    const index = OVERRIDES.indexOf(file);
    const source = fs.readFileSync(path.join(ROOT, "scripts/harness-templates", TEMPLATES[index]), "utf8");
    write(root, file, source.replace(/\r?\n/g, "\r\n"));
  }
}

test("缺失四覆盖时部署真实公开模板并通过原检查器", (t) => {
  const f = fixture(t);
  const result = bootstrap(f, { run: f.run });
  assert.equal(result.status, "PASS", JSON.stringify(result));
  assert.equal(result.ok, true);
  assert.equal(result.publicFiles.length, 67);
  assert.equal(result.preservation, "PASS");
  assert.deepEqual(checkContract(f.root), { ok: true, errors: [] });
  assert.deepEqual(result.overrides.map((item) => item.source), OVERRIDES.map(() => "created_project_default"));
  assert.equal(f.calls.length, 2);
  assert.equal(f.calls[0].selected, f.calls[1].selected);
  assert.equal(f.calls[1].cwd, f.root);
  for (const file of OVERRIDES.slice(1)) {
    const source = fs.readFileSync(path.join(f.root, file), "utf8");
    assert.match(source, /Kimi supports project custom agents/);
    assert.match(source, /built-in `coder`/);
    assert.match(source, /ask the main session to confirm the dispatch target/);
    assert.doesNotMatch(source, /does not support project custom agents|fix all issues yourself/i);
  }
});

test("既有四覆盖包含 CRLF 时保留全部字节和 hash", (t) => {
  const f = fixture(t);
  existingOverrides(f.root);
  const before = OVERRIDES.map((file) => fs.readFileSync(path.join(f.root, file)));
  const result = bootstrap(f, { run: f.run });
  assert.equal(result.ok, true, JSON.stringify(result));
  result.overrides.forEach((item, index) => {
    assert.equal(item.source, "preserved_existing");
    assert.equal(item.beforeSha256, item.afterSha256);
    assert.deepEqual(fs.readFileSync(path.join(f.root, item.path)), before[index]);
  });
});

test("混合覆盖只创建缺失项，既有公开合同逐字保留", (t) => {
  const f = fixture(t);
  existingOverrides(f.root, [OVERRIDES[0], OVERRIDES[2]]);
  const publicBytes = fs.readFileSync(path.join(ROOT, "AGENTS.md"));
  write(f.root, "AGENTS.md", publicBytes);
  const result = bootstrap(f, { run: f.run });
  assert.equal(result.ok, true, JSON.stringify(result));
  assert.deepEqual(result.overrides.map((item) => item.source), [
    "preserved_existing", "created_project_default", "preserved_existing", "created_project_default"
  ]);
  assert.deepEqual(fs.readFileSync(path.join(f.root, "AGENTS.md")), publicBytes);
});

test("既有覆盖无效时原检查器失败，字节保留且不执行 init", (t) => {
  const f = fixture(t);
  const bytes = Buffer.from("[agents]\r\nmax_depth = 9\r\n");
  write(f.root, OVERRIDES[0], bytes);
  const result = bootstrap(f, { run: f.run });
  assert.equal(result.status, "CONTRACT_FAIL");
  assert.equal(result.init, null);
  assert.ok(result.contractBefore.errors.some((error) => error.code === "DEPTH"));
  assert.equal(f.calls.length, 1);
  assert.deepEqual(fs.readFileSync(path.join(f.root, OVERRIDES[0])), bytes);
});

for (const [name, versionResult] of [
  ["版本不符", { status: 0, stdout: "0.0.1\n" }],
  ["未知版本", { status: 0, stdout: "not a version\n" }],
  ["首入口失败", { status: 7, stdout: VERSION + "\n" }],
  ["版本信号", { status: 0, signal: "SIGTERM", stdout: VERSION + "\n" }],
  ["版本超时", { error: Object.assign(new Error("fixture timeout"), { code: "ETIMEDOUT" }) }]
]) {
  test(name + "在部署前阻断，不回落入口", (t) => {
    const f = fixture(t);
    const result = bootstrap(f, { run: (...args) => { f.calls.push(args); return versionResult; } });
    assert.equal(result.status, "BLOCKED");
    assert.equal(result.init, null);
    assert.equal(f.calls.length, 1);
    assert.equal(fs.existsSync(f.root), false);
  });
}

for (const [name, initResult] of [
  ["非零退出", { status: 17 }], ["信号", { status: 0, signal: "SIGTERM" }],
  ["超时", { error: Object.assign(new Error("fixture timeout"), { code: "ETIMEDOUT" }) }]
]) {
  test("init " + name + "不能由合同和资产成功覆盖", (t) => {
    const f = fixture(t);
    const result = bootstrap(f, { run: (...args) => {
      const normal = f.run(...args);
      return args[1][0] === "init" ? { ...normal, ...initResult } : normal;
    } });
    assert.equal(result.ok, false);
    assert.equal(result.status, "INIT_FAIL");
    assert.equal(result.contractAfter.ok, true);
    assert.equal(result.preservation, "PASS");
  });
}

test("执行器抛出 init 错误时保留错误并失败", (t) => {
  const f = fixture(t);
  const result = bootstrap(f, { run: (...args) => {
    if (args[1][0] === "init") throw Object.assign(new Error("fixture runner error"), { code: "EACCES" });
    return f.run(...args);
  } });
  assert.equal(result.status, "INIT_FAIL");
  assert.equal(result.init.error.code, "EACCES");
});

test("init 改写覆盖或公开合同分别导致保存失败，不自动修复", (t) => {
  for (const file of [OVERRIDES[0], "AGENTS.md"]) {
    const f = fixture(t);
    const result = bootstrap(f, { run: (...args) => {
      const normal = f.run(...args);
      if (args[1][0] === "init") fs.appendFileSync(path.join(f.root, file), "\nfixture changed\n");
      return normal;
    } });
    assert.equal(result.status, "PRESERVATION_FAIL");
    assert.match(fs.readFileSync(path.join(f.root, file), "utf8"), /fixture changed/);
  }
});

test("init 缺平台资产时独立失败", (t) => {
  const f = fixture(t);
  const result = bootstrap(f, { run: (...args) => {
    const normal = f.run(...args);
    if (args[1][0] === "init") fs.unlinkSync(path.join(f.root, ASSETS[0]));
    return normal;
  } });
  assert.equal(result.status, "ASSET_FAIL");
  assert.equal(result.contractAfter.ok, true);
});

test("完成候选再次调用拒绝，既有文件完全不改且不重跑 init", (t) => {
  const f = fixture(t);
  assert.equal(bootstrap(f, { run: f.run }).ok, true);
  const before = [...PUBLIC_FILES, ...OVERRIDES, ...ASSETS].map((file) => fs.readFileSync(path.join(f.root, file)));
  const calls = f.calls.length;
  const result = bootstrap(f, { run: f.run });
  assert.equal(result.status, "BLOCKED");
  assert.equal(f.calls.length, calls);
  [...PUBLIC_FILES, ...OVERRIDES, ...ASSETS].forEach((file, index) => {
    assert.deepEqual(fs.readFileSync(path.join(f.root, file)), before[index]);
  });
});

test("当前树、祖先、非临时目标、私有文件和已有生成目录均在写入前拒绝", (t) => {
  const f = fixture(t);
  for (const root of [ROOT, path.dirname(ROOT), path.join(ROOT, "candidate"), os.tmpdir(), "relative-target"]) {
    const result = bootstrap({ root, entry: f.entry }, { run: f.run });
    assert.equal(result.status, "BLOCKED");
  }
  for (const file of ["clash-verge-ai-residential.local.toml", ".claude/settings.json", "monitor.sqlite3"]) {
    const candidate = path.join(f.parent, file.replace(/[^a-z]/g, "") + "-candidate");
    write(candidate, file, "private fixture\n");
    const result = bootstrap({ root: candidate, entry: f.entry }, { run: f.run });
    assert.equal(result.status, "BLOCKED");
    assert.deepEqual(fs.readdirSync(candidate), [file.split("/")[0]]);
  }
  assert.equal(f.calls.length, 0);
});

test("目标 junction 与覆盖硬链接拒绝且链接目标字节保留", (t) => {
  const f = fixture(t);
  const destination = path.join(f.parent, "outside");
  fs.mkdirSync(destination);
  fs.symlinkSync(destination, f.root, process.platform === "win32" ? "junction" : "dir");
  assert.equal(bootstrap(f, { run: f.run }).status, "BLOCKED");
  assert.deepEqual(fs.readdirSync(destination), []);
  fs.unlinkSync(f.root);
  const linked = path.join(f.parent, "linked-file");
  fs.writeFileSync(linked, "private fixture\n");
  fs.mkdirSync(path.join(f.root, ".codex"), { recursive: true });
  fs.linkSync(linked, path.join(f.root, OVERRIDES[0]));
  assert.equal(bootstrap(f, { run: f.run }).status, "BLOCKED");
  assert.equal(fs.readFileSync(linked, "utf8"), "private fixture\n");
  assert.equal(f.calls.length, 0);
});

test("缺失公开来源在部署前阻断", (t) => {
  const f = fixture(t);
  const sourceRoot = path.join(f.parent, "public-source");
  fs.mkdirSync(sourceRoot);
  const result = bootstrap(f, { sourceRoot, run: f.run });
  assert.equal(result.status, "BLOCKED");
  assert.equal(fs.existsSync(f.root), false);
  assert.equal(f.calls.length, 0);
});

for (const [name, content] of [["缺失", null], ["无效", "not a version\n"]]) {
  test("公开项目版本" + name + "时在部署前阻断", (t) => {
    const f = fixture(t);
    const sourceRoot = path.join(f.parent, "public-source");
    for (const file of PUBLIC_FILES.filter((item) => item !== ".trellis/.version")) {
      write(sourceRoot, file, fs.readFileSync(path.join(ROOT, file)));
    }
    for (const file of TEMPLATES) {
      write(sourceRoot, "scripts/harness-templates/" + file, fs.readFileSync(path.join(ROOT, "scripts/harness-templates", file)));
    }
    if (content !== null) write(sourceRoot, ".trellis/.version", content);
    const result = bootstrap(f, { sourceRoot, run: f.run });
    assert.equal(result.status, "BLOCKED");
    assert.equal(fs.existsSync(f.root), false);
    assert.equal(f.calls.length, 0);
  });
}

test("已有公开合同漂移在部署前拒绝且字节保留", (t) => {
  const f = fixture(t);
  write(f.root, "AGENTS.md", "fixture mismatch\n");
  const result = bootstrap(f, { run: f.run });
  assert.equal(result.status, "BLOCKED");
  assert.equal(fs.readFileSync(path.join(f.root, "AGENTS.md"), "utf8"), "fixture mismatch\n");
  assert.equal(fs.existsSync(path.join(f.root, ".codex")), false);
  assert.equal(f.calls.length, 0);
});

test("显式 JS 入口以 argv 执行，shell 特殊字符不解释", (t) => {
  const f = fixture(t);
  const entry = path.join(f.parent, "cli & literal.js");
  fs.writeFileSync(entry, "console.log(JSON.stringify(process.argv.slice(2)));\n");
  const args = ["value ; & ` $()", "two words"];
  const result = runEntry(entry, args, f.parent);
  assert.equal(result.status, 0);
  assert.deepEqual(JSON.parse(result.stdout), args);
  assert.deepEqual(entryCommand(entry), { command: process.execPath, prefix: [entry] });
  for (const suffix of ["cmd", "bat", "ps1"]) {
    const wrapper = path.join(f.parent, "trellis." + suffix);
    fs.writeFileSync(wrapper, "fixture\n");
    assert.throws(() => entryCommand(wrapper), /shell 包装器/);
  }
});

test("CLI 无显式参数时退出失败，导入不运行，重复参数拒绝", () => {
  const result = spawnSync(process.execPath, [path.join(ROOT, "scripts/bootstrap-harnesses.js")], { encoding: "utf8" });
  assert.equal(result.status, 1);
  assert.equal(JSON.parse(result.stdout).status, "BLOCKED");
  assert.throws(() => parseArgs(["node", "script", "--root", "one", "--root", "two"]), /重复/);
  assert.throws(() => parseArgs(["node", "script", "--install"]), /未知/);
  assert.deepEqual(parseArgs(["node", "script", "--help"]), { help: true });
});

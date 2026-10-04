"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const { test } = require("node:test");
const checker = require("../scripts/check-harness-environment.js");

function fixture(t, version = "9.8.7-beta.2") {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "harness-environment-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  fs.mkdirSync(path.join(root, ".trellis"));
  fs.writeFileSync(path.join(root, ".trellis", ".version"), version + "\n");
  for (const rel of checker.OVERRIDES) {
    fs.mkdirSync(path.dirname(path.join(root, rel)), { recursive: true });
    fs.writeFileSync(path.join(root, rel), "fixture override\n");
  }
  const calls = [];
  const dependencies = {
    discover: (tool) => [path.join(root, tool)],
    run: (entry) => {
      calls.push(entry);
      return { status: 0, stdout: version + "\n", stderr: "" };
    }
  };
  return { root, version, calls, dependencies };
}

test("版本解析忽略升级警告中的项目版本，只使用明确的版本行", () => {
  assert.equal(checker.parseVersion("trellis", "Your CLI (1.2.3) is older than project (9.8.7)\nRun: trellis upgrade\n\n1.2.3\n"), "1.2.3");
  assert.equal(checker.parseVersion("trellis", "project version 9.8.7"), null);
  assert.equal(checker.parseVersion("trellis", "1.2.3\n9.8.7\n"), null);
  assert.equal(checker.parseVersion("claude", "2.3.4 (Claude Code)"), "2.3.4");
  assert.equal(checker.parseVersion("codex", "codex-cli 3.4.5"), "3.4.5");
  assert.equal(checker.parseVersion("grok", "grok 4.5.6 (abcdef) [alpha]"), "4.5.6");
  assert.equal(checker.parseVersion("kimi", "5.6.7"), "5.6.7");
  assert.equal(checker.parseVersion("omp", "omp/6.7.8"), "6.7.8");
});

test("Windows 发现与版本探针保留中文和单引号路径", { skip: process.platform !== "win32" }, (t) => {
  const f = fixture(t);
  const directory = path.join(f.root, "中文's tool");
  const entry = path.join(directory, "trellis.ps1");
  fs.mkdirSync(directory);
  // Windows PowerShell 读取 UTF-8 脚本需要 BOM；进程输出单独按 UTF-8 解码。
  fs.writeFileSync(entry, "\ufeffWrite-Output '" + f.version + "'; [Console]::Error.WriteLine('中文诊断')", "utf8");
  const originalPath = process.env.PATH;
  try {
    process.env.PATH = directory + path.delimiter + originalPath;
    const entries = checker.discoverEntries("trellis", f.root);
    // os.tmpdir() 可能是 8.3 短路径，Get-Command 返回长路径。
    const samePath = (target) => fs.realpathSync.native(target).replace(/^\\\\\?\\/, "").toLowerCase();
    assert.equal(samePath(entries[0]), samePath(entry));
    assert.match(entries[0], /中文's tool[\\/]trellis\.ps1$/);
    const result = checker.probeEntry("trellis", entries[0], f.root, checker.runVersion);
    assert.equal(result.status, "available", JSON.stringify(result));
    assert.equal(result.version, f.version);
    assert.equal(result.stderr.trim(), "中文诊断");
  } finally {
    if (originalPath === undefined) delete process.env.PATH;
    else process.env.PATH = originalPath;
  }
});

test("项目版本匹配时只报告 bootstrap 前提，不执行 init", (t) => {
  const f = fixture(t);
  const before = checker.OVERRIDES.map((rel) => fs.readFileSync(path.join(f.root, rel)));
  const result = checker.checkEnvironment({ root: f.root }, f.dependencies);
  assert.equal(result.ok, true);
  assert.equal(result.project.expectedTrellisVersion, f.version);
  assert.equal(result.bootstrap.status, "READY");
  assert.equal(result.bootstrap.executesBootstrap, false);
  assert.equal(result.permissions.projectTrust, "UNVERIFIED");
  assert.equal(result.permissions.hooks, "UNVERIFIED");
  assert.deepEqual(f.calls, checker.TOOLS.map((tool) => path.join(f.root, tool)));
  assert.deepEqual(checker.OVERRIDES.map((rel) => fs.readFileSync(path.join(f.root, rel))), before);
});

test("版本不匹配阻断 bootstrap，不能采用警告中的匹配版本", (t) => {
  const f = fixture(t);
  f.dependencies.run = () => ({ status: 0, stdout: "Project " + f.version + "\n1.2.3\n" });
  const result = checker.checkEnvironment({ root: f.root }, f.dependencies);
  assert.equal(result.ok, false);
  assert.equal(result.bootstrap.status, "BLOCKED");
  assert.ok(result.bootstrap.blockers.includes("所选 Trellis 版本与项目不匹配"));
});

for (const [name, output, expected] of [
  ["入口缺失", { error: { code: "ENOENT", message: "missing" }, status: null }, "missing"],
  ["启动权限失败", { error: { code: "EACCES", message: "denied" }, status: null }, "permission-denied"],
  ["非零但输出匹配版本", { status: 1, stdout: "9.8.7-beta.2" }, "failed"],
  ["包装器运行失败", { status: 1, stderr: "Missing optional dependency" }, "failed"],
  ["版本未知", { status: 0, stdout: "ready" }, "unknown-version"],
  ["版本只出现在 stderr", { status: 0, stdout: "", stderr: "9.8.7-beta.2" }, "unknown-version"],
  ["进程被信号终止", { status: null, signal: "SIGTERM" }, "failed"],
  ["版本探针超时", { error: { code: "ETIMEDOUT", message: "timeout" }, status: null }, "failed"]
]) {
  test(name + "阻断 bootstrap", (t) => {
    const f = fixture(t);
    f.dependencies.run = () => output;
    const result = checker.checkEnvironment({ root: f.root }, f.dependencies);
    assert.equal(result.ok, false);
    assert.equal(result.bootstrap.status, "BLOCKED");
    assert.equal(result.tools.find((item) => item.tool === "trellis").selected.status, expected);
  });
}

test("可用替代入口不掩盖首选包装器失败，bootstrap 与客户端分别报告", (t) => {
  const f = fixture(t);
  const alternative = path.join(f.root, "desktop-codex.exe");
  const run = f.dependencies.run;
  f.dependencies.run = (entry) => entry === path.join(f.root, "codex")
    ? { status: 1, stderr: "Missing optional dependency" } : run(entry);
  const result = checker.checkEnvironment({ root: f.root, alternatives: { codex: [alternative] } }, f.dependencies);
  const codex = result.tools.find((item) => item.tool === "codex");
  assert.equal(codex.selected.status, "failed");
  assert.equal(codex.alternatives[0].path, alternative);
  assert.equal(codex.alternatives[0].status, "available");
  assert.equal(result.bootstrap.status, "READY");
  assert.equal(result.ok, false);
});

test("显式入口不回落 PATH，发现失败和零候选均报告阻断", (t) => {
  const f = fixture(t);
  const explicit = path.join(f.root, "chosen.js");
  const options = { root: f.root, entries: Object.fromEntries(checker.TOOLS.map((tool) => [tool, explicit])) };
  f.dependencies.discover = () => { throw new Error("should not discover"); };
  assert.equal(checker.checkEnvironment(options, f.dependencies).ok, true);
  const failed = checker.checkEnvironment({ root: f.root }, f.dependencies);
  assert.equal(failed.bootstrap.status, "BLOCKED");
  assert.equal(failed.tools[0].discoveryError, "should not discover");
  f.dependencies.discover = () => [];
  const missing = checker.checkEnvironment({ root: f.root }, f.dependencies);
  assert.equal(missing.bootstrap.status, "BLOCKED");
  assert.equal(missing.tools[0].selected.path, null);
});

test("缺失项目版本不能放行 bootstrap；本机覆盖缺失只报告不阻断", (t) => {
  const f = fixture(t);
  fs.writeFileSync(path.join(f.root, ".trellis", ".version"), "unknown\n");
  assert.equal(checker.checkEnvironment({ root: f.root }, f.dependencies).bootstrap.status, "BLOCKED");
  fs.rmSync(path.join(f.root, ".trellis", ".version"));
  assert.equal(checker.checkEnvironment({ root: f.root }, f.dependencies).project.expectedTrellisVersion, null);
  fs.writeFileSync(path.join(f.root, ".trellis", ".version"), f.version);
  fs.rmSync(path.join(f.root, checker.OVERRIDES[0]));
  const result = checker.checkEnvironment({ root: f.root }, f.dependencies);
  assert.equal(result.bootstrap.status, "READY");
  assert.equal(result.project.overrides[0].present, false);
});

test("参数拒绝未知工具、相对入口、重复入口和执行型开关", () => {
  for (const args of [["--entry", "unknown=/bin/tool"], ["--entry", "trellis=relative.js"], ["--entry"], ["--init"], ["--install"]]) {
    assert.throws(() => checker.parseArgs(["node", "script", ...args]));
  }
  const value = "trellis=" + path.resolve("chosen.js");
  assert.throws(() => checker.parseArgs(["node", "script", "--entry", value, "--entry", value]));
  const parsed = checker.parseArgs(["node", "script", "--entry", value, "--alternative", "codex=" + path.resolve("app.exe")]);
  assert.equal(parsed.entries.trellis, path.resolve("chosen.js"));
});

test("显式入口和替代入口使用规范化绝对路径", () => {
  const entry = (path.resolve("folder") + path.sep + ".." + path.sep + "chosen.js").replaceAll("\\", "/");
  const parsed = checker.parseArgs(["node", "script", "--entry", "trellis=" + entry, "--alternative", "codex=" + entry]);
  assert.equal(parsed.entries.trellis, path.resolve(entry));
  assert.deepEqual(parsed.alternatives.codex, [path.resolve(entry)]);
});

test("CLI fixture 保留 JSON 与失败退出码，不依赖本机客户端", (t) => {
  const f = fixture(t);
  const available = path.join(f.root, "available.js");
  const broken = path.join(f.root, "broken.js");
  fs.writeFileSync(available, "console.log(" + JSON.stringify(f.version) + ");\n");
  fs.writeFileSync(broken, "console.log(" + JSON.stringify(f.version) + "); process.exitCode = 1;\n");
  const entries = checker.TOOLS.flatMap((tool) => ["--entry", tool + "=" + (tool === "trellis" ? broken : available)]);
  const result = spawnSync(process.execPath, [path.join(__dirname, "..", "scripts", "check-harness-environment.js"), "--root", f.root, ...entries], { encoding: "utf8" });
  assert.equal(result.status, 1);
  const report = JSON.parse(result.stdout);
  assert.equal(report.bootstrap.status, "BLOCKED");
  assert.equal(report.tools.find((item) => item.tool === "trellis").selected.exitCode, 1);
});

test("根质量门只接入环境检查语法与 fixture，不要求本机客户端可用", () => {
  const pkg = require("../package.json");
  assert.ok(pkg.scripts.check.includes("node --check scripts/check-harness-environment.js"));
  assert.ok(pkg.scripts.check.includes("node --check tests/check-harness-environment.test.js"));
  assert.ok(pkg.scripts.test.includes("tests/check-harness-environment.test.js"));
  assert.equal(/(?:^|&&)\s*node scripts\/check-harness-environment\.js/.test(pkg.scripts.check), false);
});

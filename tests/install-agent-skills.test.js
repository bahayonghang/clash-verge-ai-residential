"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { test } = require("node:test");

const installer = require("../scripts/install-agent-skills.js");
const { buildInputs } = require("../skills/residential-rule-tuning/scripts/build-inputs.js");

function makeRepo() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "install-skills-"));
  const source = path.join(root, "skills", "residential-rule-tuning");
  fs.mkdirSync(path.join(source, "scripts"), { recursive: true });
  fs.writeFileSync(path.join(source, "SKILL.md"), "# skill\n");
  fs.writeFileSync(path.join(source, "reference.md"), "# ref\n");
  fs.writeFileSync(path.join(source, "scripts", "build-inputs.js"), "module.exports = {};\n");
  fs.mkdirSync(path.join(root, ".claude"), { recursive: true });
  fs.mkdirSync(path.join(root, ".cursor"), { recursive: true });
  return root;
}

test("安装器写入已存在平台且幂等", () => {
  const root = makeRepo();
  const first = installer.install(root, { force: false, check: false }, new Date("2026-08-31T00:00:00Z"));
  assert.equal(first.written, 6);
  const second = installer.install(root, { force: false, check: false }, new Date("2026-08-31T00:00:00Z"));
  assert.equal(second.written, 0);
  assert.equal(
    fs.readFileSync(path.join(root, ".claude", "skills", "residential-rule-tuning", "SKILL.md"), "utf8"),
    "# skill\n"
  );
});

test("不创建缺失的平台目录", () => {
  const root = makeRepo();
  installer.install(root, { force: false, check: false }, new Date("2026-08-31T00:00:00Z"));
  assert.equal(fs.existsSync(path.join(root, ".omp")), false);
  assert.equal(fs.existsSync(path.join(root, ".grok")), false);
});

test("同名不同内容默认拒绝且不写入任何目标", () => {
  const root = makeRepo();
  installer.install(root, { force: false, check: false }, new Date("2026-08-31T00:00:00Z"));
  const dest = path.join(root, ".claude", "skills", "residential-rule-tuning", "SKILL.md");
  fs.writeFileSync(dest, "# changed\n");
  const cursor = path.join(root, ".cursor", "skills", "residential-rule-tuning", "SKILL.md");
  const beforeCursor = fs.readFileSync(cursor);
  assert.throws(
    () => installer.install(root, { force: false, check: false }, new Date("2026-08-31T00:00:00Z")),
    (error) => error.exitCode === 1
  );
  assert.equal(fs.readFileSync(dest, "utf8"), "# changed\n");
  assert.deepEqual(fs.readFileSync(cursor), beforeCursor);
});

test("--force 先备份再替换", () => {
  const root = makeRepo();
  const now = new Date("2026-08-31T12:34:56Z");
  installer.install(root, { force: false, check: false }, now);
  const dest = path.join(root, ".claude", "skills", "residential-rule-tuning", "SKILL.md");
  fs.writeFileSync(dest, "# old\n");
  installer.install(root, { force: true, check: false }, now);
  assert.equal(fs.readFileSync(dest, "utf8"), "# skill\n");
  const backup = `${dest}.bak-20260831T123456Z`;
  assert.equal(fs.readFileSync(backup, "utf8"), "# old\n");
});

test("--check 在有差异时非零退出", () => {
  const root = makeRepo();
  assert.throws(
    () => installer.install(root, { force: false, check: true }, new Date("2026-08-31T00:00:00Z")),
    (error) => error.exitCode === 1
  );
  installer.install(root, { force: false, check: false }, new Date("2026-08-31T00:00:00Z"));
  const ok = installer.install(root, { force: false, check: true }, new Date("2026-08-31T00:00:00Z"));
  assert.equal(ok.ok, true);
});

test("--platforms 只写入指定目录", () => {
  const root = makeRepo();
  const result = installer.install(
    root,
    {
      force: false,
      check: false,
      create: true,
      platforms: [".agents", ".claude"]
    },
    new Date("2026-08-31T00:00:00Z")
  );
  assert.deepEqual(result.platforms, [".agents", ".claude"]);
  assert.equal(
    fs.existsSync(path.join(root, ".agents", "skills", "residential-rule-tuning", "SKILL.md")),
    true
  );
  assert.equal(
    fs.existsSync(path.join(root, ".claude", "skills", "residential-rule-tuning", "SKILL.md")),
    true
  );
  assert.equal(
    fs.existsSync(path.join(root, ".cursor", "skills", "residential-rule-tuning", "SKILL.md")),
    false
  );
});

test("未知 --platforms 值被拒绝", () => {
  assert.throws(
    () => installer.parseArgs(["node", "install-agent-skills.js", "--platforms", ".foo"]),
    /未知平台目录/
  );
});

test("SKILL.md 含 YAML frontmatter 的 name 与 description", () => {
  const skill = fs
    .readFileSync(
      path.join(__dirname, "..", "skills", "residential-rule-tuning", "SKILL.md"),
      "utf8"
    )
    .replace(/\r\n/g, "\n");
  assert.ok(skill.startsWith("---\n"));
  const close = skill.indexOf("\n---\n", 4);
  assert.ok(close > 0, "frontmatter 未闭合");
  const frontmatter = skill.slice(4, close);
  assert.match(frontmatter, /^name:\s*residential-rule-tuning\s*$/m);
  assert.match(frontmatter, /^description:\s*/m);
  assert.match(frontmatter, /ResiWatch/);
  assert.match(frontmatter, /家宽/);
  assert.match(frontmatter, /Exclude|不要/);
});

test("SKILL.md 含六个规定小节", () => {
  const skill = fs.readFileSync(
    path.join(__dirname, "..", "skills", "residential-rule-tuning", "SKILL.md"),
    "utf8"
  );
  for (const heading of [
    "## 触发条件",
    "## 生成器用法",
    "## 命令顺序",
    "## 四类结果判读",
    "## 改动落点",
    "## 禁止项"
  ]) {
    assert.ok(skill.includes(heading), heading);
  }
});

test("生成器对 routing 表 21 个开关做完整性检查", () => {
  const built = buildInputs(path.join(__dirname, ".."));
  assert.equal(built.routingCount, 21);
  assert.equal(
    Object.keys(built.switches.supported).length + built.switches.unsupported.length,
    21
  );
  assert.ok(Array.isArray(built.rules.rules));
  assert.ok(built.rules.rules.length > 0);
  assert.ok(built.switches.unsupported.includes("openai_shared_dependencies"));
  assert.ok(built.switches.supported.openai_core.length > 0);
});

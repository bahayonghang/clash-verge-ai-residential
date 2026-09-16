"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { test } = require("node:test");

const installer = require("../scripts/install-agent-skills.js");
const { buildInputs } = require("../skills/residential-rule-tuning/scripts/build-inputs.js");
const { constants } = require("../clash-verge-ai-residential.js");

const REAL_SKILL_SOURCE = path.join(__dirname, "..", "skills", "residential-rule-tuning");
const SKILL_FILES = ["SKILL.md", "reference.md", "scripts/build-inputs.js"];
const EXPECTED_PLATFORM_ROOTS = [
  ".agents",
  ".claude",
  ".codex",
  ".cursor",
  ".omp",
  ".grok",
  ".kimi-code"
];

function makeRepo(platformRoots) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "install-skills-"));
  const source = path.join(root, "skills", "residential-rule-tuning");
  fs.mkdirSync(path.join(source, "scripts"), { recursive: true });
  fs.writeFileSync(path.join(source, "SKILL.md"), "# skill\n");
  fs.writeFileSync(path.join(source, "reference.md"), "# ref\n");
  fs.writeFileSync(path.join(source, "scripts", "build-inputs.js"), "module.exports = {};\n");
  const roots = platformRoots === undefined ? [".claude", ".cursor"] : platformRoots;
  for (const name of roots) {
    fs.mkdirSync(path.join(root, name), { recursive: true });
  }
  return root;
}

function makeRealRepo() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "install-skills-real-"));
  fs.cpSync(REAL_SKILL_SOURCE, path.join(root, "skills", "residential-rule-tuning"), { recursive: true });
  for (const name of EXPECTED_PLATFORM_ROOTS) {
    fs.mkdirSync(path.join(root, name), { recursive: true });
  }
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

test("--check 在零个平台根时成功，不证明已安装", () => {
  const root = makeRepo([]);
  const result = installer.install(root, { force: false, check: true }, new Date("2026-08-31T00:00:00Z"));
  assert.equal(result.ok, true);
  assert.equal(result.written, 0);
  assert.equal(result.skippedPlatforms, installer.PLATFORM_ROOTS.length);
  for (const name of installer.PLATFORM_ROOTS) {
    assert.equal(fs.existsSync(path.join(root, name)), false);
  }
});

test("真实 payload 写入全部七个平台且二次安装幂等", () => {
  assert.deepEqual(installer.PLATFORM_ROOTS, EXPECTED_PLATFORM_ROOTS);
  const live = installer.planInstall(path.join(__dirname, ".."), { force: false, check: true });
  assert.deepEqual(live.files, SKILL_FILES);

  const root = makeRealRepo();
  const options = { force: false, check: false };
  const now = new Date("2026-08-31T00:00:00Z");
  const first = installer.install(root, options, now);
  assert.equal(first.written, EXPECTED_PLATFORM_ROOTS.length * SKILL_FILES.length);
  assert.deepEqual(first.platforms, EXPECTED_PLATFORM_ROOTS);
  const checked = installer.install(root, { force: false, check: true }, now);
  assert.equal(checked.ok, true);
  assert.equal(checked.written, 0);
  for (const platform of EXPECTED_PLATFORM_ROOTS) {
    for (const rel of SKILL_FILES) {
      const repoBytes = fs.readFileSync(path.join(REAL_SKILL_SOURCE, ...rel.split("/")));
      const sourceBytes = fs.readFileSync(
        path.join(root, "skills", "residential-rule-tuning", ...rel.split("/"))
      );
      const destBytes = fs.readFileSync(
        path.join(root, platform, "skills", "residential-rule-tuning", ...rel.split("/"))
      );
      assert.ok(sourceBytes.equals(repoBytes), `${rel} 夹具不是仓库源文件`);
      assert.ok(destBytes.equals(repoBytes), `${platform}/${rel} 与源文件不一致`);
    }
  }
  const extra = path.join(root, ".claude", "skills", "residential-rule-tuning", "user-notes.md");
  fs.writeFileSync(extra, "keep extra\n");
  const afterExtra = installer.install(root, { force: false, check: true }, now);
  assert.equal(afterExtra.ok, true);
  const second = installer.install(root, options, now);
  assert.equal(second.written, 0);
  assert.equal(fs.readFileSync(extra, "utf8"), "keep extra\n");
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

test("生成器对 routing 表 25 个开关做完整性检查", () => {
  const built = buildInputs(path.join(__dirname, ".."));
  assert.equal(built.routingCount, 25);
  assert.equal(Object.keys(built.switches.supported).length, 13);
  assert.deepEqual(built.switches.unsupported, [
    "openai_shared_dependencies", "claude_shared_dependencies",
    "antigravity_google_auth", "antigravity_project_apis", "antigravity_update_and_telemetry",
    "cursor_process_fallback", "claude_code_auxiliary", "ai_process_fallback",
    "anthropic_ip_fallback", "shared_realtime_infrastructure", "global_realtime_ports",
    "public_encrypted_dns"
  ]);
  assert.equal(
    Object.keys(built.switches.supported).length + built.switches.unsupported.length,
    25
  );
  assert.ok(Array.isArray(built.rules.rules));
  assert.ok(built.rules.rules.length > 0);
  assert.ok(built.switches.unsupported.includes("openai_shared_dependencies"));
  assert.ok(built.switches.supported.openai_core.length > 0);
  assert.deepEqual(built.switches.supported.extra, ["anyrouter.top"]);
});

test("新增核心开关仅映射各自域名，不将 IP 或进程记为域名归属", () => {
  const built = buildInputs(path.join(__dirname, ".."));
  const expected = {
    anthropic_core: ["claude.ai", "claude.com", "claudemcpcontent.com", "claudeusercontent.com",
      "api.anthropic.com", "mcp-proxy.anthropic.com", "assets-proxy.anthropic.com"],
    gemini_api_core: ["generativelanguage.googleapis.com"],
    antigravity_core: ["cloudcode-pa.googleapis.com", "daily-cloudcode-pa.googleapis.com",
      "cloudaicompanion.googleapis.com", "antigravity.google"]
  };
  for (const [key, hosts] of Object.entries(expected)) {
    assert.deepEqual([...built.switches.supported[key]].sort(), [...hosts].sort(), key);
    for (const host of hosts) {
      assert.deepEqual(Object.entries(built.switches.supported)
        .filter(([, domains]) => domains.includes(host)).map(([owner]) => owner), [key], host);
      assert.ok(built.rules.rules.some((rule) =>
        rule === `DOMAIN,${host},${constants.AI_GROUP}` ||
        rule === `DOMAIN-SUFFIX,${host},${constants.AI_GROUP}`), host);
    }
  }
});

test("grok_web_assets 不把 auth.x.ai 算作该开关独有流量", () => {
  const built = buildInputs(path.join(__dirname, ".."));
  const grokCore = built.switches.supported.grok_core;
  const webAssets = built.switches.supported.grok_web_assets;
  assert.ok(Array.isArray(grokCore));
  assert.ok(Array.isArray(webAssets));
  for (const host of constants.GROK_EXACT_DOMAINS) {
    assert.equal(grokCore.includes(host), true, `grok_core 应包含 ${host}`);
    assert.equal(webAssets.includes(host), false, `grok_web_assets 不应包含 ${host}`);
  }
  const exclusive = webAssets.filter((host) => !grokCore.includes(host));
  for (const host of constants.GROK_EXACT_DOMAINS) {
    assert.equal(exclusive.includes(host), false, `${host} 不是 grok_web_assets 独有流量`);
  }
  assert.deepEqual(
    [...webAssets].sort(),
    [...constants.GROK_STRICT_EXACT_DOMAINS].sort()
  );
});

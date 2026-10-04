"use strict";

const fs = require("node:fs");
const path = require("node:path");

const DEFAULT_ROOT = path.resolve(__dirname, "..");
// 本机 harness 覆盖不进版本控制：存在才校验内容，干净 clone 缺失不算失败。
const OVERRIDES = [
  ".codex/config.toml",
  ...["implement", "check", "research"].map(
    (role) => `.kimi-code/skills/trellis-${role}/SKILL.md`
  )
];
const FILES = [
  "AGENTS.md",
  "CLAUDE.md",
  ".trellis/workflow.md",
  ".trellis/spec/frontend/index.md",
  ".trellis/spec/frontend/quality-guidelines.md",
  "docs/agents/harnesses.md",
  "skills/residential-rule-tuning/SKILL.md",
  ".github/workflows/ci.yml",
  "justfile",
  "package.json"
];
const STATUSES = [
  "no_task", "planning", "planning-inline", "in_progress",
  "in_progress-inline", "completed"
];

// 只检查确定性结构；语义与客户端运行证据仍由独立审查负责。
function checkContract(root = DEFAULT_ROOT) {
  const errors = [];
  const sources = {};
  const fail = (code, file, detail) => errors.push({ code, file, detail });
  for (const file of FILES) {
    try {
      sources[file] = fs.readFileSync(path.join(root, file), "utf8").replace(/\r\n/g, "\n");
    } catch (error) {
      fail("FILE", file, `无法读取必需文件：${error.code || error.message}`);
      sources[file] = "";
    }
  }
  // 未跟踪的本机覆盖：可读才纳入检查，缺失时静默跳过。
  for (const file of OVERRIDES) {
    try {
      sources[file] = fs.readFileSync(path.join(root, file), "utf8").replace(/\r\n/g, "\n");
    } catch (error) {
      if (error.code !== "ENOENT") {
        fail("FILE", file, `无法读取本机覆盖：${error.code || error.message}`);
      }
    }
  }
  const requirePattern = (file, pattern, code, detail) => {
    if (sources[file] === undefined) return;
    if (!pattern.test(sources[file].replace(/\s+/g, " "))) fail(code, file, detail);
  };
  const requireText = (file, text, code) => {
    if (sources[file] === undefined) return;
    if (!sources[file].replace(/\s+/g, " ").includes(text)) {
      fail(code, file, `缺少合同内容：${text}`);
    }
  };

  const agents = "AGENTS.md";
  const workflow = ".trellis/workflow.md";
  const harnesses = "docs/agents/harnesses.md";
  const skill = "skills/residential-rule-tuning/SKILL.md";
  const frontend = ".trellis/spec/frontend/index.md";
  const quality = ".trellis/spec/frontend/quality-guidelines.md";
  for (const heading of [
    "Project contract", "Authorization and review", "What this repository contains",
    "Toolchains by path", "Verification", "Boundaries", "Spec navigation"
  ]) {
    if (!sources[agents].split("\n").some((line) => /^#{1,2} /.test(line) && line.replace(/^#+ /, "").trim() === heading)) {
      fail("SHARED", agents, `缺少共享合同标题：${heading}`);
    }
  }
  for (const text of ["Claude Code", "Codex", "Grok Build", "Kimi Code", "OMP",
    "clash-verge-ai-residential.js", "residential-monitor/", "docs/", "CONTEXT.md"]) {
    requireText(agents, text, "SHARED");
  }
  if (/(?:^|[\s`])@(?:\.\/)?CLAUDE\.md\b/i.test(sources[agents])) {
    fail("IMPORT", agents, "共享合同不得反向导入 CLAUDE.md");
  }
  if (!/^@(?:\.\/)?AGENTS\.md\s*$/m.test(sources["CLAUDE.md"])) {
    fail("IMPORT", "CLAUDE.md", "Claude 入口必须单向导入 AGENTS.md");
  }
  requireText(frontend, "Read the shared `AGENTS.md`", "ENTRY");
  if (/^- Read `CLAUDE\.md`/m.test(sources[frontend])) {
    fail("ENTRY", frontend, "共享 frontend 入口不得要求先读 Claude 入口");
  }

  requirePattern(agents, /Read-only review.*?must not repair product code or configuration/i, "AUTH", "缺少只读审查写入边界");
  requirePattern(agents, /Self-fix.*?only after implementation is approved.*?approved task and file scope/i, "AUTH", "缺少 self-fix 的批准与文件范围条件");
  requireText(agents, "user explicitly authorizes the exception", "AUTH");
  requireText(agents, "Never hand-edit generated `*.local.js`", "AUTH");
  requireText(skill, "默认只输出", "AUTH");
  requireText(skill, "由用户应用", "AUTH");
  requireText(skill, "例外必须由用户明确授权", "AUTH");
  requireText(workflow, "### Review authorization", "AUTH");
  requireText(workflow, "Read-only review must not repair product code or configuration", "AUTH");
  requireText(workflow, "approved task and file scope", "AUTH");
  requireText(workflow, "ask the main session to confirm the dispatch target", "AUTH");
  const kimiCheck = ".kimi-code/skills/trellis-check/SKILL.md";
  requireText(kimiCheck, "Follow `AGENTS.md`", "AUTH");
  requireText(kimiCheck, "Read-only review must not repair product code or configuration", "AUTH");
  requireText(kimiCheck, "approved task and file scope", "AUTH");
  requireText(kimiCheck, "even when the dispatch prompt supplies a path", "DISPATCH");
  requireText(kimiCheck, "ask the main session to confirm the dispatch target", "DISPATCH");

  for (const status of STATUSES) {
    const open = `[workflow-state:${status}]`;
    const close = `[/workflow-state:${status}]`;
    const lines = sources[workflow].split("\n");
    const starts = lines.flatMap((line, index) => line.trim() === open ? [index] : []);
    const ends = lines.flatMap((line, index) => line.trim() === close ? [index] : []);
    if (starts.length !== 1 || ends.length !== 1 || starts[0] >= ends[0]) {
      fail("WORKFLOW", workflow, `阶段标记必须唯一且成对：${status}`);
    } else if (status.startsWith("in_progress") && !lines.slice(starts[0] + 1, ends[0]).some((line) => line.startsWith("Authorization: follow AGENTS.md; self-fix only within approved implementation and file scope."))) {
      fail("AUTH", workflow, `执行阶段缺少授权提示：${status}`);
    }
  }
  for (const file of [workflow, harnesses]) {
    requireText(file, "Active task: <path>", "DISPATCH");
    requireText(file, "built-in `coder`", "DISPATCH");
    requireText(file, "task.py current --source", "DISPATCH");
  }
  for (const file of OVERRIDES) {
    requireText(harnesses, file, "OVERRIDE");
  }
  const codexConfig = ".codex/config.toml";
  const config = sources[codexConfig];
  if (config !== undefined) {
    const agentsSection = config.match(/^\[agents\]\s*\n([\s\S]*?)(?=^\[|(?![\s\S]))/m);
    if (!agentsSection || !/^max_depth\s*=\s*1\s*(?:#.*)?$/m.test(agentsSection[1])) {
      fail("DEPTH", codexConfig, "保留 [agents] max_depth = 1");
    }
    requireText(codexConfig, "V1 agent threads only; V2 ignores this field", "DEPTH");
    requireText(codexConfig, "prompt guards are not a hard sandbox", "DEPTH");
  }

  for (const file of [agents, quality]) {
    for (const gate of ["just ci", "npm run ci", "just docs-build", "just dependency-audit", "Required checks", "--include=dev --audit-level=high"]) {
      requireText(file, gate, "GATE");
    }
    requireText(file, "seven separate pwsh native", "GATE");
  }
  requirePattern(".github/workflows/ci.yml", /name: Required checks needs: \[test, monitor, docs\]/, "GATE", "Required checks 名称或依赖发生漂移");
  for (const file of ["justfile", ".github/workflows/ci.yml"]) {
    for (const pkg of ["residential-monitor", "docs"]) {
      requireText(file, `npm --prefix ${pkg} audit --include=dev --audit-level=high`, "GATE");
    }
  }
  for (const recipe of ["ci", "docs-build", "dependency-audit"]) {
    if (!new RegExp(`^${recipe}:`, "m").test(sources.justfile)) fail("GATE", "justfile", `缺少 recipe：${recipe}`);
  }
  try {
    const scripts = JSON.parse(sources["package.json"]).scripts || {};
    if (scripts["check:agents"] !== "node scripts/check-agent-contract.js" ||
        !/(?:^|&&)\s*npm run check:agents\s*(?:&&|$)/.test(scripts.check || "") ||
        !(scripts.check || "").includes("node --check scripts/check-agent-contract.js") ||
        !(scripts.check || "").includes("node --check tests/check-agent-contract.test.js") ||
        !(scripts.test || "").split(/\s+/).includes("tests/check-agent-contract.test.js")) {
      fail("WIRING", "package.json", "check/test 必须接入说明检查器及其测试");
    }
  } catch (error) {
    fail("WIRING", "package.json", `无法解析 package.json：${error.message}`);
  }

  // 只解析当前说明中的具体仓库路径；占位符和忽略的安装态不属于此检查。
  for (const file of [harnesses, workflow]) {
    for (const match of sources[file].matchAll(/`(\.trellis\/(?:tasks|scripts)\/[^`]+)`/g)) {
      const reference = match[1];
      if (/[<>*{}\s]/.test(reference)) continue;
      const target = path.resolve(root, reference);
      const relative = path.relative(path.resolve(root), target);
      if (relative === ".." || relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative) || !fs.existsSync(target)) {
        fail("REFERENCE", file, `仓库引用不存在或越界：${reference}`);
      }
    }
  }
  return { ok: errors.length === 0, errors };
}

if (require.main === module) {
  const result = checkContract();
  for (const error of result.errors) {
    console.error(`[${error.code}] ${error.file}：${error.detail}`);
  }
  if (result.ok) console.log("说明合同结构检查通过；语义与客户端运行状态仍需独立验证。");
  process.exitCode = result.ok ? 0 : 1;
}

module.exports = { checkContract, FILES, OVERRIDES };

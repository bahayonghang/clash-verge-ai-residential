"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const { test } = require("node:test");
const { checkContract, FILES, OVERRIDES } = require("../scripts/check-agent-contract.js");

const ROOT = path.resolve(__dirname, "..");
const WORKFLOW = ".trellis/workflow.md";
const HARNESS = "docs/agents/harnesses.md";
const SKILL = "skills/residential-rule-tuning/SKILL.md";
const CHECK_ROLE = ".kimi-code/skills/trellis-check/SKILL.md";
// 干净 clone 不持有本机覆盖；占位内容只满足检查器的结构要求，不是本机配置的副本。
const OVERRIDE_PLACEHOLDERS = {
  ".codex/config.toml": [
    "[agents]",
    "max_depth = 1",
    "# V1 agent threads only; V2 ignores this field",
    "# prompt guards are not a hard sandbox",
    ""
  ].join("\n"),
  [CHECK_ROLE]: [
    "Follow `AGENTS.md`.",
    "Read-only review must not repair product code or configuration.",
    "Self-fix applies only after implementation is approved, within the approved task and file scope.",
    "Stop and ask the main session to confirm the dispatch target even when the dispatch prompt supplies a path.",
    ""
  ].join("\n")
};

function fixture(t, crlf = false) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "agent-contract-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  for (const file of [...FILES, ...OVERRIDES, "scripts/check-agent-contract.js"]) {
    const target = path.join(root, file);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    const origin = path.join(ROOT, file);
    const source = fs.existsSync(origin) ? fs.readFileSync(origin, "utf8") : OVERRIDE_PLACEHOLDERS[file];
    if (source === undefined) continue;
    const text = source.replace(/\r\n/g, "\n");
    fs.writeFileSync(target, crlf ? text.replace(/\n/g, "\r\n") : text);
  }
  // 路径 fixture 只提供公开占位内容；不复制真实任务证据或用户配置。
  for (const file of [
    ".trellis/scripts/get_context.py",
    ".trellis/scripts/task.py",
    ".trellis/scripts/common/workflow_phase.py",
    ".trellis/tasks/09-30-evergreen-five-harness-audit/research/audit.md",
    ".trellis/tasks/archive/2026-09/09-07-evergreen-harness-audit/research/harness-audit.md"
  ]) {
    fs.mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
    fs.writeFileSync(path.join(root, file), "fixture\n");
  }
  return root;
}

function replace(root, file, before, after) {
  const target = path.join(root, file);
  const source = fs.readFileSync(target, "utf8");
  assert.ok(source.includes(before), `负例的替换目标必须存在：${file} / ${before}`);
  fs.writeFileSync(target, source.replace(before, after));
}

function rejects(root, code, file) {
  const result = checkContract(root);
  assert.equal(result.ok, false);
  assert.ok(result.errors.some((error) => error.code === code && error.file === file), JSON.stringify(result));
}

test("正常仓库的说明合同通过", () => {
  assert.deepEqual(checkContract(ROOT), { ok: true, errors: [] });
});

for (const crlf of [false, true]) {
  test(`正常临时 fixture 通过且不改文件：${crlf ? "CRLF" : "LF"}`, (t) => {
    const root = fixture(t, crlf);
    const before = FILES.map((file) => fs.readFileSync(path.join(root, file)));
    assert.deepEqual(checkContract(root), { ok: true, errors: [] });
    FILES.forEach((file, index) => assert.deepEqual(fs.readFileSync(path.join(root, file)), before[index]));
  });
}

const negativeCases = [
  ["缺少共享产品合同", "SHARED", "AGENTS.md", "## What this repository contains", "## Removed"],
  ["共享合同反向导入 Claude", "IMPORT", "AGENTS.md", "# Project contract", "# Project contract\n@CLAUDE.md"],
  ["Claude 缺少共享导入", "IMPORT", "CLAUDE.md", "@AGENTS.md", "AGENTS.md"],
  ["frontend 恢复 Claude 专属入口", "ENTRY", ".trellis/spec/frontend/index.md", "Read the shared `AGENTS.md`", "Read `CLAUDE.md`"],
  ["根合同缺少只读边界", "AUTH", "AGENTS.md", "must not repair product code or configuration", "may repair product code or configuration"],
  ["根合同缺少实施批准", "AUTH", "AGENTS.md", "only after implementation is approved", "whenever tools exist"],
  ["skill 移除用户例外授权", "AUTH", SKILL, "例外必须由用户明确授权", "例外由 skill 授权"],
  ["skill 移除用户应用建议", "AUTH", SKILL, "由用户应用", "由 agent 自动应用"],
  ["check 角色缺少只读边界", "AUTH", CHECK_ROLE, "Read-only review must not repair product code or", "Read-only review may repair product code or"],
  ["Kimi 跳过派发路径核对", "DISPATCH", CHECK_ROLE, "even when the dispatch prompt supplies a path", "only when the dispatch prompt has no path"],
  ["Kimi 忽略任务路径冲突", "DISPATCH", CHECK_ROLE, "ask the main session to confirm the dispatch target", "trust the first task path"],
  ["workflow 缺少授权段", "AUTH", WORKFLOW, "### Review authorization", "### Removed"],
  ["workflow 缺少路径不一致处理", "AUTH", WORKFLOW, "ask the main session to confirm the dispatch target", "trust the injected target"],
  ["workflow 缺少阶段闭合", "WORKFLOW", WORKFLOW, "[/workflow-state:in_progress-inline]", ""],
  ["workflow 重复阶段", "WORKFLOW", WORKFLOW, "[workflow-state:in_progress]\n", "[workflow-state:in_progress]\n[workflow-state:in_progress]\n"],
  ["workflow 缺少执行授权提示", "AUTH", WORKFLOW, "Authorization: follow AGENTS.md; self-fix only within approved implementation and file scope.", "Authorization: self-fix without limits."],
  ["harness 缺少派发首行", "DISPATCH", HARNESS, "Dispatch 首行 `Active task: <path>`", "Dispatch 首行 `Task path:`"],
  ["Codex 深度发生变化", "DEPTH", ".codex/config.toml", "max_depth = 1", "max_depth = 2"],
  ["Codex 声称 V2 受深度限制", "DEPTH", ".codex/config.toml", "V1 agent threads only; V2 ignores this field", "V1 and V2 agent threads"],
  ["Codex 声称提示词硬隔离", "DEPTH", ".codex/config.toml", "prompt guards are not a hard sandbox", "prompt guards are a hard sandbox"],
  ["共享说明门禁名称漂移", "GATE", "AGENTS.md", "`Required checks`", "`Optional checks`"],
  ["spec 的 Windows step 数回退", "GATE", ".trellis/spec/frontend/quality-guidelines.md", "seven separate pwsh native", "six separate pwsh native"],
  ["CI 聚合门名称漂移", "GATE", ".github/workflows/ci.yml", "name: Required checks", "name: Optional checks"],
  ["CI 聚合门漏掉 docs", "GATE", ".github/workflows/ci.yml", "needs: [test, monitor, docs]", "needs: [test, monitor]"],
  ["CI docs 审计漏掉开发依赖", "GATE", ".github/workflows/ci.yml", "npm --prefix docs audit --include=dev", "npm --prefix docs audit"],
  ["just 独立审计门被重命名", "GATE", "justfile", "dependency-audit:", "optional-audit:"],
  ["说明引用旧的未归档路径", "REFERENCE", HARNESS, ".trellis/tasks/archive/2026-09/09-07-evergreen-harness-audit/research/harness-audit.md", ".trellis/tasks/09-07-evergreen-harness-audit/research/harness-audit.md"]
];

for (const [name, code, file, before, after] of negativeCases) {
  test(name, (t) => {
    const root = fixture(t);
    replace(root, file, before, after);
    // 派发前缀在表格中重复；本负例删除所有前缀以模拟入口整体缺失。
    if (code === "DISPATCH" && file === HARNESS) {
      const target = path.join(root, file);
      fs.writeFileSync(target, fs.readFileSync(target, "utf8").replaceAll("Active task: <path>", "Task path:"));
    }
    rejects(root, code, file);
  });
}

test("本机覆盖缺失时不阻断：干净 clone 不持有 .codex/ 与 .kimi-code/", (t) => {
  const root = fixture(t);
  for (const file of OVERRIDES) fs.rmSync(path.join(root, file), { force: true });
  assert.deepEqual(checkContract(root), { ok: true, errors: [] });
});

for (const key of ["check:agents", "check", "test"]) {
  test(`根命令接线缺失：${key}`, (t) => {
    const root = fixture(t);
    const file = path.join(root, "package.json");
    const pkg = JSON.parse(fs.readFileSync(file, "utf8"));
    delete pkg.scripts[key];
    fs.writeFileSync(file, JSON.stringify(pkg));
    rejects(root, "WIRING", "package.json");
  });
}

test("引用越出仓库时失败", (t) => {
  const root = fixture(t);
  fs.appendFileSync(path.join(root, HARNESS), "\n`.trellis/tasks/../../../outside.md`\n");
  rejects(root, "REFERENCE", HARNESS);
});

test("CLI 正常退出 0，破坏共享合同后退出 1", (t) => {
  const root = fixture(t);
  const script = path.join(root, "scripts/check-agent-contract.js");
  const ok = spawnSync(process.execPath, [script], { cwd: root, encoding: "utf8" });
  assert.equal(ok.status, 0, ok.stderr);
  replace(root, "AGENTS.md", "## Verification", "## Removed");
  const bad = spawnSync(process.execPath, [script], { cwd: root, encoding: "utf8" });
  assert.equal(bad.status, 1);
  assert.match(bad.stderr, /\[SHARED\] AGENTS\.md/);
});

"use strict";

const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const crypto = require("node:crypto");
const { spawnSync } = require("node:child_process");
const { parseVersion } = require("./check-harness-environment.js");
const { checkContract, OVERRIDES } = require("./check-agent-contract.js");

const SOURCE_ROOT = path.resolve(__dirname, "..");
const INIT_ARGS = ["init", "--claude", "--codex", "--grok", "--kimi", "--omp", "--skip-existing", "-y"];
const TEMPLATES = ["codex-config.toml", "kimi-trellis-implement.md", "kimi-trellis-check.md", "kimi-trellis-research.md"];
const ASSETS = [
  ".claude/agents/trellis-implement.md", ".codex/hooks.json",
  ".grok/agents/trellis-implement.md", ".kimi-code/skills/trellis-start/SKILL.md",
  ".omp/agents/trellis-implement.md"
];
// 已审查的 67 项公开最低合同；不递归复制目录，不读取本机覆盖、数据库或历史配置。
// 该候选不是完整 fresh checkout，不能用候选 package.json 执行全产品门。
const PUBLIC_FILES = [
  ".github/workflows/ci.yml", ".gitignore", ".trellis/.version", ".trellis/config.yaml",
  ".trellis/scripts/__init__.py", ".trellis/scripts/add_session.py",
  ...["__init__", "active_task", "cli_adapter", "config", "developer", "git", "git_context",
    "io", "log", "packages_context", "paths", "safe_commit", "session_context", "spec_inject",
    "spec_match", "task_context", "task_queue", "task_store", "task_utils", "tasks", "trellis_config",
    "types", "workflow_phase", "workflow_selection"].map((name) => `.trellis/scripts/common/${name}.py`),
  ".trellis/scripts/get_context.py", ".trellis/scripts/get_developer.py",
  ".trellis/scripts/hooks/linear_sync.py", ".trellis/scripts/init_developer.py", ".trellis/scripts/task.py",
  ...["component-guidelines", "directory-structure", "hook-guidelines", "index", "quality-guidelines",
    "state-management", "type-safety"].map((name) => `.trellis/spec/frontend/${name}.md`),
  ...["code-reuse-thinking-guide", "cross-layer-thinking-guide", "index"].map((name) => `.trellis/spec/guides/${name}.md`),
  ".trellis/spec/residential-monitor/backend/index.md",
  ".trellis/spec/residential-monitor/backend/modules-and-errors.md",
  ".trellis/spec/residential-monitor/backend/secrets-and-cancellation.md",
  ".trellis/spec/residential-monitor/frontend/dto-and-decoding.md",
  ".trellis/spec/residential-monitor/frontend/index.md",
  ".trellis/spec/residential-monitor/frontend/view-state.md",
  ".trellis/spec/residential-monitor/index.md",
  ".trellis/spec/residential-monitor/storage/index.md",
  ".trellis/spec/residential-monitor/storage/sqlite-contract.md",
  ".trellis/tasks/09-30-evergreen-five-harness-audit/research/audit.md",
  ".trellis/tasks/archive/2026-09/09-07-evergreen-harness-audit/research/harness-audit.md",
  ".trellis/workflow.md", "AGENTS.md", "CLAUDE.md", "CONTEXT.md",
  "docs/agents/harnesses.md", "docs/agents/residential-rule-tuning.md", "justfile", "package.json",
  "scripts/check-agent-contract.js", "scripts/check-harness-environment.js", "skills/residential-rule-tuning/SKILL.md"
];

function sha256(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}

function within(parent, child) {
  const relative = path.relative(parent, child);
  return relative !== "" && relative !== ".." && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative);
}

// 逐层检查 junction/符号链接；文件硬链接也不作为隔离副本。
function checkPath(target) {
  const absolute = path.resolve(target);
  let current = path.parse(absolute).root;
  for (const segment of absolute.slice(current.length).split(path.sep).filter(Boolean)) {
    current = path.join(current, segment);
    let stat;
    try { stat = fs.lstatSync(current); } catch (error) {
      if (error.code === "ENOENT") break;
      throw error;
    }
    if (stat.isSymbolicLink() || (stat.isFile() && stat.nlink > 1)) throw new Error("拒绝链接路径：" + current);
    if (!stat.isDirectory() && !stat.isFile()) throw new Error("拒绝特殊文件：" + current);
  }
}

function validateTarget(root, sourceRoot) {
  const temp = path.resolve(os.tmpdir());
  if (!path.isAbsolute(root)) throw new Error("--root 必须为绝对路径");
  if (!within(temp, root) || !path.relative(temp, root).split(path.sep)[0].startsWith("trellis-")) {
    throw new Error("目标必须位于系统临时目录的 trellis- 独立目录中");
  }
  if (root === sourceRoot || within(sourceRoot, root) || within(root, sourceRoot)) throw new Error("拒绝当前工作树、子目录或祖先目录");
  checkPath(root);
  const allowed = new Set([...PUBLIC_FILES, ...OVERRIDES]);
  const visit = (directory) => {
    for (const name of fs.readdirSync(directory)) {
      const entry = path.join(directory, name);
      checkPath(entry);
      const relative = path.relative(root, entry).split(path.sep).join("/");
      const stat = fs.lstatSync(entry);
      if (stat.isDirectory()) {
        if (![...allowed].some((file) => file.startsWith(relative + "/"))) throw new Error("目标含非隔离目录或已生成资产：" + relative);
        visit(entry);
      } else if (!allowed.has(relative)) throw new Error("目标含未批准文件或已生成资产：" + relative);
    }
  };
  if (fs.existsSync(root)) {
    if (!fs.statSync(root).isDirectory()) throw new Error("目标必须为目录");
    visit(root);
  }
}

function entryCommand(entry) {
  if (!entry || !path.isAbsolute(entry)) throw new Error("--entry 必须为绝对路径");
  checkPath(entry);
  if (!fs.statSync(entry).isFile()) throw new Error("所选入口必须为普通文件");
  if (/\.(?:c?js|mjs)$/i.test(entry)) return { command: process.execPath, prefix: [entry] };
  if (/\.(?:cmd|bat|ps1)$/i.test(entry) || (process.platform === "win32" && !/\.exe$/i.test(entry))) {
    throw new Error("拒绝 shell 包装器；请显式选择固定 JS 或 EXE 入口");
  }
  return { command: entry, prefix: [] };
}

function runEntry(entry, args, root) {
  const selected = entryCommand(entry);
  return spawnSync(selected.command, [...selected.prefix, ...args], {
    cwd: root, encoding: "utf8", timeout: 60000, maxBuffer: 8 * 1024 * 1024,
    windowsHide: true, shell: false
  });
}

function receipt(result) {
  return {
    exitCode: result.status ?? null, signal: result.signal || null,
    error: result.error ? { code: result.error.code || null, message: result.error.message } : null,
    stdout: String(result.stdout || ""), stderr: String(result.stderr || "")
  };
}

function succeeded(result) {
  return result.exitCode === 0 && !result.error && !result.signal;
}

function invoke(run, entry, args, root) {
  try { return receipt(run(entry, args, root)); } catch (error) { return receipt({ error }); }
}

function snapshot(root, files) {
  return files.map((file) => {
    const target = path.join(root, file);
    checkPath(target);
    try { return { path: file, present: true, sha256: sha256(fs.readFileSync(target)) }; } catch (error) {
      if (error.code === "ENOENT") return { path: file, present: false, sha256: null };
      throw error;
    }
  });
}

function bootstrap(options, dependencies = {}) {
  const sourceRoot = dependencies.sourceRoot || SOURCE_ROOT;
  const run = dependencies.run || runEntry;
  const result = { ok: false, status: "BLOCKED", root: options.root || null, entry: options.entry || null,
    errors: [], version: null, init: null, contractBefore: null, contractAfter: null,
    overrides: [], publicFiles: [], assets: [], preservation: "NOT_RUN",
    boundary: "仅初始化与本地合同；原生角色、hook、权限、hosted 与产品运行仍需独立验证。" };
  try {
    if (!options.root) throw new Error("必须显式提供 --root");
    const root = path.resolve(options.root);
    validateTarget(options.root, sourceRoot);
    entryCommand(options.entry);
    if (options.entry === root || within(root, options.entry)) throw new Error("工具入口必须位于候选目录之外");
    const inputs = PUBLIC_FILES.map((file) => {
      const source = path.join(sourceRoot, file);
      checkPath(source);
      return { path: file, bytes: fs.readFileSync(source) };
    });
    const templates = TEMPLATES.map((file) => {
      const source = path.join(sourceRoot, "scripts", "harness-templates", file);
      checkPath(source);
      return fs.readFileSync(source);
    });
    const expected = inputs.find((item) => item.path === ".trellis/.version").bytes.toString("utf8").trim();
    if (parseVersion("trellis", expected) !== expected) throw new Error("公开项目版本缺失或无效");
    // 在全部预检查结束前不建立目标或部署文件。
    for (const item of inputs) {
      const target = path.join(root, item.path);
      if (fs.existsSync(target) && !fs.readFileSync(target).equals(item.bytes)) throw new Error("已有公开合同与来源不同：" + item.path);
    }
    result.version = invoke(run, options.entry, ["--version"], sourceRoot);
    result.version.expected = expected;
    result.version.actual = parseVersion("trellis", result.version.stdout);
    if (!succeeded(result.version) || result.version.actual !== expected) throw new Error("所选入口失败、版本未知或与项目版本不匹配");
    const before = snapshot(root, OVERRIDES);
    fs.mkdirSync(root, { recursive: true });
    for (const item of inputs) {
      const target = path.join(root, item.path);
      if (!fs.existsSync(target)) {
        fs.mkdirSync(path.dirname(target), { recursive: true });
        fs.writeFileSync(target, item.bytes, { flag: "wx" });
      }
      result.publicFiles.push({ path: item.path, sha256: sha256(item.bytes) });
    }
    for (let index = 0; index < OVERRIDES.length; index += 1) {
      const file = OVERRIDES[index];
      if (!before[index].present) {
        const target = path.join(root, file);
        fs.mkdirSync(path.dirname(target), { recursive: true });
        fs.writeFileSync(target, templates[index], { flag: "wx" });
      }
    }
    const deployed = snapshot(root, OVERRIDES);
    result.overrides = deployed.map((item, index) => ({ ...item,
      source: before[index].present ? "preserved_existing" : "created_project_default",
      beforeSha256: before[index].sha256, deployedSha256: item.sha256 }));
    result.contractBefore = checkContract(root);
    if (!result.contractBefore.ok) { result.status = "CONTRACT_FAIL"; return result; }
    result.init = invoke(run, options.entry, INIT_ARGS, root);
    // 保留 native 首失败；后检查与 hash 比较不能覆盖退出码。
    const after = snapshot(root, OVERRIDES);
    result.overrides.forEach((item, index) => { item.afterSha256 = after[index].sha256; });
    const publicAfter = snapshot(root, PUBLIC_FILES);
    const preserved = after.every((item, index) => item.sha256 === deployed[index].sha256) &&
      publicAfter.every((item, index) => item.sha256 === result.publicFiles[index].sha256);
    result.publicFiles.forEach((item, index) => { item.afterSha256 = publicAfter[index].sha256; });
    result.preservation = preserved ? "PASS" : "FAIL";
    result.contractAfter = checkContract(root);
    result.assets = snapshot(root, ASSETS);
    result.ok = succeeded(result.init) && preserved && result.contractAfter.ok && result.assets.every((item) => item.present);
    result.status = !succeeded(result.init) ? "INIT_FAIL" : !preserved ? "PRESERVATION_FAIL" :
      !result.contractAfter.ok ? "CONTRACT_FAIL" : !result.assets.every((item) => item.present) ? "ASSET_FAIL" : "PASS";
  } catch (error) {
    result.errors.push(error.message);
    if (result.init) result.status = succeeded(result.init) ? "POSTCHECK_FAIL" : "INIT_FAIL";
  }
  return result;
}

function parseArgs(argv) {
  const options = {};
  for (let index = 2; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--help" || arg === "-h") options.help = true;
    else if (["--root", "--entry"].includes(arg)) {
      const value = argv[++index];
      if (!value || value.startsWith("--") || options[arg.slice(2)]) throw new Error("参数缺失或重复：" + arg);
      options[arg.slice(2)] = value;
    } else throw new Error("未知参数：" + arg);
  }
  return options;
}

function main(argv) {
  const options = parseArgs(argv);
  if (options.help) {
    console.log("用法: node scripts/bootstrap-harnesses.js --root <系统Temp/trellis-独立目录/新候选> --entry <固定JS/EXE绝对路径>");
    return 0;
  }
  const result = bootstrap(options);
  console.log(JSON.stringify(result, null, 2));
  return result.ok ? 0 : 1;
}

if (require.main === module) {
  try { process.exitCode = main(process.argv); } catch (error) { console.error(error.message); process.exitCode = 1; }
}

module.exports = { PUBLIC_FILES, TEMPLATES, ASSETS, INIT_ARGS, bootstrap, main, parseArgs, runEntry, entryCommand };

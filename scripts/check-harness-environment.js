"use strict";

const fs = require("node:fs");
const path = require("node:path");
const crypto = require("node:crypto");
const { spawnSync } = require("node:child_process");

const REPO_ROOT = path.resolve(__dirname, "..");
const TOOLS = ["claude", "codex", "grok", "kimi", "omp", "trellis"];
// 本机 harness 覆盖不进版本控制：只报告存在性与哈希，不参与 bootstrap 判定。
const OVERRIDES = [
  ".codex/config.toml",
  ".kimi-code/skills/trellis-implement/SKILL.md",
  ".kimi-code/skills/trellis-check/SKILL.md",
  ".kimi-code/skills/trellis-research/SKILL.md"
];
const VERSION = "[0-9]+\\.[0-9]+\\.[0-9]+(?:-[0-9A-Za-z]+(?:[.-][0-9A-Za-z]+)*)?(?:\\+[0-9A-Za-z]+(?:[.-][0-9A-Za-z]+)*)?";

function parseVersion(tool, output) {
  const prefixes = {
    claude: "(?:claude(?:-code)?\\s+)?",
    codex: "(?:codex-cli\\s+)?",
    grok: "(?:grok\\s+)?",
    kimi: "(?:kimi(?:-cli)?\\s+)?",
    omp: "(?:omp/)?",
    trellis: "(?:trellis\\s+)?"
  };
  const pattern = new RegExp("^" + prefixes[tool] + "v?(" + VERSION + ")(?:\\s+\\([^)]*\\))?(?:\\s+\\[[^\\]]*\\])?$");
  const versions = new Set();
  for (const line of String(output || "").replace(/\u001b\[[0-9;]*m/g, "").split(/\r?\n/)) {
    const match = line.trim().match(pattern);
    if (match) versions.add(match[1]);
  }
  return versions.size === 1 ? [...versions][0] : null;
}

function processOptions(root) {
  return { cwd: root, encoding: "utf8", timeout: 10000, maxBuffer: 1024 * 1024, windowsHide: true };
}

function discoverEntries(tool, root) {
  if (process.platform === "win32") {
    const command = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false); $ErrorActionPreference='Stop'; $found=@(Get-Command -Name '" + tool +
      "' -All -ErrorAction SilentlyContinue | Where-Object { $_.CommandType -eq 'Application' -or $_.CommandType -eq 'ExternalScript' } | Select-Object -ExpandProperty Source); ConvertTo-Json -InputObject $found -Compress";
    const result = spawnSync("powershell.exe", ["-NoLogo", "-NoProfile", "-NonInteractive", "-Command", command], processOptions(root));
    if (result.error || result.status !== 0) {
      throw new Error("入口发现失败：" + (result.error ? result.error.message : result.stderr.trim()));
    }
    return [...new Set(JSON.parse(result.stdout || "[]"))];
  }
  const found = [];
  for (const dir of (process.env.PATH || "").split(path.delimiter).filter(Boolean)) {
    const candidate = path.resolve(dir, tool);
    try {
      fs.accessSync(candidate, fs.constants.X_OK);
      if (fs.statSync(candidate).isFile() && !found.includes(candidate)) found.push(candidate);
    } catch {
      // 不可执行的 PATH 项不能作为候选入口。
    }
  }
  return found;
}

function runVersion(entry, root) {
  if (/\.(?:c?js|mjs)$/i.test(entry)) {
    return spawnSync(process.execPath, [entry, "--version"], processOptions(root));
  }
  if (process.platform === "win32" && /\.(?:ps1|cmd|bat)$/i.test(entry)) {
    const command = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false); $ErrorActionPreference='Stop'; & '" + entry.replaceAll("'", "''") +
      "' --version; if ($null -ne $LASTEXITCODE) { exit $LASTEXITCODE }";
    return spawnSync("powershell.exe", ["-NoLogo", "-NoProfile", "-NonInteractive", "-Command", command], processOptions(root));
  }
  return spawnSync(entry, ["--version"], processOptions(root));
}

function probeEntry(tool, entry, root, run) {
  if (!entry) return { path: null, status: "missing", version: null, exitCode: null };
  let result;
  try {
    result = run(entry, root);
  } catch (error) {
    result = { error, status: null };
  }
  const error = result.error ? { code: result.error.code || null, message: result.error.message } : null;
  const stdout = String(result.stdout || "");
  const stderr = String(result.stderr || "");
  const version = parseVersion(tool, stdout);
  let status = "available";
  if (error && error.code === "ENOENT") status = "missing";
  else if (error && ["EACCES", "EPERM"].includes(error.code)) status = "permission-denied";
  else if (error || result.status !== 0 || result.signal) status = "failed";
  else if (!version) status = "unknown-version";
  return { path: entry, status, version, exitCode: result.status ?? null, signal: result.signal || null, error, stdout, stderr };
}

function checkEnvironment(options = {}, dependencies = {}) {
  const root = path.resolve(options.root || REPO_ROOT);
  const discover = dependencies.discover || discoverEntries;
  const run = dependencies.run || runVersion;
  const versionPath = path.join(root, ".trellis", ".version");
  let expectedVersion = null;
  let versionError = null;
  try {
    const value = fs.readFileSync(versionPath, "utf8").trim();
    if (!new RegExp("^" + VERSION + "$").test(value)) throw new Error("项目 Trellis 版本格式无效");
    expectedVersion = value;
  } catch (error) {
    versionError = error.message;
  }
  const overrides = OVERRIDES.map((rel) => {
    try {
      const bytes = fs.readFileSync(path.join(root, rel));
      return { path: rel, present: true, sha256: crypto.createHash("sha256").update(bytes).digest("hex") };
    } catch (error) {
      return { path: rel, present: false, error: error.message };
    }
  });
  const tools = TOOLS.map((tool) => {
    const explicit = options.entries && options.entries[tool];
    let discovered = [];
    let discoveryError = null;
    if (!explicit) {
      try { discovered = discover(tool, root); } catch (error) { discoveryError = error.message; }
    }
    const selected = probeEntry(tool, explicit || discovered[0], root, run);
    const alternativePaths = [...new Set([
      ...discovered.slice(1), ...((options.alternatives && options.alternatives[tool]) || [])
    ])].filter((entry) => entry !== selected.path);
    const alternatives = alternativePaths.map((entry) => probeEntry(tool, entry, root, run));
    return { tool, selection: explicit ? "explicit" : "discovered", discoveryError, selected, alternatives };
  });
  const trellis = tools.find((item) => item.tool === "trellis");
  const blockers = [];
  if (!expectedVersion) blockers.push("项目 .trellis/.version 缺失或无效");
  if (trellis.discoveryError || trellis.selected.status !== "available") blockers.push("所选 Trellis 入口不可用或版本未知");
  if (expectedVersion && trellis.selected.version && expectedVersion !== trellis.selected.version) {
    blockers.push("所选 Trellis 版本与项目不匹配");
  }
  const bootstrap = { status: blockers.length ? "BLOCKED" : "READY", blockers, executesBootstrap: false };
  return {
    ok: bootstrap.status === "READY" && tools.every((item) => !item.discoveryError && item.selected.status === "available"),
    project: { root, versionPath, expectedTrellisVersion: expectedVersion, versionError, overrides },
    tools,
    bootstrap,
    permissions: { projectTrust: "UNVERIFIED", hooks: "UNVERIFIED", note: "版本命令不证明项目或 hook 获准；本检查不修改权限、信任或 PATH。" }
  };
}

function parseArgs(argv) {
  const options = { entries: {}, alternatives: {} };
  for (let i = 2; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--help" || arg === "-h") options.help = true;
    else if (["--root", "--entry", "--alternative"].includes(arg)) {
      const value = argv[++i];
      if (!value || value.startsWith("--")) throw new Error(arg + " 缺少参数");
      if (arg === "--root") options.root = path.resolve(value);
      else {
        const split = value.indexOf("=");
        const tool = value.slice(0, split);
        const entry = value.slice(split + 1);
        if (split < 1 || !TOOLS.includes(tool) || !path.isAbsolute(entry)) throw new Error("入口必须为 工具名=绝对路径");
        if (arg === "--entry") {
          if (options.entries[tool]) throw new Error("重复选择入口：" + tool);
          options.entries[tool] = path.resolve(entry);
        } else (options.alternatives[tool] ||= []).push(path.resolve(entry));
      }
    } else throw new Error("未知参数 " + arg);
  }
  return options;
}

function main(argv) {
  const options = parseArgs(argv);
  if (options.help) {
    console.log("用法: node scripts/check-harness-environment.js [--root <项目目录>] [--entry <工具名=绝对路径>] [--alternative <工具名=绝对路径>]");
    return 0;
  }
  const result = checkEnvironment(options);
  console.log(JSON.stringify(result, null, 2));
  return result.ok ? 0 : 1;
}

if (require.main === module) {
  try { process.exitCode = main(process.argv); } catch (error) { console.error(error.message); process.exitCode = 1; }
}

module.exports = { TOOLS, OVERRIDES, checkEnvironment, discoverEntries, main, parseArgs, parseVersion, probeEntry, runVersion };

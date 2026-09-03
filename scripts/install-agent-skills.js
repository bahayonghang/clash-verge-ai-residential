"use strict";

const fs = require("node:fs");
const path = require("node:path");

const REPO_ROOT = path.resolve(__dirname, "..");
const SKILL_ID = "residential-rule-tuning";
const SOURCE_DIR = path.join("skills", SKILL_ID);
const PLATFORM_ROOTS = [
  ".agents",
  ".claude",
  ".codex",
  ".cursor",
  ".omp",
  ".grok",
  ".kimi-code"
];

function parsePlatformList(value) {
  const names = value
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
  for (const name of names) {
    if (!PLATFORM_ROOTS.includes(name)) {
      throw new Error(`未知平台目录 ${name}`);
    }
  }
  return names;
}

function parseArgs(argv) {
  const options = { force: false, check: false, create: false, platforms: null };
  const args = argv.slice(2);
  for (let i = 0; i < args.length; i += 1) {
    const arg = args[i];
    if (arg === "--force") options.force = true;
    else if (arg === "--check") options.check = true;
    else if (arg === "--create") options.create = true;
    else if (arg === "--platforms") {
      const value = args[i + 1];
      if (!value || value.startsWith("-")) {
        throw new Error("--platforms 需要逗号分隔的目录列表");
      }
      options.platforms = parsePlatformList(value);
      i += 1;
    } else if (arg === "--help" || arg === "-h") options.help = true;
    else throw new Error(`未知参数 ${arg}`);
  }
  return options;
}

function listFiles(root) {
  const out = [];
  function walk(current, rel) {
    for (const entry of fs.readdirSync(current, { withFileTypes: true })) {
      const nextRel = rel ? `${rel}/${entry.name}` : entry.name;
      const nextPath = path.join(current, entry.name);
      if (entry.isDirectory()) walk(nextPath, nextRel);
      else out.push(nextRel.replaceAll("\\", "/"));
    }
  }
  walk(root, "");
  return out.sort();
}

function copyFile(from, to) {
  fs.mkdirSync(path.dirname(to), { recursive: true });
  fs.copyFileSync(from, to);
}

function joinRel(root, rel) {
  return path.join(root, ...rel.split("/"));
}

function utcStamp(date) {
  return date.toISOString().replace(/[-:]/g, "").replace(/\.\d{3}Z$/, "Z");
}

function selectedRoots(options) {
  return options && options.platforms ? options.platforms : PLATFORM_ROOTS;
}

function planInstall(repoRoot, options) {
  const source = path.join(repoRoot, SOURCE_DIR);
  if (!fs.existsSync(source)) {
    throw new Error(`缺少 skill 源目录 ${SOURCE_DIR}`);
  }
  const files = listFiles(source);
  const platforms = [];
  let skipped = 0;
  for (const rootName of selectedRoots(options)) {
    const platformRoot = path.join(repoRoot, rootName);
    if (!fs.existsSync(platformRoot)) {
      if (options && options.create) {
        fs.mkdirSync(platformRoot, { recursive: true });
      } else {
        skipped += 1;
        continue;
      }
    } else if (!fs.statSync(platformRoot).isDirectory()) {
      skipped += 1;
      continue;
    }
    platforms.push({
      rootName,
      destDir: path.join(platformRoot, "skills", SKILL_ID)
    });
  }
  return { source, files, platforms, skipped };
}

function fileStatus(sourceFile, destFile) {
  if (!fs.existsSync(destFile)) return "missing";
  const left = fs.readFileSync(sourceFile);
  const right = fs.readFileSync(destFile);
  if (left.equals(right)) return "same";
  return "different";
}

function install(repoRoot, options, now) {
  const { source, files, platforms, skipped } = planInstall(repoRoot, options);
  const conflicts = [];
  const diffs = [];
  for (const platform of platforms) {
    for (const rel of files) {
      const from = joinRel(source, rel);
      const to = joinRel(platform.destDir, rel);
      const status = fileStatus(from, to);
      if (status !== "same") {
        diffs.push({ platform: platform.rootName, rel, status });
      }
      if (status === "different") {
        conflicts.push(path.join(platform.rootName, "skills", SKILL_ID, rel));
      }
    }
  }

  if (options.check) {
    if (diffs.length === 0) {
      return { ok: true, written: 0, skippedPlatforms: skipped };
    }
    const error = new Error(
      `skill 与已安装副本不一致：${diffs.map((item) => `${item.platform}/${item.rel}:${item.status}`).join(", ")}`
    );
    error.exitCode = 1;
    throw error;
  }

  if (conflicts.length > 0 && !options.force) {
    const error = new Error(
      `目标存在同名不同内容文件，已拒绝写入任何目录：${conflicts.join(", ")}`
    );
    error.exitCode = 1;
    throw error;
  }

  let written = 0;
  const stamp = utcStamp(now || new Date());
  for (const platform of platforms) {
    for (const rel of files) {
      const from = joinRel(source, rel);
      const to = joinRel(platform.destDir, rel);
      const status = fileStatus(from, to);
      if (status === "same") continue;
      if (status === "different" && options.force) {
        fs.copyFileSync(to, `${to}.bak-${stamp}`);
      }
      copyFile(from, to);
      written += 1;
    }
  }
  return {
    ok: true,
    written,
    platforms: platforms.map((item) => item.rootName),
    skippedPlatforms: skipped
  };
}

function main(argv, repoRoot) {
  const options = parseArgs(argv);
  if (options.help) {
    console.log(
      "用法: node scripts/install-agent-skills.js [--check] [--force] [--create] [--platforms .agents,.claude]"
    );
    return;
  }
  const result = install(repoRoot || REPO_ROOT, options);
  if (!options.check) {
    console.log(
      `已处理 ${result.platforms.length} 个平台目录，写入 ${result.written} 个文件，跳过 ${result.skippedPlatforms} 个缺失平台。`
    );
  }
}

if (require.main === module) {
  try {
    main(process.argv);
  } catch (error) {
    console.error(error.message);
    process.exitCode = error.exitCode || 1;
  }
}

module.exports = {
  PLATFORM_ROOTS,
  SKILL_ID,
  install,
  main,
  parseArgs,
  planInstall
};

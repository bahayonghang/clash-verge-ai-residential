"use strict";

const fs = require("node:fs");
const path = require("node:path");

const DEFAULT_ROOT = path.resolve(__dirname, "..");
const PACKAGE_NAME = "residential-monitor";
const VERSION_PATTERN =
  /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/;

function monitorPaths(root) {
  const monitor = path.join(root, PACKAGE_NAME);
  return {
    packageJson: path.join(monitor, "package.json"),
    packageLock: path.join(monitor, "package-lock.json"),
    tauriConf: path.join(monitor, "src-tauri", "tauri.conf.json"),
    cargoToml: path.join(monitor, "src-tauri", "Cargo.toml"),
    cargoLock: path.join(monitor, "src-tauri", "Cargo.lock")
  };
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function relativeLabel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join("/");
}

function readRequired(filePath, label) {
  if (!fs.existsSync(filePath)) {
    throw new Error(`缺少 ${label}`);
  }
  return fs.readFileSync(filePath, "utf8");
}

function readPackageVersion(root) {
  const filePath = monitorPaths(root).packageJson;
  const source = readRequired(filePath, "residential-monitor/package.json");
  let parsed;
  try {
    parsed = JSON.parse(source);
  } catch (error) {
    throw new Error(`无法解析 residential-monitor/package.json：${error.message}`);
  }
  if (typeof parsed.version !== "string" || !VERSION_PATTERN.test(parsed.version)) {
    throw new Error(
      `residential-monitor/package.json 的 version 不是有效 SemVer：${parsed.version}`
    );
  }
  return parsed.version;
}

function replaceFirstJsonVersion(text, version, label) {
  const pattern = /("version"\s*:\s*")([^"]*)(")/;
  const match = text.match(pattern);
  if (!match) {
    throw new Error(`${label} 缺少 version 字段`);
  }
  return {
    previous: match[2],
    next: text.replace(pattern, `$1${version}$3`)
  };
}

function replaceNamedTomlVersion(text, name, version, label) {
  const pattern = new RegExp(
    `(name = "${escapeRegExp(name)}"\\r?\\nversion = ")([^"]+)(")`
  );
  const match = text.match(pattern);
  if (!match) {
    throw new Error(`${label} 未找到 name = "${name}" 后的 version 字段`);
  }
  return {
    previous: match[2],
    next: text.replace(pattern, `$1${version}$3`)
  };
}

function replaceNamedJsonVersions(text, name, version, label) {
  const pattern = new RegExp(
    `("name"\\s*:\\s*"${escapeRegExp(name)}"\\s*,\\s*"version"\\s*:\\s*")([^"]+)(")`,
    "g"
  );
  let previous;
  let count = 0;
  const next = text.replace(pattern, (full, prefix, old, suffix) => {
    count += 1;
    if (previous !== undefined && previous !== old) {
      throw new Error(`${label} 中 ${name} 的 version 不一致`);
    }
    previous = old;
    return `${prefix}${version}${suffix}`;
  });
  if (count === 0) {
    throw new Error(`${label} 未找到 ${name} 的 version`);
  }
  return { previous, next, count };
}

function collectTargets(root, version) {
  const files = monitorPaths(root);
  const tauriLabel = relativeLabel(root, files.tauriConf);
  const cargoTomlLabel = relativeLabel(root, files.cargoToml);
  const cargoLockLabel = relativeLabel(root, files.cargoLock);
  const lockLabel = relativeLabel(root, files.packageLock);

  const tauriText = readRequired(files.tauriConf, tauriLabel);
  const tauri = replaceFirstJsonVersion(tauriText, version, tauriLabel);
  JSON.parse(tauri.next);

  const cargoTomlText = readRequired(files.cargoToml, cargoTomlLabel);
  const cargoToml = replaceNamedTomlVersion(
    cargoTomlText,
    PACKAGE_NAME,
    version,
    cargoTomlLabel
  );

  const cargoLockText = readRequired(files.cargoLock, cargoLockLabel);
  const cargoLock = replaceNamedTomlVersion(
    cargoLockText,
    PACKAGE_NAME,
    version,
    cargoLockLabel
  );

  const lockText = readRequired(files.packageLock, lockLabel);
  const lock = replaceNamedJsonVersions(lockText, PACKAGE_NAME, version, lockLabel);
  const lockJson = JSON.parse(lock.next);
  if (lockJson.version !== version) {
    throw new Error(`${lockLabel} 根 version 写入后仍为 ${lockJson.version}`);
  }
  if (lockJson.packages?.[""]?.version !== version) {
    throw new Error(`${lockLabel} packages[""].version 写入后未对齐`);
  }

  return [
    {
      path: files.tauriConf,
      label: tauriLabel,
      previous: tauri.previous,
      nextText: tauri.next
    },
    {
      path: files.cargoToml,
      label: cargoTomlLabel,
      previous: cargoToml.previous,
      nextText: cargoToml.next
    },
    {
      path: files.cargoLock,
      label: cargoLockLabel,
      previous: cargoLock.previous,
      nextText: cargoLock.next
    },
    {
      path: files.packageLock,
      label: lockLabel,
      previous: lock.previous,
      nextText: lock.next
    }
  ];
}

function syncMonitorVersion(root = DEFAULT_ROOT, options = {}) {
  const check = options.check === true;
  const version = readPackageVersion(root);
  const targets = collectTargets(root, version);
  const changed = targets.filter((target) => target.previous !== version);

  if (!check) {
    for (const target of changed) {
      fs.writeFileSync(target.path, target.nextText);
    }
  }

  return {
    version,
    check,
    changed: changed.map((target) => ({
      label: target.label,
      previous: target.previous
    }))
  };
}

function parseArgs(argv) {
  const check = argv.includes("--check");
  const unknown = argv.filter((arg) => arg.startsWith("-") && arg !== "--check");
  if (unknown.length > 0) {
    throw new Error(`不支持的参数：${unknown.join(" ")}`);
  }
  return { check };
}

function main() {
  const { check } = parseArgs(process.argv.slice(2));
  const result = syncMonitorVersion(DEFAULT_ROOT, { check });
  if (result.changed.length === 0) {
    console.log(`家宽监控版本已对齐 ${result.version}`);
    return;
  }
  const details = result.changed
    .map((item) => `${item.label} 为 ${item.previous}`)
    .join("，");
  if (check) {
    throw new Error(
      `家宽监控版本未对齐：package.json 为 ${result.version}，${details}。请运行 just monitor-sync-version。`
    );
  }
  console.log(
    `已将家宽监控版本同步为 ${result.version}：${result.changed.map((item) => item.label).join("、")}`
  );
}

if (require.main === module) {
  try {
    main();
  } catch (error) {
    console.error(`✗ ${error.message}`);
    process.exitCode = 1;
  }
}

module.exports = {
  PACKAGE_NAME,
  collectTargets,
  monitorPaths,
  readPackageVersion,
  syncMonitorVersion
};

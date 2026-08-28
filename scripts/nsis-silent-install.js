"use strict";

const childProcess = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");

const DEFAULT_ROOT = path.resolve(__dirname, "..");
const PACKAGE_NAME = "residential-monitor";
const BINARY_NAME = "residential-monitor";
const NSIS_BUNDLE_DIR = path.join(
  PACKAGE_NAME,
  "src-tauri",
  "target",
  "release",
  "bundle",
  "nsis"
);
const UNINSTALL_KEY_PREFIX =
  "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\";
const MANUFACTURER_KEY_PREFIX = "Software\\github\\";
const LEGACY_PRODUCT_NAMES = ["家宽流量监控"];

function monitorPaths(root) {
  const monitor = path.join(root, PACKAGE_NAME);
  return {
    packageJson: path.join(monitor, "package.json"),
    tauriConf: path.join(monitor, "src-tauri", "tauri.conf.json"),
    nsisDir: path.join(root, NSIS_BUNDLE_DIR)
  };
}

function readJson(filePath, label) {
  if (!fs.existsSync(filePath)) {
    throw new Error(`缺少 ${label}`);
  }
  try {
    return JSON.parse(fs.readFileSync(filePath, "utf8"));
  } catch (error) {
    throw new Error(`无法解析 ${label}：${error.message}`);
  }
}

function readProductName(root) {
  const parsed = readJson(monitorPaths(root).tauriConf, "tauri.conf.json");
  if (typeof parsed.productName !== "string" || parsed.productName.trim() === "") {
    throw new Error("tauri.conf.json 缺少 productName");
  }
  return parsed.productName;
}

function readPackageVersion(root) {
  const parsed = readJson(monitorPaths(root).packageJson, "residential-monitor/package.json");
  if (typeof parsed.version !== "string" || parsed.version.trim() === "") {
    throw new Error("无法读取 residential-monitor/package.json 的 version。");
  }
  return parsed.version;
}

function defaultInstallDir(localAppData, productName) {
  if (!localAppData) {
    throw new Error("缺少 LOCALAPPDATA，无法确定 current-user 安装目录。");
  }
  return path.win32.join(localAppData, productName);
}

function normalizeWinPath(value) {
  return path.win32.resolve(value).replace(/[\\/]+$/, "").toLowerCase();
}

function isUnderDir(target, root) {
  if (!target || !root) {
    return false;
  }
  const resolvedTarget = normalizeWinPath(target);
  const resolvedRoot = normalizeWinPath(root);
  if (resolvedTarget === resolvedRoot) {
    return true;
  }
  const prefix = resolvedRoot.endsWith("\\") ? resolvedRoot : `${resolvedRoot}\\`;
  return resolvedTarget.startsWith(prefix);
}

function stripOuterQuotes(value) {
  const trimmed = String(value).trim();
  if (
    (trimmed.startsWith("\"") && trimmed.endsWith("\"")) ||
    (trimmed.startsWith("'") && trimmed.endsWith("'"))
  ) {
    return trimmed.slice(1, -1);
  }
  return trimmed;
}

function findSetupExe(nsisDir, version) {
  if (!fs.existsSync(nsisDir)) {
    throw new Error(`未找到版本 ${version} 的 NSIS 安装包。`);
  }
  const needle = `_${version}_`;
  const matches = fs
    .readdirSync(nsisDir)
    .filter((name) => name.includes(needle) && name.endsWith("-setup.exe"))
    .map((name) => {
      const fullPath = path.join(nsisDir, name);
      return { fullPath, name, mtime: fs.statSync(fullPath).mtimeMs };
    })
    .sort((a, b) => b.mtime - a.mtime);
  if (matches.length === 0) {
    throw new Error(`未找到版本 ${version} 的 NSIS 安装包。`);
  }
  return matches[0];
}

function planRelocation(prevInstallDir, destDir, tempDir, localAppData) {
  const dest = destDir;
  const sameDir =
    prevInstallDir && normalizeWinPath(prevInstallDir) === normalizeWinPath(destDir);
  const relocate = Boolean(prevInstallDir) && !sameDir;
  const cleanup =
    relocate &&
    (isUnderDir(prevInstallDir, tempDir) || isUnderDir(prevInstallDir, localAppData));
  return {
    destDir: dest,
    migrateFrom: relocate ? path.win32.join(prevInstallDir, "data") : null,
    cleanupDir: cleanup ? prevInstallDir : null
  };
}

function migrateDataDir(fromData, toData) {
  const sourceDb = path.join(fromData, "monitor.sqlite3");
  if (!fs.existsSync(sourceDb)) {
    return "skip-no-source";
  }
  const destDb = path.join(toData, "monitor.sqlite3");
  if (fs.existsSync(destDb)) {
    return "skip-dest-has-db";
  }
  fs.mkdirSync(path.dirname(toData), { recursive: true });
  if (fs.existsSync(toData)) {
    const leftovers = fs.readdirSync(toData);
    if (leftovers.length > 0) {
      throw new Error(`目标数据目录已有内容，未迁移：${toData}`);
    }
    fs.rmdirSync(toData);
  }
  fs.renameSync(fromData, toData);
  return "moved";
}

function cleanupTempInstall(prevDir, tempDir) {
  cleanupAbandonedInstall(prevDir, tempDir);
}

function cleanupAbandonedInstall(prevDir, tempDir) {
  if (!prevDir || !fs.existsSync(prevDir)) {
    return;
  }
  for (const name of [`${BINARY_NAME}.exe`, "monitor-bench.exe", "uninstall.exe"]) {
    const filePath = path.join(prevDir, name);
    if (fs.existsSync(filePath)) {
      fs.unlinkSync(filePath);
    }
  }
  let current = prevDir;
  try {
    fs.rmdirSync(current);
  } catch (error) {
    if (error && error.code === "ENOTEMPTY") {
      return;
    }
    if (error && error.code !== "ENOENT") {
      throw error;
    }
  }
  if (!tempDir) {
    return;
  }
  current = path.win32.dirname(current);
  while (
    current &&
    isUnderDir(current, tempDir) &&
    normalizeWinPath(current) !== normalizeWinPath(tempDir)
  ) {
    try {
      fs.rmdirSync(current);
    } catch (error) {
      if (error && error.code !== "ENOTEMPTY" && error.code !== "ENOENT") {
        throw error;
      }
      break;
    }
    current = path.win32.dirname(current);
  }
}

function quotePsSingle(value) {
  return `'${String(value).replace(/'/g, "''")}'`;
}

function readPreviousInstallDir(productName, spawn = childProcess.spawnSync) {
  return queryUninstallLocation(productName, spawn);
}

function queryUninstallLocation(productName, spawn = childProcess.spawnSync) {
  const key = UNINSTALL_KEY_PREFIX + productName;
  const command = [
    "$k = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey(" + quotePsSingle(key) + ");",
    "if ($null -eq $k) { exit 0 };",
    "[Console]::Out.Write($k.GetValue('InstallLocation'))"
  ].join(" ");
  const result = spawn(
    "powershell.exe",
    ["-NoLogo", "-NoProfile", "-Command", command],
    { encoding: "utf8" }
  );
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    return null;
  }
  const raw = (result.stdout || "").trim();
  if (!raw) {
    return null;
  }
  return stripOuterQuotes(raw);
}

function hasMonitorDb(dir) {
  return Boolean(dir) && fs.existsSync(path.join(dir, "data", "monitor.sqlite3"));
}

function discoverPreviousInstallDir(
  productName,
  localAppData,
  destDir,
  spawn = childProcess.spawnSync
) {
  const seen = new Set();
  const candidates = [];
  function add(dir) {
    if (!dir) {
      return;
    }
    const key = normalizeWinPath(dir);
    if (seen.has(key)) {
      return;
    }
    seen.add(key);
    candidates.push(dir);
  }
  const names = [productName, ...LEGACY_PRODUCT_NAMES];
  for (const name of names) {
    add(queryUninstallLocation(name, spawn));
    if (localAppData) {
      // 文件系统探测用宿主分隔符：Ubuntu CI 上 win32.join 会拼出不存在的路径。
      add(path.join(localAppData, name));
    }
  }
  const destKey = destDir ? normalizeWinPath(destDir) : null;
  const withDb = candidates.filter((dir) => hasMonitorDb(dir));
  const otherWithDb = destKey
    ? withDb.filter((dir) => normalizeWinPath(dir) !== destKey)
    : withDb;
  if (otherWithDb.length > 0) {
    return otherWithDb[0];
  }
  if (withDb.length > 0) {
    return withDb[0];
  }
  return null;
}

function removeRegistryKey(key, spawn = childProcess.spawnSync) {
  const command =
    "Remove-Item -LiteralPath " +
    quotePsSingle(`HKCU:\\${key}`) +
    " -Recurse -Force -ErrorAction SilentlyContinue";
  const result = spawn("powershell.exe", ["-NoLogo", "-NoProfile", "-Command", command], {
    encoding: "utf8"
  });
  if (result.error) {
    throw result.error;
  }
}

function unlinkIfExists(filePath) {
  if (filePath && fs.existsSync(filePath)) {
    fs.unlinkSync(filePath);
  }
}

function removeLegacyProductTraces(legacyNames, env, spawn = childProcess.spawnSync) {
  const startMenu = env.APPDATA
    ? path.join(env.APPDATA, "Microsoft", "Windows", "Start Menu", "Programs")
    : null;
  const desktop = env.USERPROFILE ? path.join(env.USERPROFILE, "Desktop") : null;
  for (const name of legacyNames) {
    removeRegistryKey(UNINSTALL_KEY_PREFIX + name, spawn);
    removeRegistryKey(MANUFACTURER_KEY_PREFIX + name, spawn);
    if (startMenu) {
      unlinkIfExists(path.join(startMenu, `${name}.lnk`));
    }
    if (desktop) {
      unlinkIfExists(path.join(desktop, `${name}.lnk`));
    }
  }
}

function stopMonitorProcess(spawn = childProcess.spawnSync) {
  const listed = spawn("tasklist.exe", ["/FI", `IMAGENAME eq ${BINARY_NAME}.exe`, "/NH"], {
    encoding: "utf8"
  });
  const output = `${listed.stdout || ""}${listed.stderr || ""}`;
  if (!output.toLowerCase().includes(`${BINARY_NAME}.exe`.toLowerCase())) {
    return;
  }
  console.log(`停止正在运行的 ${BINARY_NAME} 进程。`);
  const killed = spawn("taskkill.exe", ["/F", "/IM", `${BINARY_NAME}.exe`], {
    encoding: "utf8"
  });
  if (killed.status !== 0 && killed.status !== 128) {
    throw new Error(`无法结束正在运行的 ${BINARY_NAME}，安装会覆盖锁定文件。`);
  }
  const deadline = Date.now() + 20_000;
  while (Date.now() < deadline) {
    spawn("powershell.exe", ["-NoLogo", "-NoProfile", "-Command", "Start-Sleep -Milliseconds 250"], {
      encoding: "utf8"
    });
    const again = spawn("tasklist.exe", ["/FI", `IMAGENAME eq ${BINARY_NAME}.exe`, "/NH"], {
      encoding: "utf8"
    });
    const text = `${again.stdout || ""}${again.stderr || ""}`;
    if (!text.toLowerCase().includes(`${BINARY_NAME}.exe`.toLowerCase())) {
      return;
    }
  }
  throw new Error(`无法结束正在运行的 ${BINARY_NAME}，安装会覆盖锁定文件。`);
}

function runSetup(setupPath, destDir, spawn = childProcess.spawnSync) {
  // NSIS /D= 必须是最后一个参数，路径两边不能加引号。
  const command = `& ${quotePsSingle(setupPath)} /S /D=${destDir}`;
  const result = spawn(
    "powershell.exe",
    ["-NoLogo", "-NoProfile", "-Command", command],
    { encoding: "utf8", windowsHide: true }
  );
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    const detail = (result.stderr || result.stdout || "").trim();
    throw new Error(
      detail
        ? `安装失败，退出码 ${result.status}。${detail}`
        : `安装失败，退出码 ${result.status}。`
    );
  }
}

function main(root = DEFAULT_ROOT, env = process.env) {
  if (process.platform !== "win32") {
    throw new Error("家宽监控 v1 只提供 Windows 11 NSIS current-user 安装。");
  }
  const paths = monitorPaths(root);
  const version = readPackageVersion(root);
  const productName = readProductName(root);
  const setup = findSetupExe(paths.nsisDir, version);
  const destDir = defaultInstallDir(env.LOCALAPPDATA, productName);
  const prevInstallDir = discoverPreviousInstallDir(
    productName,
    env.LOCALAPPDATA,
    destDir
  );
  const plan = planRelocation(
    prevInstallDir,
    destDir,
    env.TEMP || env.TMP,
    env.LOCALAPPDATA
  );
  stopMonitorProcess();
  console.log(`正在静默安装 ${setup.name}`);
  console.log(`安装目录 ${destDir}`);
  runSetup(setup.fullPath, destDir);
  if (plan.migrateFrom) {
    const outcome = migrateDataDir(plan.migrateFrom, path.win32.join(destDir, "data"));
    if (outcome === "moved") {
      console.log(`已将数据目录从 ${plan.migrateFrom} 迁到 ${path.win32.join(destDir, "data")}`);
    }
  }
  if (plan.cleanupDir && fs.existsSync(plan.cleanupDir)) {
    cleanupAbandonedInstall(plan.cleanupDir, env.TEMP || env.TMP);
  }
  removeLegacyProductTraces(LEGACY_PRODUCT_NAMES, env);
  console.log("安装完成。");
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
  BINARY_NAME,
  LEGACY_PRODUCT_NAMES,
  cleanupAbandonedInstall,
  cleanupTempInstall,
  defaultInstallDir,
  discoverPreviousInstallDir,
  findSetupExe,
  isUnderDir,
  migrateDataDir,
  monitorPaths,
  planRelocation,
  readPackageVersion,
  readPreviousInstallDir,
  readProductName,
  stripOuterQuotes
};

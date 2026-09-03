"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { test } = require("node:test");

const {
  cleanupTempInstall,
  defaultInstallDir,
  discoverPreviousInstallDir,
  findSetupExe,
  isUnderDir,
  migrateDataDir,
  planRelocation,
  readPackageVersion,
  readProductName,
  startMonitorApp,
  stripOuterQuotes
} = require("../scripts/nsis-silent-install.js");

function withTemporaryDirectory(fn) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "nsis-silent-install-"));
  try {
    fn(directory);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

test("defaultInstallDir 使用 LocalAppData 与产品名", () => {
  assert.equal(
    defaultInstallDir("C:\\Users\\lyh\\AppData\\Local", "ResiWatch"),
    "C:\\Users\\lyh\\AppData\\Local\\ResiWatch"
  );
});

test("isUnderDir 识别 Temp 前缀，不把 LocalAppData 产品目录当 Temp", () => {
  const tempDir = "C:\\Users\\lyh\\AppData\\Local\\Temp";
  const local = "C:\\Users\\lyh\\AppData\\Local";
  assert.equal(
    isUnderDir(`${tempDir}\\nsis-hook-test\\app`, tempDir),
    true
  );
  assert.equal(isUnderDir(`${local}\\ResiWatch`, tempDir), false);
  assert.equal(isUnderDir(`${local}\\TemporaryFiles`, tempDir), false);
});

test("planRelocation 从 Temp 或旧产品目录迁 data", () => {
  const tempDir = "C:\\Users\\lyh\\AppData\\Local\\Temp";
  const local = "C:\\Users\\lyh\\AppData\\Local";
  const destDir = `${local}\\ResiWatch`;
  const prevTemp = `${tempDir}\\nsis-hook-test\\app`;
  const fromTemp = planRelocation(prevTemp, destDir, tempDir, local);
  assert.equal(fromTemp.destDir, destDir);
  assert.equal(fromTemp.migrateFrom, `${prevTemp}\\data`);
  assert.equal(fromTemp.cleanupDir, prevTemp);

  const prevChinese = `${local}\\家宽流量监控`;
  const fromLegacy = planRelocation(prevChinese, destDir, tempDir, local);
  assert.equal(fromLegacy.migrateFrom, `${prevChinese}\\data`);
  assert.equal(fromLegacy.cleanupDir, prevChinese);

  const alreadyGood = planRelocation(destDir, destDir, tempDir, local);
  assert.equal(alreadyGood.migrateFrom, null);
  assert.equal(alreadyGood.cleanupDir, null);
});

test("stripOuterQuotes 去掉注册表 InstallLocation 的引号", () => {
  assert.equal(
    stripOuterQuotes("\"C:\\Users\\lyh\\AppData\\Local\\Temp\\nsis-hook-test\\app\""),
    "C:\\Users\\lyh\\AppData\\Local\\Temp\\nsis-hook-test\\app"
  );
  assert.equal(stripOuterQuotes("C:\\plain"), "C:\\plain");
});

test("migrateDataDir 在目标无库时整目录改名，目标已有库则跳过", () => {
  withTemporaryDirectory((directory) => {
    const fromData = path.join(directory, "old", "data");
    const toData = path.join(directory, "app", "data");
    fs.mkdirSync(fromData, { recursive: true });
    fs.writeFileSync(path.join(fromData, "monitor.sqlite3"), "db");
    fs.writeFileSync(path.join(fromData, "monitor.sqlite3-wal"), "wal");
    assert.equal(migrateDataDir(fromData, toData), "moved");
    assert.equal(fs.readFileSync(path.join(toData, "monitor.sqlite3"), "utf8"), "db");
    assert.equal(fs.existsSync(fromData), false);

    const from2 = path.join(directory, "old2", "data");
    fs.mkdirSync(from2, { recursive: true });
    fs.writeFileSync(path.join(from2, "monitor.sqlite3"), "legacy");
    assert.equal(migrateDataDir(from2, toData), "skip-dest-has-db");
    assert.equal(fs.readFileSync(path.join(toData, "monitor.sqlite3"), "utf8"), "db");
    assert.equal(fs.existsSync(path.join(from2, "monitor.sqlite3")), true);
  });
});

test("findSetupExe 按版本过滤并取最新安装包", () => {
  withTemporaryDirectory((directory) => {
    const stale = path.join(directory, "ResiWatch_0.2.0_x64-setup.exe");
    const fresh = path.join(directory, "ResiWatch_0.3.0_x64-setup.exe");
    fs.writeFileSync(stale, "old");
    fs.writeFileSync(fresh, "new");
    const past = new Date("2026-08-01T00:00:00Z");
    fs.utimesSync(stale, past, past);
    const found = findSetupExe(directory, "0.3.0");
    assert.equal(found.name, "ResiWatch_0.3.0_x64-setup.exe");
    assert.throws(() => findSetupExe(directory, "9.9.9"), /未找到版本 9.9.9/);
  });
});

test("cleanupTempInstall 删除 Temp 旧安装并去掉空父目录", () => {
  withTemporaryDirectory((directory) => {
    const tempDir = path.join(directory, "Temp");
    const prev = path.join(tempDir, "nsis-hook-test", "app");
    fs.mkdirSync(prev, { recursive: true });
    fs.writeFileSync(path.join(prev, "residential-monitor.exe"), "exe");
    fs.writeFileSync(path.join(prev, "uninstall.exe"), "un");
    cleanupTempInstall(prev, tempDir);
    assert.equal(fs.existsSync(path.join(tempDir, "nsis-hook-test")), false);
    assert.equal(fs.existsSync(tempDir), true);
  });
});

test("discoverPreviousInstallDir 优先选带库的旧目录，而不是空的新安装目录", () => {
  withTemporaryDirectory((directory) => {
    const local = path.join(directory, "Local");
    const dest = path.join(local, "ResiWatch");
    const legacy = path.join(local, "家宽流量监控");
    fs.mkdirSync(dest, { recursive: true });
    fs.writeFileSync(path.join(dest, "residential-monitor.exe"), "new");
    fs.mkdirSync(path.join(legacy, "data"), { recursive: true });
    fs.writeFileSync(path.join(legacy, "data", "monitor.sqlite3"), "db");
    const spawn = () => ({ status: 0, stdout: dest, error: null });
    const found = discoverPreviousInstallDir("ResiWatch", local, dest, spawn);
    assert.equal(path.resolve(found), path.resolve(legacy));
  });
});

test("仓库 tauri.conf.json 的 productName 与 package.json version 可读", () => {
  assert.equal(readProductName(path.resolve(__dirname, "..")), "ResiWatch");
  assert.match(readPackageVersion(path.resolve(__dirname, "..")), /^\d+\.\d+\.\d+/);
});

test("startMonitorApp 以 detached 启动安装目录中的 exe", () => {
  withTemporaryDirectory((directory) => {
    const exe = path.win32.join(directory, "residential-monitor.exe");
    fs.writeFileSync(exe, "x");
    const calls = [];
    const child = {
      unref() {
        calls.push("unref");
      }
    };
    startMonitorApp(directory, (file, args, options) => {
      calls.push({ file, args, options });
      return child;
    });
    assert.equal(calls[0].file, exe);
    assert.deepEqual(calls[0].args, []);
    assert.equal(calls[0].options.detached, true);
    assert.equal(calls[0].options.stdio, "ignore");
    assert.equal(calls[0].options.windowsHide, false);
    assert.equal(calls[1], "unref");
  });
});

test("startMonitorApp 在缺少 exe 时失败", () => {
  withTemporaryDirectory((directory) => {
    assert.throws(() => startMonitorApp(directory, () => {
      throw new Error("不应启动");
    }), /未找到/);
  });
});

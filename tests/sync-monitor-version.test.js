"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { test } = require("node:test");

const {
  syncMonitorVersion
} = require("../scripts/sync-monitor-version.js");

function withTemporaryDirectory(fn) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "sync-monitor-version-"));
  try {
    fn(directory);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

function writeTree(root, version, options = {}) {
  const eol = options.eol || "\n";
  const monitor = path.join(root, "residential-monitor");
  const tauri = path.join(monitor, "src-tauri");
  fs.mkdirSync(tauri, { recursive: true });
  fs.writeFileSync(
    path.join(monitor, "package.json"),
    `{\n  "name": "residential-monitor",\n  "version": "${version}"\n}\n`,
    "utf8"
  );
  const lock = [
    "{",
    '  "name": "residential-monitor",',
    `  "version": "${options.lockVersion || version}",`,
    '  "packages": {',
    '    "": {',
    '      "name": "residential-monitor",',
    `      "version": "${options.lockPackageVersion || options.lockVersion || version}"`,
    "    },",
    '    "node_modules/yocto-queue": {',
    '      "version": "0.1.0"',
    "    }",
    "  }",
    "}",
    ""
  ].join(eol);
  fs.writeFileSync(path.join(monitor, "package-lock.json"), lock, "utf8");
  fs.writeFileSync(
    path.join(tauri, "tauri.conf.json"),
    `{${eol}  "productName": "家宽流量监控",${eol}  "version": "${options.tauriVersion || version}"${eol}}${eol}`,
    "utf8"
  );
  const cargoToml = [
    "[package]",
    'name = "residential-monitor"',
    `version = "${options.cargoVersion || version}"`,
    "",
    "[[bin]]",
    'name = "residential-monitor"',
    'path = "src/main.rs"',
    "",
    "[dependencies]",
    'hyper-util = { version = "0.1", features = [] }',
    ""
  ].join(eol);
  fs.writeFileSync(path.join(tauri, "Cargo.toml"), cargoToml, "utf8");
  const cargoLock = [
    "[[package]]",
    'name = "residential-monitor"',
    `version = "${options.cargoLockVersion || options.cargoVersion || version}"`,
    "",
    "[[package]]",
    'name = "vswhom"',
    'version = "0.1.0"',
    ""
  ].join(eol);
  fs.writeFileSync(path.join(tauri, "Cargo.lock"), cargoLock, "utf8");
}

test("写入模式把 package.json 版本同步到 Tauri、Cargo 与 lockfile", () => {
  withTemporaryDirectory((directory) => {
    writeTree(directory, "0.1.0");
    fs.writeFileSync(
      path.join(directory, "residential-monitor", "package.json"),
      '{\n  "name": "residential-monitor",\n  "version": "0.2.0"\n}\n',
      "utf8"
    );

    const result = syncMonitorVersion(directory);
    assert.equal(result.version, "0.2.0");
    assert.equal(result.changed.length, 4);

    const tauri = JSON.parse(
      fs.readFileSync(
        path.join(directory, "residential-monitor", "src-tauri", "tauri.conf.json"),
        "utf8"
      )
    );
    assert.equal(tauri.version, "0.2.0");

    const cargoToml = fs.readFileSync(
      path.join(directory, "residential-monitor", "src-tauri", "Cargo.toml"),
      "utf8"
    );
    assert.match(cargoToml, /\[package\]\r?\nname = "residential-monitor"\r?\nversion = "0.2.0"/);
    assert.match(cargoToml, /\[\[bin\]\]\r?\nname = "residential-monitor"\r?\npath = /);
    assert.match(cargoToml, /hyper-util = \{ version = "0.1"/);

    const cargoLock = fs.readFileSync(
      path.join(directory, "residential-monitor", "src-tauri", "Cargo.lock"),
      "utf8"
    );
    assert.match(cargoLock, /name = "residential-monitor"\r?\nversion = "0.2.0"/);
    assert.match(cargoLock, /name = "vswhom"\r?\nversion = "0.1.0"/);

    const lock = JSON.parse(
      fs.readFileSync(
        path.join(directory, "residential-monitor", "package-lock.json"),
        "utf8"
      )
    );
    assert.equal(lock.version, "0.2.0");
    assert.equal(lock.packages[""].version, "0.2.0");
    assert.equal(lock.packages["node_modules/yocto-queue"].version, "0.1.0");
  });
});

test("检查模式发现漂移时不写文件", () => {
  withTemporaryDirectory((directory) => {
    writeTree(directory, "0.1.0");
    fs.writeFileSync(
      path.join(directory, "residential-monitor", "package.json"),
      '{\n  "name": "residential-monitor",\n  "version": "0.2.0"\n}\n',
      "utf8"
    );
    const cargoPath = path.join(
      directory,
      "residential-monitor",
      "src-tauri",
      "Cargo.toml"
    );
    const before = fs.readFileSync(cargoPath, "utf8");

    const result = syncMonitorVersion(directory, { check: true });
    assert.equal(result.check, true);
    assert.equal(result.version, "0.2.0");
    assert.ok(result.changed.some((item) => item.label.endsWith("Cargo.toml")));
    assert.equal(fs.readFileSync(cargoPath, "utf8"), before);
  });
});

test("已对齐时写入模式不改内容", () => {
  withTemporaryDirectory((directory) => {
    writeTree(directory, "0.2.0");
    const cargoPath = path.join(
      directory,
      "residential-monitor",
      "src-tauri",
      "Cargo.toml"
    );
    const before = fs.readFileSync(cargoPath, "utf8");
    const result = syncMonitorVersion(directory);
    assert.deepEqual(result.changed, []);
    assert.equal(fs.readFileSync(cargoPath, "utf8"), before);
  });
});

test("保留 Cargo.toml 的 CRLF", () => {
  withTemporaryDirectory((directory) => {
    writeTree(directory, "0.1.0", { eol: "\r\n" });
    fs.writeFileSync(
      path.join(directory, "residential-monitor", "package.json"),
      '{\n  "name": "residential-monitor",\n  "version": "1.2.3"\n}\n',
      "utf8"
    );
    syncMonitorVersion(directory);
    const cargoToml = fs.readFileSync(
      path.join(directory, "residential-monitor", "src-tauri", "Cargo.toml"),
      "utf8"
    );
    assert.equal(cargoToml.includes("\r\n"), true);
    assert.match(cargoToml, /version = "1.2.3"/);
  });
});

test("非法 SemVer 失败", () => {
  withTemporaryDirectory((directory) => {
    writeTree(directory, "0.1.0");
    fs.writeFileSync(
      path.join(directory, "residential-monitor", "package.json"),
      '{\n  "name": "residential-monitor",\n  "version": "latest"\n}\n',
      "utf8"
    );
    assert.throws(
      () => syncMonitorVersion(directory),
      /不是有效 SemVer/
    );
  });
});

test("缺少 tauri.conf.json 失败", () => {
  withTemporaryDirectory((directory) => {
    writeTree(directory, "0.2.0");
    fs.unlinkSync(
      path.join(directory, "residential-monitor", "src-tauri", "tauri.conf.json")
    );
    assert.throws(
      () => syncMonitorVersion(directory),
      /缺少 residential-monitor\/src-tauri\/tauri.conf.json/
    );
  });
});

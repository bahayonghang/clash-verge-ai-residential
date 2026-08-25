#!/usr/bin/env node
/**
 * 从 icon-source.png 调用 `tauri icon`，只把 Windows 需要的文件拷回 icons/。
 * 不覆盖 tray-*.png、icon-source.png 与 *.json。
 */
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const appRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const iconsDir = path.join(appRoot, "src-tauri", "icons");
const source = path.join(iconsDir, "icon-source.png");
const keep = ["icon.ico", "icon.png", "32x32.png", "128x128.png", "128x128@2x.png"];

if (!fs.existsSync(source)) {
  console.error("build-icons: 缺少 src-tauri/icons/icon-source.png");
  process.exit(1);
}

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "rm-icons-"));

function runTauriIcon() {
  const npxBin = process.platform === "win32" ? "npx.cmd" : "npx";
  const result = spawnSync(
    npxBin,
    ["--no-install", "tauri", "icon", source, "-o", tmp],
    { cwd: appRoot, stdio: "inherit" }
  );
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(`tauri icon 退出码 ${result.status}`);
  }
}

try {
  runTauriIcon();
  const written = fs.readdirSync(tmp);
  for (const name of keep) {
    const from = path.join(tmp, name);
    if (!fs.existsSync(from)) {
      console.error(`build-icons: tauri icon 未生成 ${name}；输出为 ${written.join(", ")}`);
      process.exit(1);
    }
    fs.copyFileSync(from, path.join(iconsDir, name));
  }
  console.log(`build-icons: 已写入 ${keep.join(", ")}`);
} finally {
  fs.rmSync(tmp, { recursive: true, force: true });
}

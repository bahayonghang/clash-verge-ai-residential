#!/usr/bin/env node
/**
 * 断言 Windows 安装图标：icon.ico 含 16/32/48/256 层，配套 PNG 尺寸正确。
 * 不引入额外依赖；只读 ICO 目录表。
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const appRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const iconsDir = path.join(appRoot, "src-tauri", "icons");

const REQUIRED_ICO_SIZES = [16, 32, 48, 256];
const REQUIRED_PNG = [
  ["32x32.png", 32],
  ["128x128.png", 128],
  ["128x128@2x.png", 256],
  ["icon.png", 512],
  ["icon-source.png", 1024]
];

function fail(message) {
  console.error(`check-icons: ${message}`);
  process.exit(1);
}

/**
 * @param {Buffer} buf
 * @returns {{ type: number, sizes: number[], entries: { width: number, imageOffset: number }[] }}
 */
export function parseIcoDirectory(buf) {
  if (buf.length < 6) {
    throw new Error("ICO 短于目录头");
  }
  const reserved = buf.readUInt16LE(0);
  const type = buf.readUInt16LE(2);
  const count = buf.readUInt16LE(4);
  if (reserved !== 0) {
    throw new Error(`ICO reserved 应为 0，实际 ${reserved}`);
  }
  if (type !== 1) {
    throw new Error(`ICO type 应为 1，实际 ${type}`);
  }
  if (count < 1) {
    throw new Error("ICO 没有图像层");
  }
  const need = 6 + count * 16;
  if (buf.length < need) {
    throw new Error(`ICO 目录表不完整，需要 ${need} 字节`);
  }
  const entries = [];
  for (let i = 0; i < count; i += 1) {
    const entry = 6 + i * 16;
    const raw = buf.readUInt8(entry);
    entries.push({
      width: raw === 0 ? 256 : raw,
      imageOffset: buf.readUInt32LE(entry + 12)
    });
  }
  return { type, sizes: entries.map((item) => item.width), entries };
}

function pngSize(filePath) {
  const buf = fs.readFileSync(filePath);
  if (buf.length < 24 || buf.toString("ascii", 1, 4) !== "PNG") {
    throw new Error(`${path.basename(filePath)} 不是 PNG`);
  }
  return { width: buf.readUInt32BE(16), height: buf.readUInt32BE(20) };
}

function main() {
  const icoPath = path.join(iconsDir, "icon.ico");
  if (!fs.existsSync(icoPath)) {
    fail(`缺少 ${icoPath}`);
  }

  let parsed;
  try {
    parsed = parseIcoDirectory(fs.readFileSync(icoPath));
  } catch (err) {
    fail(err instanceof Error ? err.message : String(err));
  }

  const have = new Set(parsed.sizes);
  const missing = REQUIRED_ICO_SIZES.filter((size) => !have.has(size));
  if (missing.length > 0) {
    fail(`icon.ico 缺少尺寸 ${missing.join(", ")}；现有 ${[...have].sort((a, b) => a - b).join(", ")}`);
  }

  const icoBuf = fs.readFileSync(icoPath);
  const layer256 = parsed.entries.find((item) => item.width === 256);
  if (!layer256 || layer256.imageOffset + 8 > icoBuf.length) {
    fail("icon.ico 256 层偏移无效");
  }
  if (icoBuf.toString("ascii", layer256.imageOffset + 1, layer256.imageOffset + 4) !== "PNG") {
    fail("icon.ico 256 层应为 PNG 压缩");
  }

  const conf = JSON.parse(
    fs.readFileSync(path.join(appRoot, "src-tauri", "tauri.conf.json"), "utf8")
  );
  const listed = conf?.bundle?.icon;
  if (!Array.isArray(listed) || listed.length === 0) {
    fail("tauri.conf.json bundle.icon 为空");
  }
  for (const rel of listed) {
    if (!fs.existsSync(path.join(appRoot, "src-tauri", rel))) {
      fail(`bundle.icon 缺少文件 ${rel}`);
    }
  }

  for (const [name, expected] of REQUIRED_PNG) {
    const filePath = path.join(iconsDir, name);
    if (!fs.existsSync(filePath)) {
      fail(`缺少 ${name}`);
    }
    let size;
    try {
      size = pngSize(filePath);
    } catch (err) {
      fail(err instanceof Error ? err.message : String(err));
    }
    if (size.width !== expected || size.height !== expected) {
      fail(`${name} 应为 ${expected}×${expected}，实际 ${size.width}×${size.height}`);
    }
  }

  const trays = [
    "tray-collecting.png",
    "tray-connecting.png",
    "tray-paused.png",
    "tray-fault.png"
  ];
  for (const name of trays) {
    if (!fs.existsSync(path.join(iconsDir, name))) {
      fail(`缺少 ${name}`);
    }
  }

  console.log(
    `check-icons: icon.ico 层 ${parsed.sizes.sort((a, b) => a - b).join(", ")}；PNG 与托盘齐全`
  );
}

const invoked = path.normalize(fileURLToPath(import.meta.url));
const argvPath = process.argv[1] ? path.normalize(path.resolve(process.argv[1])) : "";
if (invoked === argvPath) {
  main();
}

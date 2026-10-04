"use strict";

const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { discoverEntries } = require("../../../../scripts/check-harness-environment.js");

const parent = fs.mkdtempSync(path.join(os.tmpdir(), "harness-discovery-review-"));
const directory = path.join(parent, String.fromCodePoint(0x4e2d, 0x6587) + "'s tool");
const expected = path.join(directory, "trellis.ps1");
const originalPath = process.env.PATH;
let report;
try {
  fs.mkdirSync(directory);
  fs.writeFileSync(expected, "throw 'Discovery must not execute this fixture'", "utf8");
  process.env.PATH = directory + path.delimiter + originalPath;
  const entries = discoverEntries("trellis", process.cwd());
  report = { expected, selected: entries[0], samePath: entries[0] === expected, selectedExists: fs.existsSync(entries[0]), count: entries.length };
} catch (error) {
  report = { expected, error: error.message };
} finally {
  process.env.PATH = originalPath;
  const relative = path.relative(os.tmpdir(), parent);
  if (path.isAbsolute(relative) || relative.startsWith("..") || !relative.startsWith("harness-discovery-review-")) {
    throw new Error("临时清理路径越界");
  }
  fs.rmSync(parent, { recursive: true, force: true });
}
const evidence = process.argv[2] === "after" ? "review-discovery-after.json" : "review-discovery-before.json";
fs.writeFileSync(path.join(__dirname, evidence), JSON.stringify(report, null, 2) + "\n");
console.log(JSON.stringify(report));

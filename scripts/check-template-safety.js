"use strict";

const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const scriptPath = path.join(root, "clash-verge-ai-residential.js");
const source = fs.readFileSync(scriptPath, "utf8");
const failures = [];

function readStringProperty(objectSource, property) {
  const escaped = property.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = objectSource.match(new RegExp(`(?:^|\\n)\\s*(?:["']${escaped}["']|${escaped})\\s*:\\s*["']([^"']*)["']`));
  return match ? match[1] : null;
}

const templateMatch = source.match(/const\s+HOME_PROXY_TEMPLATE\s*=\s*\{([\s\S]*?)\n\};/);
if (!templateMatch) {
  failures.push("无法定位 HOME_PROXY_TEMPLATE");
} else {
  const template = templateMatch[1];
  const allowed = new Set(["", "xxx"]);
  for (const property of ["server", "username", "password"]) {
    const value = readStringProperty(template, property);
    if (value === null) {
      failures.push(`HOME_PROXY_TEMPLATE.${property} 缺失或不是字符串`);
    } else if (!allowed.has(value)) {
      failures.push(`HOME_PROXY_TEMPLATE.${property} 不能在公共模板中保存真实值`);
    }
  }
}

const forbiddenPatterns = [
  { name: "private key", regex: /-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/ },
  { name: "GitHub token", regex: /\b(?:ghp|github_pat)_[A-Za-z0-9_]{20,}\b/ },
  { name: "AWS access key", regex: /\bAKIA[0-9A-Z]{16}\b/ },
  { name: "generic bearer token", regex: /\bBearer\s+[A-Za-z0-9._~+\/-]{24,}=*\b/i }
];

const scanExtensions = new Set([".js", ".json", ".md", ".yml", ".yaml"]);
const ignoredDirectories = new Set([".git", "node_modules"]);
const ignoredLocalFiles = new Set([
  "clash-verge-ai-residential.local.toml",
  "clash-verge-ai-residential.local.js"
]);

function isIgnoredLocalFile(relative) {
  return ignoredLocalFiles.has(relative) || relative.endsWith(".local.js");
}

function walk(directory) {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    if (ignoredDirectories.has(entry.name)) continue;
    const absolute = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      walk(absolute);
      continue;
    }
    if (!scanExtensions.has(path.extname(entry.name))) continue;
    const relative = path.relative(root, absolute);
    if (isIgnoredLocalFile(relative)) continue;
    const content = fs.readFileSync(absolute, "utf8");
    for (const pattern of forbiddenPatterns) {
      if (pattern.regex.test(content)) {
        failures.push(`${relative}: 检测到疑似 ${pattern.name}`);
      }
    }
  }
}

walk(root);

if (failures.length > 0) {
  console.error("Template safety check failed:");
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}

console.log("Template safety check passed.");

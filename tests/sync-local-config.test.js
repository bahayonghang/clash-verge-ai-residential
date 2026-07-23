"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");

const {
  parseHomeProxyToml,
  syncLocalConfig,
  validateHomeProxyConfig
} = require("../scripts/sync-local-config.js");

const root = path.resolve(__dirname, "..");
const templatePath = path.join(root, "clash-verge-ai-residential.js");

let passed = 0;
function test(name, fn) {
  try {
    fn();
    passed += 1;
    console.log(`ok ${passed} - ${name}`);
  } catch (error) {
    console.error(`not ok - ${name}`);
    throw error;
  }
}

function withTemporaryDirectory(fn) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "clash-verge-ai-residential-"));
  try {
    fn(directory);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

const validToml = `# This file intentionally contains test-only values.
[home_proxy]
name = "家宽-SOCKS5"
type = "socks5"
server = "home.example.test"
port = 1080
username = "home-user"
password = "home-pass"
udp = true
dialer-proxy = "🚀节点选择"
`;

test("TOML 配置会同步到新的本地脚本，且不修改公开模板", () => {
  withTemporaryDirectory((directory) => {
    const configPath = path.join(directory, "proxy.local.toml");
    const outputPath = path.join(directory, "proxy.local.js");
    const originalTemplate = fs.readFileSync(templatePath, "utf8");
    fs.writeFileSync(configPath, validToml, "utf8");

    syncLocalConfig({ templatePath, configPath, outputPath });

    const output = fs.readFileSync(outputPath, "utf8");
    assert.match(output, /由 clash-verge-ai-residential\.js 与 proxy\.local\.toml 自动生成/);
    assert.match(output, /server: "home\.example\.test"/);
    assert.match(output, /port: 1080/);
    assert.match(output, /username: "home-user"/);
    assert.match(output, /"dialer-proxy": "🚀节点选择"/);
    assert.doesNotMatch(output, /server: "xxx"/);
    assert.equal(fs.readFileSync(templatePath, "utf8"), originalTemplate);
  });
});

test("同步前会拒绝不完整或不安全的代理配置", () => {
  const incompleteToml = validToml.replace('port = 1080\n', "");
  assert.throws(
    () => validateHomeProxyConfig(parseHomeProxyToml(incompleteToml), "家宽-SOCKS5"),
    /缺少字段 port/
  );

  const invalidPortToml = validToml.replace("port = 1080", "port = 65536");
  assert.throws(
    () => validateHomeProxyConfig(parseHomeProxyToml(invalidPortToml), "家宽-SOCKS5"),
    /port 必须是 1-65535 的整数/
  );
});

console.log(`${passed} local configuration sync tests passed.`);

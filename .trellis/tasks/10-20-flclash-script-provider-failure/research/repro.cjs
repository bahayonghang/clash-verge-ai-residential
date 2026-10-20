"use strict";

// 仅从公开模板读取，在内存中构造脱敏配置；不读取本地凭据，也不连接网络。
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { createHash } = require("node:crypto");

const root = path.resolve(__dirname, "../../../..");
const scriptPath = path.join(root, "clash-verge-ai-residential.js");
const original = fs.readFileSync(scriptPath, "utf8");
const message = "[AI-家宽] rule-providers 必须是对象，拒绝覆盖已有配置";
const cases = [
  ["omitted", undefined, false, true],
  ["explicit-undefined", undefined, true, true],
  ["null", null, true, false],
  ["empty-mapping", {}, true, true],
  ["empty-array", [], true, false],
  ["string", "sanitized", true, false],
  ["number", 42, true, false],
  ["boolean", false, true, false]
];

function loadScript(regexEnabled) {
  let source = original;
  if (!regexEnabled) {
    for (const name of ["ROUTE_VERTEX_AI_ENDPOINTS", "ROUTE_CURSOR_REPOSITORY_INDEXING"]) {
      const pattern = new RegExp(`^const ${name} = (?:true|false);$`, "m");
      assert.match(source, pattern);
      source = source.replace(pattern, `const ${name} = false;`);
    }
  }
  const sandbox = {
    module: { exports: {} },
    console: { info() {}, warn() {}, error() {} }
  };
  vm.runInNewContext(source, sandbox, { filename: scriptPath });
  return sandbox.module.exports;
}

function fixture() {
  return {
    proxies: [
      { name: "Airport", type: "ss", server: "airport.example.test", port: 443,
        cipher: "aes-128-gcm", password: "fake-airport", udp: true },
      { name: "家宽-SOCKS5", type: "socks5", server: "home.example.test", port: 1080,
        username: "fake-user", password: "fake-password", udp: true }
    ],
    "proxy-groups": [{ name: "Proxy", type: "select", proxies: ["Airport"] }],
    rules: ["MATCH,Proxy"]
  };
}

const rows = [];
for (const regexEnabled of [true, false]) {
  const script = loadScript(regexEnabled);
  for (const [label, value, present, succeeds] of cases) {
    const input = fixture();
    if (present) input["rule-providers"] = structuredClone(value);
    const before = structuredClone(input);
    let output;
    let error;
    try { output = script.main(input, "sanitized-repro"); }
    catch (caught) { error = caught; }
    assert.deepEqual(input, before, `输入应保持不变: ${label}`);
    assert.equal(!error, succeeds, `结果不符合当前脚本契约: ${label}`);
    if (error) assert.equal(error.message, message);
    if (output) {
      assert.equal(output.proxies.length, 2);
      assert.equal(output["proxy-groups"].length, 2);
      assert.equal(Boolean(output["rule-providers"]?.["AI-家宽-DNS-REGEX"]), regexEnabled);
    }
    rows.push({ mode: regexEnabled ? "public-defaults" : "both-regex-switches-off",
      input: label, result: error ? "THROW" : "RETURN", inputUnchanged: true,
      message: error?.message,
      outputProxyCount: output?.proxies.length,
      outputGroupCount: output?.["proxy-groups"].length });
  }
}
process.stdout.write(JSON.stringify({ node: process.version,
  sourceSha256: createHash("sha256").update(original).digest("hex"), rows }, null, 2) + "\n");

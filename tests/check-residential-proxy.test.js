"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { test } = require("node:test");

const {
  createPainter,
  maskPassword,
  parseCommandLineArgs,
  loadProxyConfig,
  evaluateIpSafety,
  renderReport
} = require("../scripts/check-residential-proxy.js");

test("maskPassword 应隐藏密码", () => {
  assert.equal(maskPassword("secret123"), "******");
  assert.equal(maskPassword(""), "******");
});

test("parseCommandLineArgs 应正确解析参数", () => {
  const defaults = parseCommandLineArgs([process.execPath, "check-residential-proxy.js"]);
  assert.match(defaults.configPath, /clash-verge-ai-residential\.local\.toml$/);
  assert.equal(defaults.timeoutSeconds, 15);
  assert.equal(defaults.showHelp, false);

  const custom = parseCommandLineArgs([
    process.execPath,
    "check-residential-proxy.js",
    "my-config.toml",
    "--timeout",
    "30",
    "--help"
  ]);
  assert.match(custom.configPath, /my-config\.toml$/);
  assert.equal(custom.timeoutSeconds, 30);
  assert.equal(custom.showHelp, true);
});

test("evaluateIpSafety - ISP 且无伪装判定为 PASS", () => {
  const result = evaluateIpSafety({
    basicInfo: {
      ip: "1.2.3.4",
      org: "AS701 Verizon Business",
      city: "Richmond",
      region: "Virginia",
      country: "US"
    },
    detailData: {
      asn: { type: "isp", asn: "AS701", name: "Verizon" },
      privacy: { vpn: false, proxy: false, tor: false, relay: false, hosting: false }
    },
    homeProxy: { server: "1.2.3.4" }
  });

  assert.equal(result.status, "PASS");
  assert.equal(result.isIsp, true);
  assert.equal(result.allPrivacyFalse, true);
  assert.equal(result.ipMatchesServer, true);
  assert.match(result.gradeText, /合格/);
});

test("evaluateIpSafety - 机房 IP (Hosting) 判定为 FAIL", () => {
  const result = evaluateIpSafety({
    basicInfo: {
      ip: "5.6.7.8",
      org: "AS14061 DigitalOcean, LLC"
    },
    detailData: {
      asn: { type: "hosting", asn: "AS14061", name: "DigitalOcean" },
      privacy: { vpn: false, proxy: true, tor: false, relay: false, hosting: true }
    },
    homeProxy: { server: "5.6.7.8" }
  });

  assert.equal(result.status, "FAIL");
  assert.equal(result.isHosting, true);
  assert.match(result.gradeText, /不合格/);
});

test("evaluateIpSafety - 商业宽带 (Business) 判定为 WARN", () => {
  const result = evaluateIpSafety({
    basicInfo: {
      ip: "9.10.11.12",
      org: "AS1234 Enterprise Net"
    },
    detailData: {
      asn: { type: "business", asn: "AS1234", name: "Enterprise" },
      privacy: { vpn: false, proxy: false, tor: false, relay: false, hosting: false }
    },
    homeProxy: { server: "9.10.11.12" }
  });

  assert.equal(result.status, "WARN");
  assert.equal(result.isBusiness, true);
  assert.match(result.gradeText, /注意/);
});

test("evaluateIpSafety - ISP 但存在 Privacy 标记判定为 WARN", () => {
  const result = evaluateIpSafety({
    basicInfo: {
      ip: "13.14.15.16",
      org: "AS701 Verizon"
    },
    detailData: {
      asn: { type: "isp", asn: "AS701", name: "Verizon" },
      privacy: { vpn: true, proxy: false, tor: false, relay: false, hosting: false }
    },
    homeProxy: { server: "13.14.15.16" }
  });

  assert.equal(result.status, "WARN");
  assert.equal(result.allPrivacyFalse, false);
  assert.match(result.gradeText, /风险/);
});

test("loadProxyConfig - 拒绝不存在的文件与占位符配置", () => {
  assert.throws(
    () => loadProxyConfig("non-existent-file-path-12345.toml"),
    /配置文件不存在/
  );

  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "proxy-test-"));
  try {
    const placeholderToml = path.join(tempDir, "test.toml");
    fs.writeFileSync(
      placeholderToml,
      `[home_proxy]\nname = "家宽-SOCKS5"\ntype = "socks5"\nserver = "xxx"\nport = 443\nusername = "xxx"\npassword = "xxx"\nudp = true\n`
    );

    assert.throws(
      () => loadProxyConfig(placeholderToml),
      /占位符 \(xxx\)/
    );
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

test("renderReport - 渲染无异常抛出", () => {
  const paint = createPainter(null);
  assert.doesNotThrow(() => {
    renderReport({
      configPath: "clash-verge-ai-residential.local.toml",
      homeProxy: {
        name: "家宽-SOCKS5",
        type: "socks5",
        server: "1.2.3.4",
        port: 1080,
        username: "user",
        password: "pwd",
        "dialer-proxy": "节点选择"
      },
      connectivity: {
        ok: true,
        displayCommand: "curl --proxy socks5h://user:******@1.2.3.4:1080 https://ipinfo.io",
        durationMs: 120,
        basicInfo: {
          ip: "1.2.3.4",
          city: "Richmond",
          region: "Virginia",
          country: "US",
          timezone: "America/New_York",
          org: "AS701 Verizon"
        }
      },
      evaluation: {
        exitIp: "1.2.3.4",
        ipMatchesServer: true,
        asnType: "isp",
        asnNumber: "AS701",
        asnName: "Verizon",
        privacyChecked: true,
        privacyFlags: { vpn: false, proxy: false, tor: false, relay: false, hosting: false },
        status: "PASS",
        gradeText: "合格 (纯净静态住宅 IP)",
        reasons: [],
        advice: "测试建议",
        links: {
          ipinfo: "https://ipinfo.io/1.2.3.4",
          ping0: "https://ping0.cc/ip/1.2.3.4"
        }
      },
      paint
    });
  });
});

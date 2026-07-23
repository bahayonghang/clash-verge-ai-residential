"use strict";

const path = require("node:path");
const assert = require("node:assert/strict");

const scriptPath = process.env.CLASH_SCRIPT_PATH ||
  path.join(__dirname, "..", "clash-verge-ai-residential.js");

const script = require(scriptPath);
const {
  main,
  buildInjectedRules,
  buildNameserverPolicy,
  buildUpstreamDoh,
  cleanAndMigrateExistingRules,
  constants
} = script;

const {
  SCRIPT_VERSION,
  AI_GROUP,
  HOME_PROXY_NAME,
  HOME_PROXY_TEMPLATE: template,
  PROFILE_UPSTREAM_OVERRIDES: overrides,
  RESIDENTIAL_DOH,
  NON_AI_DOH_ENDPOINTS,
  PRESERVE_UNMANAGED_NAMESERVER_POLICY,
  ROUTE_GEMINI_WEB_CORE,
  ROUTE_CURSOR_CORE,
  ROUTE_CURSOR_PROCESS_FALLBACK,
  GEMINI_WEB_SUFFIX_DOMAINS,
  GEMINI_WEB_EXACT_DOMAINS,
  GEMINI_DOMAIN_REGEXES,
  CURSOR_SUFFIX_DOMAINS,
  CURSOR_EXACT_DOMAINS,
  CURSOR_DOMAIN_REGEXES,
  LEGACY_V53_SUFFIX_DOMAINS,
  LEGACY_V53_NON_AI_EXACT_DOMAINS
} = constants;

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

function quietMain(config, profileName) {
  const originalInfo = console.info;
  const originalWarn = console.warn;
  console.info = () => {};
  console.warn = () => {};
  try {
    return main(config, profileName);
  } finally {
    console.info = originalInfo;
    console.warn = originalWarn;
  }
}

function airportNode(name, extra = {}) {
  return {
    name,
    type: "ss",
    server: `${String(name).replace(/[^a-z0-9]+/gi, "-").toLowerCase()}.example.test`,
    port: 443,
    cipher: "aes-128-gcm",
    password: "airport-secret",
    udp: true,
    ...extra
  };
}

function homeNode(extra = {}) {
  return {
    name: HOME_PROXY_NAME,
    type: "socks5",
    server: "home.example.test",
    port: 1080,
    username: "home-user",
    password: "home-pass",
    udp: true,
    ...extra
  };
}

function group(name, proxies = [], extra = {}) {
  return {
    name,
    type: "select",
    proxies,
    ...extra
  };
}

function configFixture({
  proxies = [],
  groups = [],
  rules = [],
  dns = {},
  tun,
  findProcessMode,
  includeHome = true
} = {}) {
  const config = {
    proxies: [...(includeHome ? [homeNode()] : []), ...proxies],
    "proxy-groups": groups,
    rules,
    dns
  };
  if (tun !== undefined) config.tun = tun;
  if (findProcessMode !== undefined) config["find-process-mode"] = findProcessMode;
  return config;
}

function findProxy(config, name) {
  return config.proxies.find((entry) => entry && entry.name === name);
}

function findGroup(config, name) {
  return config["proxy-groups"].find((entry) => entry && entry.name === name);
}

function countNamed(items, name) {
  return items.filter((entry) => entry && entry.name === name).length;
}

function parseRule(rule) {
  if (typeof rule !== "string") return null;
  const parts = rule.split(",");
  if (parts.length < 3) return null;
  return {
    type: parts[0],
    value: parts[1],
    target: parts[parts.length - 1]
  };
}

function ruleMatchesHost(rules, host, target = AI_GROUP) {
  return rules.some((rule) => {
    const parsed = parseRule(rule);
    if (!parsed || parsed.target !== target) return false;
    if (parsed.type === "DOMAIN") return host === parsed.value;
    if (parsed.type === "DOMAIN-SUFFIX") {
      return host === parsed.value || host.endsWith(`.${parsed.value}`);
    }
    if (parsed.type === "DOMAIN-REGEX") {
      return new RegExp(parsed.value).test(host);
    }
    return false;
  });
}

function assertNoAiRoute(rules, hosts) {
  for (const host of hosts) {
    assert.equal(ruleMatchesHost(rules, host), false, `不应由 ${AI_GROUP} 匹配：${host}`);
  }
}

function assertAiRoute(rules, hosts) {
  for (const host of hosts) {
    assert.equal(ruleMatchesHost(rules, host), true, `应由 ${AI_GROUP} 匹配：${host}`);
  }
}

// ---------------------------------------------------------------------------
// 基础配置与多 Profile
// ---------------------------------------------------------------------------

test("脚本版本与默认 dialer-proxy 正确", () => {
  assert.equal(SCRIPT_VERSION, "5.4.0");
  assert.equal(template["dialer-proxy"], "🚀节点选择");
});

test("默认选择 🚀节点选择，并复用 Profile 内已有家宽节点凭据", () => {
  const config = configFixture({
    proxies: [airportNode("HK"), airportNode("JP")],
    groups: [group("🚀节点选择", ["HK"]), group("Proxy", ["JP"])]
  });
  const output = quietMain(config, "赔钱机场");
  const home = findProxy(output, HOME_PROXY_NAME);
  assert.equal(home["dialer-proxy"], "🚀节点选择");
  assert.equal(home.server, "home.example.test");
  assert.equal(home.port, 1080);
  assert.equal(home.username, "home-user");
  assert.equal(home.password, "home-pass");
  assert.equal(typeof home["dialer-proxy"], "string");
});

test("奈云 Profile 在没有 🚀节点选择 时选择 Proxy", () => {
  const config = configFixture({
    proxies: [airportNode("JP 09"), airportNode("HK 04")],
    groups: [
      group("Proxy", ["JP 09", "自动选择"]),
      group("自动选择", ["HK 04"], { type: "url-test" })
    ]
  });
  const output = quietMain(config, "奈云");
  assert.equal(findProxy(output, HOME_PROXY_NAME)["dialer-proxy"], "Proxy");
});

test("候选均未命中时可从最终 MATCH 规则解析上游", () => {
  const config = configFixture({
    proxies: [airportNode("US")],
    groups: [group("主路由", ["US"])],
    rules: ["DOMAIN-SUFFIX,example.com,DIRECT", "MATCH,主路由"]
  });
  const output = quietMain(config, "自定义订阅");
  assert.equal(findProxy(output, HOME_PROXY_NAME)["dialer-proxy"], "主路由");
});

test("Profile 覆盖失效时继续尝试通用候选", () => {
  const hadOriginal = Object.prototype.hasOwnProperty.call(overrides, "极速云");
  const original = overrides["极速云"];
  overrides["极速云"] = ["不存在的组"];
  try {
    const config = configFixture({
      proxies: [airportNode("JP")],
      groups: [group("Proxy", ["JP"])]
    });
    const output = quietMain(config, "极速云");
    assert.equal(findProxy(output, HOME_PROXY_NAME)["dialer-proxy"], "Proxy");
  } finally {
    if (hadOriginal) overrides["极速云"] = original;
    else delete overrides["极速云"];
  }
});

test("候选名仅 emoji/空格不同仍可归一化匹配", () => {
  const config = configFixture({
    proxies: [airportNode("JP")],
    groups: [group("🛰️　节点选择", ["JP"])]
  });
  const output = quietMain(config, "未知订阅");
  assert.equal(findProxy(output, HOME_PROXY_NAME)["dialer-proxy"], "🛰️　节点选择");
});

test("没有安全上游时 fail closed，不回落 DIRECT", () => {
  const config = configFixture({
    proxies: [airportNode("US")],
    groups: [group("未知出口", ["US"])],
    rules: ["MATCH,DIRECT"]
  });
  assert.throws(() => quietMain(config, "未知订阅"), /找不到可用 dialer-proxy/);
});

// ---------------------------------------------------------------------------
// 递归、UDP 与保留名称
// ---------------------------------------------------------------------------

test("include-all 上游排除家宽节点，避免动态递归", () => {
  const config = configFixture({
    proxies: [airportNode("HK")],
    groups: [
      group("Proxy", [], {
        "include-all-proxies": true,
        "exclude-filter": "^过期节点$"
      })
    ]
  });
  const output = quietMain(config, "奈云");
  const upstream = findGroup(output, "Proxy");
  assert.match(upstream["exclude-filter"], /\^家宽-SOCKS5\$/);
  assert.match(upstream["exclude-filter"], /\^过期节点\$/);
});

test("拒绝选定上游可达的代理组循环依赖", () => {
  const config = configFixture({
    groups: [
      group("Proxy", ["自动选择"]),
      group("自动选择", ["Proxy"], { type: "url-test" })
    ]
  });
  assert.throws(
    () => quietMain(config, "奈云"),
    /循环依赖：Proxy -> 自动选择 -> Proxy/
  );
});

test("上游组清理脚本对象后为空时拒绝生成链路", () => {
  const config = configFixture({
    groups: [group("Proxy", [HOME_PROXY_NAME, AI_GROUP])]
  });
  assert.throws(() => quietMain(config, "奈云"), /没有可用节点来源/);
});

test("显式禁用 UDP 的上游被拒绝", () => {
  const config = configFixture({
    proxies: [airportNode("HK")],
    groups: [group("🚀节点选择", ["HK"], { "disable-udp": true })]
  });
  assert.throws(() => quietMain(config, "赔钱机场"), /显式禁用了 UDP/);
});

test("跨代理/代理组命名空间冲突被拒绝", () => {
  const config = configFixture({
    proxies: [airportNode("HK"), airportNode(AI_GROUP)],
    groups: [group("🚀节点选择", ["HK"])]
  });
  assert.throws(() => quietMain(config, "赔钱机场"), /已被代理节点占用/);
});

test("重复保留代理组被拒绝，不静默覆盖未知配置", () => {
  const config = configFixture({
    proxies: [airportNode("HK")],
    groups: [
      group(AI_GROUP, [HOME_PROXY_NAME]),
      group(AI_GROUP, [HOME_PROXY_NAME]),
      group("🚀节点选择", ["HK"])
    ]
  });
  assert.throws(() => quietMain(config, "赔钱机场"), /多个同名代理组/);
});

test("只配置 endpoint 而保留 xxx 凭据时明确报错", () => {
  const original = {
    server: template.server,
    port: template.port,
    username: template.username,
    password: template.password
  };
  template.server = "configured.example.test";
  template.port = 1080;
  template.username = "xxx";
  template.password = "xxx";

  try {
    const config = configFixture({
      includeHome: false,
      proxies: [airportNode("JP")],
      groups: [group("🚀节点选择", ["JP"])]
    });
    assert.throws(
      () => quietMain(config, "赔钱机场"),
      /username\/password 仍是占位值 xxx/
    );
  } finally {
    template.server = original.server;
    template.port = original.port;
    template.username = original.username;
    template.password = original.password;
  }
});

// ---------------------------------------------------------------------------
// AI-only 域名边界
// ---------------------------------------------------------------------------

test("Gemini 核心产品、Developer API 与 Vertex AI 端点走家宽", () => {
  assert.equal(ROUTE_GEMINI_WEB_CORE, true);
  assert.deepEqual(GEMINI_WEB_SUFFIX_DOMAINS, ["gemini.google.com", "aistudio.google.com"]);
  assert.ok(GEMINI_WEB_EXACT_DOMAINS.includes("alkalicore-pa.clients6.google.com"));
  assert.ok(GEMINI_WEB_EXACT_DOMAINS.includes("alkalimakersuite-pa.clients6.google.com"));
  assert.ok(GEMINI_WEB_EXACT_DOMAINS.includes("webchannel-alkalimakersuite-pa.clients6.google.com"));
  assert.ok(GEMINI_DOMAIN_REGEXES.includes("^[a-z0-9-]+-aiplatform\\.googleapis\\.com$"));

  const rules = buildInjectedRules();
  assertAiRoute(rules, [
    "gemini.google.com",
    "aistudio.google.com",
    "alkalicore-pa.clients6.google.com",
    "alkalimakersuite-pa.clients6.google.com",
    "webchannel-alkalimakersuite-pa.clients6.google.com",
    "generativelanguage.googleapis.com",
    "aiplatform.googleapis.com",
    "us-central1-aiplatform.googleapis.com",
    "aiplatform.us.rep.googleapis.com",
    "cloudaicompanion.googleapis.com",
    "cloudcode-pa.googleapis.com"
  ]);
});

test("Gemini 的 YouTube、Maps、广告、统计与通用 Google 资源不走家宽", () => {
  const rules = buildInjectedRules();
  assertNoAiRoute(rules, [
    "www.youtube.com",
    "i.ytimg.com",
    "yt3.ggpht.com",
    "maps.googleapis.com",
    "maps.gstatic.com",
    "www.google.com",
    "www.googleapis.com",
    "ssl.gstatic.com",
    "fonts.googleapis.com",
    "www.googletagmanager.com",
    "www.google-analytics.com",
    "static.doubleclick.net",
    "googleads.g.doubleclick.net"
  ]);
});

test("Cursor 仅匹配 AI API、Tab、Agent、索引、Cloud Agent 与专属认证", () => {
  assert.equal(ROUTE_CURSOR_CORE, true);
  assert.deepEqual(
    CURSOR_SUFFIX_DOMAINS,
    ["api2.cursor.sh", "api5.cursor.sh", "gcpp.cursor.sh", "authentication.cursor.sh"]
  );
  assert.ok(CURSOR_EXACT_DOMAINS.includes("api.cursor.com"));
  assert.ok(CURSOR_DOMAIN_REGEXES.includes("^repo[0-9]+\\.cursor\\.sh$"));

  const rules = buildInjectedRules();
  assertAiRoute(rules, [
    "api2.cursor.sh",
    "feature.api2.cursor.sh",
    "api3.cursor.sh",
    "api4.cursor.sh",
    "agent.api5.cursor.sh",
    "agentn.global.api5.cursor.sh",
    "repo42.cursor.sh",
    "repo99.cursor.sh",
    "prod.authentication.cursor.sh",
    "authenticator.cursor.sh",
    "us-eu.gcpp.cursor.sh",
    "api.cursor.com"
  ]);
});

test("Cursor 插件市场、下载、CDN、更新与 Remote-SSH 资产不走家宽", () => {
  const rules = buildInjectedRules();
  assertNoAiRoute(rules, [
    "marketplace.cursorapi.com",
    "downloads.cursor.com",
    "cursor-cdn.com",
    "anysphere-binaries.s3.us-east-1.amazonaws.com",
    "cursor.blob.core.windows.net",
    "www.cursor.com",
    "docs.cursor.com",
    "forum.cursor.com"
  ]);

  for (const suffix of ["cursor.sh", "cursor.com", "cursorapi.com", "cursor-cdn.com"]) {
    assert.equal(
      rules.includes(`DOMAIN-SUFFIX,${suffix},${AI_GROUP}`),
      false,
      `不应存在 Cursor 宽泛后缀：${suffix}`
    );
  }
});

test("Cursor 进程级兜底默认关闭，避免插件、GitHub、npm、MCP 与用户后端被全量代理", () => {
  assert.equal(ROUTE_CURSOR_PROCESS_FALLBACK, false);
  const processRules = buildInjectedRules().filter((rule) => rule.startsWith("PROCESS-"));
  assert.deepEqual(processRules, []);
});

test("公共 DoH/DoT 与通用 STUN/TURN/Voice 端口默认不走家宽", () => {
  const rules = buildInjectedRules();
  assert.equal(rules.some((rule) => rule.includes("DST-PORT,853")), false);
  assert.equal(rules.some((rule) => rule.includes("DST-PORT,3478")), false);
  assert.equal(rules.some((rule) => rule.includes("DST-PORT,19302")), false);
  assert.equal(rules.some((rule) => /stun|turn|livekit|xirsys|metered/i.test(rule)), false);
  assertNoAiRoute(rules, ["dns.google", "cloudflare-dns.com", "stun.l.google.com"]);
});

test("Claude、ChatGPT、Antigravity 核心域名仍走家宽，共享第三方依赖不走", () => {
  const rules = buildInjectedRules();
  assertAiRoute(rules, [
    "claude.ai",
    "api.anthropic.com",
    "chatgpt.com",
    "api.openai.com",
    "antigravity.google",
    "daily-cloudcode-pa.googleapis.com"
  ]);
  assertNoAiRoute(rules, [
    "oaistatic.com",
    "intercom.io",
    "sentry.io",
    "statsigapi.net",
    "js.stripe.com",
    "accounts.google.com",
    "serviceusage.googleapis.com",
    "update.googleapis.com",
    "open-vsx.org"
  ]);
});

// ---------------------------------------------------------------------------
// 迁移与幂等
// ---------------------------------------------------------------------------

test("v5.3 宽泛 Cursor/Gemini 与非 AI 规则会被精确清理", () => {
  assert.ok(LEGACY_V53_SUFFIX_DOMAINS.includes("cursor.com"));
  assert.ok(LEGACY_V53_NON_AI_EXACT_DOMAINS.includes("www.youtube.com"));
  assert.ok(LEGACY_V53_NON_AI_EXACT_DOMAINS.includes("marketplace.cursorapi.com"));

  const oldRules = [
    `DOMAIN-SUFFIX,oaistatic.com,${AI_GROUP}`,
    `DOMAIN-SUFFIX,cursor.sh,${AI_GROUP}`,
    `DOMAIN-SUFFIX,cursor.com,${AI_GROUP}`,
    `DOMAIN-SUFFIX,cursorapi.com,${AI_GROUP}`,
    `DOMAIN-SUFFIX,cursor-cdn.com,${AI_GROUP}`,
    `DOMAIN-SUFFIX,gemini.google,${AI_GROUP}`,
    `DOMAIN,www.youtube.com,${AI_GROUP}`,
    `DOMAIN,maps.googleapis.com,${AI_GROUP}`,
    `DOMAIN,marketplace.cursorapi.com,${AI_GROUP}`,
    `DOMAIN,downloads.cursor.com,${AI_GROUP}`,
    `DOMAIN,anysphere-binaries.s3.us-east-1.amazonaws.com,${AI_GROUP}`,
    `DOMAIN,cursor.blob.core.windows.net,${AI_GROUP}`,
    "MATCH,🚀节点选择"
  ];
  const cleaned = cleanAndMigrateExistingRules(oldRules);
  assert.deepEqual(cleaned, ["MATCH,🚀节点选择"]);
});

test("脚本执行两次保持幂等，并保留用户自定义非托管规则", () => {
  const customAiRule = `DOMAIN,custom-ai.example,${AI_GROUP}`;
  const normalYoutubeRule = "DOMAIN-SUFFIX,youtube.com,Proxy";
  const normalMarketplaceRule = "DOMAIN,marketplace.cursorapi.com,Proxy";
  const config = configFixture({
    proxies: [airportNode("HK")],
    groups: [group("🚀节点选择", ["HK"]), group("Proxy", ["HK"])],
    rules: [
      customAiRule,
      customAiRule,
      normalYoutubeRule,
      normalMarketplaceRule,
      `DOMAIN,www.youtube.com,${AI_GROUP}`,
      `DOMAIN,marketplace.cursorapi.com,${AI_GROUP}`,
      `DOMAIN-SUFFIX,cursor.com,${AI_GROUP}`,
      `IP-CIDR,160.79.104.0/21,${AI_GROUP},no-resolve`,
      `IP-CIDR6,2607:6bc0::/32,${AI_GROUP},no-resolve`,
      "MATCH,Proxy"
    ]
  });

  quietMain(config, "赔钱机场");
  quietMain(config, "赔钱机场");

  assert.equal(countNamed(config.proxies, HOME_PROXY_NAME), 1);
  assert.equal(countNamed(config["proxy-groups"], AI_GROUP), 1);
  assert.equal(config.rules.filter((rule) => rule === customAiRule).length, 1);
  assert.equal(config.rules.includes(normalYoutubeRule), true);
  assert.equal(config.rules.includes(normalMarketplaceRule), true);
  assert.equal(config.rules.includes(`DOMAIN,www.youtube.com,${AI_GROUP}`), false);
  assert.equal(config.rules.includes(`DOMAIN,marketplace.cursorapi.com,${AI_GROUP}`), false);
  assert.equal(config.rules.includes(`DOMAIN-SUFFIX,cursor.com,${AI_GROUP}`), false);
  assert.equal(config.rules.includes(`IP-CIDR,160.79.104.0/21,${AI_GROUP},no-resolve`), false);
  assert.equal(config.rules.includes(`IP-CIDR6,2607:6bc0::/32,${AI_GROUP},no-resolve`), false);
  assert.equal(config.rules.includes(`IP-CIDR,160.79.104.0/23,${AI_GROUP},no-resolve`), true);
  assert.equal(config.rules.includes(`IP-CIDR6,2607:6bc0::/48,${AI_GROUP},no-resolve`), true);
  assert.equal(new Set(config.rules).size, config.rules.length);
});

// ---------------------------------------------------------------------------
// DNS：AI 查询走家宽，其他查询走机场上游
// ---------------------------------------------------------------------------

test("非 AI 默认 DoH 绑定当前 Profile 上游，而不是 AI-家宽", () => {
  const doh = buildUpstreamDoh("🚀节点选择");
  assert.equal(doh.length, NON_AI_DOH_ENDPOINTS.length);
  assert.ok(doh.every((value) => value.includes("#🚀节点选择&disable-ipv6=true")));
  assert.ok(doh.every((value) => !value.includes(`#${AI_GROUP}`)));
});

test("AI DNS policy 仅覆盖 AI 核心域名，排除 YouTube 与插件市场", () => {
  const policy = buildNameserverPolicy({});
  assert.deepEqual(policy["+.gemini.google.com"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["+.aistudio.google.com"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["generativelanguage.googleapis.com"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["+.api2.cursor.sh"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["authenticator.cursor.sh"], RESIDENTIAL_DOH);

  for (const key of [
    "www.youtube.com",
    "i.ytimg.com",
    "maps.googleapis.com",
    "marketplace.cursorapi.com",
    "downloads.cursor.com",
    "+.cursor.com",
    "+.cursorapi.com",
    "+.cursor-cdn.com"
  ]) {
    assert.equal(key in policy, false, `DNS policy 不应包含：${key}`);
  }
});

test("严格 DNS 模式移除独立旁路，并保持私有域名使用系统 DNS", () => {
  assert.equal(PRESERVE_UNMANAGED_NAMESERVER_POLICY, false);
  const config = configFixture({
    proxies: [airportNode("HK")],
    groups: [group("🚀节点选择", ["HK"])],
    dns: {
      "fake-ip-filter-mode": "rule",
      "fake-ip-filter": ["MATCH,fake-ip"],
      fallback: ["tls://9.9.9.9"],
      "fallback-filter": { geoip: true },
      "proxy-server-nameserver-policy": {
        "node.example.test": ["https://1.1.1.1/dns-query"]
      },
      "nameserver-policy": {
        "geosite:gfw": ["https://doh.pub/dns-query"],
        "+.user-owned.example": ["https://9.9.9.9/dns-query"],
        "www.youtube.com": RESIDENTIAL_DOH,
        "marketplace.cursorapi.com": RESIDENTIAL_DOH
      }
    }
  });
  const output = quietMain(config, "赔钱机场");
  const dns = output.dns;

  assert.equal("fallback" in dns, false);
  assert.equal("fallback-filter" in dns, false);
  assert.equal("proxy-server-nameserver-policy" in dns, false);
  assert.equal(dns["fake-ip-filter-mode"], "blacklist");
  assert.equal(dns["fake-ip-filter"].includes("MATCH,fake-ip"), false);
  assert.equal("geosite:gfw" in dns["nameserver-policy"], false);
  assert.equal("+.user-owned.example" in dns["nameserver-policy"], false);
  assert.equal("www.youtube.com" in dns["nameserver-policy"], false);
  assert.equal("marketplace.cursorapi.com" in dns["nameserver-policy"], false);
  assert.deepEqual(dns["nameserver-policy"]["geosite:private"], ["system"]);
  assert.deepEqual(dns["nameserver-policy"]["+.local"], ["system"]);
  assert.ok(dns.nameserver.every((value) => value.includes("#🚀节点选择&disable-ipv6=true")));
  assert.ok(dns.nameserver.every((value) => !value.includes(`#${AI_GROUP}`)));
  assert.equal(dns["respect-rules"], true);
  assert.equal(dns["prefer-h3"], false);
});

test("已开启 TUN 时只补齐 DNS 劫持；AI-only 模式不强制进程匹配", () => {
  const config = configFixture({
    proxies: [airportNode("HK")],
    groups: [group("🚀节点选择", ["HK"])],
    tun: { enable: true, "dns-hijack": ["udp://any:53"] },
    findProcessMode: "off"
  });
  const output = quietMain(config, "赔钱机场");
  assert.equal(output.tun["dns-hijack"].includes("any:53"), true);
  assert.equal(output.tun["dns-hijack"].includes("tcp://any:53"), true);
  assert.equal(output["find-process-mode"], "off");
});

test("最终注入规则不存在重复项", () => {
  const rules = buildInjectedRules();
  assert.equal(new Set(rules).size, rules.length);
});

console.log(`All ${passed} v5.4 regression tests passed.`);

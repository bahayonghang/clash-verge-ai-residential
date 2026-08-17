"use strict";

const path = require("node:path");
const assert = require("node:assert/strict");
const { test } = require("node:test");

const scriptPath = process.env.CLASH_SCRIPT_PATH ||
  path.join(__dirname, "..", "clash-verge-ai-residential.js");

const script = require(scriptPath);
const {
  main,
  buildInjectedRules,
  buildNameserverPolicy,
  buildOutboundIndex,
  buildUpstreamDoh,
  cleanExistingManagedRules,
  findOutbound,
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
  ROUTE_GROK_CORE,
  ROUTE_CURSOR_PROCESS_FALLBACK,
  GEMINI_WEB_SUFFIX_DOMAINS,
  GEMINI_WEB_EXACT_DOMAINS,
  GEMINI_DOMAIN_REGEXES,
  CURSOR_SUFFIX_DOMAINS,
  CURSOR_EXACT_DOMAINS,
  CURSOR_DOMAIN_REGEXES,
  GROK_SUFFIX_DOMAINS,
  GROK_EXACT_DOMAINS,
  OPENAI_CORE_EXACT_DOMAINS
} = constants;

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

function captureMain(config, profileName) {
  const warnings = [];
  const originalInfo = console.info;
  const originalWarn = console.warn;
  console.info = () => {};
  console.warn = (message) => warnings.push(String(message));
  try {
    return { output: main(config, profileName), warnings };
  } finally {
    console.info = originalInfo;
    console.warn = originalWarn;
  }
}

function extractQuotedNames(message) {
  return Array.from(String(message).matchAll(/“([^”]+)”/g), (match) => match[1]);
}

function udpDisabledWarnings(warnings) {
  return warnings.filter((line) => line.includes("显式关闭 UDP"));
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
  assert.equal(SCRIPT_VERSION, "5.8.1");
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

test("移除上游组中的保留名引用时输出 warn 说明原因", () => {
  const warnings = [];
  const originalWarn = console.warn;
  const originalInfo = console.info;
  console.warn = (message) => warnings.push(String(message));
  console.info = () => {};
  try {
    const config = configFixture({
      proxies: [airportNode("HK")],
      groups: [group("🚀节点选择", [HOME_PROXY_NAME, AI_GROUP, "HK"])]
    });
    const output = main(config, "赔钱机场");
    assert.deepEqual(
      findGroup(output, "🚀节点选择").proxies,
      ["HK"]
    );
  } finally {
    console.warn = originalWarn;
    console.info = originalInfo;
  }

  const removalWarning = warnings.find((line) => line.includes("引用已被移除"));
  assert.ok(removalWarning, `应输出引用移除 warn，实际：${warnings.join(" | ")}`);
  assert.match(removalWarning, /🚀节点选择/);
  assert.match(removalWarning, new RegExp(HOME_PROXY_NAME));
  assert.match(removalWarning, /递归/);
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

test("findOutbound 缺索引时抛错，合法索引可解析唯一节点", () => {
  assert.throws(() => findOutbound(), /需要 outbound 索引/);
  assert.throws(() => findOutbound({}, "x"), /需要 outbound 索引/);
  assert.throws(() => findOutbound({ groups: new Map() }, "x"), /需要 outbound 索引/);

  const index = buildOutboundIndex(configFixture({
    proxies: [airportNode("HK")],
    groups: [group("🚀节点选择", ["HK"])]
  }));
  assert.equal(findOutbound(index, "HK").kind, "proxy");
  assert.equal(findOutbound(index, "🚀节点选择").kind, "group");
});

test("普通名两个同名节点被拒绝", () => {
  const config = configFixture({
    proxies: [airportNode("HK"), airportNode("HK")],
    groups: [group("🚀节点选择", ["HK"])]
  });
  assert.throws(() => quietMain(config, "赔钱机场"), /歧义/);
});

test("普通名两个同名组被拒绝", () => {
  const config = configFixture({
    proxies: [airportNode("HK")],
    groups: [
      group("🚀节点选择", ["HK"]),
      group("🚀节点选择", ["HK"])
    ]
  });
  assert.throws(() => quietMain(config, "赔钱机场"), /歧义/);
});

test("普通名同时被组与节点占用时被拒绝", () => {
  const config = configFixture({
    proxies: [airportNode("HK")],
    groups: [group("HK", ["HK"])],
    rules: ["MATCH,HK"]
  });
  assert.throws(() => quietMain(config, "未知订阅"), /歧义/);
});

test("归一化命中唯一字符串后组与节点同名仍拒绝", () => {
  const config = configFixture({
    proxies: [airportNode("🚀 节点选择")],
    groups: [group("🚀 节点选择", ["🚀 节点选择"])]
  });
  assert.throws(() => quietMain(config, "未知订阅"), /歧义/);
});

test("单叶子 udp:false 汇总含名称与路径", () => {
  const config = configFixture({
    proxies: [airportNode("HK", { udp: false }), airportNode("JP")],
    groups: [group("🚀节点选择", ["HK", "JP"])]
  });
  const { warnings } = captureMain(config, "赔钱机场");
  const udpWarnings = udpDisabledWarnings(warnings);
  assert.equal(udpWarnings.length, 1);
  assert.match(udpWarnings[0], /HK/);
  assert.match(udpWarnings[0], /路径：🚀节点选择 -> HK/);
});

test("同名 udp:false 节点挂两个组时只计一次且保留首次路径", () => {
  const config = configFixture({
    proxies: [airportNode("HK", { udp: false })],
    groups: [
      group("🚀节点选择", ["A", "B"]),
      group("A", ["HK"]),
      group("B", ["HK"])
    ]
  });
  const { warnings } = captureMain(config, "赔钱机场");
  const udpWarnings = udpDisabledWarnings(warnings);
  assert.equal(udpWarnings.length, 1);
  assert.match(udpWarnings[0], /1 个可达节点显式关闭 UDP/);
  assert.match(udpWarnings[0], /路径：🚀节点选择 -> A -> HK/);
  assert.equal(udpWarnings[0].includes("路径：🚀节点选择 -> B -> HK"), false);
});

test("9 个不同名 udp:false 叶子只展示前 8 个样本", () => {
  const leafNames = [];
  const proxies = [];
  for (let index = 1; index <= 9; index += 1) {
    const name = `L${index}`;
    leafNames.push(name);
    proxies.push(airportNode(name, { udp: false }));
  }
  const config = configFixture({
    proxies,
    groups: [group("🚀节点选择", leafNames)]
  });
  const { warnings } = captureMain(config, "赔钱机场");
  const udpWarnings = udpDisabledWarnings(warnings);
  assert.equal(udpWarnings.length, 1);
  const sampleNames = extractQuotedNames(udpWarnings[0]);
  assert.deepEqual(sampleNames, leafNames.slice(0, 8));
  assert.equal(udpWarnings[0].includes("L9"), false);
  assert.match(udpWarnings[0], /9/);
});

test("2000 叶子中 1000 个不同名 udp:false 只汇总一条警告", () => {
  const leafNames = [];
  const proxies = [];
  for (let index = 0; index < 2000; index += 1) {
    const name = `N${String(index).padStart(4, "0")}`;
    leafNames.push(name);
    proxies.push(airportNode(name, { udp: index < 1000 ? false : true }));
  }
  const config = configFixture({
    proxies,
    groups: [group("🚀节点选择", leafNames)]
  });
  const { output, warnings } = captureMain(config, "赔钱机场");
  assert.ok(output);
  const udpWarnings = udpDisabledWarnings(warnings);
  assert.equal(udpWarnings.length, 1);
  const sampleNames = extractQuotedNames(udpWarnings[0]);
  assert.ok(sampleNames.length <= 8);
  assert.equal(udpWarnings[0].includes("N0008"), false);
  assert.match(udpWarnings[0], /1000/);
});

test("2000 叶子同一对象连续两次 main 保持规则、policy 与 dialer-proxy", () => {
  const leafNames = [];
  const proxies = [];
  for (let index = 0; index < 2000; index += 1) {
    const name = `N${String(index).padStart(4, "0")}`;
    leafNames.push(name);
    proxies.push(airportNode(name, { udp: index < 1000 ? false : true }));
  }
  const config = configFixture({
    proxies,
    groups: [group("🚀节点选择", leafNames)]
  });

  quietMain(config, "赔钱机场");
  const firstRules = config.rules.slice();
  const firstPolicy = structuredClone(config.dns["nameserver-policy"]);
  const firstHome = findProxy(config, HOME_PROXY_NAME);
  const firstServer = firstHome.server;
  const firstPort = firstHome.port;
  const firstDialer = firstHome["dialer-proxy"];
  quietMain(config, "赔钱机场");

  const secondHome = findProxy(config, HOME_PROXY_NAME);
  assert.deepEqual(config.rules, firstRules);
  assert.deepEqual(config.dns["nameserver-policy"], firstPolicy);
  assert.equal(secondHome.server, firstServer);
  assert.equal(secondHome.port, firstPort);
  assert.equal(secondHome["dialer-proxy"], firstDialer);
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

test("Cursor 核心路由默认开启，并保持窄范围目录", () => {
  assert.equal(ROUTE_CURSOR_CORE, true);
  assert.deepEqual(
    CURSOR_SUFFIX_DOMAINS,
    [
      "api2.cursor.sh",
      "api5.cursor.sh",
      "gcpp.cursor.sh",
      "authenticate.cursor.sh",
      "authentication.cursor.sh",
      "cursorvm.com"
    ]
  );
  assert.deepEqual(
    CURSOR_EXACT_DOMAINS,
    ["api3.cursor.sh", "api4.cursor.sh", "authenticator.cursor.sh", "api.cursor.com"]
  );
  assert.deepEqual(
    CURSOR_DOMAIN_REGEXES,
    ["^repo[0-9]+\\.cursor\\.sh$", "^adminportal[0-9]+\\.cursor\\.sh$"]
  );
  assert.equal(
    new RegExp(CURSOR_DOMAIN_REGEXES[0]).test("repo42.cursor.sh"),
    true
  );
  assert.equal(
    new RegExp(CURSOR_DOMAIN_REGEXES[0]).test("repo99.cursor.sh"),
    true
  );
  assert.equal(
    new RegExp(CURSOR_DOMAIN_REGEXES[1]).test("adminportal42.cursor.sh"),
    true
  );
  assert.equal(
    new RegExp(CURSOR_DOMAIN_REGEXES[1]).test("adminportal.cursor.sh"),
    false
  );

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
    "authenticate.cursor.sh",
    "prod.authentication.cursor.sh",
    "authenticator.cursor.sh",
    "adminportal42.cursor.sh",
    "us-eu.gcpp.cursor.sh",
    "vm.cursorvm.com",
    "us-east.vm.cursorvm.com",
    "api.cursor.com"
  ]);
});

test("Grok Build 核心域默认走家宽，共享第三方与安装域名不走", () => {
  assert.equal(ROUTE_GROK_CORE, true);
  assert.deepEqual(GROK_SUFFIX_DOMAINS, ["grok.com"]);
  assert.deepEqual(GROK_EXACT_DOMAINS, ["auth.x.ai", "api.x.ai"]);

  const rules = buildInjectedRules();
  assertAiRoute(rules, [
    "grok.com",
    "cli-chat-proxy.grok.com",
    "auth.x.ai",
    "api.x.ai"
  ]);
  assertNoAiRoute(rules, [
    "api.mixpanel.com",
    "x.ai",
    "www.x.ai",
    "storage.googleapis.com"
  ]);

  const policy = buildNameserverPolicy({});
  assert.deepEqual(policy["+.grok.com"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["auth.x.ai"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["api.x.ai"], RESIDENTIAL_DOH);
  assert.equal("api.mixpanel.com" in policy, false);
  assert.equal("x.ai" in policy, false);
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
  assert.deepEqual(OPENAI_CORE_EXACT_DOMAINS, [
    "chat.openai.com",
    "android.chat.openai.com",
    "desktop.chat.openai.com",
    "ios.chat.openai.com",
    "tcr9i.chat.openai.com"
  ]);

  const rules = buildInjectedRules();
  assertAiRoute(rules, [
    "claude.ai",
    "api.anthropic.com",
    "a-api.anthropic.com",
    "mcp-proxy.anthropic.com",
    "assets-proxy.anthropic.com",
    "chatgpt.com",
    "api.openai.com",
    "us.api.openai.com",
    "eu.api.openai.com",
    "chat.openai.com",
    "android.chat.openai.com",
    "desktop.chat.openai.com",
    "ios.chat.openai.com",
    "tcr9i.chat.openai.com",
    "antigravity.google",
    "daily-cloudcode-pa.googleapis.com"
  ]);
  assertNoAiRoute(rules, [
    "oaistatic.com",
    "oaistatsig.com",
    "intercom.io",
    "sentry.io",
    "statsigapi.net",
    "js.stripe.com",
    "auth.openai.com",
    "www.anthropic.com",
    "www.openai.com",
    "docs.anthropic.com",
    "support.anthropic.com",
    "status.anthropic.com",
    "telemetry.anthropic.com",
    "accounts.google.com",
    "serviceusage.googleapis.com",
    "update.googleapis.com",
    "open-vsx.org"
  ]);
  assert.equal(rules.includes(`DOMAIN-SUFFIX,chat.openai.com,${AI_GROUP}`), false);
  assert.equal(rules.includes(`DOMAIN-SUFFIX,openai.com,${AI_GROUP}`), false);
});

// ---------------------------------------------------------------------------
// 当前托管规则与幂等
// ---------------------------------------------------------------------------

test("开关关闭后清理当前托管规则，并保留退役或用户自写规则", () => {
  const currentManagedRules = [
    `DOMAIN,a-api.anthropic.com,${AI_GROUP}`,
    `DOMAIN,mcp-proxy.anthropic.com,${AI_GROUP}`,
    `DOMAIN,assets-proxy.anthropic.com,${AI_GROUP}`,
    `DOMAIN-SUFFIX,api.openai.com,${AI_GROUP}`,
    `DOMAIN,api.openai.com,${AI_GROUP}`,
    `DOMAIN,chat.openai.com,${AI_GROUP}`,
    `DOMAIN-SUFFIX,chat.openai.com,${AI_GROUP}`,
    `DOMAIN,auth.x.ai,${AI_GROUP}`,
    `DOMAIN,api.x.ai,${AI_GROUP}`,
    `DOMAIN-SUFFIX,api2.cursor.sh,${AI_GROUP}`,
    `DOMAIN-SUFFIX,authenticate.cursor.sh,${AI_GROUP}`,
    `DOMAIN-REGEX,^adminportal[0-9]+\\.cursor\\.sh$,${AI_GROUP}`,
    `DOMAIN-SUFFIX,cursorvm.com,${AI_GROUP}`,
    `DOMAIN,api.cursor.com,${AI_GROUP}`,
    `DOMAIN-SUFFIX,grok.com,${AI_GROUP}`,
    `DOMAIN-REGEX,^repo[0-9]+\\.cursor\\.sh$,${AI_GROUP}`
  ];
  const userOwnedRules = [
    `DOMAIN,repo42.cursor.sh,${AI_GROUP}`,
    `DOMAIN-REGEX,^[a-z0-9-]+\\.api5\\.cursor\\.sh$,${AI_GROUP}`,
    `DOMAIN-REGEX,^(?:us-asia|us-eu|us-only)\\.gcpp\\.cursor\\.sh$,${AI_GROUP}`,
    `DOMAIN-SUFFIX,cursor.com,${AI_GROUP}`,
    `DOMAIN,www.youtube.com,${AI_GROUP}`,
    "MATCH,🚀节点选择"
  ];
  const cleaned = cleanExistingManagedRules([
    ...currentManagedRules,
    ...userOwnedRules
  ]);
  assert.deepEqual(cleaned, userOwnedRules);
});

test("脚本执行两次保持幂等，并保留用户自定义非托管规则", () => {
  const customAiRule = `DOMAIN,custom-ai.example,${AI_GROUP}`;
  const anthropicFallbackRule = "DOMAIN-SUFFIX,anthropic.com,GPT";
  const normalYoutubeRule = "DOMAIN-SUFFIX,youtube.com,Proxy";
  const normalMarketplaceRule = "DOMAIN,marketplace.cursorapi.com,Proxy";
  const retiredCursorRules = [
    `DOMAIN,repo42.cursor.sh,${AI_GROUP}`,
    `DOMAIN-REGEX,^[a-z0-9-]+\\.api5\\.cursor\\.sh$,${AI_GROUP}`,
    `DOMAIN-REGEX,^(?:us-asia|us-eu|us-only)\\.gcpp\\.cursor\\.sh$,${AI_GROUP}`
  ];
  const config = configFixture({
    proxies: [airportNode("HK")],
    groups: [
      group("🚀节点选择", ["HK"]),
      group("GPT", ["HK"]),
      group("Proxy", ["HK"])
    ],
    rules: [
      customAiRule,
      customAiRule,
      anthropicFallbackRule,
      normalYoutubeRule,
      normalMarketplaceRule,
      `DOMAIN,www.youtube.com,${AI_GROUP}`,
      `DOMAIN,marketplace.cursorapi.com,${AI_GROUP}`,
      `DOMAIN-SUFFIX,cursor.com,${AI_GROUP}`,
      `DOMAIN-SUFFIX,api2.cursor.sh,${AI_GROUP}`,
      // v5.6 遗留的 exact 形态规则应被托管清理并按 suffix 重新注入一次。
      `DOMAIN,api.openai.com,${AI_GROUP}`,
      `DOMAIN,chat.openai.com,${AI_GROUP}`,
      `DOMAIN-SUFFIX,chat.openai.com,${AI_GROUP}`,
      ...retiredCursorRules,
      `IP-CIDR,160.79.104.0/21,${AI_GROUP},no-resolve`,
      `IP-CIDR6,2607:6bc0::/32,${AI_GROUP},no-resolve`,
      "MATCH,Proxy"
    ]
  });

  quietMain(config, "赔钱机场");
  const firstNameserverPolicy = structuredClone(config.dns["nameserver-policy"]);
  quietMain(config, "赔钱机场");

  assert.equal(countNamed(config.proxies, HOME_PROXY_NAME), 1);
  assert.equal(countNamed(config["proxy-groups"], AI_GROUP), 1);
  assert.equal(config.rules.includes(anthropicFallbackRule), true);
  for (const host of ["api.anthropic.com", "a-api.anthropic.com"]) {
    const exactRule = `DOMAIN,${host},${AI_GROUP}`;
    assert.equal(config.rules.filter((rule) => rule === exactRule).length, 1);
    assert.ok(config.rules.indexOf(exactRule) < config.rules.indexOf(anthropicFallbackRule));
  }
  assert.equal(config.rules.filter((rule) => rule === customAiRule).length, 1);
  assert.equal(config.rules.includes(normalYoutubeRule), true);
  assert.equal(config.rules.includes(normalMarketplaceRule), true);
  assert.equal(config.rules.includes(`DOMAIN,www.youtube.com,${AI_GROUP}`), true);
  assert.equal(config.rules.includes(`DOMAIN,marketplace.cursorapi.com,${AI_GROUP}`), true);
  assert.equal(config.rules.includes(`DOMAIN-SUFFIX,cursor.com,${AI_GROUP}`), true);
  // cursor_core 默认开启：用户预置的同形托管规则被清理后恰好重新注入一次。
  assert.equal(
    config.rules.filter((rule) => rule === `DOMAIN-SUFFIX,api2.cursor.sh,${AI_GROUP}`).length,
    1
  );
  assert.equal(
    config.rules.filter((rule) => rule === `DOMAIN-SUFFIX,grok.com,${AI_GROUP}`).length,
    1
  );
  assert.equal(
    config.rules.filter((rule) => rule === `DOMAIN,api.openai.com,${AI_GROUP}`).length,
    0
  );
  assert.equal(
    config.rules.filter((rule) => rule === `DOMAIN-SUFFIX,api.openai.com,${AI_GROUP}`).length,
    1
  );
  assert.equal(
    config.rules.filter((rule) => rule === `DOMAIN-SUFFIX,chat.openai.com,${AI_GROUP}`).length,
    0
  );
  for (const host of OPENAI_CORE_EXACT_DOMAINS) {
    assert.equal(
      config.rules.filter((rule) => rule === `DOMAIN,${host},${AI_GROUP}`).length,
      1,
      `exact 主机应重注一次：${host}`
    );
  }
  for (const rule of retiredCursorRules) assert.equal(config.rules.includes(rule), true);
  assert.equal(config.rules.includes(`IP-CIDR,160.79.104.0/21,${AI_GROUP},no-resolve`), true);
  assert.equal(config.rules.includes(`IP-CIDR6,2607:6bc0::/32,${AI_GROUP},no-resolve`), true);
  assert.equal(config.rules.includes(`IP-CIDR,160.79.104.0/23,${AI_GROUP},no-resolve`), true);
  assert.equal(config.rules.includes(`IP-CIDR6,2607:6bc0::/48,${AI_GROUP},no-resolve`), true);
  assert.equal(new Set(config.rules).size, config.rules.length);
  assert.deepEqual(config.dns["nameserver-policy"], firstNameserverPolicy);
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

test("非 AI DoH 拒绝会破坏 Mihomo URL 绑定语义的上游名称", () => {
  assert.throws(
    () => buildUpstreamDoh("Proxy#fallback"),
    /Proxy#fallback.*不能包含 # 或 &/
  );
  assert.throws(
    () => buildUpstreamDoh("Proxy&fallback"),
    /Proxy&fallback.*不能包含 # 或 &/
  );
});

test("AI DNS policy 仅覆盖 AI 核心域名，排除相邻非核心域名", () => {
  const policy = buildNameserverPolicy({});
  assert.deepEqual(policy["api.anthropic.com"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["a-api.anthropic.com"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["+.gemini.google.com"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["+.aistudio.google.com"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["generativelanguage.googleapis.com"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["+.api2.cursor.sh"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["+.authenticate.cursor.sh"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["+.cursorvm.com"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["authenticator.cursor.sh"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["+.grok.com"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["+.chatgpt.com"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["+.api.openai.com"], RESIDENTIAL_DOH);
  for (const host of OPENAI_CORE_EXACT_DOMAINS) {
    assert.deepEqual(policy[host], RESIDENTIAL_DOH);
  }
  assert.equal("+.chat.openai.com" in policy, false);

  for (const key of [
    "www.youtube.com",
    "+.anthropic.com",
    "www.anthropic.com",
    "docs.anthropic.com",
    "support.anthropic.com",
    "status.anthropic.com",
    "telemetry.anthropic.com",
    "i.ytimg.com",
    "maps.googleapis.com",
    "marketplace.cursorapi.com",
    "downloads.cursor.com",
    "+.cursor.com",
    "+.cursorapi.com",
    "+.cursor-cdn.com",
    "+.mixpanel.com",
    "+.x.ai",
    "+.googleapis.com"
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

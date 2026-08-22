"use strict";

const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const childProcess = require("node:child_process");
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
  ROUTE_VERTEX_AI_ENDPOINTS,
  ROUTE_CURSOR_CORE,
  ROUTE_CURSOR_REPOSITORY_INDEXING,
  ROUTE_GROK_CORE,
  ROUTE_GROK_WEB_ASSETS,
  ROUTE_CURSOR_PROCESS_FALLBACK,
  ROUTE_OPENAI_AUTH,
  ROUTE_OPENAI_WEB_ASSETS,
  GEMINI_WEB_SUFFIX_DOMAINS,
  GEMINI_WEB_EXACT_DOMAINS,
  VERTEX_AI_EXACT_DOMAINS,
  VERTEX_AI_DOMAIN_REGEXES,
  CURSOR_SUFFIX_DOMAINS,
  CURSOR_EXACT_DOMAINS,
  CURSOR_REPOSITORY_INDEXING_DOMAIN_REGEXES,
  GROK_SUFFIX_DOMAINS,
  GROK_STRICT_EXACT_DOMAINS,
  GROK_EXACT_DOMAINS,
  OPENAI_CORE_EXACT_DOMAINS,
  OPENAI_AUTH_SUFFIX_DOMAINS,
  OPENAI_AUTH_EXACT_DOMAINS,
  OPENAI_WEB_ASSET_SUFFIX_DOMAINS
} = constants;

function quietMainWith(scriptModule, config, profileName) {
  const originalInfo = console.info;
  const originalWarn = console.warn;
  console.info = () => {};
  console.warn = () => {};
  try {
    return scriptModule.main(config, profileName);
  } finally {
    console.info = originalInfo;
    console.warn = originalWarn;
  }
}

function quietMain(config, profileName) {
  return quietMainWith(script, config, profileName);
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

function withPatchedSwitches(replacements, fn) {
  const directory = fs.mkdtempSync(
    path.join(os.tmpdir(), "clash-verge-patched-switches-")
  );
  try {
    const patchedPath = path.join(directory, "script.js");
    let source = fs.readFileSync(scriptPath, "utf8");
    for (const [name, value] of Object.entries(replacements)) {
      const pattern = new RegExp(
        `^const ${name} = (?:true|false);$`,
        "m"
      );
      assert.match(source, pattern, `应找到布尔常量 ${name}`);
      source = source.replace(pattern, `const ${name} = ${value};`);
    }
    fs.writeFileSync(patchedPath, source, "utf8");
    fn(require(patchedPath));
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

function withPatchedCursorSwitches(options, fn) {
  withPatchedSwitches({
    ROUTE_CURSOR_CORE: options.cursorCore,
    ROUTE_CURSOR_REPOSITORY_INDEXING: options.cursorRepositoryIndexing
  }, fn);
}

function withPatchedGrokWebAssets(enabled, fn) {
  withPatchedSwitches({ ROUTE_GROK_WEB_ASSETS: enabled }, fn);
}

function withPatchedVertexAiEndpoints(enabled, fn) {
  withPatchedSwitches({ ROUTE_VERTEX_AI_ENDPOINTS: enabled }, fn);
}

function withPatchedOpenAiSwitches(openaiAuth, openaiWebAssets, fn) {
  withPatchedSwitches({
    ROUTE_OPENAI_AUTH: openaiAuth,
    ROUTE_OPENAI_WEB_ASSETS: openaiWebAssets
  }, fn);
}

const CURSOR_CORE_HOSTS = [
  "api2.cursor.sh",
  "api3.cursor.sh",
  "api4.cursor.sh",
  "agent.api5.cursor.sh",
  "agentn.global.api5.cursor.sh",
  "authenticate.cursor.sh",
  "prod.authentication.cursor.sh",
  "authenticator.cursor.sh",
  "adminportal42.cursor.sh",
  "us-eu.gcpp.cursor.sh",
  "vm.cursorvm.com",
  "us-east.vm.cursorvm.com",
  "api.cursor.com"
];
const CURSOR_REPOSITORY_INDEXING_HOSTS = [
  "repo0.cursor.sh",
  "repo42.cursor.sh",
  "repo99.cursor.sh"
];
const CURSOR_NEGATIVE_HOSTS = [
  "marketplace.cursorapi.com",
  "downloads.cursor.com",
  "cursor-cdn.com",
  "anysphere-binaries.s3.us-east-1.amazonaws.com",
  "www.cursor.com",
  "repo.cursor.sh",
  "adminportal.cursor.sh",
  "adminportal0.cursor.sh",
  "adminportal999.cursor.sh",
  "feature.api2.cursor.sh",
  "www.api2.cursor.sh"
];

// ---------------------------------------------------------------------------
// 基础配置与多 Profile
// ---------------------------------------------------------------------------

test("脚本版本与默认 dialer-proxy 正确", () => {
  assert.equal(SCRIPT_VERSION, "5.11.0");
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
  assert.equal(ROUTE_VERTEX_AI_ENDPOINTS, true);
  assert.deepEqual(GEMINI_WEB_SUFFIX_DOMAINS, ["gemini.google.com", "aistudio.google.com"]);
  assert.ok(GEMINI_WEB_EXACT_DOMAINS.includes("alkalicore-pa.clients6.google.com"));
  assert.ok(GEMINI_WEB_EXACT_DOMAINS.includes("alkalimakersuite-pa.clients6.google.com"));
  assert.ok(GEMINI_WEB_EXACT_DOMAINS.includes("webchannel-alkalimakersuite-pa.clients6.google.com"));
  assert.deepEqual(VERTEX_AI_EXACT_DOMAINS, [
    "aiplatform.googleapis.com",
    "aiplatform.us.rep.googleapis.com",
    "aiplatform.eu.rep.googleapis.com"
  ]);
  assert.deepEqual(
    VERTEX_AI_DOMAIN_REGEXES,
    ["^[a-z0-9-]+-aiplatform\\.googleapis\\.com$"]
  );

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
    "aiplatform.eu.rep.googleapis.com",
    "cloudaicompanion.googleapis.com",
    "cloudcode-pa.googleapis.com",
    "daily-cloudcode-pa.googleapis.com"
  ]);
});

test("Antigravity language_server 的 daily cloudcode 端点走家宽", () => {
  const rules = buildInjectedRules();
  assert.equal(
    rules.includes(`DOMAIN,daily-cloudcode-pa.googleapis.com,${AI_GROUP}`),
    true
  );
  assertAiRoute(rules, ["daily-cloudcode-pa.googleapis.com"]);
  assertNoAiRoute(rules, ["daily-cloudcode-pa.sandbox.googleapis.com"]);
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

test("Cursor 核心路由默认开启，仓库索引默认不走家宽", () => {
  assert.equal(ROUTE_CURSOR_CORE, true);
  assert.equal(ROUTE_CURSOR_REPOSITORY_INDEXING, false);
  assert.deepEqual(
    CURSOR_SUFFIX_DOMAINS,
    [
      "api5.cursor.sh",
      "gcpp.cursor.sh",
      "authentication.cursor.sh",
      "cursorvm.com"
    ]
  );
  assert.deepEqual(
    CURSOR_EXACT_DOMAINS,
    [
      "api2.cursor.sh",
      "api3.cursor.sh",
      "api4.cursor.sh",
      "authenticate.cursor.sh",
      "authenticator.cursor.sh",
      "adminportal42.cursor.sh",
      "api.cursor.com"
    ]
  );
  assert.deepEqual(
    CURSOR_REPOSITORY_INDEXING_DOMAIN_REGEXES,
    ["^repo[0-9]+\\.cursor\\.sh$"]
  );

  const rules = buildInjectedRules();
  assertAiRoute(rules, CURSOR_CORE_HOSTS);
  assertNoAiRoute(rules, CURSOR_REPOSITORY_INDEXING_HOSTS);
  assertNoAiRoute(rules, CURSOR_NEGATIVE_HOSTS);
  assert.equal(
    rules.includes(`DOMAIN-REGEX,^repo[0-9]+\\.cursor\\.sh$,${AI_GROUP}`),
    false
  );
  assert.equal(
    rules.includes(`DOMAIN-REGEX,^adminportal[0-9]+\\.cursor\\.sh$,${AI_GROUP}`),
    false
  );
  assert.equal(
    rules.includes(`DOMAIN,adminportal42.cursor.sh,${AI_GROUP}`),
    true
  );
  assert.equal(
    rules.includes(`DOMAIN,api2.cursor.sh,${AI_GROUP}`),
    true
  );
  assert.equal(
    rules.includes(`DOMAIN-SUFFIX,api2.cursor.sh,${AI_GROUP}`),
    false
  );
  assert.equal(
    rules.includes(`DOMAIN,authenticate.cursor.sh,${AI_GROUP}`),
    true
  );
  assert.equal(
    rules.includes(`DOMAIN-SUFFIX,authenticate.cursor.sh,${AI_GROUP}`),
    false
  );

  const firstAiRule = rules.findIndex((rule) => rule.endsWith(`,${AI_GROUP}`));
  assert.equal(rules[0], "DOMAIN,localhost,DIRECT");
  assert.ok(firstAiRule > 0, "私有网段 DIRECT 必须排在 AI 规则之前");
  assert.ok(
    rules.slice(0, firstAiRule).every((rule) => rule.includes(",DIRECT")),
    "AI 规则之前只应出现私有直连规则"
  );
  assert.deepEqual(
    rules.filter((rule) => rule.startsWith("PROCESS-")),
    [],
    "默认不注入进程兜底"
  );
});

test("Cursor 核心与仓库索引开关可独立组合", () => {
  const cases = [
    {
      cursorCore: true,
      cursorRepositoryIndexing: false,
      expectCore: true,
      expectRepo: false
    },
    {
      cursorCore: true,
      cursorRepositoryIndexing: true,
      expectCore: true,
      expectRepo: true
    },
    {
      cursorCore: false,
      cursorRepositoryIndexing: false,
      expectCore: false,
      expectRepo: false
    },
    {
      cursorCore: false,
      cursorRepositoryIndexing: true,
      expectCore: false,
      expectRepo: true
    }
  ];

  for (const item of cases) {
    withPatchedCursorSwitches(item, (patched) => {
      const rules = patched.buildInjectedRules();
      const target = patched.constants.AI_GROUP;
      const label =
        `core=${item.cursorCore}, indexing=${item.cursorRepositoryIndexing}`;
      for (const host of CURSOR_CORE_HOSTS) {
        assert.equal(
          ruleMatchesHost(rules, host, target),
          item.expectCore,
          `${label} 时核心主机应变为 ${item.expectCore}：${host}`
        );
      }
      for (const host of CURSOR_REPOSITORY_INDEXING_HOSTS) {
        assert.equal(
          ruleMatchesHost(rules, host, target),
          item.expectRepo,
          `${label} 时索引主机应变为 ${item.expectRepo}：${host}`
        );
      }
      for (const host of CURSOR_NEGATIVE_HOSTS) {
        assert.equal(
          ruleMatchesHost(rules, host, target),
          false,
          `${label} 时负向主机仍不应走家宽：${host}`
        );
      }
      assert.equal(
        rules.includes(`DOMAIN-REGEX,^repo[0-9]+\\.cursor\\.sh$,${target}`),
        item.expectRepo,
        `${label} 时 repo 正则注入状态错误`
      );
    });
  }
});

test("Grok Build 核心域默认走家宽，共享第三方与安装域名不走", () => {
  assert.equal(ROUTE_GROK_CORE, true);
  assert.equal(ROUTE_GROK_WEB_ASSETS, true);
  assert.deepEqual(GROK_SUFFIX_DOMAINS, ["grok.com", "api.x.ai"]);
  assert.deepEqual(GROK_EXACT_DOMAINS, ["auth.x.ai"]);
  assert.deepEqual(
    GROK_STRICT_EXACT_DOMAINS,
    ["grok.com", "cli-chat-proxy.grok.com", "code.grok.com"]
  );

  const rules = buildInjectedRules();
  assertAiRoute(rules, [
    "grok.com",
    "cli-chat-proxy.grok.com",
    "code.grok.com",
    "assets.grok.com",
    "auth.x.ai",
    "api.x.ai",
    "eu-west-1.api.x.ai",
    "mtls.api.x.ai"
  ]);
  assertNoAiRoute(rules, [
    "api.mixpanel.com",
    "x.ai",
    "www.x.ai",
    "storage.googleapis.com"
  ]);
  assert.equal(rules.includes(`DOMAIN-SUFFIX,grok.com,${AI_GROUP}`), true);
  assert.equal(rules.includes(`DOMAIN-SUFFIX,api.x.ai,${AI_GROUP}`), true);
  assert.equal(rules.includes(`DOMAIN,api.x.ai,${AI_GROUP}`), false);

  const policy = buildNameserverPolicy({});
  assert.deepEqual(policy["+.grok.com"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["auth.x.ai"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["+.api.x.ai"], RESIDENTIAL_DOH);
  assert.equal("api.x.ai" in policy, false);
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
  assert.equal(rules.includes(`DOMAIN,antigravity.google,${AI_GROUP}`), true);
  assert.equal(rules.includes(`DOMAIN-SUFFIX,antigravity.google,${AI_GROUP}`), false);
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

test("OpenAI 第一方认证与网页资源开关默认关闭且可独立组合", () => {
  assert.equal(ROUTE_OPENAI_AUTH, false);
  assert.equal(ROUTE_OPENAI_WEB_ASSETS, false);
  assert.deepEqual(OPENAI_AUTH_SUFFIX_DOMAINS, ["auth.openai.com"]);
  assert.deepEqual(OPENAI_AUTH_EXACT_DOMAINS, ["auth0.openai.com"]);
  assert.deepEqual(OPENAI_WEB_ASSET_SUFFIX_DOMAINS, ["oaistatic.com"]);

  const authHosts = [
    "auth.openai.com",
    "setup.auth.openai.com",
    "tenant.auth.openai.com",
    "auth0.openai.com"
  ];
  const assetHosts = ["oaistatic.com", "cdn.oaistatic.com"];
  const alwaysExcluded = [
    "www.openai.com",
    "login.openai.com",
    "child.auth0.openai.com",
    "oaistatsig.com",
    "intercom.io",
    "challenges.cloudflare.com"
  ];
  const cases = [
    { auth: false, assets: false },
    { auth: true, assets: false },
    { auth: false, assets: true },
    { auth: true, assets: true }
  ];

  for (const item of cases) {
    withPatchedOpenAiSwitches(item.auth, item.assets, (patched) => {
      const rules = patched.buildInjectedRules();
      const policy = patched.buildNameserverPolicy({});
      const target = patched.constants.AI_GROUP;
      const label = `auth=${item.auth}, assets=${item.assets}`;

      for (const host of authHosts) {
        assert.equal(
          ruleMatchesHost(rules, host, target),
          item.auth,
          `${label} 时认证主机状态错误：${host}`
        );
      }
      for (const host of assetHosts) {
        assert.equal(
          ruleMatchesHost(rules, host, target),
          item.assets,
          `${label} 时网页资源主机状态错误：${host}`
        );
      }
      for (const host of alwaysExcluded) {
        assert.equal(
          ruleMatchesHost(rules, host, target),
          false,
          `${label} 时相邻或共享主机不应走家宽：${host}`
        );
      }

      assert.equal("+.auth.openai.com" in policy, item.auth);
      assert.equal("auth0.openai.com" in policy, item.auth);
      assert.equal("+.auth0.openai.com" in policy, false);
      assert.equal("+.oaistatic.com" in policy, item.assets);
      assert.equal(rules.includes(`DOMAIN-SUFFIX,openai.com,${target}`), false);
      assert.equal(new Set(rules).size, rules.length);

      if (item.auth && item.assets) {
        const config = configFixture({
          proxies: [airportNode("HK")],
          groups: [group("🚀节点选择", ["HK"])],
          rules: ["MATCH,🚀节点选择"]
        });
        quietMainWith(patched, config, "赔钱机场");
        const firstRules = structuredClone(config.rules);
        const firstPolicy = structuredClone(config.dns["nameserver-policy"]);
        quietMainWith(patched, config, "赔钱机场");
        assert.deepEqual(config.rules, firstRules);
        assert.deepEqual(config.dns["nameserver-policy"], firstPolicy);
      }
    });
  }
});

test("OpenAI 开关分别由开启切换为关闭时清理规则与 DNS，并保留用户自写规则", () => {
  const customAiRule = `DOMAIN,login.openai.com,${AI_GROUP}`;
  const cases = [
    {
      authEnabledAfter: false,
      assetsEnabledAfter: true
    },
    {
      authEnabledAfter: true,
      assetsEnabledAfter: false
    }
  ];

  for (const item of cases) {
    const config = configFixture({
      proxies: [airportNode("HK")],
      groups: [group("🚀节点选择", ["HK"])],
      rules: [customAiRule, "MATCH,🚀节点选择"]
    });

    withPatchedOpenAiSwitches(true, true, (enabled) => {
      quietMainWith(enabled, config, "赔钱机场");
    });
    assert.equal(ruleMatchesHost(config.rules, "auth.openai.com"), true);
    assert.equal(ruleMatchesHost(config.rules, "auth0.openai.com"), true);
    assert.equal(ruleMatchesHost(config.rules, "oaistatic.com"), true);
    assert.equal("+.auth.openai.com" in config.dns["nameserver-policy"], true);
    assert.equal("auth0.openai.com" in config.dns["nameserver-policy"], true);
    assert.equal("+.oaistatic.com" in config.dns["nameserver-policy"], true);

    withPatchedOpenAiSwitches(
      item.authEnabledAfter,
      item.assetsEnabledAfter,
      (disabled) => {
        quietMainWith(disabled, config, "赔钱机场");
      }
    );

    assert.equal(
      ruleMatchesHost(config.rules, "auth.openai.com"),
      item.authEnabledAfter
    );
    assert.equal(
      ruleMatchesHost(config.rules, "auth0.openai.com"),
      item.authEnabledAfter
    );
    assert.equal(
      ruleMatchesHost(config.rules, "oaistatic.com"),
      item.assetsEnabledAfter
    );
    assert.equal(
      "+.auth.openai.com" in config.dns["nameserver-policy"],
      item.authEnabledAfter
    );
    assert.equal(
      "auth0.openai.com" in config.dns["nameserver-policy"],
      item.authEnabledAfter
    );
    assert.equal(
      "+.oaistatic.com" in config.dns["nameserver-policy"],
      item.assetsEnabledAfter
    );
    assert.equal(config.rules.includes(customAiRule), true);
    assert.equal(new Set(config.rules).size, config.rules.length);
  }
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
    `DOMAIN-SUFFIX,auth.openai.com,${AI_GROUP}`,
    `DOMAIN,auth0.openai.com,${AI_GROUP}`,
    `DOMAIN-SUFFIX,oaistatic.com,${AI_GROUP}`,
    `DOMAIN,auth.x.ai,${AI_GROUP}`,
    `DOMAIN,api.x.ai,${AI_GROUP}`,
    `DOMAIN-SUFFIX,api2.cursor.sh,${AI_GROUP}`,
    `DOMAIN-SUFFIX,authenticate.cursor.sh,${AI_GROUP}`,
    `DOMAIN-REGEX,^adminportal[0-9]+\\.cursor\\.sh$,${AI_GROUP}`,
    `DOMAIN-SUFFIX,cursorvm.com,${AI_GROUP}`,
    `DOMAIN,api.cursor.com,${AI_GROUP}`,
    `DOMAIN-SUFFIX,grok.com,${AI_GROUP}`,
    `DOMAIN-REGEX,^repo[0-9]+\\.cursor\\.sh$,${AI_GROUP}`,
    `DOMAIN-SUFFIX,clau.de,${AI_GROUP}`,
    `DOMAIN-SUFFIX,claudemcpclient.com,${AI_GROUP}`,
    `DOMAIN,daily-cloudcode-pa.googleapis.com,${AI_GROUP}`,
    `DOMAIN,geminicloudassist.googleapis.com,${AI_GROUP}`,
    `DOMAIN-SUFFIX,antigravity.google,${AI_GROUP}`
  ];
  const userOwnedRules = [
    `DOMAIN,repo42.cursor.sh,${AI_GROUP}`,
    `DOMAIN-REGEX,^[a-z0-9-]+\\.api5\\.cursor\\.sh$,${AI_GROUP}`,
    `DOMAIN-REGEX,^(?:us-asia|us-eu|us-only)\\.gcpp\\.cursor\\.sh$,${AI_GROUP}`,
    `DOMAIN-SUFFIX,cursor.com,${AI_GROUP}`,
    `DOMAIN,www.youtube.com,${AI_GROUP}`,
    `DOMAIN-SUFFIX,example-user-rule.com,${AI_GROUP}`,
    `DOMAIN,login.openai.com,${AI_GROUP}`,
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
  {
    const exactRule = `DOMAIN,api.anthropic.com,${AI_GROUP}`;
    assert.equal(config.rules.filter((rule) => rule === exactRule).length, 1);
    assert.ok(config.rules.indexOf(exactRule) < config.rules.indexOf(anthropicFallbackRule));
  }
  assert.equal(
    config.rules.filter((rule) => rule === `DOMAIN,a-api.anthropic.com,${AI_GROUP}`).length,
    0
  );
  assert.equal(config.rules.filter((rule) => rule === customAiRule).length, 1);
  assert.equal(config.rules.includes(normalYoutubeRule), true);
  assert.equal(config.rules.includes(normalMarketplaceRule), true);
  assert.equal(config.rules.includes(`DOMAIN,www.youtube.com,${AI_GROUP}`), true);
  assert.equal(config.rules.includes(`DOMAIN,marketplace.cursorapi.com,${AI_GROUP}`), true);
  assert.equal(config.rules.includes(`DOMAIN-SUFFIX,cursor.com,${AI_GROUP}`), true);
  // cursor_core 默认开启：旧 suffix 被清理，改注入 exact 一次。
  assert.equal(
    config.rules.filter((rule) => rule === `DOMAIN-SUFFIX,api2.cursor.sh,${AI_GROUP}`).length,
    0
  );
  assert.equal(
    config.rules.filter((rule) => rule === `DOMAIN,api2.cursor.sh,${AI_GROUP}`).length,
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
  assert.equal(
    config.rules.includes(`DOMAIN-REGEX,^repo[0-9]+\\.cursor\\.sh$,${AI_GROUP}`),
    false,
    "默认关闭仓库索引后不应重新注入托管 repo 正则"
  );
  assert.equal(config.rules.includes(`IP-CIDR,160.79.104.0/21,${AI_GROUP},no-resolve`), true);
  assert.equal(config.rules.includes(`IP-CIDR6,2607:6bc0::/32,${AI_GROUP},no-resolve`), true);
  assert.equal(config.rules.includes(`IP-CIDR,160.79.104.0/23,${AI_GROUP},no-resolve`), true);
  assert.equal(config.rules.includes(`IP-CIDR6,2607:6bc0::/48,${AI_GROUP},no-resolve`), true);
  assert.equal(new Set(config.rules).size, config.rules.length);
  assert.deepEqual(config.dns["nameserver-policy"], firstNameserverPolicy);
});

test("关闭仓库索引后二次运行会移除托管 repo 正则，并保留用户自有规则", () => {
  const managedRepoRule = `DOMAIN-REGEX,^repo[0-9]+\\.cursor\\.sh$,${AI_GROUP}`;
  const unknownAiRule = `DOMAIN,custom-repo-upload.example,${AI_GROUP}`;
  const retiredExactRule = `DOMAIN,repo42.cursor.sh,${AI_GROUP}`;
  const retiredRegexRule = `DOMAIN-REGEX,^[a-z0-9-]+\\.api5\\.cursor\\.sh$,${AI_GROUP}`;
  const config = configFixture({
    proxies: [airportNode("HK")],
    groups: [group("🚀节点选择", ["HK"])],
    rules: [
      managedRepoRule,
      managedRepoRule,
      unknownAiRule,
      retiredExactRule,
      retiredRegexRule,
      "MATCH,🚀节点选择"
    ]
  });

  quietMain(config, "赔钱机场");
  assert.equal(config.rules.includes(managedRepoRule), false);
  assert.equal(config.rules.filter((rule) => rule === unknownAiRule).length, 1);
  assert.equal(config.rules.includes(retiredExactRule), true);
  assert.equal(config.rules.includes(retiredRegexRule), true);
  assert.equal(
    config.rules.filter((rule) => rule === `DOMAIN-SUFFIX,api2.cursor.sh,${AI_GROUP}`).length,
    0
  );
  assert.equal(
    config.rules.filter((rule) => rule === `DOMAIN,api2.cursor.sh,${AI_GROUP}`).length,
    1
  );

  quietMain(config, "赔钱机场");
  assert.equal(config.rules.includes(managedRepoRule), false);
  assert.equal(config.rules.filter((rule) => rule === unknownAiRule).length, 1);
  assert.equal(config.rules.includes(retiredExactRule), true);
  assert.equal(config.rules.includes(retiredRegexRule), true);
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
  assert.equal("a-api.anthropic.com" in policy, false);
  assert.deepEqual(policy["+.gemini.google.com"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["+.aistudio.google.com"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["generativelanguage.googleapis.com"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["api2.cursor.sh"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["authenticate.cursor.sh"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["adminportal42.cursor.sh"], RESIDENTIAL_DOH);
  assert.equal("+.api2.cursor.sh" in policy, false);
  assert.equal("+.authenticate.cursor.sh" in policy, false);
  assert.deepEqual(policy["+.cursorvm.com"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["authenticator.cursor.sh"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["antigravity.google"], RESIDENTIAL_DOH);
  assert.equal("+.antigravity.google" in policy, false);
  assert.deepEqual(policy["daily-cloudcode-pa.googleapis.com"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["cloudcode-pa.googleapis.com"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["+.grok.com"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["+.chatgpt.com"], RESIDENTIAL_DOH);
  assert.deepEqual(policy["+.api.openai.com"], RESIDENTIAL_DOH);
  for (const host of OPENAI_CORE_EXACT_DOMAINS) {
    assert.deepEqual(policy[host], RESIDENTIAL_DOH);
  }
  assert.equal("+.chat.openai.com" in policy, false);
  assert.equal("+.auth.openai.com" in policy, false);
  assert.equal("auth0.openai.com" in policy, false);
  assert.equal("+.oaistatic.com" in policy, false);

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

test("已开启 TUN 时只补齐 DNS 劫持；AI-only 写顶层查找进程 always，不注入进程路由", () => {
  const config = configFixture({
    proxies: [airportNode("HK")],
    groups: [group("🚀节点选择", ["HK"])],
    tun: { enable: true, "dns-hijack": ["udp://any:53"] },
    findProcessMode: "off"
  });
  const output = quietMain(config, "赔钱机场");
  assert.equal(output.tun["dns-hijack"].includes("any:53"), true);
  assert.equal(output.tun["dns-hijack"].includes("tcp://any:53"), true);
  assert.equal(output["find-process-mode"], "always");
  assert.deepEqual(
    output.rules.filter((rule) => String(rule).startsWith("PROCESS-")),
    []
  );
});

test("profile 嵌套的 find-process-mode 仍写出顶层 always", () => {
  const config = configFixture({
    proxies: [airportNode("HK")],
    groups: [group("🚀节点选择", ["HK"])]
  });
  config.profile = { "store-selected": true, "find-process-mode": "always" };
  const output = quietMain(config, "赔钱机场");
  assert.equal(output["find-process-mode"], "always");
  assert.equal(output.profile["find-process-mode"], "always");
});

test("开启进程路由时注入 PROCESS 规则，查找进程仍为顶层 always", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "ai-residential-process-"));
  try {
    const patched = fs.readFileSync(scriptPath, "utf8").replace(
      "const ENABLE_AI_PROCESS_FALLBACK = false;",
      "const ENABLE_AI_PROCESS_FALLBACK = true;"
    );
    assert.match(patched, /const ENABLE_AI_PROCESS_FALLBACK = true;/);
    const patchedPath = path.join(directory, "script.js");
    fs.writeFileSync(patchedPath, patched, "utf8");
    const probeSource = [
      '"use strict";',
      "const script = require(process.argv[1]);",
      "const home = script.constants.HOME_PROXY_NAME;",
      "const config = {",
      "  proxies: [",
      "    { name: home, type: 'socks5', server: 'home.example.test', port: 1080, username: 'home-user', password: 'home-pass', udp: true },",
      "    { name: 'HK', type: 'ss', server: 'hk.example.test', port: 443, cipher: 'aes-128-gcm', password: 'airport-secret', udp: true }",
      "  ],",
      "  'proxy-groups': [{ name: '🚀节点选择', type: 'select', proxies: ['HK'] }],",
      "  rules: [],",
      "  dns: {}",
      "};",
      "const originalInfo = console.info;",
      "const originalWarn = console.warn;",
      "console.info = () => {};",
      "console.warn = () => {};",
      "let output;",
      "try { output = script.main(config, '赔钱机场'); }",
      "finally { console.info = originalInfo; console.warn = originalWarn; }",
      "process.stdout.write(JSON.stringify({",
      "  findProcessMode: output['find-process-mode'],",
      "  processRules: output.rules.filter((rule) => String(rule).startsWith('PROCESS-'))",
      "}));"
    ].join("\n");
    const probe = JSON.parse(childProcess.execFileSync(
      process.execPath,
      ["-e", probeSource, patchedPath],
      { encoding: "utf8" }
    ));
    assert.equal(probe.findProcessMode, "always");
    assert.ok(probe.processRules.length > 0);
    assert.ok(probe.processRules.some((rule) => rule.startsWith("PROCESS-NAME")));
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("最终注入规则不存在重复项", () => {
  const rules = buildInjectedRules();
  assert.equal(new Set(rules).size, rules.length);
});

test("默认注入 45 条 AI-家宽 规则", () => {
  const rules = buildInjectedRules().filter((rule) => rule.includes(AI_GROUP));
  assert.equal(rules.length, 45);
});

test("v5.10 审计后的正向主机走家宽，退出与收窄主机不走", () => {
  const rules = buildInjectedRules();
  assertAiRoute(rules, [
    "platform.claude.com",
    "bridge.claudeusercontent.com",
    "x.frame.claudeusercontent.com",
    "assets-proxy.anthropic.com",
    "eu-west-1.api.x.ai",
    "mtls.api.x.ai",
    "api.x.ai",
    "api2.cursor.sh",
    "authenticate.cursor.sh",
    "prod.authentication.cursor.sh",
    "agent.api5.cursor.sh",
    "us-eu.gcpp.cursor.sh",
    "adminportal42.cursor.sh",
    "api.cursor.com",
    "antigravity.google",
    "daily-cloudcode-pa.googleapis.com",
    "aiplatform.us.rep.googleapis.com",
    "aiplatform.eu.rep.googleapis.com",
    "cloudaicompanion.googleapis.com",
    "us-central1-aiplatform.googleapis.com",
    "widget.claudemcpcontent.com",
    "alkalicore-pa.clients6.google.com",
    "alkalimakersuite-pa.clients6.google.com",
    "webchannel-alkalimakersuite-pa.clients6.google.com",
    "chatgpt.com",
    "ws.chatgpt.com",
    "api.openai.com",
    "us.api.openai.com"
  ]);
  assert.equal(rules.includes(`DOMAIN-SUFFIX,chatgpt.com,${AI_GROUP}`), true);
  assert.equal(rules.includes(`DOMAIN-SUFFIX,claudemcpcontent.com,${AI_GROUP}`), true);
  assert.equal(rules.includes(`DOMAIN-SUFFIX,claude.ai,${AI_GROUP}`), true);
  assert.equal(rules.includes(`DOMAIN-SUFFIX,claude.com,${AI_GROUP}`), true);
  assertNoAiRoute(rules, [
    "clau.de",
    "claudemcpclient.com",
    "a-api.anthropic.com",
    "geminicloudassist.googleapis.com",
    "adminportal0.cursor.sh",
    "adminportal999.cursor.sh",
    "www.api2.cursor.sh",
    "feature.api2.cursor.sh",
    "docs.antigravity.google",
    "download.antigravity.google",
    "www.antigravity.google"
  ]);
});

test("grok_web_assets 关闭后排除 assets.grok.com，仍覆盖 CLI 与会话主机", () => {
  withPatchedGrokWebAssets(false, (patched) => {
    const rules = patched.buildInjectedRules();
    const target = patched.constants.AI_GROUP;
    for (const host of ["grok.com", "cli-chat-proxy.grok.com", "code.grok.com"]) {
      assert.equal(
        ruleMatchesHost(rules, host, target),
        true,
        `grok_web_assets=false 时应走家宽：${host}`
      );
    }
    assert.equal(
      ruleMatchesHost(rules, "assets.grok.com", target),
      false,
      "grok_web_assets=false 时 assets.grok.com 不应走家宽"
    );
    assert.equal(
      ruleMatchesHost(rules, "eu-west-1.api.x.ai", target),
      true,
      "关闭网页资源开关不得拿掉 api.x.ai 后缀"
    );
    assert.equal(rules.includes(`DOMAIN-SUFFIX,api.x.ai,${target}`), true);
    assert.equal(rules.includes(`DOMAIN-SUFFIX,grok.com,${target}`), false);
    assert.equal(rules.includes(`DOMAIN,grok.com,${target}`), true);
    assert.equal(rules.includes(`DOMAIN,cli-chat-proxy.grok.com,${target}`), true);
    assert.equal(rules.includes(`DOMAIN,code.grok.com,${target}`), true);
  });

  withPatchedGrokWebAssets(true, (patched) => {
    const rules = patched.buildInjectedRules();
    const target = patched.constants.AI_GROUP;
    assert.equal(ruleMatchesHost(rules, "assets.grok.com", target), true);
    assert.equal(rules.includes(`DOMAIN-SUFFIX,grok.com,${target}`), true);
  });
});

test("vertex_ai_endpoints 一次控制全部四条 Vertex 规则", () => {
  const vertexHosts = [
    "aiplatform.googleapis.com",
    "aiplatform.us.rep.googleapis.com",
    "aiplatform.eu.rep.googleapis.com",
    "us-central1-aiplatform.googleapis.com"
  ];

  const vertexRules = (target) => [
    `DOMAIN,aiplatform.googleapis.com,${target}`,
    `DOMAIN,aiplatform.us.rep.googleapis.com,${target}`,
    `DOMAIN,aiplatform.eu.rep.googleapis.com,${target}`,
    `DOMAIN-REGEX,^[a-z0-9-]+-aiplatform\\.googleapis\\.com$,${target}`
  ];

  withPatchedVertexAiEndpoints(false, (patched) => {
    const rules = patched.buildInjectedRules();
    const target = patched.constants.AI_GROUP;
    for (const host of vertexHosts) {
      assert.equal(
        ruleMatchesHost(rules, host, target),
        false,
        `vertex_ai_endpoints=false 时不应走家宽：${host}`
      );
    }
    for (const rule of vertexRules(target)) {
      assert.equal(rules.includes(rule), false, `关闭 Vertex 后不应注入：${rule}`);
    }
    assert.equal(
      ruleMatchesHost(rules, "alkalicore-pa.clients6.google.com", target),
      true,
      "关闭 Vertex 开关不得拿掉 alkali* AI Studio 主机"
    );
  });

  withPatchedVertexAiEndpoints(true, (patched) => {
    const rules = patched.buildInjectedRules();
    const target = patched.constants.AI_GROUP;
    for (const host of vertexHosts) {
      assert.equal(
        ruleMatchesHost(rules, host, target),
        true,
        `vertex_ai_endpoints=true 时应走家宽：${host}`
      );
    }
    for (const rule of vertexRules(target)) {
      assert.equal(rules.includes(rule), true, `开启 Vertex 后应注入：${rule}`);
    }
  });
});

test("退出激活主机不再出现 nameserver-policy 键", () => {
  const policy = buildNameserverPolicy({});
  for (const key of [
    "+.clau.de",
    "clau.de",
    "+.claudemcpclient.com",
    "claudemcpclient.com",
    "a-api.anthropic.com",
    "+.a-api.anthropic.com",
    "geminicloudassist.googleapis.com"
  ]) {
    assert.equal(key in policy, false, `DNS policy 不应包含退出激活键：${key}`);
  }
});

"use strict";

/**
 * Clash Verge Rev 全局扩展脚本
 * Claude / ChatGPT / Gemini / Google Antigravity / Cursor AI 核心家宽链路 · v5.4
 *
 * 数据路径：
 *   本机 -> 当前 Profile 的机场代理组/节点 -> 家宽 SOCKS5 -> AI 服务
 *
 * v5.4 重点：
 *   - 默认只让 AI 产品核心、模型推理、代码补全、Agent、索引与产品专属认证流量走家宽。
 *   - 删除 Cursor 插件市场、更新、下载、CDN、Remote-SSH 资产等非推理地址。
 *   - 不注入 YouTube、Maps、广告、统计、通用 Google 静态资源等共享域名。
 *   - Cursor 不再使用 cursor.sh / cursor.com / cursorapi.com / cursor-cdn.com 宽泛后缀。
 *   - 默认关闭进程级兜底、共享遥测、通用 STUN/TURN、公共 DoH/DoT 劫持。
 *   - AI 域名 DNS 经家宽；其他域名 DNS 经当前 Profile 的机场上游，不再默认占用家宽。
 *   - Vertex AI 与 Cursor 滚动端点使用窄范围 DOMAIN-REGEX；已知主机同时进入 DNS policy。
 *   - 保留多 Profile 上游解析、递归链防护、严格配置校验与幂等迁移。
 *   - 自动清理 v5.3 的宽泛规则和旧 DNS policy。
 *
 * 运行环境：Clash Verge Rev 的 JavaScript 扩展脚本环境。
 * 入口签名：main(config, profileName)
 *
 * @file clash-verge-ai-residential.js
 */

// ============================================================
// 0. 脚本标识与保留名称
// ============================================================

const SCRIPT_VERSION = "5.4.0";
const AI_GROUP = "AI-家宽";
const HOME_PROXY_NAME = "家宽-SOCKS5";
const LEGACY_GROUPS = ["Claude-家宽"];

// ============================================================
// 1. 用户配置
// ============================================================

/**
 * 家宽 SOCKS5 模板。
 *
 * 两种配置方式：
 *   1. 直接替换 server / port / username / password；
 *   2. 在 Profile 中预置一个同名“家宽-SOCKS5”节点，并保留下面的 xxx。
 *
 * 方式 2 会复用已有节点的地址、端口和凭据，减少明文凭据散落在脚本中。
 */
const HOME_PROXY_TEMPLATE = {
  name: HOME_PROXY_NAME,
  type: "socks5",
  server: "xxx",                 // TODO: 家宽 SOCKS5 地址
  port: 443,                     // TODO: 1-65535 的整数
  username: "xxx",              // TODO: 无认证时改成空字符串
  password: "xxx",              // TODO: 无认证时改成空字符串
  udp: true,
  "dialer-proxy": "🚀节点选择"  // 用户指定的默认上游；解析优先级最高
};

/**
 * Profile 专属候选覆盖。
 *
 * value 可写字符串或字符串数组。数组只是按顺序尝试，绝不会原样写入 dialer-proxy。
 * 覆盖项未命中时会继续尝试通用候选和当前 Profile 的最终 MATCH/FINAL 目标。
 */
const PROFILE_UPSTREAM_OVERRIDES = {
  // 截图中的“奈云”顶层 selector 名为 Proxy；仍优先尝试用户最常用的 🚀节点选择。
  "奈云": ["🚀节点选择", "Proxy", "自动选择"]

  // 其他 Profile 可按实际名称补充：
  // "赔钱机场": ["🚀节点选择", "Proxy", "自动选择"],
  // "极速云": ["🚀节点选择", "Proxy", "自动选择"]
};

/**
 * 跨 Profile 的上游代理组/节点候选名，按优先级排列。
 */
const UPSTREAM_CANDIDATES = [
  "🚀节点选择",
  "🚀 节点选择",
  "Proxy",
  "PROXY",
  "节点选择",
  "代理选择",
  "自动选择",
  "♻️ 自动选择",
  "自动选优",
  "故障转移",
  "GLOBAL",
  "Global"
];

// 候选均未命中时，允许使用当前 Profile 最后一个 MATCH/FINAL 规则的目标。
const ALLOW_FINAL_RULE_UPSTREAM_FALLBACK = true;

// 仍无法命中时，是否按组名语义猜测。隐私链路默认关闭，避免静默选错出口。
const ALLOW_HEURISTIC_UPSTREAM_FALLBACK = false;

// WorkOS / Intercom / Sentry / Datadog / Stripe 等共享依赖不是模型推理，默认不走家宽。
const ROUTE_OPENAI_SHARED_DEPENDENCIES = false;

// Claude 的统计、客服、风控与共享第三方依赖默认不走家宽。
const ROUTE_CLAUDE_SHARED_DEPENDENCIES = false;

// Google OAuth 是所有 Google 产品共享入口，默认不通过家宽。
const ROUTE_ANTIGRAVITY_GOOGLE_AUTH = false;

// Service Usage / Resource Manager / IAM / API Hub 属于项目配置，不是推理流量。
const ROUTE_ANTIGRAVITY_PROJECT_APIS = false;

// Antigravity 更新、扩展市场和遥测默认关闭。
const ROUTE_ANTIGRAVITY_UPDATE_AND_TELEMETRY = false;

// Gemini Web / Google AI Studio 产品入口。
const ROUTE_GEMINI_WEB_CORE = true;

// Cursor AI API、Tab、Agent、索引、Cloud Agent 与产品专属认证。
const ROUTE_CURSOR_CORE = true;

// Cursor 进程会访问插件市场、GitHub、npm、MCP 和用户后端；默认不做进程级全量代理。
const ROUTE_CURSOR_PROCESS_FALLBACK = false;

// Claude Code 安装、更新、文档与包管理不是推理流量。
const ROUTE_CLAUDE_CODE_AUXILIARY = false;

// 全局进程兜底会扩大作用域，严格 AI-only 模式默认关闭。
const ENABLE_AI_PROCESS_FALLBACK = false;

// 使用 Anthropic 官方入站网段兜底，覆盖域名嗅探失败或直连 IP 的情况。
const ENABLE_ANTHROPIC_IP_FALLBACK = true;

// 通用 STUN/TURN 基础设施会被大量非 AI 应用复用，默认完全不注入。
const ROUTE_SHARED_REALTIME_INFRASTRUCTURE = false;

// 即便启用共享实时基础设施，也默认不按通用端口捕获所有应用。
const ROUTE_GLOBAL_REALTIME_PORTS = false;

// 公共 DoH/DoT 是全局共享服务，不通过家宽。
const ROUTE_PUBLIC_ENCRYPTED_DNS = false;

// 严格模式不继承订阅中未知 nameserver-policy，避免其优先级旁路家宽。
const PRESERVE_UNMANAGED_NAMESERVER_POLICY = false;

// 域名嗅探用于补偿纯 IP 连接和 DNS 映射缺失；采用保守的全局 override-destination=false。
const ENABLE_DOMAIN_SNIFFER = true;

// 仅当用户已经启用 TUN 时补齐 DNS 劫持，不擅自开启 TUN。
const HARDEN_EXISTING_TUN_DNS_HIJACK = true;

// Windows strict-route 可降低多宿主 DNS 泄漏，但可能影响虚拟机或特殊路由。
const ENABLE_TUN_STRICT_ROUTE = false;

// 发现可达子组/节点显式禁用 UDP 时输出警告；顶层上游禁用 UDP 会直接拒绝。
const WARN_ON_REACHABLE_UDP_DISABLED = true;

// ============================================================
// 2. AI 域名清单
// ============================================================

const CORE_SUFFIX_DOMAINS = [
  // Claude Web / Desktop / generated content
  "claude.ai",
  "claude.com",
  "clau.de",
  "claudemcpclient.com",
  "claudemcpcontent.com",
  "claudeusercontent.com",

  // ChatGPT Web / user-uploaded and generated content；通用静态 CDN 不走家宽
  "chatgpt.com",
  "oaiusercontent.com",

  // Google Antigravity 产品域
  "antigravity.google"
];

const CORE_EXACT_DOMAINS = [
  // 第一方模型 API；避免 anthropic.com / openai.com 宽泛后缀。
  "api.anthropic.com",
  "api.openai.com",

  // Antigravity / Gemini Code Assist / Gemini Developer API / Vertex AI
  "cloudcode-pa.googleapis.com",
  "daily-cloudcode-pa.googleapis.com",
  "cloudaicompanion.googleapis.com",
  "geminicloudassist.googleapis.com",
  "generativelanguage.googleapis.com",
  "aiplatform.googleapis.com"
];

// Gemini Web / AI Studio：只保留产品入口，不纳入共享 Google 服务清单。
const GEMINI_WEB_SUFFIX_DOMAINS = [
  "gemini.google.com",
  "aistudio.google.com"
];

const GEMINI_WEB_EXACT_DOMAINS = [
  // Google AI Studio 浏览器端 RPC、权限与流式通道后端。
  "alkalicore-pa.clients6.google.com",
  "alkalimakersuite-pa.clients6.google.com",
  "webchannel-alkalimakersuite-pa.clients6.google.com",

  // Vertex AI 多区域服务端点。
  "aiplatform.us.rep.googleapis.com",
  "aiplatform.eu.rep.googleapis.com"
];

const GEMINI_DOMAIN_REGEXES = [
  // Vertex AI 区域端点，例如 us-central1-aiplatform.googleapis.com。
  "^[a-z0-9-]+-aiplatform\\.googleapis\\.com$"
];

// Cursor 仅保留 AI API / Tab / Agent / 索引 / 专属认证的窄范围后缀。
const CURSOR_SUFFIX_DOMAINS = [
  "api2.cursor.sh",
  "api5.cursor.sh",
  "gcpp.cursor.sh",
  "authentication.cursor.sh"
];

const CURSOR_EXACT_DOMAINS = [
  "api3.cursor.sh",
  "api4.cursor.sh",
  "repo42.cursor.sh",
  "authenticator.cursor.sh",

  // Cursor Cloud Agent / Bugbot AI API；不会匹配 cursor.com 其他页面。
  "api.cursor.com"
];

const CURSOR_DOMAIN_REGEXES = [
  // Cursor API5 的滚动子域，例如 agent.api5.cursor.sh。
  "^[a-z0-9-]+\\.api5\\.cursor\\.sh$",

  // 代码库索引端点可能滚动编号，例如 repo42.cursor.sh。
  "^repo[0-9]+\\.cursor\\.sh$",

  // Cursor Tab 的区域化 GCPP 端点。
  "^(?:us-asia|us-eu|us-only)\\.gcpp\\.cursor\\.sh$"
];

// 仅用于删除 v5.3 及更早版本注入的宽泛/非推理规则，不会重新注入。
const LEGACY_V53_SUFFIX_DOMAINS = [
  "anthropic.com",
  "openai.com",
  "oaistatic.com",
  "oaistatsig.com",
  "gemini.google",
  "cursor.sh",
  "cursor.com",
  "cursorapi.com",
  "cursor-cdn.com"
];

const LEGACY_V53_NON_AI_EXACT_DOMAINS = [
  // 旧 Claude/OpenAI Web、认证和静态基础设施
  "servd-anthropic-website.b-cdn.net",
  "anthropic.com.cdn.cloudflare.net",
  "anthropic.auth0.com",
  "anthropic-com.ghost.io",
  "cdn.openaimerge.com",

  // 旧 Gemini 共享/辅助主机：产品 UI、YouTube、Maps、广告与遥测
  "jnn-pa.googleapis.com",
  "waa-pa.clients6.google.com",
  "ogads-pa.clients6.google.com",
  "optimizationguide-pa.googleapis.com",
  "content-autofill.googleapis.com",
  "lh5.googleusercontent.com",
  "www.googleapis.com",
  "ssl.gstatic.com",
  "fonts.googleapis.com",
  "play.google.com",
  "ogs.google.com",
  "www.google.com",
  "apis.google.com",
  "i.ytimg.com",
  "yt3.ggpht.com",
  "lh3.googleusercontent.com",
  "maps.gstatic.com",
  "lh3.google.com",
  "csp.withgoogle.com",
  "www.youtube.com",
  "fonts.gstatic.com",
  "maps.googleapis.com",
  "www.gstatic.com",
  "encrypted-tbn0.gstatic.com",
  "encrypted-tbn1.gstatic.com",
  "encrypted-tbn2.gstatic.com",
  "encrypted-tbn3.gstatic.com",
  "streetviewpixels-pa.googleapis.com",
  "www.googletagmanager.com",
  "static.doubleclick.net",
  "td.doubleclick.net",
  "googleads.g.doubleclick.net",
  "www.google-analytics.com",

  // 旧 Cursor 插件市场、下载、CDN、更新和远程开发资产
  "marketplace.cursorapi.com",
  "downloads.cursor.com",
  "anysphere-binaries.s3.us-east-1.amazonaws.com",
  "cursor.blob.core.windows.net"
];

const OPENAI_SHARED_SUFFIX_DOMAINS = [
  "ct.sendgrid.net",
  "intercom.io",
  "intercomcdn.com",
  "workos.com",
  "workoscdn.com"
];

const OPENAI_SHARED_EXACT_DOMAINS = [
  "cdn.workos.com",
  "forwarder.workos.com",
  "images.workoscdn.com",
  "setup.workos.com",
  "workos.imgix.net",
  "challenges.cloudflare.com",
  "js.stripe.com",
  "humb.apple.com",
  "js.intercomcdn.com",
  "o207216.ingest.sentry.io",
  "o33249.ingest.sentry.io",
  "rum.browser-intake-datadoghq.com"
];

const CLAUDE_SHARED_SUFFIX_DOMAINS = [
  "statsigapi.net",
  "intercom.io",
  "intercomcdn.com",
  "sentry.io"
];

const CLAUDE_SHARED_EXACT_DOMAINS = [
  "browser-intake-us5-datadoghq.com",
  "http-intake.logs.us5.datadoghq.com",
  "cdn.usefathom.com"
];

const ANTIGRAVITY_GOOGLE_AUTH_DOMAINS = [
  "accounts.google.com",
  "oauth2.googleapis.com",
  "openidconnect.googleapis.com",
  "people.googleapis.com",
  "lh3.googleusercontent.com",
  "lh5.googleusercontent.com",
  "ssl.gstatic.com",
  "www.gstatic.com"
];

const ANTIGRAVITY_PROJECT_API_DOMAINS = [
  "serviceusage.googleapis.com",
  "cloudresourcemanager.googleapis.com",
  "iam.googleapis.com",
  "apihub.googleapis.com"
];

const ANTIGRAVITY_UPDATE_AND_TELEMETRY_EXACT_DOMAINS = [
  "update.googleapis.com",
  "dl.google.com",
  "firebaselogging-pa.googleapis.com",
  "feedback-pa.googleapis.com"
];

const ANTIGRAVITY_UPDATE_AND_TELEMETRY_SUFFIX_DOMAINS = [
  "open-vsx.org"
];

const CLAUDE_CODE_AUXILIARY_EXACT_DOMAINS = [
  "storage.googleapis.com",
  "raw.githubusercontent.com",
  "formulae.brew.sh",
  "registry.npmjs.org"
];

// 仅用于清理 v4/v5.0 旧规则，不会默认重新注入。
const LEGACY_EXACT_DOMAINS = [
  "antigravity.google.com",
  "www.googleapis.com",
  "ws.chatgpt.com"
];

// ============================================================
// 3. WebRTC / DoH / IP 清单
// ============================================================

const REALTIME_EXACT_DOMAINS = [
  "stun.cloudflare.com",
  "stun.l.google.com",
  "stun1.l.google.com",
  "stun2.l.google.com",
  "stun3.l.google.com",
  "stun4.l.google.com",
  "stun.services.mozilla.com"
];

const REALTIME_SUFFIX_DOMAINS = [
  "relay.metered.ca",
  "xirsys.com",
  "livekit.cloud"
];

const PUBLIC_DOH_SUFFIX_DOMAINS = [
  "dns.google",
  "cloudflare-dns.com",
  "one.one.one.one",
  "dns.quad9.net",
  "dns10.quad9.net",
  "dns11.quad9.net",
  "dns.nextdns.io",
  "doh.opendns.com",
  "mozilla.cloudflare-dns.com",
  "adguard-dns.com",
  "doh.cleanbrowsing.org",
  "doh.mullvad.net",
  "dns.adguard-dns.com",
  "dns-family.adguard.com"
];

const PUBLIC_DOH_IPS = [
  "1.1.1.1/32",
  "1.0.0.1/32",
  "8.8.8.8/32",
  "8.8.4.4/32",
  "9.9.9.9/32",
  "149.112.112.112/32",
  "208.67.222.222/32",
  "208.67.220.220/32",
  "94.140.14.14/32",
  "94.140.15.15/32"
];

// Anthropic 官方“入站”网段。旧版 /21 是 Anthropic 发起外连时使用的 outbound 范围。
const ANTHROPIC_INBOUND_IP_RULE_TEMPLATES = [
  "IP-CIDR,160.79.104.0/23,{GROUP},no-resolve",
  "IP-CIDR6,2607:6bc0::/48,{GROUP},no-resolve"
];

// ============================================================
// 4. DNS 常量
// ============================================================

const PRIVATE_DNS = ["system"];

const DIRECT_DOH = [
  "https://dns.alidns.com/dns-query#DIRECT&disable-ipv6=true",
  "https://doh.pub/dns-query#DIRECT&disable-ipv6=true"
];

const RESIDENTIAL_DOH = [
  `https://1.1.1.1/dns-query#${AI_GROUP}&disable-ipv6=true`,
  `https://8.8.8.8/dns-query#${AI_GROUP}&disable-ipv6=true`
];

const NON_AI_DOH_ENDPOINTS = [
  "https://1.1.1.1/dns-query",
  "https://8.8.8.8/dns-query"
];

function buildUpstreamDoh(upstreamName) {
  const target = String(
    upstreamName || HOME_PROXY_TEMPLATE["dialer-proxy"] || ""
  ).trim();

  if (!target) {
    throw new Error(`[${AI_GROUP}] 无法为非 AI DNS 构造机场上游`);
  }

  return NON_AI_DOH_ENDPOINTS.map(
    (endpoint) => `${endpoint}#${target}&disable-ipv6=true`
  );
}

const DEFAULT_FAKE_IP_FILTER = [
  "+.lan",
  "+.local",
  "+.localhost",
  "+.home.arpa",
  "localhost",
  "time.*.com",
  "ntp.*.com",
  "+.msftncsi.com",
  "+.msftconnecttest.com",
  "localhost.ptlogin2.qq.com"
];

const PRIVATE_DNS_POLICY_KEYS = [
  "localhost",
  "+.localhost",
  "+.local",
  "+.lan",
  "+.home.arpa"
];

// ============================================================
// 5. 通用工具
// ============================================================

function isPlainObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function cloneObject(value) {
  return isPlainObject(value) ? { ...value } : {};
}

function uniqueStrings(items) {
  const result = [];
  const seen = new Set();

  for (const item of Array.isArray(items) ? items : []) {
    if (typeof item !== "string" || item.length === 0 || seen.has(item)) continue;
    seen.add(item);
    result.push(item);
  }

  return result;
}

function uniqueScalars(items) {
  const result = [];
  const seen = new Set();

  for (const item of Array.isArray(items) ? items : []) {
    if (typeof item !== "string" && typeof item !== "number") continue;
    const key = `${typeof item}:${String(item)}`;
    if (seen.has(key)) continue;
    seen.add(key);
    result.push(item);
  }

  return result;
}

function dedupeRuleEntries(items) {
  const result = [];
  const seenStrings = new Set();

  for (const item of Array.isArray(items) ? items : []) {
    if (typeof item !== "string") {
      result.push(item);
      continue;
    }
    if (seenStrings.has(item)) continue;
    seenStrings.add(item);
    result.push(item);
  }

  return result;
}

function toStringArray(value) {
  if (typeof value === "string") return value.length > 0 ? [value] : [];
  if (!Array.isArray(value)) return [];
  return value.filter((item) => typeof item === "string" && item.length > 0);
}

function normalizeName(value) {
  return String(value || "")
    .toLowerCase()
    .replace(/[^a-z0-9\u3400-\u9fff]+/g, "");
}

function escapeRegExp(value) {
  return String(value).replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function countNamedItems(items, name) {
  if (!Array.isArray(items)) return 0;
  return items.filter((item) => item && item.name === name).length;
}

function findNamedItem(items, name) {
  if (!Array.isArray(items)) return null;
  for (const item of items) {
    if (item && item.name === name) return item;
  }
  return null;
}

function namedItems(items, name) {
  if (!Array.isArray(items)) return [];
  return items.filter((item) => item && item.name === name);
}

function allGroupNames(config) {
  return uniqueStrings(
    (Array.isArray(config["proxy-groups"]) ? config["proxy-groups"] : [])
      .map((group) => group && group.name)
  );
}

function allProxyNames(config) {
  return uniqueStrings(
    (Array.isArray(config.proxies) ? config.proxies : [])
      .map((proxy) => proxy && proxy.name)
  );
}

function allOutboundNames(config) {
  return uniqueStrings([...allGroupNames(config), ...allProxyNames(config)]);
}

function isForbiddenUpstreamName(name) {
  return [
    AI_GROUP,
    HOME_PROXY_NAME,
    ...LEGACY_GROUPS,
    "DIRECT",
    "REJECT",
    "REJECT-DROP",
    "PASS",
    "COMPATIBLE"
  ].indexOf(name) !== -1;
}

function formatAvailableOutbounds(config) {
  const names = allOutboundNames(config).filter((name) => !isForbiddenUpstreamName(name));
  if (names.length === 0) return "<无可用代理组或节点>";
  const shown = names.slice(0, 30).join("、");
  return names.length > 30 ? `${shown}……（共 ${names.length} 个）` : shown;
}

function warn(message) {
  if (typeof console !== "undefined" && typeof console.warn === "function") {
    console.warn(message);
  }
}

function info(message) {
  if (typeof console !== "undefined" && typeof console.info === "function") {
    console.info(message);
  }
}

// ============================================================
// 6. 保留名称与 outbound 查找
// ============================================================

function validateReservedNameCollisions(config) {
  const proxies = Array.isArray(config.proxies) ? config.proxies : [];
  const groups = Array.isArray(config["proxy-groups"]) ? config["proxy-groups"] : [];

  if (countNamedItems(proxies, HOME_PROXY_NAME) > 1) {
    throw new Error(`[${AI_GROUP}] 存在多个同名代理节点“${HOME_PROXY_NAME}”，无法确定要更新哪一个`);
  }
  if (countNamedItems(groups, AI_GROUP) > 1) {
    throw new Error(`[${AI_GROUP}] 存在多个同名代理组“${AI_GROUP}”，无法安全更新`);
  }
  if (findNamedItem(groups, HOME_PROXY_NAME)) {
    throw new Error(`[${AI_GROUP}] 保留名称“${HOME_PROXY_NAME}”已被代理组占用`);
  }
  if (findNamedItem(proxies, AI_GROUP)) {
    throw new Error(`[${AI_GROUP}] 保留名称“${AI_GROUP}”已被代理节点占用`);
  }

  for (const legacyName of LEGACY_GROUPS) {
    if (findNamedItem(proxies, legacyName)) {
      throw new Error(`[${AI_GROUP}] 旧保留名称“${legacyName}”已被代理节点占用`);
    }
  }

  const existingHome = findNamedItem(proxies, HOME_PROXY_NAME);
  if (existingHome && String(existingHome.type || "").toLowerCase() !== "socks5") {
    throw new Error(
      `[${AI_GROUP}] 同名“${HOME_PROXY_NAME}”节点类型为 ${existingHome.type || "<未设置>"}，` +
      "为避免覆盖用户节点，脚本拒绝继续"
    );
  }

  const existingAiGroup = findNamedItem(groups, AI_GROUP);
  if (existingAiGroup) {
    const proxiesInGroup = Array.isArray(existingAiGroup.proxies)
      ? existingAiGroup.proxies
      : [];
    const isManagedShape =
      existingAiGroup.type === "select" &&
      proxiesInGroup.length === 1 &&
      proxiesInGroup[0] === HOME_PROXY_NAME;

    if (!isManagedShape) {
      throw new Error(
        `[${AI_GROUP}] 已存在非脚本管理的同名代理组“${AI_GROUP}”。` +
        "请重命名该组，或确认其类型为 select 且仅包含 家宽-SOCKS5"
      );
    }
  }
}

function findOutbound(config, name) {
  const groups = namedItems(config["proxy-groups"], name);
  const proxies = namedItems(config.proxies, name);

  if (groups.length > 1 || proxies.length > 1 || (groups.length === 1 && proxies.length === 1)) {
    throw new Error(
      `[${AI_GROUP}] outbound 名称“${name}”存在歧义（同名组/节点或重复定义），` +
      "无法安全用于 dialer-proxy"
    );
  }
  if (groups.length === 1) return { kind: "group", value: groups[0] };
  if (proxies.length === 1) return { kind: "proxy", value: proxies[0] };
  return null;
}

// ============================================================
// 7. 多 Profile 上游解析
// ============================================================

function profileOverrideCandidates(profileName) {
  if (typeof profileName !== "string" || profileName.length === 0) return [];

  if (Object.prototype.hasOwnProperty.call(PROFILE_UPSTREAM_OVERRIDES, profileName)) {
    return toStringArray(PROFILE_UPSTREAM_OVERRIDES[profileName]);
  }

  const normalizedProfileName = normalizeName(profileName);
  if (!normalizedProfileName) return [];

  const matchingKeys = Object.keys(PROFILE_UPSTREAM_OVERRIDES).filter(
    (key) => normalizeName(key) === normalizedProfileName
  );
  if (matchingKeys.length === 1) {
    return toStringArray(PROFILE_UPSTREAM_OVERRIDES[matchingKeys[0]]);
  }
  if (matchingKeys.length > 1) {
    throw new Error(`[${AI_GROUP}] Profile 覆盖配置存在归一化重名：${matchingKeys.join(" / ")}`);
  }

  return [];
}

function resolveCandidate(config, candidate) {
  if (typeof candidate !== "string" || candidate.length === 0) return null;
  if (isForbiddenUpstreamName(candidate)) return null;

  const exact = findOutbound(config, candidate);
  if (exact) return candidate;

  const normalizedCandidate = normalizeName(candidate);
  if (!normalizedCandidate) return null;

  const normalizedMatches = allOutboundNames(config).filter(
    (name) => !isForbiddenUpstreamName(name) && normalizeName(name) === normalizedCandidate
  );

  if (normalizedMatches.length === 1) {
    findOutbound(config, normalizedMatches[0]); // 触发同名歧义检测
    return normalizedMatches[0];
  }
  if (normalizedMatches.length > 1) {
    throw new Error(
      `[${AI_GROUP}] 候选“${candidate}”归一化后匹配多个 outbound：` +
      normalizedMatches.join(" / ")
    );
  }

  return null;
}

function resolveFromCandidates(config, candidates) {
  for (const candidate of uniqueStrings(candidates)) {
    const resolved = resolveCandidate(config, candidate);
    if (resolved) return resolved;
  }
  return null;
}

function extractFinalRuleTarget(rules) {
  if (!ALLOW_FINAL_RULE_UPSTREAM_FALLBACK || !Array.isArray(rules)) return null;

  for (let index = rules.length - 1; index >= 0; index -= 1) {
    const rule = rules[index];
    if (typeof rule !== "string") continue;

    const parts = rule.split(",").map((part) => part.trim());
    const ruleType = String(parts[0] || "").toUpperCase();
    if ((ruleType === "MATCH" || ruleType === "FINAL") && parts.length >= 2) {
      return parts[1];
    }
  }

  return null;
}

function resolveHeuristicUpstream(config) {
  if (!ALLOW_HEURISTIC_UPSTREAM_FALLBACK) return null;

  const allowedTypes = ["select", "url-test", "fallback", "load-balance"];
  const semanticPattern = /(节点选择|代理选择|自动选择|自动选优|proxy|select)/i;
  const groups = Array.isArray(config["proxy-groups"]) ? config["proxy-groups"] : [];
  const matches = groups.filter((group) => {
    if (!group || typeof group.name !== "string") return false;
    if (isForbiddenUpstreamName(group.name)) return false;
    if (allowedTypes.indexOf(String(group.type || "").toLowerCase()) === -1) return false;
    return semanticPattern.test(group.name);
  });

  return matches.length === 1 ? matches[0].name : null;
}

function resolveUpstreamName(config, profileName) {
  const overrideCandidates = profileOverrideCandidates(profileName);
  const globalCandidates = uniqueStrings([
    HOME_PROXY_TEMPLATE["dialer-proxy"],
    ...UPSTREAM_CANDIDATES
  ]);

  const resolvedOverride = resolveFromCandidates(config, overrideCandidates);
  if (resolvedOverride) return resolvedOverride;

  if (overrideCandidates.length > 0) {
    warn(
      `[${AI_GROUP}] Profile“${profileName}”的专属候选未命中：` +
      `${overrideCandidates.join(" / ")}；继续尝试通用候选`
    );
  }

  const resolvedGlobal = resolveFromCandidates(config, globalCandidates);
  if (resolvedGlobal) return resolvedGlobal;

  const finalTarget = extractFinalRuleTarget(config.rules);
  const resolvedFinal = resolveCandidate(config, finalTarget);
  if (resolvedFinal) return resolvedFinal;

  const heuristic = resolveHeuristicUpstream(config);
  if (heuristic) return heuristic;

  const profileText = typeof profileName === "string" && profileName.length > 0
    ? `，Profile：${profileName}`
    : "";
  const finalText = finalTarget ? `；最终规则目标：${finalTarget}` : "";

  throw new Error(
    `[${AI_GROUP}] 找不到可用 dialer-proxy${profileText}。` +
    `候选：${globalCandidates.join(" / ")}${finalText}；` +
    `当前 outbound：${formatAvailableOutbounds(config)}。` +
    "请在 PROFILE_UPSTREAM_OVERRIDES 或 UPSTREAM_CANDIDATES 中补充真实名称。"
  );
}

// ============================================================
// 8. 家宽 SOCKS5 构建与校验
// ============================================================

function isPlaceholder(value) {
  if (typeof value !== "string") return false;
  const normalized = value.trim().toLowerCase();
  return normalized === "xxx" ||
    normalized === "example" ||
    normalized.indexOf("todo") !== -1 ||
    normalized.indexOf("your-") === 0;
}

function chooseTemplateOrExisting(templateValue, existingValue) {
  if (templateValue === undefined || isPlaceholder(templateValue)) return existingValue;
  return templateValue;
}

function buildHomeProxy(config, upstreamName) {
  const existing = findNamedItem(config.proxies, HOME_PROXY_NAME) || {};
  const templateHasEndpoint = !isPlaceholder(HOME_PROXY_TEMPLATE.server);

  // 用户一旦在模板中填写 endpoint，就必须明确处理认证字段。
  // 否则 chooseTemplateOrExisting 会把 xxx 解析为 undefined，静默退化为无认证 SOCKS5。
  if (templateHasEndpoint) {
    const unresolvedCredentials = ["username", "password"].filter(
      (field) => isPlaceholder(HOME_PROXY_TEMPLATE[field]) && existing[field] === undefined
    );
    if (unresolvedCredentials.length > 0) {
      throw new Error(
        `[${AI_GROUP}] 家宽 SOCKS5 ${unresolvedCredentials.join("/")} 仍是占位值 xxx；` +
        "无认证时请显式改为空字符串"
      );
    }
  }

  const homeProxy = {
    ...existing,
    ...HOME_PROXY_TEMPLATE,
    name: HOME_PROXY_NAME,
    type: "socks5",
    server: templateHasEndpoint ? HOME_PROXY_TEMPLATE.server : existing.server,
    port: templateHasEndpoint ? HOME_PROXY_TEMPLATE.port : existing.port,
    username: chooseTemplateOrExisting(HOME_PROXY_TEMPLATE.username, existing.username),
    password: chooseTemplateOrExisting(HOME_PROXY_TEMPLATE.password, existing.password),
    udp: true,
    "dialer-proxy": upstreamName
  };

  if (homeProxy.username === undefined) delete homeProxy.username;
  if (homeProxy.password === undefined) delete homeProxy.password;
  return homeProxy;
}

function validateHomeProxy(homeProxy) {
  if (!homeProxy.server || isPlaceholder(homeProxy.server)) {
    throw new Error(
      `[${AI_GROUP}] 家宽 SOCKS5 server 未配置。` +
      `请修改 HOME_PROXY_TEMPLATE，或在当前 Profile 中预置同名“${HOME_PROXY_NAME}”节点。`
    );
  }
  if (!Number.isInteger(homeProxy.port) || homeProxy.port < 1 || homeProxy.port > 65535) {
    throw new Error(`[${AI_GROUP}] 家宽 SOCKS5 port 必须是 1-65535 的整数`);
  }
  if (isPlaceholder(homeProxy.username) || isPlaceholder(homeProxy.password)) {
    throw new Error(
      `[${AI_GROUP}] 家宽 SOCKS5 用户名/密码仍是占位值 xxx；` +
      "无认证时请改为空字符串"
    );
  }
  if (homeProxy.udp !== true) {
    throw new Error(`[${AI_GROUP}] 家宽 SOCKS5 udp 必须为 true`);
  }
  if (isForbiddenUpstreamName(homeProxy["dialer-proxy"])) {
    throw new Error(
      `[${AI_GROUP}] dialer-proxy“${homeProxy["dialer-proxy"]}”会导致直连、拒绝或递归链`
    );
  }
}

// ============================================================
// 9. 代理组图校验与递归链防护
// ============================================================

function injectedNames() {
  return [HOME_PROXY_NAME, AI_GROUP, ...LEGACY_GROUPS];
}

function groupHasAlternativeSource(group) {
  return (
    (Array.isArray(group.use) && group.use.length > 0) ||
    group["include-all"] === true ||
    group["include-all-proxies"] === true ||
    group["include-all-providers"] === true
  );
}

function removeInjectedReferencesFromGroup(group) {
  if (!group || !Array.isArray(group.proxies)) return;
  const blocked = injectedNames();
  group.proxies = uniqueStrings(
    group.proxies.filter((name) => blocked.indexOf(name) === -1)
  );
}

function appendHomeProxyExcludeFilter(group) {
  if (!group) return;
  const includesOutboundProxies =
    group["include-all"] === true || group["include-all-proxies"] === true;
  if (!includesOutboundProxies) return;

  const exactPattern = `^${escapeRegExp(HOME_PROXY_NAME)}$`;
  const existing = typeof group["exclude-filter"] === "string"
    ? group["exclude-filter"]
    : "";

  if (existing.indexOf(exactPattern) !== -1) return;
  group["exclude-filter"] = existing
    ? `(?:${existing})|(?:${exactPattern})`
    : exactPattern;
}

function hardenAllIncludeAllGroups(groups) {
  if (!Array.isArray(groups)) return;
  for (const group of groups) appendHomeProxyExcludeFilter(group);
}

function buildGroupMap(config) {
  const map = new Map();
  const groups = Array.isArray(config["proxy-groups"]) ? config["proxy-groups"] : [];

  for (const group of groups) {
    if (!group || typeof group.name !== "string" || group.name.length === 0) continue;
    if (map.has(group.name)) {
      throw new Error(`[${AI_GROUP}] 存在重复代理组名称“${group.name}”`);
    }
    map.set(group.name, group);
  }

  return map;
}

function warnForUdpDisabledLeaf(config, name, path) {
  if (!WARN_ON_REACHABLE_UDP_DISABLED) return;
  const outbound = findOutbound(config, name);
  if (!outbound) return;

  if (outbound.kind === "proxy" && outbound.value.udp === false) {
    warn(
      `[${AI_GROUP}] 可达节点“${name}”显式关闭 UDP（路径：${path.join(" -> ")}）。` +
      "当上游组选择该节点时，WebRTC/STUN 可能失败或改走其他路径。"
    );
  }
}

function hardenReachableUpstreamGraph(config, upstreamName) {
  const groupMap = buildGroupMap(config);
  const visited = new Set();
  const visiting = new Set();
  const stack = [];

  function visit(groupName) {
    if (!groupMap.has(groupName)) return;
    if (visiting.has(groupName)) {
      const start = stack.indexOf(groupName);
      const cycle = [...stack.slice(start), groupName];
      throw new Error(`[${AI_GROUP}] 上游代理组存在循环依赖：${cycle.join(" -> ")}`);
    }
    if (visited.has(groupName)) return;

    const group = groupMap.get(groupName);
    visiting.add(groupName);
    stack.push(groupName);

    removeInjectedReferencesFromGroup(group);
    appendHomeProxyExcludeFilter(group);

    if (group["disable-udp"] === true) {
      throw new Error(
        `[${AI_GROUP}] 可达上游代理组“${groupName}”显式禁用了 UDP（路径：${stack.join(" -> ")}）`
      );
    }

    const children = Array.isArray(group.proxies) ? group.proxies : [];
    if (children.length === 0 && !groupHasAlternativeSource(group)) {
      throw new Error(
        `[${AI_GROUP}] 上游代理组“${groupName}”在移除递归引用后没有可用节点来源`
      );
    }

    for (const childName of children) {
      if (groupMap.has(childName)) {
        visit(childName);
      } else {
        warnForUdpDisabledLeaf(config, childName, [...stack, childName]);
      }
    }

    stack.pop();
    visiting.delete(groupName);
    visited.add(groupName);
  }

  visit(upstreamName);
}

function validateTopLevelUpstream(config, upstreamName) {
  const outbound = findOutbound(config, upstreamName);
  if (!outbound) {
    throw new Error(`[${AI_GROUP}] 上游“${upstreamName}”不存在`);
  }

  if (outbound.kind === "group") {
    const group = outbound.value;
    if (group["disable-udp"] === true) {
      throw new Error(`[${AI_GROUP}] 上游代理组“${upstreamName}”显式禁用了 UDP`);
    }
    const explicit = Array.isArray(group.proxies) ? group.proxies : [];
    if (explicit.length === 0 && !groupHasAlternativeSource(group)) {
      throw new Error(`[${AI_GROUP}] 上游代理组“${upstreamName}”没有可用节点来源`);
    }
    return;
  }

  const proxy = outbound.value;
  const proxyType = String(proxy.type || "").toLowerCase();
  if (proxyType === "direct" || proxyType === "reject") {
    throw new Error(`[${AI_GROUP}] 上游“${upstreamName}”不是机场代理节点`);
  }
  if (proxy.udp === false) {
    throw new Error(`[${AI_GROUP}] 上游节点“${upstreamName}”显式关闭了 UDP`);
  }
}

// ============================================================
// 10. 域名、进程与路由规则生成
// ============================================================

function activeSuffixDomains() {
  return uniqueStrings([
    ...CORE_SUFFIX_DOMAINS,
    ...(ROUTE_GEMINI_WEB_CORE ? GEMINI_WEB_SUFFIX_DOMAINS : []),
    ...(ROUTE_CURSOR_CORE ? CURSOR_SUFFIX_DOMAINS : []),
    ...(ROUTE_OPENAI_SHARED_DEPENDENCIES ? OPENAI_SHARED_SUFFIX_DOMAINS : []),
    ...(ROUTE_CLAUDE_SHARED_DEPENDENCIES ? CLAUDE_SHARED_SUFFIX_DOMAINS : []),
    ...(ROUTE_ANTIGRAVITY_UPDATE_AND_TELEMETRY
      ? ANTIGRAVITY_UPDATE_AND_TELEMETRY_SUFFIX_DOMAINS
      : [])
  ]);
}

function activeExactDomains() {
  return uniqueStrings([
    ...CORE_EXACT_DOMAINS,
    ...(ROUTE_GEMINI_WEB_CORE ? GEMINI_WEB_EXACT_DOMAINS : []),
    ...(ROUTE_CURSOR_CORE ? CURSOR_EXACT_DOMAINS : []),
    ...(ROUTE_OPENAI_SHARED_DEPENDENCIES ? OPENAI_SHARED_EXACT_DOMAINS : []),
    ...(ROUTE_CLAUDE_SHARED_DEPENDENCIES ? CLAUDE_SHARED_EXACT_DOMAINS : []),
    ...(ROUTE_ANTIGRAVITY_GOOGLE_AUTH ? ANTIGRAVITY_GOOGLE_AUTH_DOMAINS : []),
    ...(ROUTE_ANTIGRAVITY_PROJECT_APIS ? ANTIGRAVITY_PROJECT_API_DOMAINS : []),
    ...(ROUTE_ANTIGRAVITY_UPDATE_AND_TELEMETRY
      ? ANTIGRAVITY_UPDATE_AND_TELEMETRY_EXACT_DOMAINS
      : []),
    ...(ROUTE_CLAUDE_CODE_AUXILIARY ? CLAUDE_CODE_AUXILIARY_EXACT_DOMAINS : [])
  ]);
}

function activeDomainRegexes() {
  return uniqueStrings([
    ...GEMINI_DOMAIN_REGEXES,
    ...(ROUTE_CURSOR_CORE ? CURSOR_DOMAIN_REGEXES : [])
  ]);
}

function allPossibleSuffixDomains() {
  return uniqueStrings([
    ...CORE_SUFFIX_DOMAINS,
    ...GEMINI_WEB_SUFFIX_DOMAINS,
    ...CURSOR_SUFFIX_DOMAINS,
    ...LEGACY_V53_SUFFIX_DOMAINS,
    ...OPENAI_SHARED_SUFFIX_DOMAINS,
    ...CLAUDE_SHARED_SUFFIX_DOMAINS,
    ...ANTIGRAVITY_UPDATE_AND_TELEMETRY_SUFFIX_DOMAINS,
    ...PUBLIC_DOH_SUFFIX_DOMAINS
  ]);
}

function allPossibleExactDomains() {
  return uniqueStrings([
    ...CORE_EXACT_DOMAINS,
    ...GEMINI_WEB_EXACT_DOMAINS,
    ...CURSOR_EXACT_DOMAINS,
    ...LEGACY_V53_NON_AI_EXACT_DOMAINS,
    ...OPENAI_SHARED_EXACT_DOMAINS,
    ...CLAUDE_SHARED_EXACT_DOMAINS,
    ...ANTIGRAVITY_GOOGLE_AUTH_DOMAINS,
    ...ANTIGRAVITY_PROJECT_API_DOMAINS,
    ...ANTIGRAVITY_UPDATE_AND_TELEMETRY_EXACT_DOMAINS,
    ...CLAUDE_CODE_AUXILIARY_EXACT_DOMAINS,
    ...REALTIME_EXACT_DOMAINS,
    ...LEGACY_EXACT_DOMAINS
  ]);
}

function allPossibleDomainRegexes() {
  return uniqueStrings([
    ...GEMINI_DOMAIN_REGEXES,
    ...CURSOR_DOMAIN_REGEXES
  ]);
}

function buildPrivateDirectRules() {
  // 这些规则必须排在进程兜底之前，避免 localhost / MCP / LAN 被送往家宽。
  return [
    "DOMAIN,localhost,DIRECT",
    "DOMAIN-SUFFIX,localhost,DIRECT",
    "DOMAIN-SUFFIX,local,DIRECT",
    "DOMAIN-SUFFIX,lan,DIRECT",
    "DOMAIN-SUFFIX,home.arpa,DIRECT",
    "IP-CIDR,127.0.0.0/8,DIRECT,no-resolve",
    "IP-CIDR,10.0.0.0/8,DIRECT,no-resolve",
    "IP-CIDR,100.64.0.0/10,DIRECT,no-resolve",
    "IP-CIDR,169.254.0.0/16,DIRECT,no-resolve",
    "IP-CIDR,172.16.0.0/12,DIRECT,no-resolve",
    "IP-CIDR,192.168.0.0/16,DIRECT,no-resolve",
    "IP-CIDR,224.0.0.0/4,DIRECT,no-resolve",
    "IP-CIDR6,::1/128,DIRECT,no-resolve",
    "IP-CIDR6,fc00::/7,DIRECT,no-resolve",
    "IP-CIDR6,fe80::/10,DIRECT,no-resolve",
    "IP-CIDR6,ff00::/8,DIRECT,no-resolve"
  ];
}

function buildDomainRules(targetGroup) {
  return uniqueStrings([
    ...activeSuffixDomains().map((domain) => `DOMAIN-SUFFIX,${domain},${targetGroup}`),
    ...activeExactDomains().map((domain) => `DOMAIN,${domain},${targetGroup}`),
    ...activeDomainRegexes().map((pattern) => `DOMAIN-REGEX,${pattern},${targetGroup}`)
  ]);
}

function buildCoreAiProcessRules(targetGroup) {
  return [
    // Antigravity 主进程与安装目录内的 language_server 等子进程
    `PROCESS-NAME-REGEX,(?i)^antigravity(?:[ _-]ide)?(?:\\.exe)?$,${targetGroup}`,
    `PROCESS-PATH-REGEX,(?i).*[/\\\\]antigravity(?:[ _-]ide)?[/\\\\].*,${targetGroup}`,

    // ChatGPT 桌面端、Electron Helper、Codex/OpenAI CLI
    `PROCESS-NAME-REGEX,(?i)^(?:chatgpt(?: helper.*)?|codex|openai)(?:\\.exe)?$,${targetGroup}`,
    `PROCESS-PATH-REGEX,(?i).*[/\\\\](?:chatgpt|codex|openai)(?:[ _-][^/\\\\]+)?[/\\\\].*,${targetGroup}`,

    // Claude Desktop / Claude Code
    `PROCESS-NAME-REGEX,(?i)^(?:claude(?: desktop)?|claude-code)(?:\\.exe)?$,${targetGroup}`,
    `PROCESS-PATH-REGEX,(?i).*[/\\\\](?:claude|claude-code)(?:[ _-][^/\\\\]+)?[/\\\\].*,${targetGroup}`
  ];
}

function buildCursorProcessRules(targetGroup) {
  return [
    `PROCESS-NAME-REGEX,(?i)^(?:cursor|cursor-agent)(?: helper.*)?(?:\\.exe)?$,${targetGroup}`,
    `PROCESS-PATH-REGEX,(?i).*[/\\\\]cursor(?:\\.app)?[/\\\\].*,${targetGroup}`
  ];
}

function buildAllProcessRules(targetGroup) {
  return uniqueStrings([
    ...buildCoreAiProcessRules(targetGroup),
    ...buildCursorProcessRules(targetGroup)
  ]);
}

function buildProcessRules(targetGroup) {
  if (!ENABLE_AI_PROCESS_FALLBACK) return [];
  return uniqueStrings([
    ...buildCoreAiProcessRules(targetGroup),
    ...(ROUTE_CURSOR_PROCESS_FALLBACK ? buildCursorProcessRules(targetGroup) : [])
  ]);
}

function buildAnthropicIpRules(targetGroup) {
  if (!ENABLE_ANTHROPIC_IP_FALLBACK) return [];
  return ANTHROPIC_INBOUND_IP_RULE_TEMPLATES.map(
    (template) => template.replace("{GROUP}", targetGroup)
  );
}

function buildRealtimeRules(targetGroup, includeDisabledRules) {
  const includeShared =
    ROUTE_SHARED_REALTIME_INFRASTRUCTURE || includeDisabledRules === true;
  if (!includeShared) return [];

  const includePorts =
    ROUTE_GLOBAL_REALTIME_PORTS || includeDisabledRules === true;
  const portRules = includePorts
    ? [
        `AND,((NETWORK,UDP),(DST-PORT,3478-3481)),${targetGroup}`,
        `DST-PORT,5349,${targetGroup}`,
        `AND,((NETWORK,UDP),(DST-PORT,19302-19309)),${targetGroup}`
      ]
    : [];

  return uniqueStrings([
    ...portRules,
    ...REALTIME_EXACT_DOMAINS.map((domain) => `DOMAIN,${domain},${targetGroup}`),
    ...REALTIME_SUFFIX_DOMAINS.map((domain) => `DOMAIN-SUFFIX,${domain},${targetGroup}`)
  ]);
}

function buildDnsLeakRules(targetGroup, includeDisabledRules) {
  if (!ROUTE_PUBLIC_ENCRYPTED_DNS && includeDisabledRules !== true) return [];

  return [
    ...PUBLIC_DOH_SUFFIX_DOMAINS.map(
      (domain) => `DOMAIN-SUFFIX,${domain},${targetGroup}`
    ),
    ...PUBLIC_DOH_IPS.map(
      (cidr) => `IP-CIDR,${cidr},${targetGroup},no-resolve`
    ),
    `DST-PORT,853,${targetGroup}`
  ];
}

function buildInjectedRules() {
  return uniqueStrings([
    ...buildPrivateDirectRules(),
    ...buildDomainRules(AI_GROUP),
    ...buildAnthropicIpRules(AI_GROUP),
    ...buildRealtimeRules(AI_GROUP, false),
    ...buildDnsLeakRules(AI_GROUP, false),
    ...buildProcessRules(AI_GROUP)
  ]);
}

// ============================================================
// 11. 旧规则清理与迁移
// ============================================================

const LEGACY_RULE_TEMPLATES = [
  "DOMAIN-KEYWORD,datadog,{GROUP}",
  "DOMAIN-KEYWORD,sift,{GROUP}",
  "DOMAIN-KEYWORD,stun,{GROUP}",
  "DOMAIN-KEYWORD,turn,{GROUP}",
  "DOMAIN-SUFFIX,stun.l.google.com,{GROUP}",
  "DST-PORT,3478-3481,{GROUP}",
  "DST-PORT,19302-19309,{GROUP}",
  "IP-CIDR,160.79.104.0/21,{GROUP},no-resolve",
  "IP-CIDR6,2607:6bc0::/32,{GROUP},no-resolve",
  "IP-ASN,399358,{GROUP},no-resolve"
];

const LEGACY_PROCESS_RULE_TEMPLATES = [
  "PROCESS-NAME-REGEX,(?i)^(?:antigravity(?:[ _-]ide)?|chatgpt(?: helper.*)?|claude(?: desktop)?)(?:\\.exe)?$,{GROUP}",
  "PROCESS-PATH-REGEX,(?i).*[/\\\\](?:antigravity|chatgpt|claude)(?:[ _-][^/\\\\]+)?[/\\\\].*,{GROUP}"
];

function buildManagedRuleSet() {
  const managed = new Set(buildPrivateDirectRules());
  const targets = [AI_GROUP, ...LEGACY_GROUPS];

  for (const target of targets) {
    for (const domain of allPossibleSuffixDomains()) {
      managed.add(`DOMAIN-SUFFIX,${domain},${target}`);
    }
    for (const domain of allPossibleExactDomains()) {
      managed.add(`DOMAIN,${domain},${target}`);
    }
    for (const pattern of allPossibleDomainRegexes()) {
      managed.add(`DOMAIN-REGEX,${pattern},${target}`);
    }
    for (const domain of REALTIME_SUFFIX_DOMAINS) {
      managed.add(`DOMAIN-SUFFIX,${domain},${target}`);
    }
    for (const rule of buildRealtimeRules(target, true)) managed.add(rule);
    for (const rule of buildDnsLeakRules(target, true)) managed.add(rule);
    for (const rule of buildAllProcessRules(target)) managed.add(rule);
    for (const template of ANTHROPIC_INBOUND_IP_RULE_TEMPLATES) {
      managed.add(template.replace("{GROUP}", target));
    }
    for (const template of LEGACY_RULE_TEMPLATES) {
      managed.add(template.replace("{GROUP}", target));
    }
    for (const template of LEGACY_PROCESS_RULE_TEMPLATES) {
      managed.add(template.replace("{GROUP}", target));
    }
  }

  return managed;
}

function migrateLegacyRuleTarget(rule) {
  if (typeof rule !== "string") return rule;
  const parts = rule.split(",");
  let changed = false;

  for (let index = 0; index < parts.length; index += 1) {
    const trimmed = parts[index].trim();
    if (LEGACY_GROUPS.indexOf(trimmed) !== -1) {
      const prefixLength = parts[index].length - parts[index].trimStart().length;
      const suffixLength = parts[index].length - parts[index].trimEnd().length;
      parts[index] =
        parts[index].slice(0, prefixLength) +
        AI_GROUP +
        (suffixLength > 0 ? parts[index].slice(parts[index].length - suffixLength) : "");
      changed = true;
    }
  }

  return changed ? parts.join(",") : rule;
}

function cleanAndMigrateExistingRules(rules) {
  const managed = buildManagedRuleSet();
  const result = [];

  for (const rule of Array.isArray(rules) ? rules : []) {
    if (typeof rule === "string" && managed.has(rule)) continue;
    result.push(migrateLegacyRuleTarget(rule));
  }

  return dedupeRuleEntries(result);
}

// ============================================================
// 12. DNS 构建
// ============================================================

function buildManagedDnsPolicyKeySet() {
  const managed = new Set([
    "geosite:cn",
    "geosite:private",
    "geosite:cn,private",
    ...PRIVATE_DNS_POLICY_KEYS
  ]);

  for (const domain of allPossibleSuffixDomains()) managed.add(`+.${domain}`);
  for (const domain of allPossibleExactDomains()) managed.add(domain);
  return managed;
}

function buildNameserverPolicy(existingPolicy) {
  const source = isPlainObject(existingPolicy) ? existingPolicy : {};
  const managedKeys = buildManagedDnsPolicyKeySet();
  const policy = {};

  if (PRESERVE_UNMANAGED_NAMESERVER_POLICY) {
    for (const key of Object.keys(source)) {
      if (!managedKeys.has(key)) policy[key] = source[key];
    }
  }

  // 私有域名不应发往公网 DoH。
  for (const key of PRIVATE_DNS_POLICY_KEYS) policy[key] = PRIVATE_DNS;

  // 具体 AI / 公共 DoH 域名优先于宽泛 geosite。
  for (const domain of activeSuffixDomains()) {
    policy[`+.${domain}`] = RESIDENTIAL_DOH;
  }
  for (const domain of activeExactDomains()) {
    policy[domain] = RESIDENTIAL_DOH;
  }
  if (ROUTE_PUBLIC_ENCRYPTED_DNS) {
    for (const domain of PUBLIC_DOH_SUFFIX_DOMAINS) {
      policy[`+.${domain}`] = RESIDENTIAL_DOH;
    }
  }

  policy["geosite:private"] = PRIVATE_DNS;
  policy["geosite:cn"] = DIRECT_DOH;
  return policy;
}

function buildFakeIpFilter(base) {
  const mode = String(base["fake-ip-filter-mode"] || "blacklist").toLowerCase();
  const existing = Array.isArray(base["fake-ip-filter"])
    ? base["fake-ip-filter"]
    : [];

  if (mode !== "blacklist") {
    warn(
      `[${AI_GROUP}] 检测到 fake-ip-filter-mode=${mode}；` +
      "脚本强制使用 blacklist，并丢弃语义不兼容的旧 fake-ip-filter 条目"
    );
    return DEFAULT_FAKE_IP_FILTER.slice();
  }

  return uniqueStrings([...existing, ...DEFAULT_FAKE_IP_FILTER]);
}

function buildDnsConfig(existingDns, upstreamName) {
  const base = cloneObject(existingDns);
  const fakeIpFilter = buildFakeIpFilter(base);

  // 这些字段可能创建第二条解析路径，严格模式统一移除。
  delete base.fallback;
  delete base["fallback-filter"];
  delete base["fallback-lazy-query"];
  delete base["proxy-server-nameserver-policy"];
  delete base["fake-ip-range6"];

  return {
    ...base,
    enable: true,
    ipv6: false,
    "cache-algorithm": base["cache-algorithm"] || "arc",
    "prefer-h3": false,
    "enhanced-mode": "fake-ip",
    "fake-ip-range": base["fake-ip-range"] || "198.18.0.1/16",
    "fake-ip-filter-mode": "blacklist",
    "fake-ip-filter": fakeIpFilter,

    // 只用于解析 DoH 上游域名；Mihomo 要求 default-nameserver 使用 IP。
    "default-nameserver": ["223.5.5.5", "119.29.29.29"],

    // 非 AI 域名 DNS 经当前 Profile 的机场上游；AI 域名由 nameserver-policy 改走家宽。
    nameserver: buildUpstreamDoh(upstreamName),

    // 代理服务器域名解析不能依赖 AI_GROUP，否则会形成 bootstrap 循环。
    "proxy-server-nameserver": DIRECT_DOH,

    // DIRECT 出口使用国内 DoH，并继续遵循 nameserver-policy。
    "direct-nameserver": DIRECT_DOH,
    "direct-nameserver-follow-policy": true,

    "nameserver-policy": buildNameserverPolicy(base["nameserver-policy"]),
    "respect-rules": true
  };
}

// ============================================================
// 13. 配置对象操作
// ============================================================

function upsertNamedItem(items, item) {
  const result = Array.isArray(items) ? items.slice() : [];
  const indexes = [];

  for (let index = 0; index < result.length; index += 1) {
    if (result[index] && result[index].name === item.name) indexes.push(index);
  }
  if (indexes.length > 1) {
    throw new Error(`[${AI_GROUP}] 无法 upsert 重复名称“${item.name}”`);
  }
  if (indexes.length === 1) result[indexes[0]] = item;
  else result.unshift(item);
  return result;
}

function migrateLegacyGroupReferences(groups) {
  const result = Array.isArray(groups) ? groups : [];

  for (const group of result) {
    if (!group || !Array.isArray(group.proxies)) continue;
    group.proxies = uniqueStrings(
      group.proxies.map((name) =>
        LEGACY_GROUPS.indexOf(name) !== -1 ? AI_GROUP : name
      )
    );
  }

  return result;
}

function removeLegacyGroups(groups) {
  return (Array.isArray(groups) ? groups : []).filter(
    (group) => !group || LEGACY_GROUPS.indexOf(group.name) === -1
  );
}

function buildAiGroup(config) {
  const existing = findNamedItem(config["proxy-groups"], AI_GROUP) || {};
  return {
    ...existing,
    name: AI_GROUP,
    type: "select",
    proxies: [HOME_PROXY_NAME],
    "disable-udp": false
  };
}

function hardenTun(config) {
  if (!HARDEN_EXISTING_TUN_DNS_HIJACK) return;
  if (!isPlainObject(config.tun) || config.tun.enable !== true) return;

  const current = Array.isArray(config.tun["dns-hijack"])
    ? config.tun["dns-hijack"]
    : [];
  config.tun["dns-hijack"] = uniqueStrings([
    ...current,
    "any:53",
    "tcp://any:53"
  ]);

  if (ENABLE_TUN_STRICT_ROUTE) config.tun["strict-route"] = true;
}

function mergeSniffProtocol(existingProtocol, defaultPorts, overrideDestination) {
  const existing = cloneObject(existingProtocol);
  return {
    ...existing,
    ports: uniqueScalars([
      ...(Array.isArray(existing.ports) ? existing.ports : []),
      ...defaultPorts
    ]),
    ...(overrideDestination === undefined
      ? {}
      : { "override-destination": overrideDestination })
  };
}

function hardenSniffer(config) {
  if (!ENABLE_DOMAIN_SNIFFER) return;

  const existing = cloneObject(config.sniffer);
  const sniff = cloneObject(existing.sniff);
  config.sniffer = {
    ...existing,
    enable: true,
    "force-dns-mapping": true,
    "parse-pure-ip": true,
    "override-destination": false,
    sniff: {
      ...sniff,
      HTTP: mergeSniffProtocol(sniff.HTTP, [80, "8080-8880"], true),
      TLS: mergeSniffProtocol(sniff.TLS, [443, 8443], undefined),
      QUIC: mergeSniffProtocol(sniff.QUIC, [443, 8443], undefined)
    }
  };
}

function ensureProcessLookup(config) {
  if (!ENABLE_AI_PROCESS_FALLBACK) return;
  if (!config["find-process-mode"] || config["find-process-mode"] === "off") {
    config["find-process-mode"] = "strict";
  }
}

// ============================================================
// 14. 主流程
// ============================================================

function main(config, profileName) {
  if (!config || typeof config !== "object") return config;

  if (!Array.isArray(config.proxies)) config.proxies = [];
  if (!Array.isArray(config["proxy-groups"])) config["proxy-groups"] = [];
  if (!Array.isArray(config.rules)) config.rules = [];

  // 1. 在任何覆盖或迁移前检查保留名称，防止静默破坏用户配置。
  validateReservedNameCollisions(config);

  // 2. 为当前 Profile 动态解析一个真实存在的上游名称。
  const upstreamName = resolveUpstreamName(config, profileName);

  // 3. 迁移旧组引用并移除旧组本体。
  config["proxy-groups"] = migrateLegacyGroupReferences(config["proxy-groups"]);
  config["proxy-groups"] = removeLegacyGroups(config["proxy-groups"]);

  // 4. 防止 include-all / 嵌套组把家宽节点重新纳入上游，形成递归链。
  hardenAllIncludeAllGroups(config["proxy-groups"]);
  hardenReachableUpstreamGraph(config, upstreamName);
  validateTopLevelUpstream(config, upstreamName);

  // 5. 构建家宽 SOCKS5，dialer-proxy 始终是单一、已解析名称。
  const homeProxy = buildHomeProxy(config, upstreamName);
  validateHomeProxy(homeProxy);
  config.proxies = upsertNamedItem(config.proxies, homeProxy);

  // 6. 注入统一 AI 出口组；不提供 DIRECT 回退，故障时 fail closed。
  config["proxy-groups"] = upsertNamedItem(
    config["proxy-groups"],
    buildAiGroup(config)
  );

  // 7. 精确清理脚本管理的旧规则；未知自定义规则保留并迁移旧组目标。
  const existingRules = cleanAndMigrateExistingRules(config.rules);
  config.rules = dedupeRuleEntries([
    ...buildInjectedRules(),
    ...existingRules
  ]);

  // 8. 重建严格 DNS 路径。
  config.dns = buildDnsConfig(config.dns, upstreamName);

  // 9. 加固用户已经启用的 TUN 与域名嗅探；进程级全量代理默认关闭。
  hardenTun(config);
  hardenSniffer(config);
  ensureProcessLookup(config);

  // 10. 统一关闭 Mihomo IPv6；操作系统层仍需由 TUN/系统路由约束。
  config.ipv6 = false;

  info(
    `[${AI_GROUP} v${SCRIPT_VERSION}] Profile“${profileName || "<未命名>"}”` +
    `：dialer-proxy -> ${upstreamName}`
  );

  return config;
}

// Node.js 单元测试导出；Clash Verge 环境不存在 module，不受影响。
if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    main,
    resolveUpstreamName,
    buildDnsConfig,
    buildUpstreamDoh,
    buildInjectedRules,
    buildNameserverPolicy,
    cleanAndMigrateExistingRules,
    constants: {
      SCRIPT_VERSION,
      AI_GROUP,
      HOME_PROXY_NAME,
      HOME_PROXY_TEMPLATE,
      PROFILE_UPSTREAM_OVERRIDES,
      UPSTREAM_CANDIDATES,
      RESIDENTIAL_DOH,
      NON_AI_DOH_ENDPOINTS,
      DIRECT_DOH,
      PRIVATE_DNS,
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
    }
  };
}

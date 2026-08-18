"use strict";

/**
 * Clash Verge Rev 全局扩展脚本
 * Claude / ChatGPT / Gemini / Google Antigravity / Cursor / Grok Build 核心家宽链路 · v5.9.0
 *
 * 数据路径：
 *   本机 -> 当前 Profile 的机场代理组/节点 -> 家宽 SOCKS5 -> AI 服务
 *
 * v5.9.0 重点：
 *   - 仓库索引主机 repo[0-9]+.cursor.sh 从 Cursor 核心目录拆出；
 *     routing.cursor_repository_indexing 默认关闭，不再消耗家宽。
 *   - Cursor Chat/Tab/Agent/认证/Cloud Agent 仍由 routing.cursor_core 控制（默认开启）。
 *
 * v5.8.1 重点：
 *   - 大订阅 outbound 索引，避免按叶子全表扫描；UDP 叶子警告改为一条汇总。
 *
 * v5.8 重点：
 *   - 按官方 help.openai.com/9247338 以 exact 补齐五个 chat.openai.com 家族主机；
 *     不注入 DOMAIN-SUFFIX,chat.openai.com。
 *
 * v5.7 重点：
 *   - 域名对齐官方网络文档：补 Claude MCP 代理与资产代理、Grok 认证与 API 域；
 *     api.openai.com 从 exact 提升为 suffix，覆盖 Codex 的 us./eu. 数据驻留前缀。
 *   - 上游代理组中的保留名引用被移除时输出 warn，递归链清理不再静默。
 *   - 记录 Clash Verge Rev 权威字段（tun/ipv6）对脚本改写的覆盖行为并提示。
 *
 * v5.6 重点：
 *   - Cursor 核心路由默认开启；补充授权端点、SSO 管理门户与 Cloud Agent VM 域。
 *   - 新增 Grok Build（xAI grok CLI）核心域与 routing.grok_core 开关，默认开启。
 *   - 默认只让 AI 产品核心、模型推理、代码补全、Agent、索引与产品专属认证流量走家宽。
 *   - 不注入插件市场、CDN、更新下载、广告、统计、通用 Google 静态资源等共享域名。
 *   - 默认关闭进程级兜底、共享遥测、通用 STUN/TURN、公共 DoH/DoT 劫持。
 *   - AI 域名 DNS 经家宽；其他域名 DNS 经当前 Profile 的机场上游，不再默认占用家宽。
 *   - 保留多 Profile 上游解析、递归链防护、严格配置校验与幂等重建。
 *   - 只清理当前版本可生成的托管规则；未知用户规则始终保留。
 *
 * 运行环境：Clash Verge Rev 的 JavaScript 扩展脚本环境。
 * 入口签名：main(config, profileName)
 *
 * @file clash-verge-ai-residential.js
 */

// ============================================================
// 0. 脚本标识与保留名称
// ============================================================

const SCRIPT_VERSION = "5.9.0";
const AI_GROUP = "AI-家宽";
const HOME_PROXY_NAME = "家宽-SOCKS5";

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

// ChatGPT 产品、OpenAI 模型 API 与用户上传/生成内容；默认走家宽，可在本地 TOML 关闭。
const ROUTE_OPENAI_CORE = true;

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

// Cursor AI API、Tab、Agent、Cloud Agent 与产品专属认证；默认走家宽。
const ROUTE_CURSOR_CORE = true;

// Cursor 仓库索引主机；与 ROUTE_CURSOR_CORE 独立，默认关闭以免占用家宽。
const ROUTE_CURSOR_REPOSITORY_INDEXING = false;

// Grok Build（xAI grok CLI）推理 API 与产品域；默认走家宽，可在本地 TOML 关闭。
const ROUTE_GROK_CORE = true;

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

  // Google Antigravity 产品域
  "antigravity.google"
];

// ChatGPT 产品域；开关关闭后 GPT 流量改走机场，不再进家宽。
const OPENAI_CORE_SUFFIX_DOMAINS = [
  // ChatGPT Web / user-uploaded and generated content；通用静态 CDN 不走家宽
  "chatgpt.com",
  "oaiusercontent.com",

  // 官方模型 API。v5.7 从 exact 提升为 suffix：Codex API-key 路线使用
  // us. / eu. 数据驻留前缀（learn.chatgpt.com 配置文档），exact 会漏匹配。
  "api.openai.com"
];

const CORE_EXACT_DOMAINS = [
  // 第一方模型 API；避免 anthropic.com 宽泛后缀。
  "api.anthropic.com",
  "a-api.anthropic.com",

  // 官方网络文档列出的产品功能域（code.claude.com/docs network-config）：
  // claude.ai MCP connector 代理与桌面/网页资产代理（官方警告缺失会导致白屏）。
  "mcp-proxy.anthropic.com",
  "assets-proxy.anthropic.com",

  // Antigravity / Gemini Code Assist / Gemini Developer API / Vertex AI
  "cloudcode-pa.googleapis.com",
  "daily-cloudcode-pa.googleapis.com",
  "cloudaicompanion.googleapis.com",
  "geminicloudassist.googleapis.com",
  "generativelanguage.googleapis.com",
  "aiplatform.googleapis.com"
];

// 官方 help.openai.com/9247338 明文列出的 ChatGPT 应用主机；tcr9i 用途不明；
// 不添加 Voice UDP 3478。
const OPENAI_CORE_EXACT_DOMAINS = [
  "chat.openai.com",
  "android.chat.openai.com",
  "desktop.chat.openai.com",
  "ios.chat.openai.com",
  "tcr9i.chat.openai.com"
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

// Cursor 仅保留 AI API / Tab / Agent / 专属认证 / Cloud Agent VM 的窄范围后缀。
const CURSOR_SUFFIX_DOMAINS = [
  "api2.cursor.sh",
  "api5.cursor.sh",
  "gcpp.cursor.sh",

  // 官方授权端点与 JWT 签发后端。
  "authenticate.cursor.sh",
  "authentication.cursor.sh",

  // Cloud Agent 虚拟机服务（官方通配 *.cursorvm.com / *.*.cursorvm.com）。
  "cursorvm.com"
];

const CURSOR_EXACT_DOMAINS = [
  "api3.cursor.sh",
  "api4.cursor.sh",
  "authenticator.cursor.sh",

  // Cursor Cloud Agent / Bugbot AI API；不会匹配 cursor.com 其他页面。
  "api.cursor.com"
];

const CURSOR_CORE_DOMAIN_REGEXES = [
  // 企业 SSO 配置与域验证门户，编号可能滚动，例如 adminportal42.cursor.sh。
  "^adminportal[0-9]+\\.cursor\\.sh$"
];

const CURSOR_REPOSITORY_INDEXING_DOMAIN_REGEXES = [
  // 官方精确主机为 repo42.cursor.sh；数字通配是项目前向兼容策略，不是官方通配合同。
  "^repo[0-9]+\\.cursor\\.sh$"
];

// Grok Build 核心：cli-chat-proxy.grok.com 承载推理 API（/v1/responses）、
// 代码库与会话轨迹上传（/v1/storage*）；同主机也是 Grok 网页产品域，
// 与 chatgpt.com / claude.ai 的产品域处理保持一致。
// 官方网络文档另列的 marketplace.cursorapi.com、cursor-cdn.com、
// downloads.cursor.com、anysphere-binaries.s3.us-east-1.amazonaws.com 属于
// 市场/CDN/更新下载；Grok 的 api.mixpanel.com（分析）、x.ai（安装脚本）
// 与 storage.googleapis.com（共享 GCS，见 CLAUDE_CODE_AUXILIARY）均不走家宽。
const GROK_SUFFIX_DOMAINS = [
  "grok.com"
];

// 官方企业部署文档（docs.x.ai/build/enterprise）列出的主机：
// auth.x.ai 是 OAuth2/OIDC 认证（must-allow），api.x.ai 是 API-key 直连
// 推理端点；安装脚本域 x.ai 仍不走家宽。
const GROK_EXACT_DOMAINS = [
  "auth.x.ai",
  "api.x.ai"
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
  if (/[#&]/.test(target)) {
    throw new Error(`[${AI_GROUP}] 非 AI DNS 上游名称“${target}”不能包含 # 或 &`);
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

function requireOutboundIndex(outboundIndex) {
  if (
    !outboundIndex ||
    typeof outboundIndex !== "object" ||
    !(outboundIndex.groups instanceof Map) ||
    !(outboundIndex.proxies instanceof Map)
  ) {
    throw new Error(`[${AI_GROUP}] findOutbound 需要 outbound 索引`);
  }
}

// 键与 namedItems 一致：收录每个真值 item，用 item.name 原值（含空串）。
function buildOutboundIndex(config) {
  function indexItems(items, map) {
    if (!Array.isArray(items)) return;
    for (const item of items) {
      if (!item) continue;
      const existing = map.get(item.name);
      if (existing) {
        existing.count += 1;
      } else {
        map.set(item.name, { count: 1, value: item });
      }
    }
  }

  const groups = new Map();
  const proxies = new Map();
  indexItems(config && config["proxy-groups"], groups);
  indexItems(config && config.proxies, proxies);
  return { groups, proxies };
}

function findOutbound(outboundIndex, name) {
  requireOutboundIndex(outboundIndex);
  const groupEntry = outboundIndex.groups.get(name);
  const proxyEntry = outboundIndex.proxies.get(name);
  const groupCount = groupEntry ? groupEntry.count : 0;
  const proxyCount = proxyEntry ? proxyEntry.count : 0;

  if (groupCount > 1 || proxyCount > 1 || (groupCount === 1 && proxyCount === 1)) {
    throw new Error(
      `[${AI_GROUP}] outbound 名称“${name}”存在歧义（同名组/节点或重复定义），` +
      "无法安全用于 dialer-proxy"
    );
  }
  if (groupCount === 1) return { kind: "group", value: groupEntry.value };
  if (proxyCount === 1) return { kind: "proxy", value: proxyEntry.value };
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

function resolveCandidate(config, candidate, outboundIndex) {
  requireOutboundIndex(outboundIndex);
  if (typeof candidate !== "string" || candidate.length === 0) return null;
  if (isForbiddenUpstreamName(candidate)) return null;

  const exact = findOutbound(outboundIndex, candidate);
  if (exact) return candidate;

  const normalizedCandidate = normalizeName(candidate);
  if (!normalizedCandidate) return null;

  const normalizedMatches = allOutboundNames(config).filter(
    (name) => !isForbiddenUpstreamName(name) && normalizeName(name) === normalizedCandidate
  );

  if (normalizedMatches.length === 1) {
    findOutbound(outboundIndex, normalizedMatches[0]); // 触发同名歧义检测
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

function resolveFromCandidates(config, candidates, outboundIndex) {
  requireOutboundIndex(outboundIndex);
  for (const candidate of uniqueStrings(candidates)) {
    const resolved = resolveCandidate(config, candidate, outboundIndex);
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

function resolveUpstreamName(config, profileName, outboundIndex) {
  requireOutboundIndex(outboundIndex);
  const overrideCandidates = profileOverrideCandidates(profileName);
  const globalCandidates = uniqueStrings([
    HOME_PROXY_TEMPLATE["dialer-proxy"],
    ...UPSTREAM_CANDIDATES
  ]);

  const resolvedOverride = resolveFromCandidates(config, overrideCandidates, outboundIndex);
  if (resolvedOverride) return resolvedOverride;

  if (overrideCandidates.length > 0) {
    warn(
      `[${AI_GROUP}] Profile“${profileName}”的专属候选未命中：` +
      `${overrideCandidates.join(" / ")}；继续尝试通用候选`
    );
  }

  const resolvedGlobal = resolveFromCandidates(config, globalCandidates, outboundIndex);
  if (resolvedGlobal) return resolvedGlobal;

  const finalTarget = extractFinalRuleTarget(config.rules);
  const resolvedFinal = resolveCandidate(config, finalTarget, outboundIndex);
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
  return [HOME_PROXY_NAME, AI_GROUP];
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
  const removed = group.proxies.filter((name) => blocked.indexOf(name) !== -1);
  if (removed.length > 0) {
    warn(
      `[${AI_GROUP}] 代理组“${group.name || "<未命名>"}”中的 ` +
      `${removed.join("、")} 引用已被移除：上游组包含家宽链路会形成 ` +
      `dialer-proxy 递归。AI 流量请用规则指向 ${AI_GROUP}，` +
      `不要把 ${AI_GROUP} / ${HOME_PROXY_NAME} 放进上游代理组。`
    );
  }
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

function hardenReachableUpstreamGraph(config, upstreamName, outboundIndex) {
  requireOutboundIndex(outboundIndex);
  const groupMap = buildGroupMap(config);
  const visited = new Set();
  const visiting = new Set();
  const stack = [];
  const collectUdpWarnings = WARN_ON_REACHABLE_UDP_DISABLED === true;
  const udpDisabledNames = collectUdpWarnings ? new Set() : null;
  const udpDisabledSamples = collectUdpWarnings ? [] : null;
  let udpDisabledCount = 0;

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
      } else if (collectUdpWarnings) {
        const outbound = findOutbound(outboundIndex, childName);
        if (outbound && outbound.kind === "proxy" && outbound.value.udp === false) {
          if (udpDisabledNames.has(childName)) continue;
          udpDisabledNames.add(childName);
          udpDisabledCount += 1;
          if (udpDisabledSamples.length < 8) {
            udpDisabledSamples.push({
              name: childName,
              path: [...stack, childName]
            });
          }
        }
      }
    }

    stack.pop();
    visiting.delete(groupName);
    visited.add(groupName);
  }

  visit(upstreamName);

  if (udpDisabledCount > 0) {
    const sampleText = udpDisabledSamples
      .map((sample) => `“${sample.name}”（路径：${sample.path.join(" -> ")}）`)
      .join("、");
    const overflow = udpDisabledCount > 8 ? `……（共 ${udpDisabledCount} 个）` : "";
    warn(
      `[${AI_GROUP}] ${udpDisabledCount} 个可达节点显式关闭 UDP：` +
      `${sampleText}${overflow}。` +
      "当上游组选择这些节点时，WebRTC/STUN 可能失败或改走其他路径。"
    );
  }
}

function validateTopLevelUpstream(config, upstreamName, outboundIndex) {
  requireOutboundIndex(outboundIndex);
  const outbound = findOutbound(outboundIndex, upstreamName);
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
    ...(ROUTE_OPENAI_CORE ? OPENAI_CORE_SUFFIX_DOMAINS : []),
    ...(ROUTE_GEMINI_WEB_CORE ? GEMINI_WEB_SUFFIX_DOMAINS : []),
    ...(ROUTE_CURSOR_CORE ? CURSOR_SUFFIX_DOMAINS : []),
    ...(ROUTE_GROK_CORE ? GROK_SUFFIX_DOMAINS : []),
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
    ...(ROUTE_OPENAI_CORE ? OPENAI_CORE_EXACT_DOMAINS : []),
    ...(ROUTE_GEMINI_WEB_CORE ? GEMINI_WEB_EXACT_DOMAINS : []),
    ...(ROUTE_CURSOR_CORE ? CURSOR_EXACT_DOMAINS : []),
    ...(ROUTE_GROK_CORE ? GROK_EXACT_DOMAINS : []),
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
    ...(ROUTE_CURSOR_CORE ? CURSOR_CORE_DOMAIN_REGEXES : []),
    ...(ROUTE_CURSOR_REPOSITORY_INDEXING
      ? CURSOR_REPOSITORY_INDEXING_DOMAIN_REGEXES
      : [])
  ]);
}

function allPossibleSuffixDomains() {
  return uniqueStrings([
    ...CORE_SUFFIX_DOMAINS,
    ...OPENAI_CORE_SUFFIX_DOMAINS,
    // 从不注入 DOMAIN-SUFFIX,chat.openai.com；仅清理误注入的 suffix 规则与 +.chat.openai.com。
    "chat.openai.com",
    ...GEMINI_WEB_SUFFIX_DOMAINS,
    ...CURSOR_SUFFIX_DOMAINS,
    ...GROK_SUFFIX_DOMAINS,
    ...OPENAI_SHARED_SUFFIX_DOMAINS,
    ...CLAUDE_SHARED_SUFFIX_DOMAINS,
    ...ANTIGRAVITY_UPDATE_AND_TELEMETRY_SUFFIX_DOMAINS,
    ...PUBLIC_DOH_SUFFIX_DOMAINS
  ]);
}

function allPossibleExactDomains() {
  return uniqueStrings([
    ...CORE_EXACT_DOMAINS,
    ...OPENAI_CORE_EXACT_DOMAINS,
    // v5.6 曾以 exact 形式注入 api.openai.com；保留以清理旧版托管规则。
    "api.openai.com",
    ...GEMINI_WEB_EXACT_DOMAINS,
    ...CURSOR_EXACT_DOMAINS,
    ...GROK_EXACT_DOMAINS,
    ...OPENAI_SHARED_EXACT_DOMAINS,
    ...CLAUDE_SHARED_EXACT_DOMAINS,
    ...ANTIGRAVITY_GOOGLE_AUTH_DOMAINS,
    ...ANTIGRAVITY_PROJECT_API_DOMAINS,
    ...ANTIGRAVITY_UPDATE_AND_TELEMETRY_EXACT_DOMAINS,
    ...CLAUDE_CODE_AUXILIARY_EXACT_DOMAINS,
    ...REALTIME_EXACT_DOMAINS
  ]);
}

function allPossibleDomainRegexes() {
  return uniqueStrings([
    ...GEMINI_DOMAIN_REGEXES,
    ...CURSOR_CORE_DOMAIN_REGEXES,
    ...CURSOR_REPOSITORY_INDEXING_DOMAIN_REGEXES
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
// 11. 当前版本托管规则清理
// ============================================================

function buildManagedRuleSet() {
  const managed = new Set(buildPrivateDirectRules());
  for (const domain of allPossibleSuffixDomains()) {
    managed.add(`DOMAIN-SUFFIX,${domain},${AI_GROUP}`);
  }
  for (const domain of allPossibleExactDomains()) {
    managed.add(`DOMAIN,${domain},${AI_GROUP}`);
  }
  for (const pattern of allPossibleDomainRegexes()) {
    managed.add(`DOMAIN-REGEX,${pattern},${AI_GROUP}`);
  }
  for (const rule of buildRealtimeRules(AI_GROUP, true)) managed.add(rule);
  for (const rule of buildDnsLeakRules(AI_GROUP, true)) managed.add(rule);
  for (const rule of buildAllProcessRules(AI_GROUP)) managed.add(rule);
  for (const template of ANTHROPIC_INBOUND_IP_RULE_TEMPLATES) {
    managed.add(template.replace("{GROUP}", AI_GROUP));
  }

  return managed;
}

function cleanExistingManagedRules(rules) {
  const managed = buildManagedRuleSet();
  const result = [];

  for (const rule of Array.isArray(rules) ? rules : []) {
    if (typeof rule === "string" && managed.has(rule)) continue;
    result.push(rule);
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
  // 新版 Clash Verge Rev 在全局脚本执行后会按“权威字段”把 tun/ipv6 还原为
  // 应用设置页的值，此函数的改动在这类宿主上无效；TUN 的 dns-hijack 与
  // IPv6 开关需在 Verge 设置页配置。保留实现以兼容旧版宿主。
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

  // 1. 在任何覆盖前检查保留名称，防止静默破坏用户配置。
  validateReservedNameCollisions(config);

  // 2. 为当前 Profile 动态解析一个真实存在的上游名称。
  const outboundIndex = buildOutboundIndex(config);
  const upstreamName = resolveUpstreamName(config, profileName, outboundIndex);

  // 3. 防止 include-all / 嵌套组把家宽节点重新纳入上游，形成递归链。
  hardenAllIncludeAllGroups(config["proxy-groups"]);
  hardenReachableUpstreamGraph(config, upstreamName, outboundIndex);
  validateTopLevelUpstream(config, upstreamName, outboundIndex);

  // 4. 构建家宽 SOCKS5，dialer-proxy 始终是单一、已解析名称。
  const homeProxy = buildHomeProxy(config, upstreamName);
  validateHomeProxy(homeProxy);
  config.proxies = upsertNamedItem(config.proxies, homeProxy);

  // 5. 注入统一 AI 出口组；不提供 DIRECT 回退，故障时 fail closed。
  config["proxy-groups"] = upsertNamedItem(
    config["proxy-groups"],
    buildAiGroup(config)
  );

  // 6. 精确清理当前版本管理的规则；未知自定义规则原样保留。
  const existingRules = cleanExistingManagedRules(config.rules);
  config.rules = dedupeRuleEntries([
    ...buildInjectedRules(),
    ...existingRules
  ]);

  // 7. 重建严格 DNS 路径。
  config.dns = buildDnsConfig(config.dns, upstreamName);

  // 8. 加固用户已经启用的 TUN 与域名嗅探；进程级全量代理默认关闭。
  hardenTun(config);
  hardenSniffer(config);
  ensureProcessLookup(config);

  // 9. 统一关闭 Mihomo IPv6；操作系统层仍需由 TUN/系统路由约束。
  // 新版 Clash Verge Rev 会把 ipv6 还原为应用设置值（见 hardenTun 注释）。
  config.ipv6 = false;

  info(
    `[${AI_GROUP} v${SCRIPT_VERSION}] Profile“${profileName || "<未命名>"}”` +
    `：dialer-proxy -> ${upstreamName}`
  );
  info(
    `[${AI_GROUP}] 提示：新版 Clash Verge Rev 会在脚本执行后还原 tun/ipv6 ` +
    "等权威字段；TUN 的 dns-hijack 与 IPv6 开关请在 Verge 设置页配置。"
  );

  return config;
}

// Node.js 单元测试导出；Clash Verge 环境不存在 module，不受影响。
if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    main,
    resolveUpstreamName,
    buildOutboundIndex,
    findOutbound,
    buildDnsConfig,
    buildUpstreamDoh,
    buildInjectedRules,
    buildNameserverPolicy,
    cleanExistingManagedRules,
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
      ROUTE_OPENAI_CORE,
      OPENAI_CORE_SUFFIX_DOMAINS,
      OPENAI_CORE_EXACT_DOMAINS,
      ROUTE_GEMINI_WEB_CORE,
      ROUTE_CURSOR_CORE,
      ROUTE_CURSOR_REPOSITORY_INDEXING,
      ROUTE_GROK_CORE,
      ROUTE_CURSOR_PROCESS_FALLBACK,
      GEMINI_WEB_SUFFIX_DOMAINS,
      GEMINI_WEB_EXACT_DOMAINS,
      GEMINI_DOMAIN_REGEXES,
      CURSOR_SUFFIX_DOMAINS,
      CURSOR_EXACT_DOMAINS,
      CURSOR_CORE_DOMAIN_REGEXES,
      CURSOR_REPOSITORY_INDEXING_DOMAIN_REGEXES,
      GROK_SUFFIX_DOMAINS,
      GROK_EXACT_DOMAINS
    }
  };
}

# Clash Verge AI Residential

[![CI](https://github.com/bahayonghang/clash-verge-ai-residential/actions/workflows/ci.yml/badge.svg)](https://github.com/bahayonghang/clash-verge-ai-residential/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Clash Verge Rev 全局扩展脚本：只把 Claude、ChatGPT、Gemini、Google Antigravity 和 Cursor 的核心 AI 请求送入住宅 SOCKS5 链路，并保留原 Profile 对插件市场、下载、YouTube、共享 Google 服务及其他非 AI 流量的分流。

```text
本机 -> 当前 Profile 的机场代理组/节点 -> 家宽 SOCKS5 -> AI 服务
```

当前版本：`v5.4.0`。

## 核心边界

家宽链路包含：

- Claude / Anthropic 产品域、模型 API、MCP 和会话内容。
- ChatGPT / OpenAI 产品域、模型 API、上传与生成内容。
- Gemini Web、Google AI Studio 专用后端、Gemini Developer API、Vertex AI 区域/全局模型端点。
- Google Antigravity / Gemini Code Assist 的产品域和核心 Agent API。
- Cursor Chat、Tab、Agent、代码库索引、Cloud Agent/Bugbot 和产品专属认证。

家宽链路明确排除：

- Cursor Marketplace、扩展、更新、下载、CDN、Remote-SSH/WSL 资产。
- YouTube、Maps、Google Search、Fonts、Gstatic、广告和统计。
- OpenAI/Claude 的 Intercom、Sentry、Datadog、Stripe 等共享依赖。
- 公共 DoH/DoT、通用 STUN/TURN 和进程级全量代理。

完整边界见 [`docs/routing-scope.md`](docs/routing-scope.md)。

## 快速使用

打开 `clash-verge-ai-residential.js`，在顶部配置住宅 SOCKS5：

```javascript
const HOME_PROXY_TEMPLATE = {
  name: "家宽-SOCKS5",
  type: "socks5",
  server: "xxx",
  port: 443,
  username: "xxx",
  password: "xxx",
  udp: true,
  "dialer-proxy": "🚀节点选择"
};
```

不要把真实 endpoint 或凭据提交到公开仓库。更安全的做法是在 Profile 中预置同名 `家宽-SOCKS5` 节点，让脚本复用其配置；或者编辑被 `.gitignore` 排除的 `*.local.js` 副本。

在 Clash Verge Rev 中：

1. 打开全局扩展脚本。
2. 粘贴 `clash-verge-ai-residential.js` 内容。
3. 刷新当前 Profile。
4. 检查生成配置中 `家宽-SOCKS5.dialer-proxy` 是否指向真实机场组。
5. 用 Connections 验证 AI 请求命中 `AI-家宽`，插件市场、下载和 YouTube 不命中。

稳定 Raw 地址：

```text
https://raw.githubusercontent.com/bahayonghang/clash-verge-ai-residential/main/clash-verge-ai-residential.js
```

## 多 Profile

`dialer-proxy` 只能写一个名称。脚本按以下顺序解析：

1. 当前 Profile 在 `PROFILE_UPSTREAM_OVERRIDES` 中的候选。
2. 默认值 `🚀节点选择`。
3. `UPSTREAM_CANDIDATES` 中常见组名。
4. 当前配置最后一个 `MATCH` / `FINAL` 目标。
5. 全部失败则抛错，不静默回落到 `DIRECT`。

配置和限制见 [`docs/multi-profile.md`](docs/multi-profile.md)。

## DNS 行为

```text
AI 域名 DNS       -> AI-家宽 -> 家宽 SOCKS5
其他海外域名 DNS  -> 当前 Profile 的机场上游
中国域名 DNS      -> 国内 DoH / DIRECT
私有与局域网域名  -> system
```

因此普通 DNS leak test 不一定显示住宅地区。项目保证的是 AI 请求及其域名解析路径一致，而不是让所有 DNS 流量占用住宅出口。威胁边界见 [`docs/dns-and-leak-model.md`](docs/dns-and-leak-model.md)。

## 本地验证

需要 Node.js 18 或更高版本，无第三方依赖：

```bash
npm run ci
```

包含：

- JavaScript 语法检查。
- 28 项配置级回归测试。
- 公共模板凭据与常见 token 安全检查。
- Gemini/Cursor 核心域名正向测试。
- YouTube、Maps、Marketplace、下载、CDN 和静态资源负向测试。
- 多 Profile 解析、循环检测、DNS 收敛、迁移与幂等测试。

Node.js 测试不替代真实 Clash Verge Rev JavaScript 引擎、Mihomo 内核和订阅 Profile 集成测试。

## 文档

- [`docs/configuration.md`](docs/configuration.md)：凭据、开关和 Clash Verge 设置。
- [`docs/routing-scope.md`](docs/routing-scope.md)：AI-only 路由准入与排除标准。
- [`docs/multi-profile.md`](docs/multi-profile.md)：上游解析和递归保护。
- [`docs/dns-and-leak-model.md`](docs/dns-and-leak-model.md)：DNS 路径和不能单独解决的泄漏面。
- [`docs/troubleshooting.md`](docs/troubleshooting.md)：常见故障定位。
- [`SECURITY.md`](SECURITY.md)：安全报告与凭据泄漏处置。

## Domain 变更规则

新增域名必须提供官方资料或脱敏 Connections 证据，并补充负向测试。宽泛 provider suffix、插件市场、更新、下载、媒体、广告、统计和共享基础设施默认不接受。

可使用仓库的 **AI domain request** Issue 模板提交。

## 参考资料

- [Clash Verge Rev](https://github.com/clash-verge-rev/clash-verge-rev)
- [Mihomo `dialer-proxy`](https://wiki.metacubex.one/en/config/proxies/dialer-proxy/)
- [Mihomo DNS configuration](https://wiki.metacubex.one/en/config/dns/)
- [Mihomo rules](https://wiki.metacubex.one/en/config/rules/)
- [Anthropic IP addresses](https://docs.anthropic.com/en/api/ip-addresses)
- [Google Gemini API](https://ai.google.dev/gemini-api/docs)
- [Google AI Studio](https://ai.google.dev/aistudio)

## License

[MIT](LICENSE)

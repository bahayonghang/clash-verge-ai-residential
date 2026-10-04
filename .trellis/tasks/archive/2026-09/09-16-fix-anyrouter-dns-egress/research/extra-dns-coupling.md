# Research: extra 小站规则与住宅 DNS 的耦合

- Query: 为何 `anyrouter.top` 业务走 `AI-家宽` 时 DNS 也会走住宅 DoH
- Scope: internal
- Date: 2026-09-16

## Findings

### Files Found

| File Path | Description |
|---|---|
| `clash-verge-ai-residential.js` | extra 域名清单、活动后缀、规则与 `nameserver-policy` 同源 |
| `tests/regression.test.js` | extra 开/关同时断言规则与 `+.anyrouter.top` |
| `.trellis/tasks/archive/2026-09/09-16-extra-routing-group/prd.md` | 当初把「后缀规则 + 住宅 DNS」写成同一条需求 |

### Code Patterns

`EXTRA_SUFFIX_DOMAINS` 只有 `anyrouter.top`：

```360:362:clash-verge-ai-residential.js
const EXTRA_SUFFIX_DOMAINS = [
  "anyrouter.top"
];
```

`ROUTE_EXTRA` 为真时，它进入 `activeSuffixDomains()`（约 1254–1263 行）。`buildDomainRules` 把每个活动后缀写成 `DOMAIN-SUFFIX`（约 1384–1386 行）。`buildNameserverPolicy` 对**同一份**活动后缀再写 `+.${domain} = RESIDENTIAL_DOH`（约 1565–1567 行）。没有「只路由、不改 DNS」的例外表。

住宅 DoH 本身是 Cloudflare/Google，但查询出口是家宽组：

```503:506:clash-verge-ai-residential.js
const RESIDENTIAL_DOH = [
  `https://1.1.1.1/dns-query#${AI_GROUP}&disable-ipv6=true`,
  `https://8.8.8.8/dns-query#${AI_GROUP}&disable-ipv6=true`
];
```

非 AI 默认 `nameserver` 是同一对 DoH 主机，但 `#` 后是机场节点（`buildUpstreamDoh`，约 513–527 行）。GeoDNS 看到的是机场出口，不是家宽出口。

关闭 `ROUTE_EXTRA` 时，活动清单不再含 extra，但 `allPossibleSuffixDomains()` 仍含 `EXTRA_SUFFIX_DOMAINS`（约 1319 行），`buildManagedDnsPolicyKeySet` 仍管理 `+.anyrouter.top`（约 1545 行）。因此可以只停用 DNS 策略、保留清理键，而不必把域名移出托管全集。

仓库里已有「路由与 DNS 不对齐」的先例：`quality-guidelines.md` 写明 regex-only 路由没有对应 `nameserver-policy`。extra 目前没有走这条路径。

上一任务 `09-16-extra-routing-group` 的 PRD 第 3 条把「`DOMAIN-SUFFIX` + 住宅 DNS」绑成同一需求；本轮故障说明这条耦合对 GeoDNS/ESA 站点不安全。

### Related Specs

- `.trellis/spec/frontend/quality-guidelines.md` — 后缀策略键必须断言 `+.${host}`；域名变更要有官方或脱敏连接证据
- `.trellis/spec/guides/index.md` — 路径不能从 host 规则里拆开；`/console` 与 `/` 同主机

## Caveats / Not Found

- 没有现成的 `dnsExemptSuffixDomains`（或同等）辅助函数。
- Node 回归不能证明真实 TLS；真机验收仍要脱敏 Profile + Connections。

# Research: AnyRouter 根域控制台 DNS / TLS

- Query: 不用 `www`，以官方控制台 `https://anyrouter.top/console` 复核 DNS 出口与 TLS
- Scope: mixed
- Date: 2026-09-16
- Vantage: 本机 Windows，Clash Verge / verge-mihomo 运行中，Meta TUN `198.18.0.2`

## Findings

### Official product URLs

| Role | URL | Source |
|---|---|---|
| 控制台 | `https://anyrouter.top/console` | 用户提供；健康边缘 HTTP 200 |
| API Base | `https://anyrouter.top` | [docs.anyrouter.top](https://docs.anyrouter.top/)：`ANTHROPIC_BASE_URL=https://anyrouter.top` |
| 文档站 | `https://docs.anyrouter.top/` | EdgeOne Pages，与主站不同 CDN |

官方文档与 Claude Code 配置都不使用 `www`。`anyrouter.dev` 是另一套文档/产品，不在本任务范围。

### Three DNS views for `anyrouter.top`

hosts 删除前（2026-09-16 上午）：

| Path | How measured | Answer | TLS to `anyrouter.top` /console |
|---|---|---|---|
| 本机系统 / Clash UDP 53 | `Resolve-DnsName`，含 `-NoHostsFile` 与 `-Server 1.1.1.1/8.8.8.8/223.5.5.5` | `47.246.23.200`（TTL 10；hosts 命中时 TTL ~526000） | 握手失败：`schannel: failed to receive handshake` |
| 机场/本机 HTTPS DoH | `https://1.1.1.1/dns-query`、`cloudflare-dns.com`、`dns.google` | CNAME `anyrouter.top.a1.initrr.com` → `43.109.133.203/204/205` | TLS 1.3，`CN=anyrouter.top`，`/console` 与 `/` 均为 HTTP 200 |
| 国内 DIRECT DoH | AliDNS / doh.pub JSON | CNAME `bestali.030101.xyz` → `bestcf.030101.xyz` → `104.17.185.207` / `104.17.101.139` | `SEC_E_ILLEGAL_MESSAGE`，证书/握手不可用 |

`43.109.133.202`（PRD 旧证据）钉住 SNI 后对 `/console` 也是 HTTP 200，`Via: ens-cache*.hk57`。

PowerShell `-NoHostsFile` 只跳过 Windows 解析器自己的 hosts，**仍会问 Clash DNS**。Mihomo 会读系统 hosts，所以当时 UDP 路径返回 `47.246.23.200` 不能单独证明住宅 DoH 的 GeoDNS 结果。

### Local hosts pin (removed)

`%SystemRoot%\System32\drivers\etc\hosts` 曾有第 23 行 `47.246.23.200 anyrouter.top`（`LastWriteTime` 2026-09-15 10:54:29）。2026-09-16 经提升权限删除；Docker 段保留。`ipconfig /flushdns` 后该文件不再含 `anyrouter`。

hosts 会让 Mihomo 把根域当成真实 IP：业务连接变成访问 `47.246.23.200:443`，`DOMAIN-SUFFIX` / SOCKS5 域名寻址都可能失效。钉住该 IP 的 TLS 仍然失败，与是否走 Clash 无关。

### After hosts removal (2026-09-16 09:44)

| Check | Result |
|---|---|
| `Resolve-DnsName` / `-NoHostsFile` / `-Server 1.1.1.1` | Clash fake-ip `198.18.0.36` TTL 1 |
| HTTPS DoH `1.1.1.1` / `dns.google` | 仍是 `anyrouter.top.a1.initrr.com` → `43.109.133.205` |
| `curl -I https://anyrouter.top/console` | HTTP 200，`ip=198.18.0.36`，`Server: ESA`，`Via: ens-cache8.hk57` |
| `curl -I https://anyrouter.top/` | HTTP 200，同一 fake-ip |
| 浏览器 `https://anyrouter.top/console` | 可打开，重定向到 `https://anyrouter.top/login`（Any Router 登录页） |

官方控制台在去掉 hosts 后，经当前 Clash 配置即可完成 TLS。`docs/dns-and-leak-model.md` 说明 SOCKS5 出站通常转发主机名，由住宅 SOCKS5 递归解析；这与「fake-ip + 域名规则」通路一致，也解释了为何不必再连 `47.246.23.200`。

潜在风险仍在：若 Mihomo 因 hosts、fake-ip-filter 或 L3 出站必须自己解析，住宅 DoH / 钉死 IP 仍可能再次选中 `47.246.23.200`。本轮未从控制器 `/connections` 取链（`127.0.0.1:9097` 返回 401，未读本地 secret）。

`www.anyrouter.top` 与 `docs.anyrouter.top` 不在 hosts 里，走 Clash fake-ip（`198.18.0.8` / `198.18.0.6`）。

### `www` is not the official host

`www.anyrouter.top` CNAME → `anyrouter.top`。钉到健康边缘 `43.109.133.204/205` 仍是 HTTP 530，`Server: ESA`。这是源站未配置 `www` 主机名，不是本仓库 DNS 策略能修的。

### Console / API on the healthy edge

钉住 `anyrouter.top:443:43.109.133.204`（无鉴权，只看状态）：

| Path | Status |
|---|---|
| `/console` | 200 |
| `/` | 200 |
| `/v1/models` | 401（端点存在） |
| `/v1/messages` | 404 |
| `/api` | 200 |
| `/api/v1/models` | 200 |

证书：`CN=anyrouter.top`，TLS 1.3 `TLS_AES_256_GCM_SHA384`。响应头有 `Server: ESA`、`X-Tengine-Error: denied by http_custom`；带浏览器 UA 的 GET 正文是阿里云 `acw_sc__v2` JS 挑战（约 4KB），不是站点宕机。Cursor 内置浏览器打开 `/console` 得到 `chrome-error://chromewebdata/`，是系统 DNS/hosts 先撞上 `47.246.23.200` 握手失败，没有走到 WAF。

### `docs.anyrouter.top` is a different CDN

DoH（1.1.1.1 与 AliDNS 一致）：CNAME `*.dns.edgeone.app` / `docs.anyrouter.top.pages.dnsoe9.com` → `43.174.246.103` / `43.174.247.103`。经 Clash fake-ip 访问文档站 HTTP 200，`Server: edgeone-pages`。把 `docs` SNI 钉到主站 ESA `43.109.133.204` 会证书主体不匹配。文档站当前能通，不能用来证明主站控制台健康。

### Code Patterns

| File Path | Description |
|---|---|
| `clash-verge-ai-residential.js` | `EXTRA_SUFFIX_DOMAINS` 进入 `activeSuffixDomains()` 后，规则与 `+.anyrouter.top` 住宅 DNS 一起生成 |
| `tests/regression.test.js` | 默认断言 `+.anyrouter.top` 映射 `RESIDENTIAL_DOH` |
| `docs/routing-scope.md` | 产品入口写成 `https://anyrouter.top/`，未写 `/console` |

### External References

- [AnyRouter Claude Code 文档](https://docs.anyrouter.top/) — `ANTHROPIC_BASE_URL=https://anyrouter.top`
- Alibaba ESA / Tengine WAF — 健康边缘上的 `acw_sc__v2` 与 `X-Tengine-Error`

### Related Specs

- `.trellis/spec/frontend/quality-guidelines.md` — 后缀 DNS 键是 `+.${host}`；Node 测试不能声称第三方站点可用
- `.trellis/spec/guides/index.md` — Clash 只匹配 host/SNI，不能按 `/console` 与 `/` 拆路由

## Caveats / Not Found

- 未把 DoH 查询本身绑到家宽 SOCKS5 上重放；住宅视图由 Clash 劫持 UDP 53 的本机结果代理。
- 未登录控制台，未调用带令牌的 `/v1/models`。
- hosts 已于 2026-09-16 删除并 flush DNS。未从 Clash 控制器拉取 Connections 链。
- `anyrouter.dev` 未探测，按范围排除。

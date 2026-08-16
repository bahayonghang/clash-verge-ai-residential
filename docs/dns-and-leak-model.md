# DNS and Leak Model

## Data paths

```text
AI application request
  -> Mihomo rule match
  -> AI-家宽
  -> 家宽-SOCKS5
  -> current Profile upstream selected by dialer-proxy
  -> residential exit
  -> AI service
```

The transport connection to the residential SOCKS5 server is dialed through the selected airport group. The external service sees the residential exit, while the local machine does not directly connect to the residential endpoint when chaining is required.

## DNS paths

```text
AI domain query       -> residential DoH via AI-家宽
Other overseas query -> non-AI DoH via current Profile upstream
Chinese domain query -> domestic DoH via DIRECT
Private/LAN query    -> system resolver
Proxy-server lookup  -> bootstrap/direct resolver to avoid recursion
```

This is intentionally different from a global DNS-leak-test configuration. A generic DNS leak test may show the airport or domestic resolver because non-AI queries are not sent through the residential route. The invariant is narrower: AI domain resolution and AI application connections must use the intended residential path.

解析时序说明：在 `enhanced-mode: fake-ip` 下，大多数 A/AAAA 查询直接由 fake-ip 地址池应答，不会访问上游解析器。因此，`nameserver-policy` 主要作为回退路径，用于命中 fake-ip-filter、非 A/AAAA 查询类型，以及 L3 出站所需的真实 IP 查询。SOCKS5 出站会直接转发主机名（RFC 1928 域名寻址），所以 AI 连接通常由住宅 SOCKS5 服务器完成实际递归解析，这符合预期。Mihomo 必须自行解析 AI 域名时，策略条目仍可保证使用住宅侧解析。

## Clash Verge Rev 强制恢复的字段

当前版本的 Clash Verge Rev 会在全局脚本运行前保存控制平面字段（`tun`、`ipv6`、`mode`、端口等），并在运行后恢复。因此，脚本中的 `hardenTun` DNS 劫持补全和 `config.ipv6 = false` 在这些宿主上不会生效；脚本保留这些逻辑是为了兼容旧版宿主，并会记录一条 `info` 提示。请在 Clash Verge Rev 设置页面关闭 IPv6，并在 TUN 设置中配置 DNS 劫持。脚本重建的 DNS 服务器、`nameserver-policy` 和 fake-ip 字段会保留；但启用 Clash Verge Rev 的 DNS 覆盖后，`dns.ipv6` 也会从应用设置恢复。

## geosite 依赖

`nameserver-policy` 中的 `geosite:cn` 和 `geosite:private` 依赖 `geosite.dat`。Mihomo 首次使用时会自动下载该文件；全新安装且处于离线状态时，下载失败会导致配置解析失败。Clash Verge Rev 会将其显示为配置验证失败；若发生在首次启动，应用会回退到最小默认配置。大多数订阅配置已包含需要下载同一文件的地理规则，因此该风险主要影响全新离线安装。处理方法见[故障排查](troubleshooting.md)。

## Strict-DNS performance trade-off

The script deliberately rebuilds DNS policy instead of inheriting arbitrary subscription paths. Real non-AI overseas lookups are sent through DoH bound to the current Profile upstream. When a GEOIP fallback needs the first real lookup for a new domain, establishing that path can add roughly one airport round trip; subsequent cache hits do not pay the same lookup cost. v5.5 retains this trade-off for resolver consistency and pollution resistance.

## Login and model exit split

Shared authentication hosts are outside the default AI-only scope. `auth.openai.com` and `accounts.google.com` therefore use the original Profile, while core chat/model requests use the residential exit. A strict risk-control system can observe different login and model-traffic IPs and request additional verification. The script does not add either shared authentication host merely to hide this split; opt-in shared-dependency switches should be enabled only with evidence and an understood scope.

## What the script mitigates

- DNS divergence between AI application traffic and AI domain resolution.
- Accidental residential routing of unrelated media, marketplace, download, and shared-service traffic.
- Recursive chaining through `include-all` groups.
- IPv6 use inside Mihomo configuration.
- Missing TUN DNS interception entries when TUN is already enabled.

## What the script cannot guarantee alone

- Operating-system traffic that bypasses Clash Verge Rev.
- Browser or application private DoH to an unknown endpoint.
- Mihomo/TUN 路由之外的 IPv6 泄漏。对于当前 Clash Verge Rev 宿主，还需检查应用自身的 IPv6 开关，因为宿主会恢复脚本设置的 `ipv6: false`。
- 当前所选服务商节点的 UDP 支持。机场订阅节点经常省略 `udp` 字段，而 Mihomo 默认将其视为 `false`；除非服务商明确设置 `udp: true`，否则链路会静默丢弃 UDP 流量。
- Runtime selector choices such as `DIRECT`.
- WebRTC behavior of arbitrary applications when shared STUN/TURN capture is disabled.

Use the operating system firewall, TUN mode, browser DNS settings, and sanitized connection inspection as complementary controls.

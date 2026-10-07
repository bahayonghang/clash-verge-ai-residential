# DNS 与泄漏模型

## 数据路径

```text
AI 应用请求
  -> Mihomo 规则匹配
  -> AI-家宽
  -> 经 dialer-proxy 选出的当前 Profile 上游连接家宽 endpoint
  -> 家宽-SOCKS5 / 住宅出口
  -> AI 服务
```

到住宅 SOCKS5 的传输连接经选中的机场组拨出。外部服务看到的是住宅出口；需要链式拨号时，本机不直连住宅 endpoint。

## DNS 路径

```text
启用的 exact/suffix/regex AI 域名查询 -> 经 AI-家宽 的住宅 DoH
其他海外查询        -> 经当前 Profile 上游的非 AI DoH
国内域名查询        -> 经 DIRECT 的国内 DoH
私网/局域网查询     -> 系统解析器
代理服务器自身查找  -> bootstrap/直连解析，避免递归
```

这与「全局 DNS leak test」配置不同。普通 leak test 可能显示机场或国内解析器，因为非 AI 查询不走家宽。配置为启用的 exact/suffix AI 域名建立住宅策略，区域 Vertex 与可选 Cursor 索引正则通过专用本地规则集建立住宅策略；实际路径仍需结合宿主设置验证。

解析时序：在 `enhanced-mode: fake-ip` 下，大多数 A/AAAA 查询由 fake-ip 地址池直接应答，不会访问上游解析器。因此 `nameserver-policy` 主要是回退路径，用于 fake-ip-filter、非 A/AAAA 查询，以及 L3 出站需要的真实 IP。SOCKS5 出站会直接转发主机名（RFC 1928 域名寻址），AI 连接通常由住宅 SOCKS5 服务器完成实际递归解析，这符合预期。Mihomo 必须自行解析时，exact/suffix 条目与正则规则集策略指定住宅侧解析。

## 正则 DNS 规则集与配置失败的边界

区域 Vertex 的 `DOMAIN-REGEX` 和启用时的 Cursor `repo[0-9]+.cursor.sh` 同时生成专用规则集 `AI-家宽-DNS-REGEX`。该规则集使用 `type: inline`、`behavior: classical`，payload 直接取自现有业务正则，不下载远程规则。`nameserver-policy` 通过 `rule-set:AI-家宽-DNS-REGEX` 将匹配查询指向住宅 DoH；`vertex_ai_endpoints` 与 `cursor_repository_indexing` 分别控制各自模式，两者全关时移除托管规则集和策略键。脚本不添加 `+.googleapis.com`、`+.cursor.sh`、标签内星号或裸正则 DNS 键。

`AI-家宽-DNS-REGEX` 是保留名称。已有同名规则集必须具有脚本生成的三个字段和已知正则 payload；额外字段、远程来源或未知模式会报错。自定义 `RULE-SET`（包括 `sub-rules`）、其他复合 DNS 策略、`dns.fake-ip-filter`、`sniffer.skip-domain` 和 `sniffer.force-domain` 不得引用这个专用名称，避免关闭开关后留下悬空引用。未知普通规则集保持原样，所有拒绝均不修改输入。

该机制已在 Mihomo v1.19.32 的隔离环境通过真实 DNS 查询与两级本地 SOCKS5 选路验证。其他内核版本需先验证支持情况；内核不支持时应修正版本或配置，不应扩大域名后缀。隔离验证不证明公网 DoH TLS、用户住宅服务商的 UDP 能力或固定公网 IP。fake-IP 应答、SOCKS 域名转发也不能替代实际查询证据。

回退旧脚本时，应从原始 Profile 重新执行；旧脚本不能识别并清理新增的规则集与策略键，不要只把旧脚本套在新输出上。

成功生成的 `AI-家宽` 组只有家宽 SOCKS5 成员。已有同名组的额外节点来源或筛选字段会报错；但 Clash Verge Rev 在脚本抛错后可能放弃脚本输出并使用原 Profile。拒绝生成配置不能证明流量被阻断；应先修正错误并确认配置生效，再验证业务命中链。供应商实际是否提供固定 IP 也需另行确认。

## Clash Verge Rev 强制恢复的字段

当前 Clash Verge Rev 会在全局脚本运行前保存设置页管理的字段，运行后再恢复。这些字段包括：

- 顶层控制平面字段：`external-controller*`、`secret`、各端口、`mode`、`allow-lan`、`log-level`、`ipv6`、`unified-delay`。
- `tun.enable`，以及设置页已保存的 TUN 字段：`stack`、`device`、`auto-route`、`route-exclude-address`、`auto-redirect`（Linux）、`auto-detect-interface`、`dns-hijack`、`strict-route`、`mtu`。
- 启用 DNS 覆盖时：`dns_config.yaml` 中所有非空 `dns.*` 字段，以及非空 `hosts`。

Clash Verge Rev v2.5.5 起，扩展改写这些字段后，应用会弹出提示「Extensions wrote …, which Settings manages, so those values were discarded」，并丢弃写入的值。

因此脚本不写 `tun` 下任何键，也不写顶层 `ipv6`，只读取并在不安全时输出 `warn`：

- TUN 已启用且 `dns-hijack` 缺少 `any:53` 或 `tcp://any:53` 时告警。`runtime.harden_existing_tun_dns_hijack = false` 时跳过。
- `runtime.harden_existing_tun_dns_hijack` 与 `runtime.enable_tun_strict_route` 均为 `true`、TUN 已启用且 `strict-route` 未开启时告警。
- 顶层 `ipv6: true` 时告警。

请在设置页关闭 IPv6，并在 TUN 设置里配置 DNS 劫持。DNS 覆盖关闭时，脚本重建的 DNS 服务器、`nameserver-policy` 和 fake-ip 会保留。DNS 覆盖开启时，`dns_config.yaml` 中非空的 `dns.*` 字段全部以设置页为准，脚本对这些字段的改写会被丢弃，并可能出现 `dns.*` 提示。

## geosite 依赖

`nameserver-policy` 中的 `geosite:cn` 和 `geosite:private` 依赖 `geosite.dat`。Mihomo 首次使用会下载该文件；全新离线安装下载失败会导致配置解析失败。Clash Verge Rev 显示为配置验证失败；若发生在首次启动，应用会回退到最小默认配置。大多数订阅已经包含需要同一文件的地理规则，因此风险主要在全新离线安装。处理见 [故障排查](troubleshooting.md)。

## 严格 DNS 的性能取舍

脚本故意重建 DNS 策略，不继承订阅里任意的解析路径。真实的非 AI 海外查找经绑定到当前 Profile 上游的 DoH 发出。GEOIP 回退第一次为新域名做真实查找时，大约多一次机场往返；缓存命中不再付这次建立成本。该取舍保留，是为了解析一致和抗污染。

## 登录与模型出口分裂

`routing.openai_auth` 与 `routing.antigravity_google_auth` 默认开启，因此 `auth.openai.com` 和 `accounts.google.com` 与核心聊天/模型请求同走住宅出口。`oaistatic.com` 以及 WorkOS、Intercom、Stripe、Cloudflare Challenge、Sentry、Datadog 等共享依赖默认仍走原 Profile。这些第三方跳转仍可能让风控看到不同出口。共享依赖开关只在有证据且理解范围后打开。

## 脚本能缓解的

- 启用的 exact/suffix AI 域名及区域 Vertex、可选 Cursor 索引正则在配置内的业务与 DNS 路径分叉。
- 无关媒体、市场、下载和共享服务误进家宽。
- 经 `include-all` 组的递归链路。
- DNS 层的 IPv6 查询（重建的 `dns.ipv6: false`，DNS 覆盖关闭时生效）。
- 顶层 `ipv6` 开启，或 TUN 已启用但缺失 DNS 拦截项：脚本输出 `warn`，需用户在设置页修改。

## 脚本单独保证不了的

- 绕过 Clash Verge Rev 的操作系统流量。
- 浏览器或应用把私有 DoH 指到未知 endpoint。
- Mihomo/TUN 路由之外的 IPv6 泄漏。顶层 `ipv6` 由 Clash Verge Rev 设置页的 IPv6 开关决定，脚本不写入该字段。
- 当前所选机场节点的 UDP 能力。订阅节点经常省略 `udp`，Mihomo 默认视为 `false`；服务商未显式 `udp: true` 时，链路会静默丢弃 UDP。
- 运行时选择器选中 `DIRECT` 这类值。
- 未开启共享 STUN/TURN 捕获时，任意应用的 WebRTC 行为。

用操作系统防火墙、TUN、浏览器 DNS 设置和脱敏连接检查作为补充控制。

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
启用的 exact/suffix AI 域名查询 -> 经 AI-家宽 的住宅 DoH
其他海外查询        -> 经当前 Profile 上游的非 AI DoH
国内域名查询        -> 经 DIRECT 的国内 DoH
私网/局域网查询     -> 系统解析器
代理服务器自身查找  -> bootstrap/直连解析，避免递归
```

这与「全局 DNS leak test」配置不同。普通 leak test 可能显示机场或国内解析器，因为非 AI 查询不走家宽。配置对启用的 exact/suffix AI 域名写入住宅侧解析策略；正则域名存在下述例外，实际路径仍需结合宿主设置验证。

解析时序：在 `enhanced-mode: fake-ip` 下，大多数 A/AAAA 查询由 fake-ip 地址池直接应答，不会访问上游解析器。因此 `nameserver-policy` 主要是回退路径，用于 fake-ip-filter、非 A/AAAA 查询，以及 L3 出站需要的真实 IP。SOCKS5 出站会直接转发主机名（RFC 1928 域名寻址），AI 连接通常由住宅 SOCKS5 服务器完成实际递归解析，这符合预期。Mihomo 必须自行解析时，已有 exact/suffix 策略指定住宅侧解析；不能把这项覆盖推广到正则域名。

## 正则域名与配置失败的边界

区域 Vertex 的 `DOMAIN-REGEX` 和启用时的 Cursor `repo[0-9]+.cursor.sh` 只有业务路由，没有等价 `nameserver-policy`。这些主机需要本地真实查询时，可能使用默认的非 AI DoH；fake-IP 应答或 SOCKS 域名转发不证明所有查询都同出口。`respect-rules` 管理 DNS 连接，不会自动把业务正则转换为解析策略。脚本保留窄匹配，不添加 `+.googleapis.com`、`+.cursor.sh` 等宽策略。真实 DNS/UDP 路径仍为 UNVERIFIED。

成功生成的 `AI-家宽` 组只有家宽 SOCKS5 成员。已有同名组的额外节点来源或筛选字段会报错；但 Clash Verge Rev 在脚本抛错后可能放弃脚本输出并使用原 Profile。拒绝生成配置不能证明流量被阻断；应先修正错误并确认配置生效，再验证业务命中链。供应商实际是否提供固定 IP 也需另行确认。

## Clash Verge Rev 强制恢复的字段

当前 Clash Verge Rev 会在全局脚本运行前保存控制平面字段（`tun`、`ipv6`、`mode`、端口等），运行后再恢复。因此脚本里的 `hardenTun` DNS 劫持补全和 `config.ipv6 = false` 在这些宿主上不生效；脚本保留这些逻辑是为了兼容旧宿主，并打一条 `info`。请在设置页关闭 IPv6，并在 TUN 设置里配置 DNS 劫持。脚本重建的 DNS 服务器、`nameserver-policy` 和 fake-ip 会保留；但若启用 Clash Verge Rev 的 DNS 覆盖，`dns.ipv6` 也会从应用设置恢复。

## geosite 依赖

`nameserver-policy` 中的 `geosite:cn` 和 `geosite:private` 依赖 `geosite.dat`。Mihomo 首次使用会下载该文件；全新离线安装下载失败会导致配置解析失败。Clash Verge Rev 显示为配置验证失败；若发生在首次启动，应用会回退到最小默认配置。大多数订阅已经包含需要同一文件的地理规则，因此风险主要在全新离线安装。处理见 [故障排查](troubleshooting.md)。

## 严格 DNS 的性能取舍

脚本故意重建 DNS 策略，不继承订阅里任意的解析路径。真实的非 AI 海外查找经绑定到当前 Profile 上游的 DoH 发出。GEOIP 回退第一次为新域名做真实查找时，大约多一次机场往返；缓存命中不再付这次建立成本。该取舍保留，是为了解析一致和抗污染。

## 登录与模型出口分裂

共享认证主机不在默认 AI-only 范围。因此 `auth.openai.com` 和 `accounts.google.com` 走原 Profile，核心聊天/模型请求走住宅出口。严格风控可能看到登录 IP 与模型流量 IP 不同，并要求额外验证。脚本不会仅为掩盖这次分裂而加入共享认证主机；共享依赖开关只在有证据且理解范围后打开。

## 脚本能缓解的

- 启用的 exact/suffix AI 域名在配置内的业务与 DNS 路径分叉。
- 无关媒体、市场、下载和共享服务误进家宽。
- 经 `include-all` 组的递归链路。
- Mihomo 配置内部使用 IPv6。
- TUN 已启用时缺失的 TUN DNS 拦截项。

## 脚本单独保证不了的

- 绕过 Clash Verge Rev 的操作系统流量。
- 浏览器或应用把私有 DoH 指到未知 endpoint。
- Mihomo/TUN 路由之外的 IPv6 泄漏。对当前 Clash Verge Rev 宿主，还要看应用自己的 IPv6 开关，因为宿主会恢复脚本设置的 `ipv6: false`。
- 当前所选机场节点的 UDP 能力。订阅节点经常省略 `udp`，Mihomo 默认视为 `false`；服务商未显式 `udp: true` 时，链路会静默丢弃 UDP。
- 运行时选择器选中 `DIRECT` 这类值。
- 未开启共享 STUN/TURN 捕获时，任意应用的 WebRTC 行为。

用操作系统防火墙、TUN、浏览器 DNS 设置和脱敏连接检查作为补充控制。

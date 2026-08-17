# clash-verge-ai-residential.js 性能分析（2026-08-17）

分析对象：仓库根目录 `clash-verge-ai-residential.js` v5.8.0。
测量环境：本机 Node（V8），`main()` 对合成机场订阅跑一遍；宿主 Clash Verge Rev 使用 boa_engine 0.21，速度低于 V8。

## 1. 两条路径必须分开看

| 路径 | 何时发生 | 谁执行 | 用户能否感觉到 |
|---|---|---|---|
| 配置生成 | 每次 Profile 激活 / 刷新；无独立 Profile 脚本时同一全局脚本跑两遍 | `main(config, profileName)` | Profile 切换卡顿；超时或日志超额会使脚本改动被丢弃 |
| 连接匹配 | 每一条 TCP/UDP 连接 | Mihomo `tunnel.match()` 自上而下遍历 `rules` | 通常远小于链路 RTT；进程匹配、嗅探、首次 DNS 才可能被感觉到 |

脚本本身不做网络 I/O。真正的建连时延由「本机 → 机场节点 → 家宽 SOCKS5 → AI 服务」决定，这是产品设计，不是实现缺陷。

## 2. 配置生成路径（已测量）

### 2.1 热点：`warnForUdpDisabledLeaf` + `findOutbound`

默认 `WARN_ON_REACHABLE_UDP_DISABLED = true`（`clash-verge-ai-residential.js:167`）。
`hardenReachableUpstreamGraph`（`:989`）从上游组 DFS；子节点若不是组，则调用：

```js
warnForUdpDisabledLeaf(config, childName, [...stack, childName]);
```

`warnForUdpDisabledLeaf`（`:976`）每次调用 `findOutbound`。
`findOutbound`（`:674`）用 `namedItems`（`:572`）对 `proxy-groups` 和 `proxies` 各做一次 `filter`。

当顶层 `select`/`url-test` 直接挂 N 个节点时，复杂度约为 `O(N × P)`（N = 可达叶子数，P = `proxies.length`）。

本机 Node 测量（每 20 个节点有 1 个显式 `udp: false`，不含 JSON 深拷贝）：

| 节点数 | 警告开（默认） | 警告关 |
|---|---|---|
| 1000 | 12.8 ms / 50 条 warn | 1.5 ms |
| 2000 | 26.7 ms / 100 条 warn | 1.1 ms |
| 4000 | 97.3 ms / 200 条 warn | 0.9 ms |

警告关闭后耗时近似常数；打开后近似平方增长（4000 / 2000 ≈ 2，耗时 97 / 27 ≈ 3.6）。

V8 上 4000 节点约 100 ms。boa_engine 通常慢一个数量级。脚本在默认 Verge 配置下会跑两遍（见已归档研究 `08-16-script-v5.7-hardening/research/clash-verge-rev-host-behavior.md` 第 2 条）。两遍都走完整 DFS 并重复输出 UDP 警告。

### 2.2 宿主上限（已核实，不是推测）

Clash Verge Rev `script.rs`（commit `749b6c9` 及后续）：

- 执行超时 5 秒。超时后脚本失败，enhance 丢弃全部脚本改动，回退到脚本前配置（AI 流量不再进家宽）。
- `console.*` 最多 1000 条、合计 1 MB。超限时 `console.warn` 抛错。
- 本脚本的 `warn()`（`:614`）没有 try/catch。
- Verge 把参数 `JSON.stringify` 后再记日志。

`udp === false` 只统计显式关闭，省略 `udp` 字段不会告警。若订阅把大量节点写成 `udp: false`，警告条数 ≈ 叶子数。1000 条后下一次 `console.warn` 抛错，`main` 中断，家宽注入丢失。

boa 的 1000 万次循环上限主要约束 JS `for`/`while`/`for-of`。`Array.filter` 在 boa 里是原生实现，4000 次叶子扫描不一定触发该上限。更现实的失败模式是 5 秒超时和 1000 条日志上限。

### 2.3 其余生成期成本（已排除为次要）

- `activeSuffixDomains` / `allPossible*` 被多次调用，列表长度 < 100，可忽略。
- `resolveCandidate` 对每个候选调用 `allOutboundNames` + `normalizeName`：12 个候选 × 4000 节点仍是线性，远小于 DFS 警告路径。
- `cleanExistingManagedRules` 对规则数组做一次 Set 查找，订阅规则通常几百到几千条，线性。
- `upsertNamedItem` 复制并扫描 `proxies` 一次，线性。

## 3. 连接匹配路径（对照 Mihomo 源码与文档）

### 3.1 匹配模型

[Mihomo 规则文档](https://wiki.metacubex.one/en/config/rules/)：规则从上到下，先命中先生效。

`tunnel/tunnel.go` 的 `match()` 对 `getRules(metadata)` 顺序调用 `rule.Match`。Issue [#1247](https://github.com/MetaCubeX/mihomo/issues/1247) 建议给 `match()` 加缓存，维护者关闭：元数据维度多，缓存不正确，属于过度优化。

因此本脚本前置的每一条规则，都会让**未命中家宽的连接**多付一次 `Match`。默认 AI-only 下，绝大多数连接（浏览器、系统、非 AI）都会完整扫过注入块再进入用户原规则。

### 3.2 默认注入规模（本机 `buildInjectedRules()`）

66 条，分类：

| 类型 | 条数 | 默认是否启用 |
|---|---|---|
| 私有 DIRECT（localhost / LAN CIDR） | 16 | 始终注入 |
| DOMAIN-SUFFIX | 19 | 开 |
| DOMAIN | 26 | 开 |
| DOMAIN-REGEX | 3 | 开 |
| Anthropic IP-CIDR / IP-CIDR6 | 2 | `ENABLE_ANTHROPIC_IP_FALLBACK` |
| 进程 / 共享 STUN / 公共 DoH | 0 | 默认关 |

`nameserver-policy` 52 个键。Mihomo 用域名 trie / DomainSet 匹配 policy 键，不是按键线性扫描。

社区常见订阅规则是数百到数千条。多 66 条廉价比较，相对整表通常是几个百分点。把约 45 条域名规则收成一条 `RULE-SET`（`behavior: domain`，内部 `DomainSet.Has`）在这个规模上收益很小，但会改托管清理模型（当前按完整规则字符串删除）。

### 3.3 各规则类型成本（源码级）

- `DOMAIN` / `DOMAIN-SUFFIX`：单条字符串比较或后缀比较，便宜。
- `IP-CIDR` + `no-resolve`：不触发解析。脚本的私有网段和 Anthropic 网段都带 `no-resolve`（正确）。
- `IP-CIDR` 不带 `no-resolve`：匹配过程中会做 DNS。本脚本未使用。
- `DOMAIN-REGEX`：`rules/common/domain_regex.go` 使用 `github.com/dlclark/regexp2`，且 `IgnoreCase`。未命中前面 61 条的连接会连续跑 3 次正则。比 `DOMAIN` 贵，仍远小于一次 RTT。三条模式：Vertex 区域 `*-aiplatform.googleapis.com`、`repoN.cursor.sh`、`adminportalN.cursor.sh`。不能放进 `behavior: domain` 的 DomainSet。
- `DOMAIN-KEYWORD`：社区指南标为性能差、易误伤。本脚本未使用。
- `PROCESS-*-REGEX`：需要 `find-process-mode`。默认 `ENABLE_AI_PROCESS_FALLBACK = false`，且不把 `off` 改成 `strict`（回归测试覆盖）。`always` 会在每条连接上做进程查询，路由器/高连接数场景 CPU 上升。保持默认关闭。
- `AND` / 端口范围：仅共享实时基础设施开关打开时注入。

### 3.4 规则顺序

当前顺序：私有 DIRECT → AI 域名 → DOMAIN-REGEX → Anthropic IP。

进程兜底关闭时，16 条私有 DIRECT 仍排在最前。v5.7 静态分析 P4 已记录：这会覆盖用户把 RFC1918 送进代理组的意图。性能上是 16 次廉价未命中。若只在 `ENABLE_AI_PROCESS_FALLBACK` 为真时注入私有 DIRECT，可少 16 次比较，但会改变「LAN 永不进家宽」的默认保证。

`DOMAIN-REGEX` 放在精确/后缀域名之后，顺序正确。

## 4. DNS 与传输（已有文档 + 本次核对）

### 4.1 已记录的首次解析代价

`docs/dns-and-leak-model.md`「Strict-DNS performance trade-off」：非 AI 海外域的真实查询走机场绑定的 DoH，新域第一次可能多一跳机场 RTT；缓存命中不再付这笔。v5.7 研究确认：

- fake-ip 下多数 A/AAAA 本地应答，不访问上游。
- SOCKS5 支持域名寻址，AI 域的真实解析多半发生在家宽 SOCKS5 服务端。
- 脚本 nameserver 都带 `#代理名` fragment，`respect-rules` 不会改这些出口。

把非 AI DoH 改成 DIRECT 或系统解析会降低首次延迟，但会回到污染与解析路径分叉。这是已接受的取舍，不是未修缺陷。

### 4.2 嗅探

`hardenSniffer` 默认开启：`parse-pure-ip: true`、`force-dns-mapping: true`、HTTP `override-destination: true`、TLS/QUIC 嗅探 443/8443。

[嗅探文档](https://wiki.metacubex.one/en/config/sniff/)：`parse-pure-ip` 对没有域名的流量强制嗅探。这是纯 IP 连接（含 Anthropic CIDR 回退、SNI 恢复）的正确性功能。HTTP 覆盖目的地址会触发再匹配。QUIC 嗅探增加 CPU，与默认关闭的共享 WebRTC 规则无直接对应。

Mihomo 示例配置里嗅探默认 `enable: false`。本脚本打开是为域名补偿，不是性能默认值。

### 4.3 全局传输开关

Mihomo 提供 `tcp-concurrent`、`keep-alive-interval` / `keep-alive-idle`。脚本不写这些键。当前 Verge 权威字段包含 `tun` / `ipv6` / `mode` / 端口等，不含 `tcp-concurrent`。fake-ip + SOCKS5 域名寻址下，本机对 AI 连接常常不解析出多个 IP，`tcp-concurrent` 对家宽链路帮助有限。

家宽节点模板没有 `tfo` / `smux`。落地是用户提供的 SOCKS5，这些选项取决于对端，不能当作通用优化打开。

双跳（`dialer-proxy`）的 RTT 叠加是产品形态。社区「落地不要再套 hy2/tuic/wg」的警告针对第二跳；本脚本第二跳固定 socks5，与 wiki 建议一致。

## 5. 社区最佳实践对照

| 实践 | 来源 | 本脚本现状 | 结论 |
|---|---|---|---|
| 规则自上而下，高频/精确在前 | wiki 规则页 | 私有 DIRECT 在前，域名在前，正则在后 | 顺序合理；私有块在进程关闭时偏保守 |
| 大名单用 `RULE-SET` + `behavior: domain` / MRS | wiki rule-providers、官方示例配置 | 约 45 条内联 DOMAIN* | 名单太小，不值得改托管清理模型 |
| 避免 `DOMAIN-KEYWORD` | 社区分流指南 | 未使用 | 保持 |
| `DOMAIN-REGEX` 最慢、最后用 | 同上；源码 regexp2 | 3 条，放在域名之后 | 保留；不要为性能改成宽 suffix |
| IP 规则加 `no-resolve` | wiki additional params | 已加 | 保持 |
| 进程匹配默认 `strict`，路由器 `off` | wiki general、官方 config.yaml | 进程规则关，且不改用户的 `off` | 保持 |
| 嗅探默认关 | 官方 config.yaml | 脚本打开 | 正确性优先；不作为默认性能开关 |
| 私网用 `GEOIP,PRIVATE/lan,DIRECT,no-resolve` 或 `RULE-SET,private_ip` | 官方示例 | 13 条内联 CIDR + 域名 | 可少几条规则，但增加 geoip.dat 耦合 |
| 不用宽 geosite 代替窄名单 | 仓库 `docs/routing-scope.md` | 未用 `geosite:openai` 等 | 保持；宽名单会破坏 AI-only |

## 6. 发现清单（按对用户的影响）

### P1 — 大订阅上配置生成平方级 + 日志上限可让脚本失效

证据：§2.1 测量；§2.2 宿主 5 s / 1000 条日志；`warn()` 无捕获；脚本跑两遍。
影响：节点很多、且大量 `udp: false` 时，Profile 刷新变慢，或脚本被丢弃，家宽规则不生效。
方向：一次建立 name → outbound 索引；UDP 警告改汇总或封顶；路径数组只在真正告警时分配。

### P2 — 默认 3 条 DOMAIN-REGEX 对未命中连接必跑

证据：`domain_regex.go` + 默认规则第 61–63 条。
影响：CPU 微秒级，不是用户可感知的时延主因。
方向：保持；若以后区域主机列表稳定，可改成有限 exact，而不是更宽的 suffix。

### P3 — 进程关闭时仍前置 16 条私有 DIRECT

证据：`buildPrivateDirectRules` + `buildInjectedRules`；v5.7 P4。
影响：每条连接 16 次廉价未命中；并覆盖用户把局域网送进代理的意图。
方向：行为变更，需单独产品决定，不是纯性能修复。

### P4 — 嗅探对纯 IP / HTTP / QUIC 的固定 CPU

证据：`hardenSniffer`；wiki `parse-pure-ip`。
影响：高连接数时比 66 条规则更明显，但仍小于双跳 RTT。
方向：默认保持；可用 `skip-domain` 或关掉 QUIC 嗅探做可选优化，需接受纯 IP/QUIC 补偿变弱。

### 非问题（本次不作为缺陷）

- 双跳时延：产品设计。
- 非 AI 首次 DoH 多一跳：已文档化并接受。
- 66 条内联规则 vs RULE-SET：规模不够。
- 生成期重复构建域名列表：可忽略。
- `tcp-concurrent` / keep-alive / smux：证据不足，且对 SOCKS5 域名寻址帮助有限。

## 7. 第一性原理

要解决的问题：「用户切换或刷新 Profile 时，脚本必须在宿主限制内完成；连接时延由链路决定，规则表只贡献可忽略的比较。」

物理约束：boa 5 秒、1000 条日志、脚本可能跑两遍、Mihomo 顺序匹配、双跳 RTT ≫ 规则比较。

业务不变量：AI-only 范围、fail-closed 上游校验、托管规则按完整字符串清理、幂等、无第三方依赖、可粘贴进 Verge。

因此第一期应改生成期索引与日志，而不是改路由语义或引入 RULE-SET。

## 8. 规划审阅补丁（2026-08-17）

只读审阅指出：2000 节点平方级实现仍能在约 25 ms 成功跑通，因此「跑通 + 幂等 + 一条 warn」不能证明索引已接上。已写入规划的契约：

- `findOutbound(outboundIndex, name)` 缺索引立即抛错，禁止现场建表。
- 普通名歧义四条回归（双节点、双组、组/节点同名、归一化后歧义）。
- UDP 去重用 `Set`，样本上限 8，第 9 个名必须被测试断言不出现。
- 索引键与 `namedItems` 一致，不丢空串。
- README 版本纳入 R5。
- `check.jsonl` 区分「研究报告 P2–P4 不在本期」与「PRD R2/R3 必须验收」。

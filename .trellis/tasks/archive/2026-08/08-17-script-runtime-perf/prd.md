# 脚本与运行时性能优化

## Goal

降低 `clash-verge-ai-residential.js` 在 Clash Verge Rev 宿主上的配置生成成本，避免大订阅触发 5 秒超时或 1000 条日志上限后整段脚本改动被丢弃。

用户价值：节点很多的机场 Profile 仍能稳定注入家宽链路；刷新 Profile 时脚本在宿主限制内结束。

## Background

脚本入口是 `main(config, profileName)`。宿主 boa_engine 0.21：5 秒超时、`console` 最多 1000 条、合计 1 MB；无独立 Profile 脚本时同一全局脚本跑两遍；失败则回退脚本前配置，家宽规则不生效。

默认 `WARN_ON_REACHABLE_UDP_DISABLED = true`。`hardenReachableUpstreamGraph`（`clash-verge-ai-residential.js:989`）对每个可达叶子调用 `warnForUdpDisabledLeaf`（`:976`）→ `findOutbound`（`:674`）。`findOutbound` 每次对 `proxies` 与 `proxy-groups` 做全表 `filter`（`namedItems`，`:572`）。顶层组直挂 N 个节点时约为 `O(N × P)`。

本机 Node 测量（见 `research/performance-analysis.md`）：4000 节点、警告开 = 97 ms / 200 条 warn；警告关 = 0.9 ms。2000 节点、1000 个 `udp: false` 在现网平方级实现上约 25 ms 仍能成功且幂等。因此「大订阅跑通 + 日志条数 + 幂等」不能证明索引已接上。`warn()`（`:614`）不捕获异常；Verge 的 `console.warn` 超过 1000 条会抛错。

`namedItems`（`:572`）只跳过假值对象，用 `item.name === name` 匹配，空字符串与非字符串 `name` 仍可命中。`allOutboundNames` 经 `uniqueStrings` 会丢掉空串与非字符串名；两条路径语义本来就不同。

现有歧义回归只覆盖保留名：`AI-家宽` 被节点占用、重复 `AI-家宽` 组（`tests/regression.test.js:319`）。普通名重复节点、重复组、组/节点同名、归一化后再走 `findOutbound` 歧义，均无测试。

用户 2026-08-17 确认：第一期只做生成期索引与 UDP 警告汇总，不改注入规则、DNS、嗅探或传输架构。2026-08-17 规划审阅后补上强制索引契约、普通名歧义测试、UDP `Set` 去重与 8 样本验收、索引键与 `namedItems` 对齐、README 版本。

## Requirements

- R1. `main` 在 `validateReservedNameCollisions` 之后调用一次 `buildOutboundIndex(config)`。`findOutbound` 的签名为 `findOutbound(outboundIndex, name)`：第一个参数必须是含 `groups`、`proxies` 两个 `Map` 的索引对象，否则立即抛错。禁止在 `findOutbound` 内用 `config` 现场建表或回退到 `namedItems` 全表 `filter`。`resolveUpstreamName` / `resolveFromCandidates` / `resolveCandidate` / `hardenReachableUpstreamGraph` / `validateTopLevelUpstream` 必须接收并下传同一索引，缺索引即抛错。存在性、重复节点、重复组、组与节点同名的拒绝条件与现网 `findOutbound` 相同。索引键规则与 `namedItems` 相同：收录每个真值 `item`，键为 `item.name`（含 `""` 与非字符串），用 `===` 查找。
- R2. 可达叶子上显式 `udp === false` 的通知改为一次汇总 `warn`。收集结构为 `Set`（已见表）+ 计数 + 最多 8 个样本；禁止用数组 `find`/`some` 去重。文案含总数；样本为 `“name”（路径：…）`，至多 8 个；超过 8 个时追加 `……（共 N 个）`。同名节点只计一次，保留第一次路径。默认 `WARN_ON_REACHABLE_UDP_DISABLED` 仍为 `true`。顶层上游或可达组 `disable-udp: true` 仍直接拒绝。
- R3. 注入规则、`nameserver-policy`、嗅探、进程开关、家宽范围、`dialer-proxy` 解析结果与全部默认布尔开关取值保持不变。托管规则仍按完整字符串清理。`namedItems` / `findNamedItem` / `countNamedItems` 的匹配语义不收紧。
- R4. `tests/regression.test.js` 覆盖下列可失败断言（不使用耗时阈值）：
  1. `findOutbound` 在缺索引、非对象、或缺 `groups`/`proxies` `Map` 时抛错。
  2. 普通名：被选上游对应两个同名代理节点则拒绝；两个同名代理组则拒绝；同一普通名同时被组与节点占用则拒绝；精确名未命中、归一化只得到一个字符串、但该名在组与节点各有一条时仍拒绝。
  3. 至少 2000 个叶子且其中至少 1000 个不同名的显式 `udp: false`：`main` 成功；UDP 相关 `warn` 恰好 1 条；正文含总数；样本名至多 8 个；第 9 个名字不出现。
  4. 同一 `udp: false` 节点挂在两个可达组下：总数为 1，路径为第一次访问路径。
  5. 单叶子 `udp: false` 的汇总含该名与路径。
  6. 2000 叶子同一对象连续两次 `main`：规则、DNS policy、家宽节点字段一致。
  7. 现有小订阅正/负向、托管清理、幂等、循环依赖、空组、顶层 `disable-udp`、保留名冲突断言保持原目标。
- R5. `SCRIPT_VERSION`、`package.json`、`README.md` 版本行升为 `5.8.1`。`CHANGELOG.md`、`docs/configuration.md`、`docs/local-configuration.md` 写明 UDP 警告改为汇总。公开模板凭据保持 `"xxx"` / `""`。

## Acceptance Criteria

- [x] AC1. `findOutbound` 无有效索引时抛错；`buildOutboundIndex` + `findOutbound(index, name)` 能解析唯一节点/组。大订阅测试不设耗时阈值。（R1、R4.1）
- [x] AC2. 2000 叶子合成订阅上，`main` 连续两次均成功；第二次与第一次的 `rules`、`dns["nameserver-policy"]`、家宽节点的 `server`/`port`/`dialer-proxy` 一致。（R4.6）
- [x] AC3. ≥1000 个不同名的显式 `udp: false` 叶子时不抛错；UDP 关闭相关 `console.warn` 恰好 1 条；正文含总数；用 `“…”` 抽出的样本名 ≤ 8；第 9 个叶子名不在正文中。（R2、R4.3）
- [x] AC4. 单叶子 `udp: false` 汇总含节点名与路径；同名节点挂两个组时总数为 1 且路径为先访问的那条；`WARN_ON_REACHABLE_UDP_DISABLED` 默认仍为 `true`。（R2、R4.4、R4.5）
- [x] AC5. 普通名两个同名节点、两个同名组、组与节点同名、归一化后再做歧义检测，均拒绝且错误文案仍含「歧义」或现网等价用语。保留名冲突测试保持原断言。（R1、R4.2、R4.7）
- [x] AC6. 现有路由正/负向、托管清理、幂等、循环依赖、空组、顶层 `disable-udp` 断言无需为换匹配语义而改。（R3、R4.7）
- [x] AC7. `SCRIPT_VERSION`、`package.json`、`README.md` 公开版本均为 `5.8.1`；`just ci` 通过；公开模板无真实凭据。（R5）

## Out of Scope

- 研究报告中的发现 P2–P4：改 `DOMAIN-REGEX`、进程关闭时去掉前置私有 DIRECT、改默认嗅探。
- 缩短机场 + 家宽双跳时延；改 `respect-rules` 或非 AI 机场 DoH。
- 引入 `RULE-SET` / MRS / `GEOIP,PRIVATE` 替换内联规则。
- 默认打开 `tcp-concurrent`、进程兜底或共享 STUN。
- 把 `WARN_ON_REACHABLE_UDP_DISABLED` 默认改为 `false` 而不做索引。
- 用耗时阈值证明线性复杂度。
- 改 `scripts/sync-local-config.js` 或本地 TOML 字段集合。
- 在 Clash Verge Rev / boa 上实测 5 秒超时（Node 合成测试不能替代；结果标 `UNVERIFIED`）。

## Notes

- 测量与否决项：`research/performance-analysis.md`。规划审阅补丁见该文件第 8 节。
- 技术设计：`design.md`。执行清单：`implement.md`。

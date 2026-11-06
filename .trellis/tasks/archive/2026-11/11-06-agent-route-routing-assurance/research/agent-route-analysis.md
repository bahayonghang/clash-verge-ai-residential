# AgentRoute 对照研究与双客户端优化建议

研究日期：2026-11-06。当前工程基线：`b9c43a756e0f3fbf9d5276affec555901fdea46c`，公开脚本版本 5.11.0；研究开始时工作区干净。

## 1. 证据与访问范围

- 本地参考仓库：`ref/agent-route`，固定提交 `2258977368910f3a4951702476824d38ee9b097c`，读取时无本地修改。阅读 README、AGENTS、CLIENTS、DNS-WEBRTC、SYNC、MAINTAINING、Mihomo 示例、build_rules/public_formats 和 NOTICE。其 AGENTS 是研究对象，不是本项目操作授权。
- [X 原帖](https://x.com/ryuu_restart/status/2118195086899167621)：普通抓取失败，随后通过用户指定的 Ego Browser 成功读取正文、作者仓库链接回复及限定能力范围的回复。原帖宣传自动检查、配置、DNS/TUN 排查与连通性验收，未给出可验证的防封效果或一分钟完成基准。
- [作者仓库链接回复](https://x.com/ryuu_restart/status/2118195186136408187)确认项目为 `ryuu-restart/agent-route`；[作者补充](https://x.com/ryuu_restart/status/2118244423930978671)强调减少环境部署负担。社交帖不是技术验收证据。
- [Mihomo DNS 官方文档](https://wiki.metacubex.one/config/dns/)、[路由官方文档](https://wiki.metacubex.one/config/rules/)用于语义核对。
- [Clash Verge Rev 官方扩展说明](https://www.clashverge.dev/guide/extend.html)已通过网页工具和 Ego Browser 阅读。网页与下述固定源码的 DNS 所有权描述存在差异，不能用浮动网页覆盖固定版本事实。
- Clash Verge Rev 源码基线：`c83f7ad529c7c9cb3d2d54a0955ef8c1b22a411b`（参考项目标为 v2.5.10）；[enhance/mod.rs](https://github.com/clash-verge-rev/clash-verge-rev/blob/c83f7ad529c7c9cb3d2d54a0955ef8c1b22a411b/src-tauri/src/enhance/mod.rs#L338)已通过 Ego Browser 读取。未检查用户安装版本，不能声称本机就是这个版本。
- FlClash：通过 GitHub tag API 解析 `v0.9.2` 为 `7fd417fb9a9a277c6a9339a8ac87f907df7c25f7`。读取该 tag 的 task.dart，并在固定 SHA 复核 javascript.dart、system_dns.dart。版本是研究基线，不是强制升级要求或全部版本支持承诺。
- 本次没有读取私人 TOML、控制器凭据、真实订阅或流量数据库；浏览器只提取研究相关正文，未提取账户凭据。没有安装、切换代理、修改 DNS/TUN 或运行上游脚本。

## 2. 根本区别：基础分流和家宽覆盖层

AgentRoute 提供 private/reject/proxy/direct/cn-ip 五类基础规则与跨核心格式，默认海外代理；它不提供住宅节点，也不区分机场出口和住宅出口。本项目的价值是把最小必要的 AI 流量送进 `AI-家宽 → 家宽-SOCKS5`，其他业务继续交给原 Profile。

因此：
- 不把 AgentRoute proxy 名单映射为 AI-家宽；规则数量更多不等于本项目更准确。
- 不默认引入广告拦截或中国 IP/DIRECT 兜底；这些会扩大项目对原 Profile 的所有权。
- 不自动下载规则订阅或整套替换 DNS；现有精确域名、窄后缀和有界正则继续作为业务事实源。
- 借鉴的是证据、兼容、验收与维护方法，不是照搬全部名单或防封宣传。

## 3. 差异矩阵

| 主题 | AgentRoute 做法 | 当前工程证据 | 建议 |
| --- | --- | --- | --- |
| 域名粒度 | 保留 full/exact 与 suffix，跨名单冲突清理 | 根脚本 active/allPossible 目录及正负回归已存在 | 保留；新增域名仍需官方业务依据或脱敏 Connections |
| DNS 一致性 | 按用途拆解析路径，bootstrap 独立 | 根脚本:524-577、1742-1829 已区分住宅/机场/国内/私网/节点解析；正则已有 inline provider | 不重做 DNS；补最终配置和实际解析路径证据 |
| DNS 二次解析 | direct-nameserver-follow-policy | 根脚本:1824-1829 已开启 | 已有能力，不新建修复 |
| 宿主覆写 | 按真实版本核对最终有效配置 | 本项目 docs/dns-and-leak-model.md 与 troubleshooting.md 主要叙述 Verge；根脚本:1866 起也以 Verge 描述告警 | 优先补双客户端字段所有权和兼容文档、针对性 fixture |
| 私网 DNS | 追查 system 的实际上游 | 根脚本:524 使用 system，:1753-1754 为私有主机绑定它 | system 不证明 LAN 解析；加入 Windows/Ubuntu 分平台只读检查 |
| 规则顺序 | 用户例外在托管块前 | 根脚本:1977-1982 将托管块 prepend；tests/regression.test.js:335-348 锁定私网→AI→IP→用户规则 | 明确差异，不在兼容任务中更改优先级；保留未知规则不等于保留其优先级 |
| FakeIP/IPv6 | 合成地址拒绝规则在宽 ULA DIRECT 之前 | 根脚本:1498 有 fc00::/7；:1802 删除 fake-ip-range6，:1807 关闭 DNS IPv6，但不写 GUI 的顶层 IPv6 | 宿主重新开启 IPv6/注入地址池时需检查最终配置；现阶段只列风险，未证明正常默认流量发生错误直连 |
| WebRTC | ICE 候选与实际路径、通话功能分开验收 | 本项目默认不捕获通用 STUN/TURN/端口（:166-173）；现有文档声明能力边界 | 采用分层验收，不为通过测试开启宽泛 UDP 捕获 |
| 更新和来源 | 固定 SHA、摘要、失败保留旧代、发布身份 | 本项目代码版本化和默认路由 fixture 已有，来源分散在 routing-scope 与历史任务 | 首期记录客户端/内核/脚本/来源版本与复查触发点；完整机器化规则账本列 P2 后续 |
| 流量效果 | 通用路由项目，不提供本项目家宽使用证据 | ResiWatch 与 rule-tuning 支持历史观测，但本次未查询数据库 | 不承诺节省比例；以后凭 audit 覆盖率和正反样本决定收窄 |
| 供应链 | 提供同步工具及 GPL-3.0/多来源许可声明 | 本项目 MIT、根运行时零第三方依赖 | 仅借鉴方法，保持来源引用；不复制工具或大规模规则数据 |

## 4. 已独立核实的宿主事实

### Clash Verge Rev

固定源码 `enhance/mod.rs:916-961`：
1. 处理规则编辑器输出；
2. 应用设置、TUN 和 DNS 设置；
3. capture 所有权字段；
4. 执行全局及订阅扩展；
5. enforce 写回所有权字段。

`:338-359` 把 DNS 设置中接管的键写回；`:1439` 起的测试覆盖 merge/script 后恢复已填写字段。当前官方网页则称 DNS 覆写主要在扩展前、仅特定 dns.ipv6 回写。这个冲突要求按安装版本和最终配置裁决，而非断言所有版本行为统一。

### FlClash

[task.dart 固定版本](https://github.com/chen08209/FlClash/blob/7fd417fb9a9a277c6a9339a8ac87f907df7c25f7/lib/common/task.dart#L201)：
- :201 从应用 realPatchConfig 写入 find-process-mode；
- :213-226 合并 TUN 与应用选项；
- :278-310 合并 DNS override，并可把 system:// 追加到 nameserver。
这些代码证明应用会参与生成配置；script 与生成器的完整先后顺序仍须沿调用链和实机验证，不靠函数片段猜测。

[javascript.dart:13-32](https://github.com/chen08209/FlClash/blob/7fd417fb9a9a277c6a9339a8ac87f907df7c25f7/lib/common/javascript.dart#L13)传递序列化 config 并检查结果是对象；没有从这个调用层传入 profileName。不能把这一层当作整个 Rust JS 入口签名的最终证明；实施前继续核对桥接入口。

[system_dns.dart:233-238](https://github.com/chen08209/FlClash/blob/7fd417fb9a9a277c6a9339a8ac87f907df7c25f7/lib/common/system_dns.dart#L233)仅在 macOS 且非 safeModeBuild 创建系统 DNS coordinator。:184-185 保留已读配置并追加 fallback；实际 DHCP 状态仍可能影响结果。不能将参考文档的一次观察泛化成 Windows/Ubuntu 一定覆盖系统 DNS。

本项目 `cloneConfigForEdit:611-612` 已处理 FlClash 0.9.1 的顶层 rule-providers:null；该项已有实现，不重复报缺陷。规则中查找进程与按进程路由是两种能力，必须沿用 CONTEXT.md 的区分。

## 5. 已确认的平台优先级

用户明确：两个客户端都优先 Windows + Ubuntu，之后考虑 macOS + Android。

| 客户端 | Windows | Ubuntu | macOS | Android |
| --- | --- | --- | --- | --- |
| Clash Verge Rev | 首期 | 首期 | 后续 | 官方 README 未列支持；本计划不承诺 |
| FlClash | 首期 | 首期 | 后续 | 后续 |

[Verge 固定 README](https://github.com/clash-verge-rev/clash-verge-rev/blob/c83f7ad529c7c9cb3d2d54a0955ef8c1b22a411b/README.md)列 Windows/Linux/macOS；[FlClash README](https://github.com/chen08209/FlClash/blob/7fd417fb9a9a277c6a9339a8ac87f907df7c25f7/README.md)列 Android/Windows/macOS/Linux。支持 Linux 不自动证明本项目在 Ubuntu 实机已验收。Ubuntu 发行版、桌面环境、安装包及核心版本在执行时记录，不预设任意版本都兼容。

## 6. 实施优先级与边界

### P1-A：共享脚本的双宿主兼容

梳理实际入口、profileName 可用性、JSON null、CommonJS 不存在、控制面字段、DNS/sniffer 覆写、规则编辑器顺序、内核版本与能力。复用现有函数，不新增客户端检测框架或两份业务名单。已支持行为直接记录；只有失败样本证实的宿主兼容缺陷才最小修复。涉及业务规则、优先级或权限改变的发现返回父任务重新规划。

### P1-B：四组合运行验收

按客户端与 OS 分别检查生成配置、内核接受、GUI 最终配置、真实业务链、DNS 和 WebRTC；每项必须有四态结论。加入更新后的复查与离线可执行恢复说明。复用现有 docs 和回归，不在没有需求证据时新建一套全自动网络管理器。

### P2：规则证据维护与流量优化

后续可建立“模式→用途→开关→路由/DNS→来源→验证日期→正负样本”的轻量账本，并用 ResiWatch audit 选择候选收窄；先盘点现有 routing-scope/fixture 的重复成本。当前不引入自动抓取发布、不以零命中自动删规则，不修改开关默认值。

## 7. 证据边界与本轮结果

本轮完成源码与文档分析、Ego Browser 原帖读取、固定版本核对和任务规划。未在四个目标组合运行客户端、TUN、DNS 或 WebRTC 验收，未核验实际登录、流量节省、IP 固定性或账号状态。客户端使用相同 Mihomo 不是兼容性通过证据。

四组合目前均为 UNVERIFIED；缺少设备时保留未完成验收，不降低为 Node 通过即可“全兼容”。后续测试若需运行第二个核心、TUN、安装浏览器扩展、重启或改系统 DNS，必须先明确运行授权和恢复方式。

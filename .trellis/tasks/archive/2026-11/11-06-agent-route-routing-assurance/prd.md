# 参考 AgentRoute 完善双客户端兼容与路由验收

## Goal

保留最小必要 AI 家宽路由，优先让同一脚本在 Windows 与 Ubuntu 的 Clash Verge Rev、FlClash 中具有可验证的接入方式和一致的业务范围。借鉴 AgentRoute 的证据与验收方法，减少“脚本正确但宿主覆盖后无效”的盲区。

## Background

- 用户于 2026-11-06 请求阅读本地参考与 X 原帖并创建 Trellis 任务，随后明确两个客户端均优先 Windows + Ubuntu，macOS + Android 后续考虑。
- 工程基线与来源见 [研究报告](research/agent-route-analysis.md)。原帖已用 Ego Browser 获取；没有防封效果证据。
- 既有能力包括窄域名规则、正则住宅 DNS、bootstrap 独立解析、递归防护、FlClash null 归一化、正负测试；不作为待补功能重复实施。
- 确认缺口：双宿主文档和最终生效证据不完整；GUI DNS 覆写的浮动文档与固定源码冲突；system 解析器及 FakeIP/IPv6 风险须按实际配置验证，不认定为已发生泄漏。

## Requirements

| ID | 要求 | 归属 |
| --- | --- | --- |
| R1 | 同一业务脚本适配两客户端，保留当前路由、DNS 分工和本地配置边界 | 宿主兼容子任务 |
| R2 | Windows/Ubuntu × 两客户端共四组合分别有版本、安装方式、配置层次及支持边界记录 | 两子任务 |
| R3 | 路由、DNS、TUN、IPv6、WebRTC、持久化分项报告，不用配置通过替代运行证据 | 运行验收子任务 |
| R4 | 维护来源与复查依据，未知不写零，未测不写通过；用户可独立恢复 | 运行验收子任务 |
| R5 | macOS 两客户端及 Android FlClash 保留后续路线，不阻塞首期；不承诺 Android Verge | 父任务 |

## Task Map

1. [双客户端宿主兼容](../11-06-dual-client-host-compat/prd.md)：调用/覆写合同、最小必要适配、针对性回归、接入文档。
2. [双客户端运行验收](../11-06-dual-client-runtime-acceptance/prd.md)：分平台验收、证据模板、恢复与复查说明。

先完成宿主合同，再完成四组合运行验收。第二子任务可先写文档，但最终验收依赖第一个子任务的明确基线。父任务不直接实施产品改动。

## Acceptance Criteria

- [ ] AC1 → R1：两个子任务均通过其验收；AI 域名与默认开关无未经批准的变化。
- [ ] AC2 → R2：首期四组合均有明确环境与分项记录；缺少目标环境时相关项保持 UNVERIFIED，整体兼容结论不得标“全部通过”。
- [ ] AC3 → R3：脚本、内核、宿主最终配置与实际路径证据互不冒充；解析器地址、ECS、零 ICE 候选、TLS 或 HTTP 出口均不单独证明完整路径。
- [ ] AC4 → R4：报告脱敏、来源有版本、恢复方法可离线查看；测试/文档 gate 结果与实机结果分别呈现。
- [ ] AC5 → R5：文档列首期与后续支持矩阵；平台差异不通过全局禁用 IPv6、UDP 或修改用户例外掩盖。
- [x] AC6 → 初始规划交付：研究报告、父子任务 PRD/设计/实施计划及上下文清单齐备。该阶段所有任务为 planning；后续用户批准实施，当前状态见子任务及运行验收汇总。

## Out of Scope

不整合 AgentRoute 全量规则、广告拦截、中国 IP 兜底或远程自动同步；不复制上游 GPL 工具/规则数据。不新增宽泛域名、进程全量捕获、全局 UDP/DoH 路由或 DIRECT 回退。不改用户规则优先级、私网 DIRECT 既有语义、开关默认值、控制器、凭据、ResiWatch UI/数据库或依赖。

本轮不改 *.local.toml / *.local.js，不切换代理、安装客户端、重启或修改系统 DNS/TUN，不发 Issue、不推送。用户后续明确授权本地提交与归档。P2 规则机器化账本和基于历史流量的收窄另行规划。

## Approval Boundary

当前实施与验收汇总：[四组合验证结果](../11-06-dual-client-runtime-acceptance/research/validation-summary.md)。宿主合同/测试/文档已完成，实际运行仅部分验证，AC1 不满足；用户要求保留缺口并归档，未把 AC1 标为通过。

用户于 2026-11-06 明确批准最终方案：先完成 Windows/Ubuntu 双客户端宿主兼容，再做运行验收。真实网络变更和本地 TOML 例外仍需对应明确授权。

## 归档说明

2026-11-06 用户明确要求提交全部改动并归档本轮三个任务；按用户要求结项，完整实机验收仍未完成，保留现有 FAIL/UNVERIFIED，不将归档等同四组合通过。

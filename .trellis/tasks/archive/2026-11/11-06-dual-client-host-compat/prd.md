# Windows / Ubuntu 双客户端宿主兼容

## Goal

在 Clash Verge Rev 与 FlClash 的 Windows / Ubuntu 环境中复用同一份 AI 家宽脚本，提供有版本依据的接入方式、字段所有权说明与必要适配，不扩大业务路由范围。

## Background

父任务：11-06-agent-route-routing-assurance。证据见父任务 research/agent-route-analysis.md §3–5。
根脚本:611-612 已兼容顶层 rule-providers:null，:1946 支持可选 profileName，:1977-1982 保持托管块优先；现有回归已有默认快照与 null/幂等覆盖。新测试必须填真实缺口。
Verge 固定源码与官网存在 DNS 覆写说明差异；FlClash 的应用配置参与 DNS、TUN、find-process-mode 生成，完整脚本时序尚待核对。

## Requirements

| ID | 可观察要求 |
| --- | --- |
| H1 | 明确实际客户端/核心版本、脚本挂载点、调用签名、配置字段合并顺序及四组合差异；不得仅凭同内核宣称兼容 |
| H2 | 共享脚本不依赖 Node 专属运行时；缺省 profileName 有明确上游选择行为，JSON null/缺失与无效结构边界一致 |
| H3 | 区分业务规则、DNS、TUN/IPv6、查找进程和进程路由；无法由脚本控制的设置给出客户端对应操作说明 |
| H4 | 保留 AI-only 范围、未知规则所有权、默认优先级、旧托管清理、输入不变、幂等与失败路径；只修复复现的宿主兼容缺陷 |
| H5 | 中英文接入和排错文档说明版本范围、限制与最终配置核验位置；刷新订阅后的行为可复查 |

## Acceptance Criteria

- [x] HA1 → H1：四组合配置生成链表完整，引用固定源码/版本；后续真实运行证据交由运行验收子任务，未取得前明确未验证。
- [x] HA2 → H2/H4：现有及新增的针对性回归通过，覆盖两种入口、无 module 的执行、合法 null/缺失、拒绝无效结构、上游解析、重跑与输入不变；避免重复已有测试。
- [x] HA3 → H3：DNS 关键策略和应用覆写差异可核对；追加 system DNS、顶层 IPv6、进程查找关闭不能被误报为安全配置。
- [x] HA4 → H4：默认规则/DNS 语义未扩大；extra DNS 豁免、关闭核心/资源开关的清理与正负边界保留；不得为通过 fixture 无条件重录默认快照。
- [x] HA5 → H5：两个客户端在两平台的接入路径、版本与限制明确；未知字段/顺序必须标待实测。
- [ ] HA6：just ci 与 just docs-build 通过；原生内核验证按各组合的实际版本执行并记录，无环境不伪造通过。

## File Scope

预期修改：tests/regression.test.js；必要的 tests/fixtures/dual-client-*.json；docs/configuration.md、docs/local-configuration.md、docs/multi-profile.md、docs/troubleshooting.md 及其 docs/en 对应文件。
条件性允许：clash-verge-ai-residential.js 中仅宿主入口/输入归一化/字段告警相关的已复现兼容缺陷。实施前在任务中记录具体函数、失败样本及修复范围。
不改路由算法或名单、默认开关、renderer TOML 格式、控制器和依赖。证据要求新增这些范围时返回父任务规划，不能 self-fix 扩围。

## Out of Scope / Approval

不写两份客户端专用业务名单，不新增客户端自动检测框架，不为了兼容接管系统设置或增加运行时依赖。macOS/Android 留后续。
用户已于 2026-11-06 批准该规划并启动实施。真实设备修改须有独立可执行恢复方法与相应授权。


## 实施结果与验收交接

宿主入口合同、针对性测试和双语文档已实现，独立审查无发现。HA6 的完整 gate 已通过，Verge 独立核心在 Windows/Ubuntu 隔离验证通过；FlClash 自定义核心不能用独立 CLI 验证，其真实宿主/业务证据交接运行验收子任务，尚不勾选整个 HA6。详见 research/validation.md。

## 归档说明

2026-11-06 用户明确要求提交全部改动并归档本轮三个任务；按用户要求结项，完整实机验收仍未完成，保留现有 FAIL/UNVERIFIED，不将归档等同四组合通过。

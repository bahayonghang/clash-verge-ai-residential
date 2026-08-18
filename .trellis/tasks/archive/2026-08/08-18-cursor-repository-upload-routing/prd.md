# Cursor 仓库上传家宽排除优化

## Goal

在保留 Cursor Chat、Tab、Agent、认证和 Cloud Agent 等核心请求走 `AI-家宽` 的同时，
让可明确识别的 Cursor 仓库索引上传不再消耗家宽链路，并准确说明域名分流无法覆盖的边界。

## Background

- 2026-08-17 本机 Cursor 索引日志明确创建了 `https://repo42.cursor.sh` 客户端，并记录多次
  `Starting repository upload from scratch`、数千文件上传和大量失败重试。详情见
  `research/local-cursor-traffic-evidence.md`。
- 当前脚本把 `^repo[0-9]+\.cursor\.sh$` 与 SSO 管理门户正则共同放在
  `CURSOR_DOMAIN_REGEXES` 中，并由默认开启的 `ROUTE_CURSOR_CORE` 统一注入到 `AI-家宽`。
- 当前本地配置只有 `routing.cursor_core`，无法只把仓库索引留在机场上游。
- 本机 Cursor 3.16.17 未配置 `cursor.general.disableHttp2`，2026-08-17 结构化日志明确为
  `repo42.cursor.sh` 创建 HTTP/2 transport；这是昨天实际使用的索引传输路径。
- Clash 域名规则无法按 HTTPS URL path、HTTP method 或请求方向区分上传、下载和检索；
  只对官方资料或本机日志证明为索引专属的主机做拆分。

## Product Decisions

- 新增 `routing.cursor_repository_indexing`，默认值为 `false`，优先保护按量家宽额度。
- 缺失新字段的既有本地 TOML 按 `false` 自动补全；显式设置 `true` 可恢复 v5.8.1 的 repo
  索引家宽路由。
- 该开关与 `routing.cursor_core` 独立：允许只路由 Cursor 核心、只路由索引、两者都路由或两者都不路由。

## Requirements

- R1：新增独立的 Cursor 仓库索引路由控制，不要求用户关闭全部 `cursor_core`。
- R2：至少将 `repo[0-9]+.cursor.sh` 从 Cursor 核心正则目录中拆出；当索引家宽路由关闭时，
  该规则不注入 `AI-家宽`，Cursor 其他核心规则保持现有行为。
- R3：新开关必须贯通公开模板、本地 TOML 示例、配置补全、布尔校验、生成脚本和导出测试接口。
- R4：托管规则清理必须始终识别新开关开启和关闭时可能生成的规则，防止关闭后旧的
  `DOMAIN-REGEX,^repo[0-9]+\.cursor\.sh$,AI-家宽` 残留。
- R5：回归测试必须覆盖默认行为、显式开启/关闭、Cursor 核心关闭时的组合、托管旧规则清理、
  规则顺序、Cursor 非索引核心域继续走家宽，以及 Marketplace/CDN/下载仍不走家宽。
- R6：README、中文本地配置文档、英文配置文档、路由范围、故障排查和 CHANGELOG 必须同步说明
  新开关、默认值、迁移行为与域名分流边界。
- R7：如果官方或本机证据表明上传还使用 `api*.cursor.sh`、`gcpp.cursor.sh` 或其他共享主机，
  只能承诺排除经证据确认的索引专属主机，不得宣称已完全阻止仓库上传走家宽。
- R8：不得启用 Cursor 进程级兜底，也不得增加 `cursor.sh`、`cursor.com` 等宽泛后缀。
- R9：不得把共享的 `api2.cursor.sh` 从 Cursor 核心家宽路由中移除。本机 3.16.17 在
  `cursor.general.disableHttp2=true` 或服务端强制回退时可能把 RepositoryService 放到 `api2`；
  该模式必须记录为域名层无法精确隔离，而不是牺牲多数 Cursor API 来追求完整排除。

## Acceptance Criteria

- [x] AC1：公开模板和示例默认配置下，`repo42.cursor.sh` 与 `repo99.cursor.sh` 不匹配
  `AI-家宽`，Cursor 其他核心主机保持现有默认家宽路由。
- [x] AC2：显式关闭仓库索引家宽路由时，生成规则不匹配 `repo42.cursor.sh` 和
  `repo99.cursor.sh`，但仍匹配 `api2.cursor.sh`、`agent.api5.cursor.sh`、
  `api3.cursor.sh`、`adminportal42.cursor.sh` 与 `api.cursor.com`。
- [x] AC3：显式开启仓库索引家宽路由时，滚动编号的 `repo<N>.cursor.sh` 继续匹配 `AI-家宽`。
- [x] AC4：开关关闭后再次运行脚本，会移除已有的脚本托管 repo 正则规则且不产生重复规则；
  用户自有的未知 `AI-家宽` 规则仍被保留。
- [x] AC5：缺失新字段的本地 TOML 按示例默认值原子补全；非法类型、重复字段、模板常量缺失或重复
  均保持 fail-closed，且不写入半成品。
- [x] AC6：所有现有 Cursor 负向范围测试继续通过，`npm test` 和 `just ci` 通过。
- [x] AC7：文档明确列出 `repo42.cursor.sh` 是官方和本机日志共同验证的可拆分主机；
  `repo[0-9]+.cursor.sh` 是项目保留的前向兼容策略，不得冒充 Cursor 官方通配合同；共享端点或
  进程级完整归因标记为受限或未验证。
- [x] AC8：文档明确说明当前本机的 `repo42` HTTP/2 实测、`disableHttp2`/服务端回退到 `api2`
  的限制，以及 Privacy Mode 不会关闭索引上传。

## Out of Scope

- 修改 Cursor 客户端的 Privacy Mode、索引开关或 `.cursorignore`。
- 阻断 Cursor 仓库上传；本任务只改变其是否经过 `AI-家宽`，关闭后流量会回落到原 Profile 路由。
- 用 Clash 域名规则区分同一主机上的上传、下载、检索或 Chat 请求。
- 实现进程级历史流量监控；该工作由独立的 `08-18-residential-monitor-mvp` 任务负责。
- 修改或提交用户的 `.local.toml`、`.local.js`、Clash Verge 运行时 Profile 或未脱敏 Connections 日志。

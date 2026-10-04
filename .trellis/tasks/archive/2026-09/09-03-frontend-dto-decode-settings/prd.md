# 前端严格解码与设置序号

## Goal

IPC 边界对约定字段做 own-property 检查，拒绝缺字段与非法 enum。Live DTO 不含 `processPath`。设置页互不抢请求序号；secret 读取失败必须显示错误。

## Background

规格：`dto-and-decoding.md` 缺字段拒绝；CONTEXT.md 完整路径不进 DTO。证据：`dto.ts:461-507`、`use-alerts.ts:56-67`、`live-session.ts:133` 使用 `as unknown as`；`decoder.ts:236` 保留 `processPath`；`use-settings.ts` 多命令共用 `seq`；secret `catch` 为空。已归档覆盖任务明确把 AlertRule 严格解码列为前端后续，本任务收口。

## Requirements

- R1 `decodeAlertCenter` / `decodeAlertSummary` / `decodeRules` / `decodeDiagnostics` / `decodeLiveConnectionPage.rows` / `fetchTraySummary` 缺字段或非法类型抛错，界面走既有 `errorZh`。
- R2 `LiveConnectionView` 类型与解码剥离 `processPath`。
- R3 设置页 secret、连接保存、About、数据操作分序号，互不丢弃对方成功响应。
- R4 `get_controller_secret` 失败设置 `errorZh` 与重试，不把空密码框当成「无 secret」。
- R5 `components/**` 不直接 `invoke` / `renderReportHtml`（家宽 HTML 拉进 hook）。

## Acceptance Criteria

- [ ] AC1 毒化 payload（缺 `schemaVersion`、非法 enum、多余 `processPath`）被拒绝或剥离，不进视图 state。
- [ ] AC2 About 加载进行中完成保存：保存结果仍提交，不被 About 的 seq 丢弃。
- [ ] AC3 secret 读取失败：密码框不静默空白，有错误文案。
- [ ] AC4 `ipc-boundary` 测试禁止组件层 `renderReportHtml`。
- [ ] AC5 `npm --prefix residential-monitor test` 退出 0。

## Out of scope

- 在浏览器重做核算。
- 改 Rust 校验器。
- 告警中心虚拟化（低优先级，不在本任务）。

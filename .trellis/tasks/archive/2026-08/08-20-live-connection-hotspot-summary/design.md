# 设计：方向热点摘要

## DTO

将 `ConnectionPage` 扩展为 `rows`、`nextCursor`、`matchedCount`、`sampleUtc`、`summary`。`summary.topDownload/topUpload` 使用轻量 `ConnectionHotspot`，不重复发送完整 `LiveConnectionView`；label 来源于 host/进程/目标的安全 fallback，value 为对应方向累计字节。

## 后端算法

`query_connections_with_targets` 先过滤得到完整 matched 引用集合；在分页前用 `(value desc, identity asc)` 选两项，再按现有 sort/cursor 生成 rows。`AppFacade::query` 从同一 MonitorHub 锁定的数据取得 rows 和 last sample，避免跨 tick 混用。补数值、tie-break、filter、limit 独立测试。

## 前端状态与显示

`refreshLivePage` 一次提交 rows/page.summary/sample，保存应用 query token；过期响应不覆盖。renderLive 在 table 前渲染两张卡；卡片的“未知/暂无/暂停/缺口”由 summary + snapshot + tray 状态决定，绝不用 0 兜底。卡片可使用本地图标/纯 CSS，但必须有可读文本和数值。

## 兼容与失败

decoder 发现旧/缺失 summary 字段时显示能力未知，并保留表格；不从 rows fallback 计算 Top 1。Rust/TS 契约完成后再接 UI，失败可回滚卡片与字段而不改筛选、table width 或 collector。

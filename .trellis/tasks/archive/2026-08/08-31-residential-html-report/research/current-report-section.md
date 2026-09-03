# 家宽「生成报告」现状与缺口

日期：2026-08-31

## 用户截图

- 截图 1：家宽页，当前策略开，主按钮「运行报告」，状态「已生成快照 5868c078」，`targetPolicy current · v2`。下方为覆盖、数据层、导出预览。无创建时间、无顶栏窗口文案、无「查看报告」。
- 截图 2：历史策略关，状态「尚未运行报告。」覆盖块消失（`report` 为空时不渲染 `CoveragePanel`）。导出预览仍在。

## 点击路径

```
顶栏 TimeRangePicker (preset 文案)
        │
        ▼
App timeRange ──► ResidentialPage ──► ReportSection
                                        │
                                        ├─ Switch current/historical
                                        └─ Button report.run
                                              │
                                              ▼
                                   buildResidentialManualQuery(timeRange, policy)
                                              │
                                              ▼
                                   useReportArchive.runQuery
                                              │
                                              ▼
                                   invoke run_report(query, persistManual=true)
                                              │
                          ┌───────────────────┴───────────────────┐
                          ▼                                       ▼
                 snapshot token (10 min TTL)            report_archive kind=manual (7d)
                          │
                          ▼
                 页内 Coverage / Capability / Export
                 状态 = 已生成快照 {token[:8]}
```

缺失：HTML 正文 IPC、Dialog、iframe、`generatedUtc` 展示、preset 文案、离开页面后按家宽口径回看。

## 可复用

| 能力 | 位置 | 说明 |
| --- | --- | --- |
| 家宽 query 构造 | `report-section.tsx` `buildResidentialManualQuery` | 已用壳层 timeRange |
| 手动档案 | `persist_manual` | 7 天，同 fingerprint 覆盖 |
| HTML 渲染 | `c3/export.rs` `write_html` | 只写文件，不回传字符串 |
| 时间格式 | `formatUtc` | 聚合段已用 |
| 顶栏文案 | `time.recent_prefix` + `time.preset.*` | 报告卡未用 |

## 不可直接复用

- `useReportArchive.loadArchives(true)`：会水合自动日/小时档案。
- 分析报告页 `QueryForm`：独立 preset，不读壳层时钟。
- `preview_export`：无 HTML 正文。

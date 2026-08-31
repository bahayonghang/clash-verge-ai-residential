# 设计：家宽 HTML 报告创建与弹窗查看

## 边界

| 层 | 职责 |
| --- | --- |
| `ExportService` | 把 `ReportResult` 渲染成 HTML 字符串；文件导出继续走同一 `write_html` |
| `ReportArchiveService` | 按 `generated_utc` 倒序读 manual ok 行 JSON，筛家宽口径，不占 snapshot |
| `AppFacade` | `render_report_html(token, spec)`；`get_latest_residential_manual()` 命中后再 `insert` |
| `hooks/use-report-archive.ts` | 家宽页 `restoreResidentialManual`；IPC 仍只在 hooks |
| `report-section.tsx` | 创建/查看、时间文案、Dialog + iframe；不 `invoke` |

C2 不 `use rusqlite`，不直接写 `report_archive`。

## 数据流

```
顶栏 timeRange
  → buildResidentialManualQuery (host + __residential__ + policy)
  → run_report(persistManual=true)
  → snapshot token + report_archive kind=manual
  → render_report_html(token)
  → iframe srcdoc

回看：
  report_archive JSON (只读扫描)
  → 首条 grouping=host 且 category=__residential__
  → snapshots.insert 一次
  → render_report_html
```

## 合约

### `ExportService::render_html(result, spec, cancel) -> Result<String, ReportError>`

- 先 `reject_secret`。
- HTML 主阅读区：标题、本地化窗口起止、创建时间、策略、现有总量句、排名表、趋势表（表头走 i18n）。
- `metadata_line` 放在 `<pre>` 次要块，CSV 元数据对照仍成立。
- 时间：`display_timezone` 为 `utc`（忽略大小写）用 UTC；否则 `chrono::Local`。格式 `%Y-%m-%d %H:%M:%S`。
- 禁止脚本、禁止 `http://` 资源。

### `ReportArchiveService::load_latest_residential_manual(conn) -> Result<Option<ReportResult>, ReportError>`

- `where kind='manual' and status='ok' order by generated_utc desc, archive_id desc`
- `schema_version` 不匹配则跳过该行。
- 匹配：`query_echo.grouping == Host` 且 `filters.category == __residential__`。
- 不调用 `ReportSnapshotStore`。

### Commands

- `render_report_html { token, spec } -> { html: string }`
  - `get_report` 后把 `spec.ui_locale` 设为门面当前语言（与 `export_report` 相同）。
- `get_latest_residential_manual -> ReportResult | null`
  - `None` 时前端保持空态，不算错误。

### 前端

- 新 key：`residential.report.create` / `view` / `none` / `need` / `window` / `created` / `frozen` / `viewer_title`。关闭用已有 `a11y.close`。
- `report.run` / `report.idle` 留给分析报告页。
- 弹窗：`role="dialog"` `aria-modal="true"`，固定遮罩，内容区约 `min(96vw, 1100px)` × `90vh`，iframe `sandbox=""` + `srcDoc`。
- `timeRange.startUtc/endUtc` 毫秒，展示时 `/1000` 再 `formatUtc`。
- HTML 在 `reportSnapshotToken` 变化后拉取；查看按钮在 `html` 非空时启用。
- 进页 `restoreResidentialManual` 一次。非 Tauri 预览保持空态。

## 兼容

- `export_to_path` HTML 与弹窗共用 `write_html`，另存文件内容一并更可读。
- 现有 HTML 测试继续断言 `policy_version=`、`下行 20`、`@media print`、无 `http://`。
- 快照 TTL 仍 10 分钟；查看时若 render 因 token 过期失败，可再走档案水合（同一 restore 路径）。MVP：失败显示 `errorZh`，用户再点创建。

## 回滚

去掉两条 command 与 `ReportSection` 新 UI 即可回到「运行报告 + 页内面板」。档案表无 schema 变更。

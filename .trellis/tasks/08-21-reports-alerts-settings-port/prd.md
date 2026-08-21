# 报告 / 告警 / 设置页移植

## Goal

把分析报告、告警、设置 / 数据管理三页移植到 React 组件系统，功能零回退。同时把 `PRODUCT.md`、`DESIGN.md` 与 `.trellis/spec/residential-monitor/frontend/index.md` 三处与新栈冲突的约束文字更新到位。

## 父任务

`.trellis/tasks/08-21-neko-ui-refactor`。目录约定、图表封装边界见父任务 `design.md` 第 1 / 5 节；三处需同步修改的既有约束见父任务 `prd.md` 的「需要同步修改的既有约束」。

## 依赖

- 前端基座（`08-21-neko-shell-foundation`）。本子任务不依赖 C3 能力子任务。
- 按父任务 `design.md` 第 5 节，本子任务建立 `hooks/use-alerts.ts` 与 `hooks/use-settings.ts`，并复用 `hooks/use-report.ts`；`components/**` 不得直接 `invoke`。
- 表单与表格类 UI 基元（`input`、`select`、`switch`、`popover`、`table`、`tabs`）由实时页子任务或本子任务建立，先落地者登记 API。
- `ShareDonut` / `TrendArea` 图表封装：报告页需要，若聚合页子任务未建则由本子任务建立并登记。

## Confirmed facts

### 报告页

- 渲染入口 `residential-monitor/src/main.ts:727-899`（`renderReports`），入参含 `report`、`reportStatus`、`archives`、`selectedArchiveId`、`reportSource`。
- `ReportSource` 四值 `"auto-hour" | "auto-day" | "manual" | null`（`main.ts:652`）。归档 kind / status 文案由 `archiveKindLabel`（`:654`）与 `archiveStatusLabel`（`:661`）给出。
- 纯逻辑层已抽出且有单测，**只调用不重写**：
  - `src/format/report-view.ts`：`ReportForm`、`ReportPreset`（`hour | day | 7 | 30 | month`）、`ArchiveKindFilter`、`presetFromSpan`、`formFromQueryEcho`、`applyPresetRange`、`reportShareModel`、`formatSharePct`、`reportTrendModel`。
  - `src/format/report-inspect.ts`：`rankingInspectKey`、`trendInspectKey`、`inspectGroup`、`inspectKeysMatch`、`reportInspectModel` 是纯函数；`readReportScroll` / `writeReportScroll` / `inspectKeyExists` / `shouldSkipRoutinePaint` 依赖 DOM 与整页重绘模型。
- 图表是手写 SVG：`src/format/report-svg.ts` 的 `reportPieSvg` / `reportTrendSvg`，在 `main.ts:105` 引入。inspect 交互（hover / pinned 高亮）与这套 SVG 的 DOM 结构耦合：`main.ts:695-726` 的 `activeInspectKey` / `inspectMarkClass` / `inspectTipText`，模块级状态 `reportInspectPinned` / `reportInspectHover`，以及 `main.ts:1461-1468` 的失效清理。
- `shouldSkipRoutinePaint` 是为整页重绘模型做的抑制：hover / pinned 期间跳过例行重绘，避免高亮丢失。
- 报告滚动位置由 `readReportScroll` / `writeReportScroll` 在整页重绘前后存取（`main.ts:1388-1389,1458-1460`），`report-notes` 的 `<details open>` 状态由 `main.ts:1382-1387` 单独保存。
- 归档命令 `list_report_archives` / `get_report_archive`；报告命令 `run_report` / `get_report` / `release_report`；导出 `preview_export` / `export_report`。

### 告警页

- 渲染入口 `residential-monitor/src/main.ts:1145-1240`（`renderAlerts`），入参 `alerts`、`alertStatus`、`diagnostics`、`notify`。
- DTO：`AlertRule`（`src/dto.ts:118-135`，含 `kind` 三值 / `selectorKind` 四值 / `direction` 三值 / `period` 三值 / 阈值 / 恢复阈值 / 冷却 / 静默窗口）、`AlertInstance`（`:156-167`，`status` 五值）、`AlertEvidence`（`:137-154`）、`AlertCenterPage`（`:169-173`）、`AlertSummary`（`:110-116`）。
- `DiagnosticsSnapshot`（`:175-195`，Rust 侧 `c4/diagnose.rs:13-33`）含 **19** 个字段，包括 `c4Checksum`、`journalMode`、`synchronous`、`writerWatermark`、`writerReceipts`、`recentRedactedErrorClasses`。
- `NotifyCapability`（`:197-202`）含 `available`、`reasonZh`、`canFocusApp`、`focusAssistUnknown`。
- 命令：`list_alert_rules`、`upsert_alert_rule`、`list_alert_center`、`alert_summary`、`test_notification`、`get_diagnostics`、`export_diagnostics`、`scan_outbox`。
- `AlertEvidence.notEvaluableReason` 与 `AlertInstance.status = "not-evaluable"` 必须与「无告警」分开表达。

### 设置 / 数据管理页

- 渲染入口 `residential-monitor/src/main.ts:1016-1118`（`renderSettings`）。分区 `SettingsSection` 五值：`"appearance" | "connection" | "data" | "about" | "danger"`（`main.ts:203`）。草稿 `SettingsDraft = { address: string; targets: string }`（`:204`）。
- 字体选择器：`fontChoiceLabel`（`:900`）、`fontChoiceList`（`:904`）、`visibleFontChoices`（`:923`）、`fontOptionMarkup`（`:933`）、`renderFontPicker`（`:943`）。字体枚举来自 `list_ui_fonts`。
- 关于区：`aboutRow`（`:959`）、`renderAboutBody`（`:964-999`）、`selectAboutReleaseUrl`（`:1000-1015`）。`decodeAbout`（`src/dto.ts:379-387`）**拒绝 `signed: true`**，未签名候选不得标成 signed。
- 危险区：`preview_delete_local_data` → `DeletePreview`（含 `confirmPhrase` 中文短语与 `items`），`confirm_delete_local_data` → `DeleteReport`（`allDeclaredOk` + 逐项结果 + `summaryZh`）。**部分失败不得显示「已全部删除」**（frontend spec 明确列为质量门）。
- 其余命令：`get_settings`、`get_controller_secret`、`save_settings`、`save_targets`、`test_controller`、`disconnect_controller`、`retention_preview`、`run_retention`、`create_backup`、`restore_backup`、`validate_backup`、`data_directory`、`open_log_dir`、`run_user_vacuum`、`pick_file`、`start_operation`、`cancel_operation`、`complete_wizard`、`pause_collector`、`resume_collector`、`reconnect_now`。
- 长操作进度用 `OperationProgress`（`src/dto.ts:242-253`，含 `phase`、`current`/`total`、`unit`、`canCancel`、`status`、`redactedError`）。
- secret 处理：`src/ipc/secret-field.ts` + 单测。密码框可回填并切换显示；日志、SQLite、Channel、导出都不得出现 secret。
- `renderRecovery`（`main.ts:1119-1140`）与 `renderUnavailable`（`:1141-1144`）也在本子任务范围内。

### 需要同步更新的三处约束

- `PRODUCT.md:33`「固定五页」→ 新的十段路由。
- `PRODUCT.md:37`「前端是 Vanilla TypeScript + Vite，不引入 UI 框架」→ React + Tailwind；禁止远程 URL 与 CDN 的部分保留。
- `PRODUCT.md:38`「前端只保存视图选择和 DTO 缓存，不在浏览器里重做核算或 Top N」→ **不改**。
- `DESIGN.md` 整份 → 按 neko 令牌重写；`DESIGN.md:167` 的「禁止远程字体或 CDN」保留，「不引入 UI 框架」删除。
- `.trellis/spec/residential-monitor/frontend/index.md`「不引入 UI 框架」→ React + Tailwind 约定；保留禁止 `window.__TAURI__`、eval、远程 URL、CDN；保留「关于页不得把未签名候选标成 signed」与「删除部分失败不得显示已全部删除」两条质量门。

## Requirements

### R1. 报告页

- 保留查询构造（`ReportForm` 五个 preset + 归档窗口来源）、`series`、`rankings`、`coverage`、`drilldownCapability`、`policyMetadata`、`dataTier`、`namedSql` 的展示。
- 保留归档列表与 kind / status 过滤、选中归档、`reportSource` 四态。
- 保留导出预览与导出，含 `RedactMode`。
- 保留占比读数与 `formatSharePct` 的未知语义。
- 图表换成 Recharts 封装（`ShareDonut` + `TrendArea`），inspect 的 hover / pinned 高亮与 tip 行为按新实现重建，`rankingInspectKey` / `trendInspectKey` / `inspectGroup` / `inspectKeysMatch` / `reportInspectModel` 五个纯函数继续使用。
- 图表旁保留同口径数据表。

### R2. 告警页

- 保留规则列表与编辑：三种 `kind`、四种 `selectorKind`、三种 `direction`、三种 `period`、阈值、恢复阈值、冷却秒、静默窗口、时区。
- 保留告警中心分页与五种 `status`；`not-evaluable` 与「无告警」分开表达，`notEvaluableReason` 可见。
- 保留 `AlertEvidence` 的证据展示，含 `coverageSummary`、`observedValue`（可为 null → 「未知」）、`reportQuery` 跳转。
- 保留通知能力检测（`available` / `reasonZh` / `canFocusApp` / `focusAssistUnknown`）与测试通知。
- 保留诊断快照 19 个字段与诊断导出、outbox 扫描与积压数。

### R3. 设置 / 数据管理页

- 保留五个分区与其内容：外观（主题 / 字体 / 字号 / 密度 / 语言 / 侧栏宽度）、连接（地址 / secret 回填与显示切换 / 测试 / 断开 / 重连 / targets）、数据（保留预览与执行 / 备份 / 恢复 / 校验 / 数据目录 / 日志目录 / vacuum）、关于、危险区。
- 保留字体选择器的系统字体枚举与可见字体过滤。
- 关于区：未签名候选不得标成 signed；`decodeAbout` 的断言不放宽。
- 危险区：保留删除预览、中文确认短语、逐项结果；**部分失败不得显示「已全部删除」**。
- 保留长操作的 `OperationProgress` 展示与取消。
- secret 不得进日志、SQLite、Channel 与导出；不得渲染进 DOM 属性。

### R4. Recovery 与不可用态

- 保留 `renderRecovery` 的恢复壳内容与 `renderUnavailable` 的不可用页文案。

### R5. 文档与规范同步

- 按「需要同步更新的三处约束」逐条更新，`PRODUCT.md:38` 保持不改。
- `CHANGELOG.md` 加 English 条目；`residential-monitor/docs/` 相关中文文档同步（至少 `first-run.md`、`reporting.md`、`alerts.md`）。

### R6. 双语与可访问性

- 新增字符串同时进 `zh.ts` 与 `en.ts`；既有键沿用不改名。
- 保留 `<table>` 语义、`aria-sort`、键盘可达、`:focus-visible`、`prefers-contrast: more`、`prefers-reduced-motion`。

## Out of scope

- 不改任何 Rust：本子任务不动命令、DTO、schema 与查询。
- 不重写 `src/format/report-view.ts`、`report-inspect.ts` 的纯函数与其单测，也不重写 `src/ipc/secret-field.ts`。
- 不改 `src/main.ts`（随父任务收口整体删除）。
- 删除 `report-svg.ts` 与 `report-inspect.ts` 四个 DOM 函数已由父任务 `design.md` 第 2 节批准，不属于「重写稳定层」。
- 不新增告警规则类型、选择器类型、周期类型。
- 不改保留策略、备份格式、删除确认短语文本。
- 不动概览页、聚合页、实时页、家宽页。
- 不实现 neko 的多后端管理对话框、鉴权与版本检查。

## Acceptance Criteria

- [ ] AC1 (R1)：报告页五个 preset + 归档窗口来源均可用；`series` / `rankings` / `coverage` / `drilldownCapability` / `policyMetadata` / `dataTier` / `namedSql` 全部可见；`reportSource` 四态正确。
- [ ] AC2 (R1)：报告图表的 hover / pinned 高亮与 tip 行为可用；`rankingInspectKey` / `trendInspectKey` / `inspectGroup` / `inspectKeysMatch` / `reportInspectModel` 的既有单测全部通过且未删断言。
- [ ] AC3 (R1)：导出预览与导出可用，`RedactMode` 生效，导出物中无 secret。
- [ ] AC4 (R2)：告警规则的三种 kind / 四种 selectorKind / 三种 direction / 三种 period 均可创建与编辑；`upsert_alert_rule` 入参与改造前一致。
- [ ] AC5 (R2)：`not-evaluable` 与「无告警」在界面上分开表达，`notEvaluableReason` 可见；`observedValue` 为 null 时显示「未知」。
- [ ] AC6 (R2)：`DiagnosticsSnapshot` 的 19 个字段全部可见；诊断导出与 outbox 扫描可用；`recentRedactedErrorClasses` 不泄漏原始错误文本。
- [ ] AC7 (R3)：设置页五个分区内容齐备；secret 可回填与切换显示；源码中 secret 不进 DOM 属性、不进 console。
- [ ] AC8 (R3)：关于区在 `signed: true` 的伪造响应下抛错而不渲染 signed（有测试）。
- [ ] AC9 (R3)：删除本地数据在部分失败时不显示「已全部删除」，逐项结果与 `summaryZh` 正确（有测试）。
- [ ] AC10 (R3)：长操作的 `OperationProgress` 阶段、进度、单位、可取消状态与 `redactedError` 正确显示；取消可用。
- [ ] AC11 (R4)：Recovery 壳与不可用页文案正确。
- [ ] AC12 (R5)：`PRODUCT.md`、`DESIGN.md`、`.trellis/spec/residential-monitor/frontend/index.md` 已按新栈与十段路由更新；`PRODUCT.md:38` 未改（有 diff 确认）；两条既有质量门（signed / 部分失败）在 spec 中保留。
- [ ] AC13 (R5/R6)：`zh.ts` / `en.ts` 键集合一致；`CHANGELOG.md` 有 English 条目；`docs/` 中文同步。
- [ ] AC14 (R1/R2/R3)：`components/**` 内无 `invoke` 调用（源码级确认）。
- [ ] AC15：`npm --prefix residential-monitor run typecheck && lint && test && build` 通过；`cargo test --workspace` 仍通过（本页无 Rust 改动）。
- [ ] AC16 (R6)：四款主题 × 中英文 × 1200×800 / 窄窗口实拍无溢出；`aria-sort` 与键盘可达。

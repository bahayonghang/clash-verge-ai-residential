# DTO 解码

- Rust 是权威校验者。前端解码失败时显示专门中文状态，不猜测缺字段。
- 每条 Channel 消息必须检查 `schemaVersion`、`kind` 和单调 `seq`。
- 禁止把 mihomo 原始 JSON 或 SQL 行传到视图层。
- 时间展示用用户本地时区；持久时间保持 UTC integer。

## Scenario: C2 Monitor Channel

### 1. Scope / Trigger
- Trigger: 前端订阅实时状态、窗口重建、序号缺口或 schema 不兼容。

### 2. Signatures
- `subscribe_monitor(on_event) -> subscriptionId`
- `resync_monitor(subscriptionId, on_event) -> newSubscriptionId`
- 首帧：`bootstrap { schemaVersion, subscriptionId, snapshot, baseSeq, backendTime }`
- 后续：`connectionDelta | healthChanged | summaryChanged | alertChanged`，均带 `seq`

### 3. Contracts
- `schemaVersion` 必须为 `1`。
- 后续消息只接受 `seq > baseSeq`。
- `snapshot` 是概览 DTO，不含 10k 连接数组。
- 列表与详情走 `query_live_connections` / `get_connection`。

### 4. Validation & Error Matrix
- `seq == lastSeq + 1` → 应用增量
- `seq <= lastSeq` → 忽略重复或陈旧消息
- `seq > lastSeq + 1` → 冻结并 `resync`
- 未知 `kind` 或错误 `schemaVersion` → fail closed，显示升级 / 重载
- `subscriptionId` 不匹配 → 丢弃迟到消息

### 5. Good/Base/Bad Cases
- Good: bootstrap 后连续 `seq`
- Base: 窗口重建生成新 `subscriptionId`，清空 cache
- Bad: 缺口后继续猜状态；把 gap 显示成 `0`

### 6. Tests Required
- TS `reducer.test.ts`：迟到订阅、缺口、重复、`204` 后等 remove
- Rust `channel_contract_tests`：首帧 bootstrap、resync 换 identity

### 7. Wrong vs Correct
#### Wrong
前端把 `meterUpload` 与 `attributedUpload` 加成「全局流量」，缺口当 0。
#### Correct
分字段展示；`null` 显示「未知」。

## Scenario: C3 Report Commands

### 1. Scope / Trigger
- Trigger: 历史报告、导出、保留预览、用户备份与 Recovery restore。

### 2. Signatures
- `run_report(query: ReportQuery) -> ReportResult`
- `get_report(token) -> ReportResult`
- `release_report(token) -> bool`
- `preview_export(token, spec) -> ExportPreview`
- `export_report(token, spec, path) -> path`
- `create_backup(path) -> checksum`
- `restore_backup(path)`
- `retention_preview() -> RetentionPreview`
- `run_retention(delete: bool) -> RetentionPreview`

### 3. Contracts
- `ReportResult.schemaVersion` 必须为 `1`。
- UI、CSV、JSON、HTML 只消费同一 `reportSnapshotToken`，不得为导出重新查询。
- 大结果不经实时 Channel。
- `restore` 可在 `recovery-only` 分支执行，不初始化 `ReportService`。

### 4. Validation & Error Matrix
- 非法 range / timezone / page / cursor → `invalid_query`
- raw / 精确 Top N 过期 → `capability_unsupported`
- 用户取消 / deadline → `cancelled` / `deadline_exceeded`
- token TTL / 缺失 → `token_expired`
- 活动 token 或 spool 超限 → `quota_exceeded`
- 磁盘不足 → `insufficient_space`

### 5. Good/Base/Bad Cases
- Good: 同一 token 的 UI 与三种导出 totals 一致
- Base: 空区间 totals 为 0，coverage=`empty`
- Bad: 过期 raw 仍返回截断 Top N；token 仍持有 read transaction

### 6. Tests Required
- Rust `c3::query` / `c3::service` / `c3::export` / `c3::backup` / `c3::retention`
- TS `decodeReportResult` 拒绝缺 token

### 7. Wrong vs Correct
#### Wrong
前端自己按表格重算 Top N，或导出时再跑一遍 SQL。
#### Correct
图表、数据表和导出都读当前 `ReportResult`。

## Scenario: C4 Alert Commands

### 1. Scope / Trigger
- Trigger: 告警中心、规则编辑、测试通知、诊断预览与导出。

### 2. Signatures
- `list_alert_rules() -> AlertRule[]`
- `upsert_alert_rule(rule) -> AlertRule`
- `list_alert_center(status, after) -> AlertCenterPage`
- `alert_summary() -> AlertSummary`
- `test_notification() -> NotifyCapability`
- `get_diagnostics() -> DiagnosticsSnapshot`
- `export_diagnostics(path) -> path`
- `scan_outbox() -> count`

### 3. Contracts
- `AlertCenterPage.schemaVersion` 与 `DiagnosticsSnapshot.schemaVersion` 必须为 `1`。
- 测试通知不得写入真实告警历史。
- 诊断失败不影响采集或告警提交。

### 4. Validation & Error Matrix
- 无效滞回 / 周期 / 时区 → `invalid_rule`
- C3 能力不支持或 coverage 不足 → 实例 `not-evaluable`，观测值不得写成零
- 通知不可用 → 应用内记录仍完整


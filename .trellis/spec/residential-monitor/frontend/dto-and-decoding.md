# DTO 解码

- Rust 是权威校验者。前端解码失败时显示专门中文状态，不猜测缺字段。
- 每条 Channel 消息必须检查 `schemaVersion`、`kind` 和单调 `seq`。
- 禁止把 mihomo 原始 JSON 或 SQL 行传到视图层。
- 时间展示用用户本地时区；持久时间保持 UTC integer。
- `BootstrapDto.uiLocale` 缺字段时按 `zh`。`messageZh` 字段名保持不变，内容为当前语言。

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
- `subscribe_monitor` / `resync_monitor` 必须保存 Tauri `Channel`，后续 `publish` 转发到该 Channel。只发 bootstrap 后丢弃 Channel，实时页会一直空。
- 前端用 `@tauri-apps/api/core` 的 `Channel` + `invoke`。禁止把 `window.message` 当成 Monitor Channel。
- bootstrap 与 `connectionDelta` 之后必须再查默认第一页（`sortField=identity`，`limit=LIST_PAGE_DEFAULT`）。表格以查询页为准，不以 Channel upsert 排序。

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
- TS `live-session.test.ts`：产品源码不含 `window.message` Channel
- Rust `channel_contract_tests`：首帧 bootstrap、resync 换 identity
- Rust `subscription_forward_tests`：存下的 sink 能收到 bootstrap 之后的 delta

### 7. Wrong vs Correct
#### Wrong
前端把 `meterUpload` 与 `attributedUpload` 加成「全局流量」，缺口当 0。监听 `window.message` 或只渲染 `bootstrap.snapshot`。
#### Correct
分字段展示；`null` 显示「未知」。用 Tauri Channel 订阅，再用 `query_live_connections` 填表。

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

## Scenario: C5 Release Commands

### 1. Scope / Trigger
- Trigger: 关于页、GitHub Releases 地址、应用内显式删除本地数据、用户主动 VACUUM。

### 2. Signatures
- `get_about() -> AboutDto`
- `open_releases() -> releasesUrl`
- `preview_delete_local_data() -> DeletePreview`
- `confirm_delete_local_data(phrase) -> DeleteReport`
- `run_user_vacuum()`

### 3. Contracts
- `AboutDto.schemaVersion` 必须为 `1`。`signed` 为 `false` 时不得写成已签名。
- `open_releases` 只返回固定 GitHub Releases URL，不注册 updater plugin。
- 删除确认短语必须是 `删除全部本地数据`。部分失败时 `allDeclaredOk=false`，文案不得写成「已全部删除」。
- 删除只清理数据目录声明对象和当前进程凭据引用。未再确认前不写本机 Credential Manager。
- 用户主动 VACUUM 前检查约两倍数据库空间；失败保留当前库。不自动 VACUUM。

### 4. Validation & Error Matrix
- 确认短语不匹配 → `delete_not_confirmed`，不删除任何文件
- 磁盘不足 → `insufficient_space`，不启动 VACUUM
- 未签名 about 被标成 `signed=true` → 前端解码拒绝

### 5. Good/Base/Bad Cases
- Good: 预览后短语匹配，分项结果全部 `ok`
- Base: 对象本不存在仍记 `ok`，不是失败
- Bad: 用当前 NSIS 包冒充 C0 升级基线；把 fixture 并发写成 30 天容量

### 6. Tests Required
- Rust `c5::purge` / `c5::vacuum` / `c5::about` / `c5::baseline`
- TS `decodeAbout` 拒绝 `signed=true`

### 7. Wrong vs Correct
#### Wrong
删除部分失败仍显示「已全部删除」，或 `c5-baseline` 缺失时用当前 installer 充当旧版本。
#### Correct
部分失败可见。C0 基线缺失则 `usableForUpgrade=false`，对应 AC 记未通过。


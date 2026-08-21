# DTO 解码

- Rust 是权威校验者。前端解码失败时显示专门中文状态，不猜测缺字段。
- `LiveOverview` 的所有约定字段都必须用 own-property 检查。字段存在且为 `null` 才是合法 pending/unavailable；字段缺失、undefined、非法 enum、负数或非有限数必须拒绝。`observationPhase` 合法值为 unconfigured / connecting / baselinePending / current / paused / disconnected / resyncRequired / decodeFailed。
- 每条 Channel 消息必须检查 `schemaVersion`、`kind` 和单调 `seq`。
- 禁止把 mihomo 原始 JSON 或 SQL 行传到视图层。
- 时间展示用用户本地时区；持久时间保持 UTC integer。
- `BootstrapDto.uiLocale` 缺字段时按 `zh`。`BootstrapDto.uiTheme` 缺字段时按 `mocha`。`BootstrapDto.uiFont` 缺字段或含 CSS 元字符时按 `system`；合法值为 `system`、旧别名 `yahei` / `serif` / `mono`，或校验后的本机族名。`BootstrapDto.uiFontSize` 缺字段时按 `md`。`BootstrapDto.uiDensity` 缺字段时按 `comfortable`。`BootstrapDto.uiSidebarWidth` 缺字段或非有限数字时按 `220`，并 clamp 到 160–352。`BootstrapDto.logDir` 缺字段时「打开日志目录」禁用，显示「日志目录未知」，不猜本机路径。`messageZh` 字段名保持不变，内容为当前语言。

## Scenario: list local UI fonts

### 1. Scope / Trigger
- Trigger: 外观分区首次绘制、或上次列表失败后再次进入外观。

### 2. Signatures
- `list_ui_fonts() -> string[]`
- `save_ui_font(font: string) -> string`

### 3. Contracts
- 列表是本机族名，Windows GDI 枚举，跳过 `@` 竖排面和空名，大小写不敏感去重。
- 不进 `BootstrapDto`。结果只留当前会话缓存。
- `save_ui_font` 返回规范化字符串：`system`、旧别名，或通过校验的族名。

### 4. Validation & Error Matrix
- 族名含 `" ' ; { } < > \\`、控制字符、`@` 前缀或 UTF-16 超过 31 码元 → 存 `system`
- GDI 失败 → `error.font_list` / `action.retry`，前端仍可设回系统默认

### 5. Good/Base/Bad Cases
- Good: 选 `Microsoft YaHei`，`--ui-font` 立即换栈，重启后恢复
- Base: 非 Tauri 预览只有 `system` 哨兵
- Bad: 把未校验字符串写进 stylesheet 或 `data-*`

### 6. Tests Required
- TS `parseUiFont`：合法族名、旧别名、注入串、过长、`@` 前缀
- Rust `UiFont::parse` 与 `save_ui_font` round-trip；Windows 上 `list_installed_families` 非空且无 `@`

### 7. Wrong vs Correct
#### Wrong
四档按钮写死 `yahei` / `serif` / `mono`，未知值一律丢弃。
#### Correct
`system` 哨兵 + 校验后的本机族名；旧四档继续映射原栈。

## Scenario: report archive kind manual

### 1. Scope / Trigger
- Trigger: 列表出现 `kind=manual`；`run_report` 增加 `persistManual`。

### 2. Signatures
- `run_report({ query, persistManual?: boolean })`
- `release_report({ token })`
- `ReportArchiveKind = "hour" | "day" | "manual"`
- `ArchiveKindFilter = "all" | "hour" | "day" | "manual"`

### 3. Contracts
- 解码遇到非上述 kind 拒绝整页，不猜测。
- 显式运行传 `persistManual: true`。`useReport` 现查省略或 false。
- 卸载、换 query、取消响应必须 `release_report`。已返回但不采用的 token 也要释放。
- 进页 `pickLatestArchive` 只选成功 day，否则成功 hour，跳过 manual。
- 列表失败才用 `report.archive.unavailable`。水合失败保留列表，配额不得写入能力说明。

### 4. Validation & Error Matrix
- `kind` 不是 hour/day/manual → `ReportArchivePage 无效`
- 水合 `quota_exceeded` → 结果区配额文案，列表仍可见
- 列表 IPC 失败 → `report.archive.unavailable`

### 5. Good/Base/Bad Cases
- Good: 手动行可筛选、点选、导出绑定水合 token
- Base: 无 manual 行时进页行为与改前相同
- Bad: 把配额写进「能力说明」；取消的 `run_report` 不释放 token

### 6. Tests Required
- `decodeReportArchivePage` 接受 `manual`
- `pickLatestArchive` 不选手动行
- `RankBarCard` 配额进 alert、不进 `data-capability-note`
- `shouldReleaseAbandoned` 覆盖取消与过期序号

### 7. Wrong vs Correct
#### Wrong
`loadArchives` 水合失败时把 `statusZh` 写成档案列表暂不可用。
#### Correct
列表成功则 `setArchives`；水合失败只用配额/存储原文。

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
- `snapshot` 是概览 DTO，不含 10k 连接数组。bootstrap 与每个 `connectionDelta` 都携带同 tick snapshot，使 rows、activeCount 与 phase 同步推进。
- 列表与详情走 `query_live_connections` / `get_connection`。
- `subscribe_monitor` / `resync_monitor` 必须保存 Tauri `Channel`，后续 `publish` 转发到该 Channel。只发 bootstrap 后丢弃 Channel，实时页会一直空。
- 前端用 `@tauri-apps/api/core` 的 `Channel` + `invoke`。禁止把 `window.message` 当成 Monitor Channel。
- bootstrap 与 `connectionDelta` 之后必须再查当前 `liveQuery` 的第一页（默认 `sortField=identity`，`limit=LIST_PAGE_DEFAULT`）。表格以查询页为准，不以 Channel upsert 排序。

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
前端把 `meterUpload` 与 `attributedUpload` 加成「全局流量」，缺口当 0。监听 `window.message`、只渲染 `bootstrap.snapshot`，或把缺失字段当作合法 null。
#### Correct
分字段展示；按 `observationPhase` 解释显式 `null`。用 Tauri Channel 订阅，再用 `query_live_connections` 填表。

## Scenario: C2 Live Connection Query Page

### 1. Scope / Trigger
- Trigger: 实时页刷新、筛选应用、排序、bootstrap / `connectionDelta` 后补第一页。

### 2. Signatures
- `query_live_connections(query: ConnectionQuery) -> ConnectionPage`
- `ConnectionPage { rows, nextCursor, matchedCount, sampleUtc, summary }`
- `summary { topDownload, topUpload }`，每项为 `ConnectionHotspot | null`
- `ConnectionHotspot { identity, label, host, process, destination, value }`

### 3. Contracts
- `rows`、`matchedCount`、`sampleUtc`、`summary` 是同一 hub 快照。前端一次提交，过期请求不得覆盖新状态。
- `summary` 来自完整筛选 matched 集合，在 sort / cursor / `limit` 分页之前计算。不得用当前页 `rows` 补算 Top 1。
- 热点排序：方向 `value` 降序，相同则 `identity` 升序。
- 命中为空时 `topDownload` / `topUpload` 为 `null`，不写 `0`。
- 热点字段只有 identity / 可脱敏 label / host / process / destination / value。禁止 `processPath` 或原始规则载荷。
- `sampleUtc` 为安全整数或 `null`。缺字段、非整数或非法热点 → 解码失败。

### 4. Validation & Error Matrix
- 缺 `matchedCount` / `sampleUtc` / `summary` / `topDownload` / `topUpload` → 拒绝整页
- `matchedCount` 或 `value` 不是 ≥0 安全整数 → 拒绝
- `sampleUtc` 为 `1.5` 等非整数 → 拒绝
- 热点非 `null` 但 `identity` / `label` 为空 → 拒绝
- 旧响应只有 `rows` + `nextCursor` → 拒绝，表格可保留，卡片 fail closed

### 5. Good/Base/Bad Cases
- Good: `limit=1` 与 `limit=200` 的 `summary` 相同；cursor 翻页不改 summary
- Base: 无匹配时 `matchedCount=0`，两个热点为 `null`
- Bad: 前端把当前页 download 最大值当成全量 Top 1；缺口时把旧热点画成 0

### 6. Tests Required
- TS `live-session.test.ts`：合法快照、缺字段、null 热点、非法 value / sampleUtc
- TS `live-hotspot.test.ts`：paused / gap / disconnect / `collectorRunning === null` / 未知 coverage 隐藏数值
- Rust `connection_query_tests`：完整 matched 集合、identity tie-break、limit/cursor/sort 不改 summary、空匹配为 null、序列化不含 `processPath`

### 7. Wrong vs Correct
#### Wrong
```ts
const top = page.rows.reduce((best, row) => row.download > best.download ? row : best);
```
#### Correct
```ts
const page = decodeLiveConnectionPage(raw); // summary.topDownload 已由 Rust 选定
```

## Scenario: C3 Report Commands

### 1. Scope / Trigger
- Trigger: 历史报告、导出、保留预览、用户备份与 Recovery restore。

### 2. Signatures
- `run_report(query: ReportQuery) -> ReportResult`
- `residential_share(rangeStartUtc, rangeEndUtc, displayTimezone) -> ResidentialShare`
- `get_report(token) -> ReportResult`
- `release_report(token) -> bool`
- `list_report_archives({ kind?, after?, limit? }) -> ReportArchivePage`
- `get_report_archive({ archiveId }) -> ReportResult`
- `preview_export(token, spec) -> ExportPreview`
- `export_report(token, spec, path) -> path`
- `create_backup(path) -> checksum`
- `restore_backup(path)`
- `retention_preview() -> RetentionPreview`
- `run_retention(delete: bool) -> RetentionPreview`

### 3. Contracts
- `ReportResult.schemaVersion` 必须为 `1`。
- `ReportResult.attributionQuality` 必须完整解码 known/missing upload/download/connections 与 complete/partial/unavailable；数字为非负安全整数，且 known + missing 必须精确等于 totals。rankings 每个字段也逐项验证，禁止 `as unknown as ReportResult` 绕过边界。
- UI、CSV、JSON、HTML 只消费同一 `reportSnapshotToken`，不得为导出重新查询。
- 大结果不经实时 Channel。
- `restore` 可在 `recovery-only` 分支执行，不初始化 `ReportService`。
- `queryEcho.granularity` 合法值为 `minute1` / `minute2` / `minute5` / `minute10` / `hour` / `day` / `month`。分钟档只在 raw 层有效；落到 HourlyDimension / DailyDimension / DailyCore 时返回 `capability_unsupported`，原因「分钟粒度只在 raw 保留期内可用」，不升粒度。
- 排名 `identity` 为 `"__unknown__"` 表示维度值缺失。前端按 grouping 显示维度化缺失标签并保留字节；Host 维未知可下钻检查组成，其它维保持不可下钻。
- 下钻 `filters.rule` / `filters.chain` 必须复用排名行 `identity`。`filters.chain` 是最后一跳，不是完整 `a>b>c`。单跳且有 payload 时 SQL 规则键是 `rule`，不是 `rule(payload)`。

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

## Scenario: residential_share

### 1. Scope / Trigger
- Trigger: 家宽页聚合段读取「家宽占可归因观测」比例。

### 2. Signatures
- `residential_share(range_start_utc: i64, range_end_utc: i64, display_timezone: String) -> ResidentialShare`

### 3. Contracts
- `schemaVersion` 必须为 `1`。
- 四个流量字段为 `Option<u64>`：`residentialUpload` / `residentialDownload` / `attributedUpload` / `attributedDownload`。
- 家宽分子：`a.primary_category_id IS NOT NULL`（核算口径，精确 target）。
- 分母：同区间无分类过滤的可归因观测（含未分类连接）。
- 先查 `COVERAGE_RAW`。`covered_sec == 0` 时四个字段全部为 `None`。`covered_sec > 0` 时填实测值，此时 `0` 表示区间内无家宽流量。
- `namedSql` 回显常量名，至少含 `share_residential_raw`。
- 返回 `targetCount` 与 `policyVersion`。未配置 targets 时前端走引导态，不把 `None` 画成 0%。

### 4. Validation & Error Matrix
- `range_end <= range_start` 或区间过大 → `invalid_query`
- 非法 timezone → `invalid_query`
- 无覆盖 → 四个字段 `None`，不是错误

### 5. Good/Base/Bad Cases
- Good: 有覆盖且家宽为 0 → 四个 `Some`，家宽为 0，界面写「区间内无家宽流量」
- Base: 无覆盖 → 四个 `None`，界面「未知」
- Bad: 用 totals 的 `coalesce(sum, 0)` 把无覆盖显示成 0%

### 6. Tests Required
- Rust：无覆盖 → 四个 `None`；有覆盖且家宽 0 → 四个 `Some`；`named_sql` 回显一致
- TS：`decodeResidentialShare` 拒绝缺字段；`None` 不画成 0

### 7. Wrong vs Correct
#### Wrong
空窗 totals 为 0 就显示 0%。
#### Correct
无覆盖显示「未知」；有覆盖的 0 才是真实零。

## Scenario: C3 Report Archives

### 1. Scope / Trigger
- Trigger: 已闭合本地小时 / 自然日自动出报、进分析报告页读档案、从冻结结果导出。

### 2. Signatures
- 表 `report_archive`（`user_version` 4，`c3-archive-v4`）
- `list_report_archives({ kind?: "hour"|"day", after?: string, limit?: number }) -> ReportArchivePage`
- `get_report_archive({ archiveId }) -> ReportResult`

### 3. Contracts
- `ReportArchivePage.schemaVersion` 必须为 `1`。`next` 为 `string` 或 `null`。列表项不含 `resultJson`。
- 默认查询：`displayTimezone=local`，`grouping=host`，`targetPolicy=historical`，`topN=20`，`comparison.previousEqualWindow=true`。
- 小时窗口为已闭合本地小时，日窗口为 `local_day_bounds`。同一 `(kind, range_start_utc, query_fingerprint)` 只留一份 `ok`。
- 小时档案保留 30 天，日档案保留 13 个月。过期删除只针对 `report_archive`。
- `get_report_archive` 把冻结 JSON 水合进 10 分钟 snapshot token，供现有 `export_report` 使用。不得为导出再查更新后的库。
- 每采集 tick 最多 1 份。先近后远：最近闭合小时，再最近闭合日。`failed` 可重试。
- 大结果不经实时 Channel。

### 4. Validation & Error Matrix
- 非法 `kind` / `archiveId` → `invalid_query`
- 档案 `failed` 或 JSON 缺失 → `storage_failure`
- raw / 精确 Top N 过期 → `capability_unsupported`，写 `failed`，不写假总量
- 用户取消 / deadline → `cancelled` / `deadline_exceeded`
- 磁盘不足 → `insufficient_space`，不写半份 `ok`

### 5. Good/Base/Bad Cases
- Good: 重启后 `result_json` 与生成时一致；同一小时两次调度仍一行 `ok`
- Base: 空区间 totals 为 0，coverage=`empty`，仍可归档
- Bad: 新 `ReportSnapshotStore::open(data_dir)` 清掉门面 spool；前端把 24 份小时档案加总成日总量

### 6. Tests Required
- Rust `c3::archive`：幂等、失败替换、过期删除、重启一致、next_job 顺序
- Rust `c3::query`：`local_hour_bounds` DST
- Rust `storage`：v3→v4 升级；`C3_DDL` 不含 `report_archive`
- TS `decodeReportArchivePage` 拒绝缺 `schemaVersion` / `items` / 非法 `kind`

### 7. Wrong vs Correct
#### Wrong
进页现查滚动最近 3600 秒，或把档案写进 10 分钟 `report-spool` 当历史。
#### Correct
闭合窗口的冻结 `ReportResult` 进 SQLite。进页 `list` 后 `get` 最新成功日或小时档案。

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
- `open_log_dir() -> logDir`
- `preview_delete_local_data() -> DeletePreview`
- `confirm_delete_local_data(phrase) -> DeleteReport`
- `run_user_vacuum()`

### 3. Contracts
- `AboutDto.schemaVersion` 必须为 `1`。`signed` 为 `false` 时不得写成已签名。
- `open_releases` 只返回固定 GitHub Releases URL，不注册 updater plugin。
- 删除确认短语必须是 `删除全部本地数据`。部分失败时 `allDeclaredOk=false`，文案不得写成「已全部删除」。
- 删除只清理数据目录声明对象、日志目录和当前进程凭据引用。未再确认前不写本机 Credential Manager。`open_log_dir` 不接收前端路径。
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


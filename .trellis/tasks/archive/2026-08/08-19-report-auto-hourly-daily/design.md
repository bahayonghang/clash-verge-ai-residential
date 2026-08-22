# 技术设计：分析报告自动小时与日档案

## 设计目标

1. 已闭合本地小时 / 自然日各有一份冻结 `ReportResult`，写在 SQLite，重启后仍可打开。
2. 查询、口径、导出仍只走 C3 `ReportService` / `ExportService`，不在 C4 或前端再算一套。
3. 调度挂在采集循环之后，每 tick 最多 1 份，不占 HTTP 取帧，不拖 durable commit。
4. 新表用 `user_version = 4` 前向追加。不改 `C3_DDL` / `C4_DDL` 文本。

## 模块边界

```
collector_loop_tick
    fetch_snapshot          （无 AppFacade 锁）
    apply_tick_result       （锁内：核算 + writer + C4）
    archive_tick            （见下，锁分段）

ReportArchiveService        C3 新模块，拥有档案表读写与过期删除
ReportService               复用，产出 ReportResult
ReportSnapshotStore         仅手动查询与「档案水合导出」
RetentionService            不在每个整点全量跑
AppFacade                   转发 command，不 use rusqlite
```

C2 不得直接 SQL。Recovery Shell 不构造 `ReportArchiveService` 调度。

## Schema：`user_version` 4

`c0_contract::SCHEMA_VERSION` 从 3 改为 4。新增：

```
C3_ARCHIVE_SCHEMA_VERSION = 4
C3_ARCHIVE_MIGRATION_CHECKSUM = "c3-archive-v4"
```

`storage::migrate` 在 C4 之后：

```
if current < 4 {
  execute C3_ARCHIVE_DDL
  pragma user_version = 4
  insert schema_migration(4, c3-archive-v4)
}
```

`verify_checksum` 增加 version 4。`c3/schema.rs` 的 `C3_DDL` 与 checksum `c3-report-v2` 保持原字。`c4/schema.rs` 保持原字。

表 `report_archive`（STRICT）：

| 列 | 含义 |
|---|---|
| `archive_id` | 主键，不透明 id |
| `kind` | `hour` / `day` |
| `range_start_utc` / `range_end_utc` | 半开区间 |
| `display_timezone` | 生成时所用标识，默认 `local` |
| `grouping` | 默认 `host` |
| `query_fingerprint` | 与现有 `query_fingerprint` 相同 |
| `status` | `ok` / `failed` |
| `generated_utc` | 生成时刻 |
| `data_version` | 成功时的库版本 |
| `coverage_status` | 列表用摘要 |
| `totals_upload` / `totals_download` / `connection_count` | 列表用 |
| `result_json` | 成功时完整 `ReportResult` JSON |
| `error_code` / `note_zh` | 失败时 |

唯一约束：`(kind, range_start_utc, query_fingerprint)`。  
索引：`(kind, range_start_utc desc)`。

表名加入 `C3_TABLES` / `all_table_allowlist`。过期删除只针对本表，与 `AUTO_DELETE_ENABLED` 无关。

## 默认查询

```
displayTimezone = local
grouping = host
targetPolicy = historical
comparison.previousEqualWindow = true
topN = 20
includeSessions = false
filters 全空
hour: granularity=hour, range = local_hour_bounds
day:  granularity=day,  range = local_day_bounds
```

在 `c3/query.rs` 增加 `local_hour_bounds`，算法与现有 `local_day_bounds` 相同：用 `timezone_offset_secs` + `utc_from_local_naive`，DST 不固定 3600。闭合小时是「当前本地小时起点」的前一个本地小时。

## 调度

挂在 `collector_loop_tick` 的 `apply_tick_result` **之后**，不要塞进 1 Hz 核算事务。

每个 tick：

1. 短锁：若 `branch != NormalReady` 或正在 shutdown，返回。
2. 短锁：删除过期档案（小时 `range_end_utc < now - 30d`，日 `range_end_utc < now - 396d`）。选出下一项工作：最近的缺成功档的闭合小时，否则最近的缺成功档的闭合日，否则更早缺档（先近后远）。已有 `ok` 则跳过。`failed` 可再选。无工作则结束。
3. 放锁。用 `db_path` 调 `ReportService::run`（独立 read connection + 临时 / 现有 snapshot store）。**不要**为此持 `AppFacade` 锁。
4. 短锁：`insert or replace` 档案行。成功则 `release` 临时 token。失败写 `failed` + `note_zh`。

启动后第一个采集周期即可开始补跑。每 tick 最多 1 份。30 天小时最多约 720 份、13 个月日最多约 396 份，按约 1 次/秒的采集周期分摊，不在启动时打满。

`RetentionService::run(MaterializeOnly)` 不挂在每个档案 tick 上。现实现一次事务扫即将离开 raw 的区间，放在整点全量跑会堵住 writer。30 天内默认档案走 raw。超出 raw 的日档案若日维未就绪，记 `capability_unsupported`，用户仍可在设置页手动物化后由后续 tick 重试。

## Command 合同

新增（Rust 权威校验，camelCase DTO）：

```
list_report_archives({ kind?: "hour"|"day", after?: string, limit?: number })
  -> { schemaVersion: 1, items: ReportArchiveSummary[], next: string|null }

get_report_archive({ archiveId })
  -> ReportResult
```

`get_report_archive` 把 `result_json` 反序列化后写入 `ReportSnapshotStore`，换发 10 分钟 token，供现有 `export_report` 使用。列表项不含 `result_json`、不含 secret。

`run_report` / `export_report` / `get_report(token)` 行为不变。大结果仍不进 Monitor Channel。

## 前端

`reports` 路由进入时：

1. `list_report_archives`
2. 取最新成功 `day`，否则最新成功 `hour`
3. `get_report_archive` 填总量 / 趋势表 / Top N

保留预设表单与「运行报告」。手动成功后视图标记为手动，不影响档案表。列表做最小一列：时间、类型、状态。点选再 `get`。

新 i18n 键覆盖：档案空态、补跑中、自动小时、自动日、手动、失败重试说明。中英都要加。

解码：`decodeReportArchivePage` 缺 `schemaVersion` / `items` 则失败。`decodeReportResult` 合同不变。

## 失败与回滚

- `deadline_exceeded` / `storage_busy` / `capability_unsupported` / 取消：写 `failed`，下一次选中该周期再跑。
- 磁盘不足：fail closed，不写半份 `ok`。
- 二进制回滚到不识 v4 的旧版会走 `future schema` → Recovery Shell。本任务发布后不可把 `SCHEMA_VERSION` 再降回 3。
- 功能回滚：停止调度、隐藏列表，手动 `run_report` 仍可用。已写入的 v4 表保留。

## 验证

- `local_hour_bounds`：UTC、`Asia/Shanghai`、`America/New_York` 春快 / 秋慢。
- 同一小时两次调度只有一行 `ok`。
- 进程重启后 `result_json` 字节级一致（或 totals / rankings / coverage 逐项一致）。
- 补跑顺序：先最近闭合小时，再最近闭合日。
- 过期小时 / 日不再出现在 `list`。
- `get` 后三种导出 totals 与档案一致。
- Recovery 库不创建调度、不跑 `ReportService`。
- `C3_DDL` / `C4_DDL` 文本与 checksum 未变。

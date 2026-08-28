# 模块与错误

- 稳定 identifier：`io.github.bahayonghang.residential-monitor`。
- 本机日志由 `app_log` 拥有，目录为 `%LOCALAPPDATA%\{identifier}\logs`，不复用 `data_dir`。`RESIDENTIAL_MONITOR_LOG_DIR` 可覆盖。启动时在 `AppFacade::boot` 之前 `init`。写入失败不得中断采集。`open_log_dir` 只打开该目录，不授 WebView `fs` / opener。
- 错误对前端只暴露稳定码和当前语言的下一步动作，详情脱敏。JSON 字段仍叫 `messageZh`。
- 界面语言键 `ui_locale`（`zh`/`en`）走 `put_setting`，不进控制器 JSON。`identity::PRODUCT_NAME` 与删除确认短语不随语言改。
- 外观键 `ui_theme`、`ui_font`、`ui_font_size`、`ui_density`、`ui_sidebar_width` 与实时表列布局键 `live_table_layout` 同样走 `put_setting`，不进控制器 JSON。非法值回落默认。`ui_font` 存 `system`、旧别名或校验后的本机族名。本机族名由 `list_ui_fonts` 经 GDI 枚举，失败键为 `error.font_list`。`ui_sidebar_width` 为 160–352 的整数 CSS 像素，默认 220。
- HTTP 使用成熟实现，不手写完整 HTTP/1.1 解析器。
- TCP 只接受 loopback。named pipe 不发送 secret。
- C0 候选 schema 不得复制为 C1 正式 migration。
- C2 只消费 C1：`ControllerSession`、`AccountingEngine`、`StorageCoordinator`、`LiveProjection`、`RecoveryFacade`。C2 模块不得 `use rusqlite`，不得 `create table`。
- C2 代码位于 `residential-monitor/src-tauri/src/c2/`。
- 主机 identity 由 `session_host::resolve_host_identity` 单一实现：`host` → `sniffHost` → 目的 IP，写入现有 `connection_session.host`。`ensure_session_on` 用 `prefer_host_identity` 升级空值与 IP，不得用 IP 覆盖域名。`filters.host == "__unknown__"` 匹配空 host，不把哨兵当域名绑定。不升 schema。
- 产品进程用 `c2/collector.rs` 约 1 Hz HTTP GET `/connections`。`test_controller` 只取一帧，不能代替循环。HTTP 期间不得持 `Mutex<AppFacade>`。
- `/connections` 根帧必须同时含数值型 upload/download meter 与数组型 connections；缺字段或类型错误是 `ProtocolIncompatible`，不得静默归一为 0/空列表并关闭旧连接。每个 connection 必须有非空 id 和数值型 counters；任一坏行使整帧失败，未知附加字段仍忽略。
- controller 字段覆盖诊断只输出互斥计数：Host 的 host/sniff/IP/absent、Process 的 process/path-only/absent、Chains 的 chains/provider-only/absent；不得输出 secret、完整 host/IP/processPath 或 provider 值。
- `AccountingEngine` 是每个 durable controller generation 的 canonical metadata owner。同 generation 非空字段升级、空白字段不降级；Chains 只做非空整组替换。Live row 与 MinuteFact 必须由同一 canonical snapshot 产生。`providerChains` 只作诊断，不替代 `chains`。
- `AppFacade::boot` 从 SQLite 原子保留新的 writer epoch；首次 frame、重连、meter/counter 回退、start 变化或消失后 raw id 复用时，从 SQLite 保留新的 controller epoch 后再建 baseline。进程内常量 epoch / bundle receipt 不得跨重启复用。
- `Paused` / `Resumed` / `SleepGap`、Connected 生命周期事件及瞬时 HTTP/JSON/协议错误发布时保留 `hub.rows()`、rate 与真实 `last_sample_utc`；不得把错误发生时间伪装成 controller sample。只有 `Restarted` / `Shutdown` / `Cancelled` / `CoreRestarted` 终止态清空 rows。`session_status == Cancelled` 时跳过取帧；`reconnect_now` / `resume_collector` 必须离开 `Cancelled`，不得新开第二条循环。
- `LiveOverview.observationPhase` 明确区分 unconfigured / connecting / baselinePending / current / paused / disconnected / resyncRequired / decodeFailed。每次 `connectionDelta` 携带同 tick overview；前端不得只更新 rows 而把 bootstrap 的 connecting 快照永久保留。
- `AppFacade::query` 必须经 `MonitorHub::query_snapshot` 一次锁定同时取出 rows 与 overview（含 `last_sample_utc`）。`query_connections_with_targets_at` 先过滤完整 matched 集合，再按 `(value desc, identity asc)` 选 `topDownload` / `topUpload`，然后才 sort / cursor / limit 分页。`limit` 与 cursor 不得改变 summary。空匹配热点为 `None`，不写 0。热点 DTO 不含 `process_path` 或原始规则。
- Tauri `Channel` 只放在 `lib.rs` 订阅表，不进入 `AppFacade`。
- 托盘 id `main`。Tauri 2 默认左键弹菜单，必须 `show_menu_on_left_click(false)`。左键 Up 与左键双击打开窗口，右键才是菜单。四态由 `c2::desktop::tray_chrome(collector_running, session, storage_ok)` 决定，资源是 `icons/tray-*.png`。窗口 `icon.png` 不随状态变。产品标记真源是 `icons/icon-source.png`（铺满正方形、不预做圆角）。`icon.ico` 必须含 16/32/48/256 层且 256 为 PNG 压缩，用 `just monitor-icons` 从真源生成，禁止提交单层 16×16 ICO。`scripts/check-icons.mjs` 断言层数。`just tdev` 重启后通知区才换图标。
- C3 代码位于 `residential-monitor/src-tauri/src/c3/`。C3 只通过 `StorageCoordinator` / `RecoveryFacade` 访问 SQLite，不得另建 writer 或通用 Repository。`ReportArchiveService` 拥有 `report_archive` 读写与过期删除，含 `persist_manual`。C2 不得直接写该表。
- C3 排名必须在 `LIMIT top_n` 前应用 `ReportQuery.sort`。排序字段与方向只由 `SortField` / `SortSpec` 枚举白名单生成，不接收调用方 SQL；upload / download 同值时固定以 identity 升序破同值。raw、hourly dimension、daily dimension 与 category 特例保持同一契约，默认仍为 download desc。
- 家宽判定只在 `src-tauri/src/residential.rs`：`residential_tags` / `is_residential_target`（核算，精确 target）与 `is_residential_filter`（实时筛选，精确 target 或节点名含「家宽」）。两者不得合并。`accounting::classify` 只调核算函数；`c2/query` 的「只看家宽」只调筛选函数。前端不得复制家宽字符串匹配。
- `list_routes` 与引导 DTO 共用 `c2/shell.rs` 的 `default_routes_for`。十段顺序：`overview`、`live`、`residential`、`host`、`rule`、`chain`、`process`、`reports`、`alerts`、`settings-data`。禁止再维护第二份路由表。
- `collector_loop_tick` 在 `apply_tick_result` 之后调用 `archive_tick`。`ReportService::run` 不得持 `Mutex<AppFacade>`。每 tick 最多 1 份档案。临时 snapshot 必须打开独立目录（`data_dir/archive-tick`），不得 `ReportSnapshotStore::open(data_dir)`，否则 `cleanup_orphans` 会删掉门面仍有效的 spool token。
- Recovery Shell 与 shutdown 跳过档案调度，不初始化 `ReportArchiveService` 循环。
- C4 代码位于 `residential-monitor/src-tauri/src/c4/`。`AlertEngine` 拥有告警状态机；周期用量只调用 `ReportService`；通知只经 `NotificationSink`。C4 不得另建 writer 或第二套小时 / 日 / 月聚合。
- C5 代码位于 `residential-monitor/src-tauri/src/c5/`。只做发布硬化：关于页、删除、VACUUM、故障矩阵、并发 fixture、供应链与 C0 基线核验。不得改写 C1 核算、C3 报告 / retention / backup 或 C4 告警语义。
- Recovery Shell：`restoreAvailable` 为 `true`。restore 不初始化 `ReportService`；失败必须保留当前可用库。
- C4 前向表：`alert_rule`、`alert_instance`、`alert_event`、`notification_outbox`。不得改写 C1 / C3 已发布 migration。
- AUMID 与 identifier 相同：`io.github.bahayonghang.residential-monitor`。About 固定 Releases URL，不注册 updater plugin，不新增 Windows Service。
- current-user 安装目录为 `%LOCALAPPDATA%\ResiWatch`，与 Tauri NSIS `productName` + `installMode: currentUser` 默认一致。`just tinstall` 通过 NSIS `/D=` 显式传入该路径，不沿用注册表里指向 `%TEMP%` 或旧产品名目录的上次位置。`installer.nsh` 的 `NSIS_HOOK_PREINSTALL` 在 `$INSTDIR` 位于 `$TEMP` 下时改写到该目录并搬走 `data\`。数据目录仍是 `<安装目录>\data`。identifier 与 exe 仍是 `residential-monitor`。
- 调试：`just tdev`（`tauri dev`）。出包：`just monitor-build`（只生成 NSIS，不安装）。安装：`just tinstall`（会改本机 current-user 安装态）。C5 自动门：`just monitor-c5-auto`。未再确认前不要执行 `tinstall`、本机 Credential Manager 真机测试或登录自启动写入。
- C5 完整 30 天库、24 小时 soak、安装态通知 / 签名 / GitHub Release 不得由 fixture 或 smoke 冒充完成。C0 升级基线缺失时 `monitor-bench c5-baseline` 退出码 2。

## Scenario: C3 排名排序先于 Top N

### 1. Scope / Trigger
- Trigger: `ReportQuery.sort`、任何 C3 排名 SQL 模板，或 raw / hourly / daily / category 排名路径发生变化。

### 2. Signatures
- `render_rank_sql(sql: &str, filters_sql: &str, sort: &SortSpec, layer: RankLayer) -> String`
- `fill_raw_rank(..., query: &ReportQuery, ...)`
- `fill_dimension_layer(..., query: &ReportQuery, ...)`

### 3. Contracts
- `{filters}` 只接收由 `ReportFilters` 枚举/字段生成的内部 SQL 片段；用户值只走绑定参数。
- `{order_by}` 只由 `SortField::{Upload, Download, Name, Identity}`、`SortSpec.descending` 与 `RankLayer::{Raw, Dimension}` 渲染，调用方不能传 SQL。
- upload / download 用对应层的聚合列排序，并以 identity 升序稳定破同值；name / identity 直接按第一选择列排序。
- 所有排名模板必须在 `LIMIT ?` 前完成 ORDER BY。默认 `SortSpec` 继续等价于 download desc。

### 4. Validation & Error Matrix
- 模板残留 `{filters}` 或 `{order_by}` → 测试失败；不得把带槽位 SQL 交给 SQLite。
- sort 字段超出 DTO 枚举 → 查询边界拒绝，不得回落到调用方字符串。
- raw 查询使用 dimension 别名，或反之 → SQL/EQP 测试失败。
- 非家宽高流量行出现在 `filters.category = "__residential__"` 结果 → 集成测试失败。

### 5. Good/Base/Bad Cases
- Good: `top_n=1 + upload desc` 与 `top_n=1 + download desc` 可返回不同冠军。
- Base: 未显式指定 sort 时仍返回 download desc，并保留 identity 稳定次序。
- Bad: 先按 download 固定截取 Top N，再在 Rust 或前端按 upload 重排候选。

### 6. Tests Required
- SQL corpus：8 个排名模板无残留槽位；四个字段 × 两个方向均只渲染白名单 ORDER BY。
- service raw tier：构造上传冠军与下载冠军不同的 fixture，断言 `top_n=1` 首行分别正确。
- service dimension tier：对同一 fixture 物化 hourly dimension 后重复方向断言。
- residential host：断言非家宽高流量行被过滤、域名/IP 分行，完整 Top N 时 rankings / series 与 totals 守恒。

### 7. Wrong vs Correct
#### Wrong
```rust
let mut rows = run_download_top_n(query.top_n)?;
rows.sort_by_key(|row| Reverse(row.upload));
```

#### Correct
```rust
let sql = render_rank_sql(template, filters_sql, &query.sort, RankLayer::Raw);
// SQL ORDER BY 已在 LIMIT ? 前生效。
```

## Scenario: run_report persist_manual and snapshot LRU

### 1. Scope / Trigger
- Trigger: 分析报告 / 家宽「运行报告」、告警跳转要跨重启保留结果；聚合页现查不得打满 8 格 spool。

### 2. Signatures
- `run_report(query: ReportQuery, persist_manual: Option<bool>) -> ReportResult`
- `ReportArchiveService::persist_manual(connection, result, now_utc)`
- `ReportSnapshotStore::insert` / `get` / `release`

### 3. Contracts
- `persist_manual` 缺省 false。true 时查询成功后写 `report_archive.kind=manual`，返回值仍带会话 token。失败查询不写行。
- 同一 `(kind, range_start_utc, query_fingerprint)` 的 manual 行覆盖；hour/day 已有 ok 仍不覆盖。
- 现查（`useReport`）必须 false。C2 不直接 SQL。不升 schema。
- 未过期 fingerprint 复用 token。满 8 格或超 spool 字节按 `last_access_utc` 淘汰。单 token 超 32 MiB 仍 `quota_exceeded`。TTL 600 秒。
- `get` 刷新 `last_access_utc`，因此 `get_report` / 导出路径为 `&mut self`。

### 4. Validation & Error Matrix
- 读事务仍开 → `storage_failure`
- 单 token 过大 / 淘汰后仍放不下 → `quota_exceeded`
- 未知 `list_report_archives.kind` → `invalid_query`
- Recovery-only → 既有 recovery 错误，不写档案

### 5. Good/Base/Bad Cases
- Good: 侧栏走完后再 `run_report` 成功；显式运行重启后列表可点选
- Base: `persist_manual` 省略，行为与只写 spool 相同
- Bad: 把 TTL 改成 7 天；现查也 persist；满 8 格直接拒绝且不淘汰

### 6. Tests Required
- `snapshot_store_tests`: 复用 fingerprint、第 9 次淘汰、32 MiB 拒绝
- `archive_service_tests`: manual 覆盖、7 天 purge、不挡住 auto `next_job`

### 7. Wrong vs Correct
#### Wrong
```rust
if self.items.len() >= MAX_ACTIVE_TOKENS {
    return Err(ReportError::QuotaExceeded("active token count"));
}
```
#### Correct
先 `cleanup_expired`，同 fingerprint 替换，再 LRU 淘汰，最后才 `QuotaExceeded`。


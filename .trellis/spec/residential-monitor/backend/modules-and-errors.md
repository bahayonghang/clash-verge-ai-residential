# 模块与错误

- 稳定 identifier：`io.github.bahayonghang.residential-monitor`。
- 本机日志由 `app_log` 拥有，目录为 `%LOCALAPPDATA%\{identifier}\logs`，不复用 `data_dir`。`RESIDENTIAL_MONITOR_LOG_DIR` 可覆盖。启动时在 `AppFacade::boot` 之前 `init`。写入失败不得中断采集。`open_log_dir` 只打开该目录，不授 WebView `fs` / opener。
- 错误对前端只暴露稳定码和当前语言的下一步动作，详情脱敏。JSON 字段仍叫 `messageZh`。
- 界面语言键 `ui_locale`（`zh`/`en`）走 `put_setting`，不进控制器 JSON。`identity::PRODUCT_NAME` 与删除确认短语不随语言改。
- 外观键 `ui_theme`、`ui_font`、`ui_font_size`、`ui_density`、`ui_sidebar_width` 与实时表列布局键 `live_table_layout` 同样走 `put_setting`，不进控制器 JSON。非法值回落默认。`ui_font` 存 `system`、旧别名或校验后的本机族名。本机族名由 `list_ui_fonts` 经 GDI 枚举，失败键为 `error.font_list`。`ui_sidebar_width` 为 160–352 的整数 CSS 像素，默认 220。
- HTTP 使用成熟实现，不手写完整 HTTP/1.1 解析器。
- TCP 只接受 loopback。`validate_address` 必须先拆出 `host:port`（IPv6 用 `[::1]:port`），缺 `:` 为 `invalid_address`，再对 host 做字符串白名单 `127.0.0.1` / `::1` / `localhost`。不得把 `IpAddr::is_loopback()` 的整个 127/8 当作设置页可保存地址。named pipe 不发送 secret。
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
- 家宽判定只在 `src-tauri/src/residential.rs`，实时筛选与核算写入共用一套 matcher：target 精确为 `RESIDENTIAL_SELECTOR`（`家宽`）时匹配包含该词的链路节点，其它自定义 target 只做节点全值精确匹配，空 target 集不匹配。`residential_tags` 保持 target 配置顺序并以首个命中项作为 primary；`is_residential_target` / `is_residential_filter` 不得另建分支。`accounting::classify` 与 `c2/query` 均只调用共享实现，前端不得复制字符串匹配。
- `list_routes` 与引导 DTO 共用 `c2/shell.rs` 的 `default_routes_for`。十段顺序：`overview`、`live`、`residential`、`host`、`rule`、`chain`、`process`、`reports`、`alerts`、`settings-data`。禁止再维护第二份路由表。
- `collector_loop_tick` 在 `apply_tick_result` 之后调用 `archive_tick`。`ReportService::run` 不得持 `Mutex<AppFacade>`。每 tick 最多 1 份档案。临时 snapshot 必须打开独立目录（`data_dir/archive-tick`），不得 `ReportSnapshotStore::open(data_dir)`，否则 `cleanup_orphans` 会删掉门面仍有效的 spool token。
- Recovery Shell 与 shutdown 跳过档案调度，不初始化 `ReportArchiveService` 循环。
- C4 代码位于 `residential-monitor/src-tauri/src/c4/`。`AlertEngine` 拥有告警状态机；周期用量只调用 `ReportService`；通知只经 `NotificationSink`。C4 不得另建 writer 或第二套小时 / 日 / 月聚合。`in_quiet` 必须挡住 `Activated` 与 `InstanceStatus::Active`；不得只压 outbox 仍把实例写成 Active。
- C5 代码位于 `residential-monitor/src-tauri/src/c5/`。只做发布硬化：关于页、删除、VACUUM、故障矩阵、并发 fixture、供应链与 C0 基线核验。不得改写 C1 核算、C3 报告 / retention / backup 或 C4 告警语义。
- Recovery Shell：`restoreAvailable` 为 `true`。restore 不初始化 `ReportService`；失败必须保留当前可用库。`storage.is_none()` 时 `run_report`、`save_targets`、`upsert_alert_rule`、`create_backup` 返回 `recovery_only`，不得写 `target_item` / `alert_rule`，不得把损坏热库复制为备份。
- C4 前向表：`alert_rule`、`alert_instance`、`alert_event`、`notification_outbox`。不得改写 C1 / C3 已发布 migration。
- AUMID 与 identifier 相同：`io.github.bahayonghang.residential-monitor`。About 固定 Releases URL，不注册 updater plugin，不新增 Windows Service。
- current-user 安装目录为 `%LOCALAPPDATA%\ResiWatch`，与 Tauri NSIS `productName` + `installMode: currentUser` 默认一致。`just tinstall` 通过 NSIS `/D=` 显式传入该路径，不沿用注册表里指向 `%TEMP%` 或旧产品名目录的上次位置。`installer.nsh` 的 `NSIS_HOOK_PREINSTALL` 在 `$INSTDIR` 位于 `$TEMP` 下时改写到该目录并搬走 `data\`。数据目录仍是 `<安装目录>\data`。identifier 与 exe 仍是 `residential-monitor`。
- 调试：`just tdev`（`tauri dev`）。出包：`just monitor-build`（只生成 NSIS，不安装）。安装：`just tinstall`（会改本机 current-user 安装态）。C5 自动门：`just monitor-c5-auto`。未再确认前不要执行 `tinstall`、本机 Credential Manager 真机测试或登录自启动写入。
- Windows 登录自启动由官方 Rust `tauri-plugin-autostart` 和 command-lifetime `TauriAutostartPort` 拥有，唯一参数来自 `identity::AUTOSTART_ARGUMENT`（`--background`）。前端只调用 `get_autostart_state` / `set_autostart_enabled` 自有 commands；不得安装 JS guest binding 或授予 `autostart:*` capability。`AppFacade` 不持有 adapter，`FakeAutostart` 只存在于 `#[cfg(test)]`。
- 自启动以 OS 状态为唯一真源，不写 SQLite/UI preference。set 必须 `enable|disable -> is_enabled` 回读；读取/写入失败只暴露 `autostart_unavailable` 与错误类，日志不得包含 executable path、注册表位置或平台原文。自动测试只注入 fake，不得实例化真实 manager 或写 HKCU。
- `just tinstall`、真实启动项写入和 Windows 登录验证必须另行授权；需核对安装路径、唯一 `--background`、隐藏窗口/托盘/唯一 collector 及关闭后不再登录启动。未取得该证据时保持 **UNVERIFIED**，不得归档相关验收门。
- C5 完整 30 天库、24 小时 soak、安装态通知 / 签名 / GitHub Release 不得由 fixture 或 smoke 冒充完成。C0 升级基线缺失时 `monitor-bench c5-baseline` 退出码 2。

## Scenario: Windows 登录自启动系统能力

### 1. Scope / Trigger
- Trigger: 修改 `tauri-plugin-autostart` 初始化、`AutostartPort`、自启动 commands、`--background` 生命周期或安装态验收路径。

### 2. Signatures
- `AutostartPort::{set_enabled, is_enabled} -> Result<_, AutostartError>`
- `apply_autostart(port: &dyn AutostartPort, enabled: bool) -> Result<bool, AutostartError>`
- `get_autostart_state() -> Result<AutostartStateDto, AppErrorDto>`
- `set_autostart_enabled(enabled: bool) -> Result<AutostartStateDto, AppErrorDto>`

### 3. Contracts
- Tauri builder 注册官方 Rust 插件，参数只取 `identity::AUTOSTART_ARGUMENT == "--background"`；初始化、安装和普通启动均不得隐式 enable。
- command-lifetime `TauriAutostartPort` 是生产适配器；`AppFacade` 不持有系统 adapter，`FakeAutostart` 只在 `#[cfg(test)]`。
- OS 是唯一真源，不写 SQLite 或 UI preference。set 严格执行 `enable|disable -> is_enabled`，返回回读值。
- WebView 只调用应用自有 commands；不安装 JS guest binding，不授予 `autostart:*` capability。
- `--background` 复用既有 single-instance、隐藏窗口、托盘和唯一 collector 路径，不创建第二 writer。

### 4. Validation & Error Matrix
- 启动项不存在 → get 返回 `enabled=false`，不得调用 enable/disable。
- enable/disable 或写后 readback 失败 → `autostart_unavailable`；日志只记录 operation 与稳定错误类。
- 原始错误包含 executable path、Run key 或平台文本 → IPC 与日志均不得包含原文。
- 自动测试 → 只注入 fake，不实例化真实 manager、不写 HKCU。
- `just tinstall`、启动项写入或真实登录 → 必须先取得用户授权，并区分命令采集证据与用户人工登录证据。

### 5. Good/Base/Bad Cases
- Good: 用户确认开启，plugin enable 成功且回读 true；真实登录以 `--background` 隐藏进入托盘并保持唯一 collector。
- Base: 新安装无启动项，安装器不写 Run key，设置页回读 false。
- Bad: 初始化插件时自动 enable；把请求值当成功状态；前端直接调用 guest plugin；单测写真实 Run key。

### 6. Tests Required
- Rust port/core：默认只读、enable/disable、写后回读、write/readback failure。
- IPC/log：注入 path/registry/platform 原文，断言稳定 code/错误类且敏感原文缺失。
- 静态边界：生产 `AppFacade` 无 Fake、builder 参数唯一、前端无 guest dependency、capability 无 `autostart:*`。
- 安装态门：用命令核对安装器默认不启用及 executable/参数；启用与关闭各完成一次真实登录验证。未取得证据时保持 **UNVERIFIED**。

### 7. Wrong vs Correct
#### Wrong
```rust
manager.enable()?;
Ok(AutostartStateDto { enabled: true })
```

#### Correct
```rust
apply_autostart(&port, enabled)
    .map(|enabled| AutostartStateDto { enabled })
// apply_autostart 在写入后以 is_enabled 回读 OS 真值。
```

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
- category 为空且链路不命中 target 的高流量行出现在 `filters.category = "__residential__"` 结果 → 集成测试失败；category 非空的历史归属仍保持权威。

### 5. Good/Base/Bad Cases
- Good: `top_n=1 + upload desc` 与 `top_n=1 + download desc` 可返回不同冠军。
- Base: 未显式指定 sort 时仍返回 download desc，并保留 identity 稳定次序。
- Bad: 先按 download 固定截取 Top N，再在 Rust 或前端按 upload 重排候选。

### 6. Tests Required
- SQL corpus：8 个排名模板无残留槽位；四个字段 × 两个方向均只渲染白名单 ORDER BY。
- service raw tier：构造上传冠军与下载冠军不同的 fixture，断言 `top_n=1` 首行分别正确。
- service dimension tier：对同一 fixture 物化 hourly dimension 后重复方向断言。
- residential host：断言 legacy-null + 已保存链路可恢复、多个 target / 节点不倍增、非命中高流量行被过滤、域名/IP 分行，完整 Top N 时 rankings / series 与 totals 守恒。

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

## Scenario: fail-closed address, quiet hours, Recovery writes

### 1. Scope / Trigger
- Trigger: `validate_address`、`AlertEngine::transition` 的静默窗口、或 Recovery Shell 下的 `run_report` / `save_targets` / `upsert_alert_rule` / `create_backup`。

### 2. Signatures
- `validate_address(address: &str) -> Result<(), SettingsError>`
- `AlertEngine::transition` / `in_quiet(rule, now_utc) -> bool`
- `AppFacade::{run_report, save_targets, upsert_alert_rule, create_backup}`

### 3. Contracts
- 地址必须含 `host:port`。host 白名单仅 `127.0.0.1` / `::1` / `localhost`。缺 `:` 为 `invalid_address`，非白名单为 `non_loopback`。
- `quiet_start_min > quiet_end_min` 为跨日窗口。窗口内不得发出 `Activated`，实例不得进入 `Active`。
- `storage.is_none()` 时四个写入口返回 `recovery_only`，SQLite 无新行，备份路径不得被创建。

### 4. Validation & Error Matrix
| Condition | Result |
| --- | --- |
| `127.0.0.1` 无端口 / `not-an-addr` | `invalid_address` |
| `127.0.0.2:9097` / `8.8.8.8:9097` | `non_loopback` |
| 静默窗口内三连击 | 无 `Activated`，非 `Active` |
| Recovery `create_backup` | `recovery_only`，目标文件不存在 |

### 5. Good/Base/Bad Cases
- Good: `[::1]:9097` 可保存；窗口外三连击仍激活。
- Base: NormalReady `create_backup` 仍写出 checksum。
- Bad: 缺 `:` 报 `non_loopback`；静默只压通知仍把实例写成 Active；Recovery 复制损坏热库。

### 6. Tests Required
- `validate_address_*` / `validate_targets_*`
- `quiet_hours_overnight_window_does_not_activate`
- `recovery_only_write_entry_points_return_recovery_only`
- `kill_after_facts` / `kill_after_outbox` / `kill_after_alerts` 回滚计数

### 7. Wrong vs Correct
#### Wrong
```rust
let host = address.rsplit_once(':').map(|(h, _)| h).unwrap_or(address);
reject_non_loopback(host)?;
```
#### Correct
```rust
let Some((host, _)) = address.rsplit_once(':') else {
    return Err(SettingsError::InvalidAddress);
};
```


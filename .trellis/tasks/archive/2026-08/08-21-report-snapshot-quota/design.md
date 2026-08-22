# 设计：报告快照配额与一周保留

## 1. 边界

| 层 | 职责 |
|---|---|
| `ReportSnapshotStore` | 进程内 spool：复用 fingerprint、LRU、过期清理。不升 TTL。 |
| `useReport` / `useReportArchive` | 持有 token；卸载、换 query、取消响应必须 `release_report`。 |
| `ReportArchiveService` | `kind=manual` 写入与 7 天过期删除。自动 hour/day 调度不变。 |
| 前端解码 / 档案列表 | 接受 `manual`；进页仍优先最新成功日/小时档案。 |

C2 不直接 `use rusqlite`。手动落库只经 `ReportArchiveService`。`ReportService::run` 仍不得持 `Mutex<AppFacade>`。不改 `C3_ARCHIVE_DDL`、不升 `user_version`。

## 2. Spool：复用与 LRU

`insert` 顺序：

1. 拒绝读事务仍开。
2. `cleanup_expired(now)`。
3. 单结果 `> MAX_TOKEN_BYTES` → `QuotaExceeded("single token too large")`。
4. 未过期且 `fingerprint` 相同：原地替换 `result` / bytes / checksum / spool 文件，刷新 `expires_utc` 与 `last_access_utc`，返回原 token。
5. 若 `len >= MAX_ACTIVE_TOKENS` 或 `total_bytes + bytes > MAX_SPOOL_BYTES`：按 `last_access_utc` 升序淘汰，直到能插入或无可淘汰。正在 `get` 调用栈内的 token 不淘汰（导出先 `clone` 再写文件）。
6. 仍放不下 → `QuotaExceeded`。
7. 新 token，TTL 仍 600 秒。

`get` 刷新 `last_access_utc`。`release` 行为不变。

8 格保留。峰值：概览 3 次现查、维度页父+下钻 2 次；route switch 卸载后应释放。LRU 覆盖 StrictMode 双发、取消后仍返回的 insert、档案水合。

## 3. 前端 token 生命周期

`useReport` 在 effect 内记住本次 `reportSnapshotToken`：

- 新结果成功且与旧 token 不同：释放旧 token。
- cleanup / `cancelled` / 序号过期：对已返回的 token 调 `release_report`，不写入 state。
- 卸载释放当前 token。

`useReportArchive` 在 `runQuery` / `selectArchive` / `loadArchives` 换结果时释放旧 token；现有 `release()` 接到换结果路径上，不再只作为死导出。

`run_report` 在请求已取消后仍会在后端 insert。前端必须释放那次返回值，否则 StrictMode 每页泄漏一倍。

## 4. 档案水合与导出

`get_report_archive` 仍 `insert` 冻结 JSON，以便当前会话导出走既有 `preview_export` / `export_report(token)`。依赖第 2 节复用与 LRU，不再与现查抢到 `quota_exceeded`。

进页 `loadArchives(true)` 拆开两种失败：

- `list_report_archives` 失败 → `report.archive.unavailable`。
- 列表成功、水合失败 → 保留 `archives`；结果区用配额/存储原文；不得把 `statusZh` 改成列表不可用。

`RankBarCard`：`能力说明` 只读 `drilldownCapability.noteZh`。`errorZh` 只进空态 / `role=alert`，不进能力说明。

## 5. 手动档案 `kind=manual`

产品决定 B：点「运行报告」写入 `report_archive`，按 `generated_utc` 保留 7 天，跨重启可打开。

### 写入入口

`run_report` 增加可选 `persist_manual: bool`（缺省 false）。`AppFacade` 在 `ReportService::run` 成功后，若为 true，再调 `ReportArchiveService::persist_manual`。失败查询不写行。

`persist_manual=true` 的调用方：

- 分析报告「运行报告」与告警跳转 `runQuery`
- 家宽页「生成报告」

`useReport` 现查（概览、四聚合页、家宽聚合）传 false。

### 行语义

复用表 `report_archive`。`ArchiveKind` 增 `Manual`（kebab `manual`）。`grouping` 按查询原值，不改自动档案默认 host。

唯一键仍是 `(kind, range_start_utc, query_fingerprint)`，无需新索引。同一分钟窗、同一 query 再跑：覆盖 `result_json` 与 `generated_utc`（与 hour/day「已有 ok 不覆盖」相反）。窗差一分钟则新行。

`next_job` 只走 hour/day。手动行不参与补跑。

`purge_expired` 增加：`kind = 'manual' and generated_utc < now - 7 * 86400`。常量 `ARCHIVE_MANUAL_RETAIN_DAYS = 7`。手动写入成功后也跑一次 purge。过期删除仍只针对本表。

### 列表与进页

解码 `ReportArchiveKind` 接受 `manual`。筛选增「手动」。进页 `pickLatestArchive` 仍只选成功日、否则成功小时，不自动选手动行。手动行出现在列表，点选后 `get_report_archive`。

列表 kind 列：「手动」。当前会话来源行仍用「本次手动查询」。

## 6. 数据流

```
现查 useReport
  → run_report(persist=false) → spool insert → 页面 DTO
  → unmount / 换 query / 取消 → release_report

手动运行
  → run_report(persist=true) → spool insert
  → persist_manual → report_archive kind=manual
  → 前端刷新列表，当前结果 source=manual

打开档案
  → list（不占 token）
  → get_report_archive → spool insert（fingerprint 复用 / LRU）
  → 导出走 token，不重查
```

## 7. 兼容

- 不改已发布 C1 / C3 / C4 / `C3_ARCHIVE_DDL` 文本，不升 `SCHEMA_VERSION`。
- 旧库 `kind` 为 text，写入 `manual` 即可。
- 同一次发布必须同时改 Rust 枚举与前端解码，否则列表遇到 `manual` 会整页解码失败。
- `.trellis/spec/residential-monitor/frontend/view-state.md` 中「手动运行报告不写 report_archive」一句改为：显式运行写入 `kind=manual`，7 天过期；现查不写。
- `docs/reporting.md` 同步：手动结果落库、spool 复用/LRU、token 仍 10 分钟。

## 8. 权衡

- TTL 改 7 天会让泄漏在重启前一直占满 8 格。TTL 保持 600 秒，一周留在 SQLite。
- 规则/链路/进程继续 raw 现查。7 天窗已在 30 天 raw 内。不为三维加自动小时档案。
- 导出仍要 token，所以档案水合仍 insert；用复用+LRU 满足「不因 8 格失败」，不新增 `export_report(archiveId)`。

## 9. 回滚

回滚 PR 后：旧客户端不写 `manual`，不读该 kind。已写入的 `manual` 行留在表内，新客户端 purge 仍按 `generated_utc` 删除。spool 行为回到满额拒绝；可重启进程清空 HashMap。

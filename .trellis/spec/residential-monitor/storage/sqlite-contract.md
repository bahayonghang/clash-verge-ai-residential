# SQLite 契约

启动后每个连接显式设置：

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = FULL;
PRAGMA foreign_keys = ON;
```

- 单 writer。逐行只做 bind → step → reset。
- 缺口不得写成零。
- Positive sampling coverage requires an explicit `covered` interval from consecutive valid observations, committed with the facts. The first sample is a baseline; gap, pause, generation change and clock rollback break continuity. Merge within UTC days rather than append one row each second. Legacy `epoch`/`closed` events do not prove observation; do not backfill positive historical coverage from them. Frozen-day guards also apply to zero-fact coverage mutations.
- Online Backup 必须分页。不得复制热库文件并丢掉 WAL。
- 未来 schema 或 checksum mismatch 必须 fail closed。
- `busy_timeout` 由 C0 测量后冻结，不能超过 durable commit SLO 仍称为健康。
- SQLite `user_version`：C1 = 1 / checksum `c1-core-v1`；C3 = 2 / checksum `c3-report-v2`；C4 = 3 / checksum `c4-alert-v3`；C3 档案 = 4 / checksum `c3-archive-v4`；账本生命周期 = 5 / checksum `ledger-lifecycle-v5-layout3`。不得改写已发布 migration 文本。`C3_DDL` 不得出现 `report_archive`。
- C3 追加表：`dimension_dict`、`connection_session_attr`、`traffic_hourly_dimension`、`traffic_daily_dimension`、`traffic_daily_core`、`coverage_daily`、`retention_state`、`retention_watermark`、`report_snapshot_meta`。
- C3 档案表（v4 `C3_ARCHIVE_DDL`）：`report_archive`。过期删除只针对该表，与 `AUTO_DELETE_ENABLED` 无关。`kind` 合法值 `hour` / `day` / `manual`。hour 按 `range_end_utc` 保留 30 天；day 按 `range_end_utc` 保留 13 个月；manual 按 `generated_utc` 保留 7 天。写入 `manual` 不升 schema、不改已发布 DDL。
- `ReportSnapshotStore`：未过期 `query_fingerprint` 复用 token 并续 TTL。满 `MAX_ACTIVE_TOKENS=8` 或总字节超 `MAX_SPOOL_BYTES` 时按 `last_access_utc` 淘汰后再插入。单 token 超 `MAX_TOKEN_BYTES` 仍 `quota_exceeded`。`TOKEN_TTL_SECS` 保持 600。
- C4 追加表：`alert_rule`、`alert_instance`、`alert_event`、`notification_outbox`。facts、coverage、alert 与 outbox 必须在同一 writer 事务中提交。
- `report_snapshot_token` 返回前必须关闭 SQLite read transaction。token 不持有连接或 WAL end mark。
- 自动 DELETE 保持关闭（`AUTO_DELETE_ENABLED=false`），直到守恒门通过。不自动 VACUUM。freelist 不得显示为已释放文件空间。
- 低空间 backup / restore / spool / VACUUM 必须 fail closed，不得覆盖当前可用库。
- 每个打开的连接在 `apply_required_pragmas` 之后注册 SQLite 标量函数 `last_chain_hop`（`Deterministic | Innocuous`）。新建 `StorageCoordinator` 连接后直接执行含该函数的 SQL 不得报 `no such function`。
- 同一连接还必须注册 `chain_identity`（`Deterministic | Innocuous`）。`last_chain_hop` 只给 Rule group：单跳返回 NULL 以回退 raw rule；`chain_identity` 给 Chain：单跳保留自身，多跳取末个非空 hop。`filters.chain`、raw rank、字典 intern、hourly materialization 与质量统计必须共同使用 `chain_identity`。过滤值只走绑定参数，禁止字符串插值。`namedSql` 回显常量名。
- `connection_session_attr` 的 host/process/rule/network/chain 采用字段级 non-null merge；空白 metadata 不得清除同 generation 已知值。非空 Chains 是整组 replace，不做逐 hop union。`host_id` 从 canonical `connection_session.host` intern，避免 raw Host 与物化 Host 分叉。
- `persist_slice` 每个 writer 事务只读取一次当前 `target_set(set_id=1).policy_version`，并与 live row 的 `primary_category_id` 一起写入/更新 `connection_session_attr`；未配置时为 0。不得为每条连接重复查询 policy，也不得在 target 更新后继续把新归属固定写成版本 0。
- 一次性 Chain 修复用版本 marker `chain_identity_v1`。只处理 raw 与既有派生层的交集，并在单个 `BEGIN IMMEDIATE` 中删除旧 hourly chain、intern/rebuild hourly、删除并从完整 hourly day 重建 daily、校验重建前后、raw→hourly 与 hourly→daily upload/download 守恒，最后写 marker。既有 hourly 与当前 raw 总量不等说明存在不完整原始旁证，必须拒绝重建并回滚；任一失败回滚数据和 marker，raw 已删除区间与 `report_archive.result_json` 不改。
- `dimension_dict` 新增 kind `'chain'` 与 `'rule_group'`，不得覆写既有 `'rule'`。精确维度层物化 host / process / `rule_group` / chain / network；水位键 `hourly_dim_v2`。排名 LEFT JOIN `dimension_dict`，缺失 identity 为 `"__unknown__"`，`dimension_dict.value` 不得写入该哨兵。`filters.host` 为该哨兵时 raw 层匹配 `coalesce(s.host,'')=''`，维度层匹配 `h.dimension_id = 0`。`filters.process` 为该哨兵时 raw 层匹配进程缺失谓词，维度层匹配 `h.dimension_id = 0`。`filters.category` 为 `"__residential__"` 时，raw 层使用唯一 `RESIDENTIAL_RAW_MEMBERSHIP_SQL`：`primary_category_id` 非空的历史行直接命中；仅对 category 为空的 legacy 行以 `EXISTS(connection_chain + target_item)` 恢复，target=`家宽` 做包含匹配，其它 target 精确匹配，空 target 集不恢复。维度层仍匹配 `category_id != 0`；哨兵不得 intern。
- raw 恢复谓词必须使用相关 `EXISTS`，不得把 `connection_chain` / `target_item` 连接到外层后造成多节点或多 target 倍增。它只覆盖仍保留 `connection_minute + connection_chain` 的 raw 区间，不批量回填用户库，也不从 Host/IP/进程猜测分类；raw 已删除且历史 category 为空的区间继续按现有能力边界返回未知/不支持。
- 家宽份额 named SQL `share_residential_raw` 一次扫描同时取分子与分母。分子与 `filters.category="__residential__"` 必须注入同一个 `RESIDENTIAL_RAW_MEMBERSHIP_SQL`，分母仍是窗口内全部可归因观测。
- named SQL `audit_residential_host_rule_process` 是家宽 host/规则类型/进程 identity 联合投影。窗口与 `AUDIT_MAX_ROWS=200000` 走绑定参数，家宽谓词注入 `RESIDENTIAL_RAW_MEMBERSHIP_SQL`。返回行数等于上限时 `truncation.status=truncated`，守恒字段为 `null`。
- `monitor-db` 写路径在操作期间持有 `PRAGMA locking_mode = EXCLUSIVE`，冲突 fail closed。`restore` / `vacuum` / `purge` 要求 `--offline-confirmed`，CLI 不验证 ResiWatch 是否已退出。
- CLI 查询不得写入 `ReportSnapshotStore`，以免淘汰桌面端报告 token。

## Scenario: Shared snapshot token leases

### 1. Scope / Trigger

Two views can acquire the same fingerprint token. Releasing one acquisition must not invalidate the other view.

### 2. Signatures

`ReportSnapshotStore::insert` acquires one private lease on success, including refresh/hydration that reuses a token. `get` and export do not acquire leases. `release(token)` accepts one outstanding lease; the last release removes the in-memory record and spool.

### 3. Contracts

Token reuse still refreshes its result and TTL; it is not an immutable snapshot identity. Balance each successful insert even when the returned token string matches the previous token. TTL and LRU eviction are unconditional and ignore lease count. Keep 600-second TTL, eight tokens, 32 MiB per token and 128 MiB total. Replacing a token must enforce total quota by evicting other LRU entries without selecting the replacement itself.

### 4. Validation & Error Matrix

| Condition | Required result |
| --- | --- |
| Two owners, first release | Shared token remains readable |
| Last owner releases | Token and spool are removed |
| TTL/LRU expires a leased token | Forced eviction; old holder receives existing missing-token behavior |
| Replacement exceeds total quota | Other LRU entries leave before successful replacement |
| Single result exceeds 32 MiB | Existing `quota_exceeded` error |

### 5. Good/Base/Bad Cases

Good: acquire refreshed token, then release the previous acquisition even for equal strings. Base: one insert and one release. Bad: treating token equality as permission to skip release.

### 6. Tests Required

Cover two-owner release, repeated same-token refresh, leased TTL/LRU eviction, and replacement pressure with actual spool-file byte reconciliation.

### 7. Wrong vs Correct

Wrong: `if (oldToken !== newToken) release(oldToken)`.

Correct: release every previous acquisition after successful replacement; a token string identifies the shared record, not an individual lease.

## Scenario: Durable metadata and receipt lifecycle

### 1. Scope / Trigger

Unchanged metadata must not rewrite session attributes/chains on every sample. Receipt/session cleanup must preserve retry and controller-lifecycle evidence.

### 2. Signatures

- `AccountingEngine::{dirty_metadata, confirm_metadata, closed_sessions}` expose pending canonical metadata and closure evidence.
- `StorageCoordinator::{retire_abandoned_generations, replace_controller_epoch}` persist owner-proven retirement.
- `StorageCoordinator::{prune_receipts, cleanup_retired_sessions}` perform bounded transactional maintenance with protected pending references.
- `StorageCoordinator::contains_session_id` resolves newly seen inactive connection IDs against the durable `(epoch_id, connection_id)` index before accounting consumes the input.
- v5 adds `committed_bundle.committed_utc`, `bundle_epoch.{expired_through_seq,expired_payload_digest,retired_utc}` and `controller_epoch.retired_utc`.
- `storage::durable_data_version(&Connection)` is the shared read boundary for writer allocation, reports, period usage and read-only CLI versions.

### 3. Contracts

Only a confirmed original bundle clears metadata dirtiness. Preserve the exact payload/hash on an unknown result. Resolve it before processing queued accounting inputs. Defer at most the existing eight-batch capacity, including the pending bundle; overflow exposes backpressure and a coverage gap. Generation decisions follow input order, independently of the newest UI health status. Lifecycle-only events cannot confirm stale display metadata after a target-policy change.

Accounting memory follows active connections, not lifetime connection churn. Reuse detection uses the indexed durable ledger only for IDs absent from the active engine. Preserve the original queued input if that lookup or generation reservation fails; zero-traffic sessions still establish durable identity.

Keep the union of receipts from the latest 24 hours and latest 100,000 commits. Advance the persisted expiration boundary/digest in the same transaction as deletion. Retained epoch evidence still rejects expired retries when its receipt table is empty. Contiguous sequence means contiguous, not the maximum observed sequence. Legacy commit timestamps remain NULL; retirement observation time is not a fabricated commit timestamp.

The v5 receipt table uses `data_version INTEGER PRIMARY KEY` and a unique `(writer_epoch, bundle_seq)` identity. Rebuild it transactionally from released schemas without changing versions, payload hashes or inventing commit times. Duplicate legacy versions fail the migration and roll back; earlier experimental v5 checksums fail closed.

The v5 migration removes `idx_connection_minute_utc`, whose `(utc_minute, session_pk)` keys exactly duplicate the existing primary-key index. Preserve the released v2 DDL; drop the redundant index only in the forward migration. Minute-range queries retain ordered primary-index access without a temporary sort.

The legacy `data_version.watermark` row is a durable floor. Current version is the maximum of that floor and the newest durable receipt, read in the caller's transaction. Do not rewrite the floor every commit or read it alone as the current version. Keeping the latest 100,000 receipts protects the current maximum through pruning. Fresh production-schema corpus fixtures leave the floor at zero to exercise this contract.

Positive coverage extension seeks only the newest `(kind, reason, interval_id)` row, using an immutable index. Extend it only when the endpoint matches and the start is within the same UTC day; otherwise insert a new interval. Do not index the per-tick mutable endpoint merely to locate that row.

Raw reports load one session projection, then scan `connection_minute` once through its primary minute index for each window. That scan produces totals, attribution, series, rankings and, for host/rule/process, exit sums. Do not join session or attribute rows once per minute. When the projection span, including a retained previous equal window, is at most 3120 minutes (two 26-hour days), restrict that projection to `session_pk` values distinct in the span. The restricted statement still uses the minute primary index, then integer primary-key lookups. A longer span keeps the full session projection so a 30-day report does not read the minute index twice. Sessions outside the projected span must not affect distinct counts, rankings, or comparison bytes. The diagnostic name stays `raw_session_projection`. Missing-attribution predicates must be non-null and session-constant within the read snapshot, so known bytes and distinct-session count equal total minus missing. Preserve independent equivalence tests across dimensions and filters; do not sum hourly or daily distinct counts to obtain a longer-window exact count. Diagnostic `namedSql` must name the statements actually executed: `raw_session_projection`, `raw_minute_scan`, and `coverage_raw`; add `sessions_keyset` only when the session page query runs.

Delete session/attribute/chain rows only after raw references have left, closure or durable retired-controller evidence exists, and active/pending references are absent. Legacy NULL `ended_utc` alone proves neither activity nor closure. Reopening storage for maintenance or a failed restore must not silently discard pending/deferred inputs.

Auxiliary cleanup bounds scanned candidate rows as well as deleted rows, persists its scan cursor transactionally, and shares cancellation/deadline handling. Protected prefixes must not permanently starve later eligible rows.

Read candidate pages of at most 128 rows within the existing 1,000-candidate cap, reuse prepared per-row SQL, and advance the cursor only through rows actually processed. The maintenance owner cooperatively yields session work at 100 ms and receipt work at 200 ms before its unchanged shared 250 ms VM progress-handler deadline. This threshold is not a hard wall-clock bound: measure whole-call latency, preserve overruns in evidence, and do not infer their cause without profiling. Yielded prefixes may commit atomically with deletes or expiry digests; user cancellation still rolls back the current transaction. A fixed all-or-nothing 1,000-session fanout must not repeatedly time out and lose its cursor progress.

A SQLite progress handler that remains cancelled can also interrupt transaction-drop rollback. At the maintenance owner boundary, clear the handler and explicitly roll back any still-open owned transaction before returning an error; verify autocommit and unchanged rows/watermarks. Do not leave a cancelled transaction on the shared writer.

### 4. Validation & Error Matrix

| Condition | Required result |
| --- | --- |
| Metadata unchanged | Zero actual attr/chain modifications |
| Failed/unknown commit | Exact bundle preserved; no double accounting on retry |
| Receipt sequence gap | Expiration cannot cross unproven sequence |
| Legacy NULL time, no retired owner proof | Retain receipt/session |
| Active zero-traffic or pending session | Protect session and references |
| Cancellation/deadline during cleanup | Roll back watermark/digest and deletion together |

### 5. Good/Base/Bad Cases

Good: canonical metadata changes once and is confirmed only after durable commit. Base: stable samples write facts/coverage/receipt without rewriting attributes/chains. Bad: mark every frame dirty, clear dirtiness on rollback, or retire an unresolved writer during a failed maintenance operation.

### 6. Tests Required

Assert actual SQLite attr/chain changes, non-null/host/chain/policy/generation transitions, exact unknown-commit replay, mixed snapshot/disconnect queues, overflow gaps, pending maintenance refusal, receipt window union/holes/empty-epoch reopen, and legacy/active/raw/pending session protection. Exercise 10,000 connection churn with bounded active state and durable zero-traffic ID reuse.

Also test receipt-layout migration rollback, positive legacy floors, zero-floor fresh writes, version exhaustion, pruning/reopen/unknown-result versions, read-transaction snapshots across all reader paths, and bounded coverage-tail behavior at UTC-day and clock-rollback boundaries.

### 7. Wrong vs Correct

Wrong: persist every live row and clear dirty state immediately after attempting commit.

Correct: persist the pending canonical metadata subset, retry the same bundle until confirmed, then acknowledge that subset before interpreting the next queued input.

## Scenario: Exact reports after raw deletion

### 1. Scope / Trigger

Raw configuration age is not proof that raw rows still exist. All public queries, share calculations and previous-window comparisons consult actual frozen/deleted-day evidence.

### 2. Signatures

- `ReportService` capability planning uses `retention_state` day evidence as well as the configured retention period.
- `traffic_daily_core.category_id = 0` is the total; positive categories are separate subtotals. Never sum total and subtotals together.
- Internal `query_period_usage` returns bytes and coverage without exposing nonrecoverable count/duration scalars.

### 3. Contracts

Increasing raw retention from 30 to 90 days does not restore deleted detail. Use retained complete raw where allowed; otherwise exact public aggregate reports require recoverable daily totals and ranking scalars. One complete UTC day can use daily core totals and daily dimension rankings. Hourly series are allowed only where selected rows preserve exact distinct-minute duration; overlapping identities/categories cannot be repaired by summing scalars. Arbitrary multi-day distinct sessions and category-overlap durations are unsupported once their source detail has left.

Previous-period unavailability is represented by the existing absent comparison, not zero. Residential share must not turn retained coverage without raw traffic into a covered zero. Period alerts only require bytes/coverage and may combine supported raw and verified aggregate intervals, with the end clamped to now.

Retained raw series use the same minute scan. Bucket labels follow SQLite integer division toward zero, including negative minutes. Preserve distinct session and minute counts, zero-byte observations, and omission of empty buckets. The scan shares the report snapshot, cancellation handler and whole-report deadline; moving to the next bucket must not reset that budget. Keep the former grouped query as an independent test oracle, not a runtime fallback. Exit selection sums download by raw `chain_key`, ignores blank keys, and picks the greatest sum with `chain_key` ascending as the tie break.

### 4. Validation & Error Matrix

| Condition | Required result |
| --- | --- |
| Deleted day becomes nominally Raw after settings change | Aggregate capability or explicit unsupported; never raw zero |
| Core total plus one category subtotal | Return total once |
| Same session spans hours | Daily exact count; never sum hourly distinct counts |
| Identities share a minute/category boundary | Reject unavailable scalar exactness rather than sum duplicated duration |
| Deleted previous interval unavailable | Absent comparison |

### 5. Good/Base/Bad Cases

Good: use daily core for whole-day totals and exact daily ranking rows. Base: complete retained raw answers the original query. Bad: sum all core rows or relabel a missing raw result as covered zero.

### 6. Tests Required

Test deletion followed by a larger raw setting, share over deleted days, missing previous windows, category total/subtotal arithmetic, a session spanning hours, overlapping identity durations, and byte-only monthly alerts spanning available tiers. Compare the single-pass raw fold to the grouped SQL oracle across groupings, filters, signed ranges, empty and zero-byte buckets, ranking order and exit ties. The minute scan plan must use the primary minute index and must not build a whole-window grouping sort.

### 7. Wrong vs Correct

Wrong: `sum(connection_count)` over hourly rows and call it the day's distinct-session count.

Correct: use the exact daily count computed while the complete raw day was retained; reject ranges whose distinct set can no longer be recovered.

## Scenario: Resumable daily retention

### 1. Scope / Trigger

Expired history must make bounded progress without monopolizing the writer or deleting detail before exact daily statistics are verified.

### 2. Signatures

- `RetentionService::run_chunk(coordinator, now_utc, raw_retain_days, mode, space, cancel) -> RetentionChunk` reports the day, actual raw rows deleted and whether work remains.
- v5 staging tables: `retention_build`, `retention_build_aggregate`, `retention_build_member`. Only one day is active.
- Exact-day evidence uses `retention_state.layer = 'day_exact_v1'`; `raw_delete` records the sealed write boundary.

### 3. Contracts

Build and independently check bounded raw pages with a durable cursor, exact session/minute membership and per-dimension/category counters. Preserve the complete raw day through publication, actual-output audit and coverage confirmation. A session spanning hours counts once in the daily result; overlapping minute memberships count once for that exact group.

Publish only verified values. Protect already published output through every later confirmation phase, with exemptions only for the builder's own transactional mutations. Track publication ownership before discarding unconfirmed output; invalidation during initial build must not erase pre-existing legacy aggregates. Changed raw/metadata/coverage evidence fails closed and permits a safe rebuild from complete retained evidence.

The `deleted` day state seals query/write behavior before bounded physical row deletion finishes; it does not prove zero remaining raw rows or released disk bytes. Readers must use the verified aggregate for such a day, even while residual raw awaits deletion. Every resumed destructive phase checks the current mode and gate. Production `AUTO_DELETE_ENABLED` remains false until required conservation and capacity evidence passes; a `cfg(test)` helper may exercise the identical deletion path on isolated fixtures.

Coverage source paging and auxiliary cleanup advance durable source cursors, with query plans that enforce ordered bounded scans. Coverage-only days and remaining cleanup batches must continue without requiring a dimensional row. Raw/detail retention keeps the existing 30-day default/90-day maximum, dimensions and raw coverage keep 396 days, and low-cardinality daily core/coverage remain long-term. DELETE produces reusable pages; VACUUM remains explicit and separate.

Auxiliary cleanup cursors use `-1` for a dirty sweep that must restart, positive rowids for a bounded sweep in progress, and `0` only for a completed sweep since its last invalidation. Final raw/prune completion and session reference removal invalidate the relevant cursor in the same transaction. A cursor wrap before the last reference disappears is not proof of completion. The facade and isolated capacity driver re-read pending auxiliary work after ledger cleanup, because that later step can invalidate the earlier retention result. Disabled deletion must not create a fast retry loop from a persisted dirty cursor.

### 4. Validation & Error Matrix

| Condition | Required result |
| --- | --- |
| Cancel, busy, low space, chunk deadline | Roll back that chunk and its cursor/evidence |
| Corrupt any published dimension/core/coverage before confirmation | No raw deletion or seal advancement |
| Reopen a persisted delete job with gate off | Zero further destructive changes |
| Invalidate before publication ownership | Preserve legacy output |
| Last raw hour/day verification fails | Retain the entire unconfirmed raw day |
| Coverage-only or final auxiliary batch | Continue bounded cleanup without a dimension candidate |
| Later pruning or session cleanup releases a dictionary reference | Restart the bounded sweep before reporting no pending work |

### 5. Good/Base/Bad Cases

Good: many bounded chunks finish one exact day, then delete its detail in bounded batches. Base: no expired candidate yields no work. Bad: a textual interval checksum authorizes deletion, a LIMIT hides an unbounded prior sort, or a partial raw day overwrites a frozen complete aggregate.

### 6. Tests Required

Exercise cross-hour sessions, per-dimension/category byte and count conservation, coverage overlap/gaps, restart at each destructive boundary, mode/gate changes, output corruption between publication/audit/coverage/confirmation, invalidation recovery, coverage-only cleanup, multi-page dictionary pruning across days, post-ledger reference removal with protected keys, bundled SQLite query plans and physical fixture integrity. Full 30-day capacity and installed soak remain separate evidence from focused regressions.

### 7. Wrong vs Correct

Wrong: delete each raw hour as soon as its hourly sums match, then sum hourly distinct counts for the day.

Correct: retain every raw hour until daily exact membership and actual published aggregates are confirmed, then resume bounded deletion using that frozen evidence.

# SQLite 契约

启动后每个连接显式设置：

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = FULL;
PRAGMA foreign_keys = ON;
```

- 单 writer。逐行只做 bind → step → reset。
- 缺口不得写成零。
- Online Backup 必须分页。不得复制热库文件并丢掉 WAL。
- 未来 schema 或 checksum mismatch 必须 fail closed。
- `busy_timeout` 由 C0 测量后冻结，不能超过 durable commit SLO 仍称为健康。
- SQLite `user_version`：C1 = 1 / checksum `c1-core-v1`；C3 = 2 / checksum `c3-report-v2`；C4 = 3 / checksum `c4-alert-v3`；C3 档案 = 4 / checksum `c3-archive-v4`。不得改写已发布 C1 / C3 / C4 migration 文本。`C3_DDL` 不得出现 `report_archive`。
- C3 追加表：`dimension_dict`、`connection_session_attr`、`traffic_hourly_dimension`、`traffic_daily_dimension`、`traffic_daily_core`、`coverage_daily`、`retention_state`、`retention_watermark`、`report_snapshot_meta`。
- C3 档案表（v4 `C3_ARCHIVE_DDL`）：`report_archive`。过期删除只针对该表，与 `AUTO_DELETE_ENABLED` 无关。
- C4 追加表：`alert_rule`、`alert_instance`、`alert_event`、`notification_outbox`。facts、coverage、alert 与 outbox 必须在同一 writer 事务中提交。
- `report_snapshot_token` 返回前必须关闭 SQLite read transaction。token 不持有连接或 WAL end mark。
- 自动 DELETE 保持关闭（`AUTO_DELETE_ENABLED=false`），直到守恒门通过。不自动 VACUUM。freelist 不得显示为已释放文件空间。
- 低空间 backup / restore / spool / VACUUM 必须 fail closed，不得覆盖当前可用库。
- 每个打开的连接在 `apply_required_pragmas` 之后注册 SQLite 标量函数 `last_chain_hop`（`Deterministic | Innocuous`）。新建 `StorageCoordinator` 连接后直接执行含该函数的 SQL 不得报 `no such function`。
- 同一连接还必须注册 `chain_identity`（`Deterministic | Innocuous`）。`last_chain_hop` 只给 Rule group：单跳返回 NULL 以回退 raw rule；`chain_identity` 给 Chain：单跳保留自身，多跳取末个非空 hop。`filters.chain`、raw rank、字典 intern、hourly materialization 与质量统计必须共同使用 `chain_identity`。过滤值只走绑定参数，禁止字符串插值。`namedSql` 回显常量名。
- `connection_session_attr` 的 host/process/rule/network/chain 采用字段级 non-null merge；空白 metadata 不得清除同 generation 已知值。非空 Chains 是整组 replace，不做逐 hop union。`host_id` 从 canonical `connection_session.host` intern，避免 raw Host 与物化 Host 分叉。
- 一次性 Chain 修复用版本 marker `chain_identity_v1`。只处理 raw 与既有派生层的交集，并在单个 `BEGIN IMMEDIATE` 中删除旧 hourly chain、intern/rebuild hourly、删除并从完整 hourly day 重建 daily、校验重建前后、raw→hourly 与 hourly→daily upload/download 守恒，最后写 marker。既有 hourly 与当前 raw 总量不等说明存在不完整原始旁证，必须拒绝重建并回滚；任一失败回滚数据和 marker，raw 已删除区间与 `report_archive.result_json` 不改。
- `dimension_dict` 新增 kind `'chain'` 与 `'rule_group'`，不得覆写既有 `'rule'`。精确维度层物化 host / process / `rule_group` / chain / network；水位键 `hourly_dim_v2`。排名 LEFT JOIN `dimension_dict`，缺失 identity 为 `"__unknown__"`，`dimension_dict.value` 不得写入该哨兵。`filters.host` 为该哨兵时 raw 层匹配 `coalesce(s.host,'')=''`，维度层匹配 `h.dimension_id = 0`。
- 家宽份额 named SQL `share_residential_raw` 一次扫描同时取分子与分母。分子条件 `a.primary_category_id IS NOT NULL`。

# PRD：采集缺口行去重与覆盖并集修复

来源：`08-25-home-ip-traffic-breakdown/research/findings.md` 发现 3 缺陷 B、发现 4。

## 背景与症状

- 家宽页聚合区（累积用量 / 占比 / TARGET 排名 / 趋势）全显示「未知」与「该区间无采集覆盖」。
- 概览页显示「覆盖 partial，缺口 52536600 秒」（约 608 天，明显失真）。

## 根因（已定位，三处）

1. **写入侧**：断连 / 睡眠期间每个输入帧都产出一条 `CoverageChange{gap, disconnect_or_sleep}`（`residential-monitor/src-tauri/src/accounting.rs:218-234`），逐条落库为 `coverage_interval` 新行且 `ended_utc=NULL`。现库存量 **29,187 条**未闭合行（1787071476–1787101208，约 8.3 小时，每秒一条）。落库点（started_utc 盖戳 + insert）在 storage / facade ingest 路径，实施时先定位。
2. **读取侧**：覆盖计算对 gap 行逐条求和、不去重（`c3/share.rs:116-129`、`c3/service.rs:614-628`）。29,187 条重叠开放 gap 使 `covered_sec=0` → `uncovered`；概览缺口 = 29,187 × 1,800（30 分钟窗口每条各计一个窗口长），与显示精确吻合。
3. **存量数据**：29,187 条历史脏行需要一次性事务修复，先例是 `retention.rs` 的 `repair_chain_identity_v1`（watermark 标记 + 失败回滚）。

## 需求

- R1 写入侧：同一断连期只保留一行开放 gap；断连帧重复到达时扩展既有行（不新增）；恢复采集时把开放行 `ended_utc` 闭合。
- R2 读取侧：`covered_sec` / `gap_sec` 按区间并集计算（先合并重叠区间再求和），`share.rs` 与 `service.rs` 两处口径一致。
- R3 存量修复：事务性合并既有未闭合 gap 行（同因相邻合并为一行），失败整体回滚并保留 watermark，不改动 `connection_minute` 数据。
- R4 回归测试：模拟每秒断连帧与恢复，断言行数、闭合行为、并集计算；覆盖守恒（covered + gap = span）。

## 验收标准

- A1 每秒断连帧注入 N 帧，`coverage_interval` 只新增 1 行。
- A2 恢复采集后该行 `ended_utc` 非空。
- A3 本机库（29,187 条脏行）修复后行数收敛为个位数，家宽页聚合不再显示「该区间无采集覆盖」（配合 target 已改为 `家宽-SOCKS5` 的前提）。
- A4 概览缺口回到合理量级（等于真实未采集区间并集）。
- A5 `cargo test` 全量通过，含新增回归测试。

## Out of Scope

- 核算口径 target 配置问题（用户侧已处理，非代码缺陷）。
- 数据目录迁移（另一任务 `08-25-data-dir-out-of-temp`）。
- coverage_daily 物化层如受影响，只做伴随修正，不重构保留层。

## Open Questions

- 无阻塞项。开放 gap 行的唯一性约束方式（partial unique index vs 应用层查询）留 `design.md`。

# Design：采集缺口行去重与覆盖并集修复

## 现状机制（已定位）

1. **写入**：`storage.rs:619-624` 对每个 `CoverageChange` 无条件 insert，`started_utc=本帧 utc`、`ended_utc=NULL`。断连期间每秒一个 Disconnected 帧 → 每秒一行。全库不存在任何写 `ended_utc` 的路径（`closed` 行同样永远开放）。
2. **读取**：`c3/share.rs:116-129` 与 `c3/service.rs:614-628` 对 gap 行逐条求和、不去重。29,187 条重叠开放行把 `covered_sec` 压成 0 → 家宽页 `uncovered` 全未知；概览缺口 52,536,600 = 29,187 × 1,800。
3. **存量**：29,187 条开放 gap 行（1787071476→1787101208）需要一次性合并。

## 修改设计

### 写入侧（storage.rs `persist_slice`）

- **去重**：insert 前查同 `(kind, reason)` 且 `ended_utc IS NULL` 的开放行，存在则跳过。断连 8 小时只产生 1 行，`started_utc` 即断连起点。
- **闭合**：切片处理后执行 `update coverage_interval set ended_utc = ?utc where ended_utc is null and kind not in (本切片出现的 kind)`。恢复采集的正常帧不含 gap → 开放 gap 行在恢复时刻闭合；暂停→断连切换时 `closed` 行同理被闭合。
- `epoch`（core_restart）行适用同一规则：重启风暴只留一行，恢复后闭合。

### 读取侧（共享并集助手）

- 新助手放 `c3/query.rs`（`CoverageSlice` 所在地）：`pub fn gap_union_sec(win_start, win_end, slices) -> i64`——取 kind=gap 的 `(max(started, win_start), min(ended ?? win_end, win_end))`，滤正、排序、合并重叠、求和。
- `share.rs::covered_sec_from_slices` 与 `service.rs::summarize_coverage` 改为调用该助手：`covered = span - gap_union`。两处口径不再漂移。
- 开放行按 `ended = win_end` 处理，语义不变。

### 存量修复（retention.rs，沿用 `repair_chain_identity_v1` 模式）

- watermark 层名 `coverage_open_gap_v1`。
- 按 `(kind, reason)` 分组开放行：保留一行 `started_utc = min(started)`、`ended_utc = max(started)`（最后一行出现时刻 ≈ 实际恢复时刻），删除组内其余行。仅一行的组不动（真开放，交给写入侧闭合）。
- 单事务 + watermark 标记 + 失败整体回滚，不触碰 `connection_minute`。
- 触发点与 `repair_chain_identity_v1` 相同（RetentionService 入口），保证既有调用路径自动获得修复。

### 不改的东西

- `coverage_daily` 物化 SQL（retention.rs:478-487）：写入侧去重后行不再重叠，逐行求和自然等于并集。
- `COVERAGE_RAW` 查询、告警引擎、DTO 结构：消费方只看 slices 与 covered/gap 数值。

## 权衡

- 闭合时刻用「恢复采集那一帧的 utc」，精度为一帧（1 秒），足够覆盖口径用途。
- 读取侧并集把「缺口」从 608 天修正为真实未采集区间（本机约 8.3 小时）；家宽页 `covered_sec` 随之 > 0，`partial` 状态如实反映存在缺口。
- 修复把 29,187 行收敛为 1 行；`coverage_daily` 已物化的错误数值由既有 `insert or replace` 在下次物化时自然覆盖。

## 回滚

- 三处改动相互独立：读取侧（并集）单独回滚即恢复旧口径；写入侧回滚后重新出现每秒一行（不再叠加旧数据）；修复层有 watermark，重跑幂等。

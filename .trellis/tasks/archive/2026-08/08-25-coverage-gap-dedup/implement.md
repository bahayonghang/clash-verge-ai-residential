# Implement：采集缺口行去重与覆盖并集修复

## 顺序清单

1. `c3/query.rs`：新增 `gap_union_sec(win_start, win_end, slices) -> i64` + 单测（重叠、相邻、开放行、窗口裁剪、空集）。
2. `c3/share.rs`：`covered_sec_from_slices` 改用助手；单测改为断言并集语义。
3. `c3/service.rs`：`summarize_coverage` 改用助手；补重叠开放行单测。
4. `storage.rs::persist_slice`：gap/closed/epoch 去重 + 恢复闭合；单测（两帧断连一行、恢复闭合、kind 切换闭合）。
5. `retention.rs`：`repair_coverage_open_gaps_v1`（watermark `coverage_open_gap_v1`，事务 + 回滚，仿 `repair_chain_identity_v1`）；接入 RetentionService 入口；单测（多行收敛一行、单行不动、注入失败回滚）。
6. 验证：
   - `cargo test` 全量。
   - 真实库副本演练：修复前后 `coverage_interval` 行数（29,187 → 每组 1 行）、share 查询 `covered_sec > 0`、概览窗口 `gap_sec` 回到 ~29,732s 量级。
7. 提交 + 归档。

## 验证命令

```bash
cd residential-monitor/src-tauri && cargo test
```

## 风险文件与回滚点

- `storage.rs::persist_slice`：热路径，每帧执行；闭合 UPDATE 走 `ended_utc IS NULL` 谓词，修复后行数极小，无性能顾虑；回滚 = 还原该函数。
- `retention.rs`：新修复层失败会整体回滚并保留 watermark，不影响既有 `repair_chain_identity_v1`。

## start 前检查

- prd / design / implement 就绪；`implement.jsonl` / `check.jsonl` 已加真实条目。

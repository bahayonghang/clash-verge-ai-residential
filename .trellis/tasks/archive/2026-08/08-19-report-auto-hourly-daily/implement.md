# 实施计划：分析报告自动小时与日档案

## 启动前门禁

- [x] 用户已确认本文件所在规划摘要（Goal / 范围 / AC / 关键决定）。
- [x] 之后才允许 `task.py start`。不得在同一轮规划里 start。
- [x] 实施前加载 `trellis-before-dev`，读 residential-monitor backend / frontend / storage 对应 spec。

## 执行顺序

### 1. 前向 migration v4 与档案表

- [x] 新增 `c3-archive-v4` DDL：`report_archive` STRICT、唯一键、列表索引。
- [x] `SCHEMA_VERSION = 4`。`migrate` 在 C4 之后追加。checksum 校验覆盖 v4。
- [x] `C3_TABLES` / `all_table_allowlist` 加入表名。
- [x] 不改 `C3_DDL`、`C4_DDL`、`c3-report-v2`、`c4-alert-v3` 字符串。
- [x] 测试：空库升到 4；已有 v3 库升到 4；v5 未来库 fail closed；C3/C4 checksum 仍匹配。

**回滚点**：仅 schema。停在这里不影响采集与手动报告。已应用 v4 不 down migrate。

### 2. 时区边界与默认查询

- [x] `local_hour_bounds`，复用 `timezone_offset_secs` / `utc_from_local_naive`。
- [x] 默认自动 `ReportQuery` 构造函数（小时 / 日），fingerprint 稳定。
- [x] 测试：上海整点、纽约春快 / 秋慢、`local` 与 `UTC`。

**回滚点**：纯函数，无表写入。

### 3. ReportArchiveService

- [x] `insert` 幂等：已有 `ok` 不覆盖。`failed` 可被成功替换。
- [x] `list` keyset，默认近→远。摘要不含 `result_json`。
- [x] `get` 返回冻结 JSON。
- [x] `purge_expired`：小时 30 天，日 396 天。
- [x] `next_job`：先近后远，先小时后同距离的日（见 design：先最近闭合小时，再最近闭合日）。
- [x] 测试：幂等、失败重试、过期删除、重启读同一 JSON。

**回滚点**：服务可停用；表可留空。

### 4. 调度接入采集循环

- [x] `collector_loop_tick` 在 `apply_tick_result` 之后调用 `archive_tick`。
- [x] 生成时不持 `AppFacade` 锁。每 tick 最多 1 份。
- [x] Recovery / shutdown 跳过。
- [x] 不在 tick 里调用全量 `RetentionService::run`。
- [x] 测试：模拟 now 跨过小时 / 日边界各产生一份；重复 tick 不双写；锁分段（至少用测试证明 apply 返回后才跑报告，或报告路径使用独立 reader）。

**回滚点**：去掉 `archive_tick` 调用，command 仍可读已有档案。

### 5. IPC 与前端

- [x] `list_report_archives` / `get_report_archive`。`get` 水合进 snapshot store。
- [x] TS DTO + decode 测试。
- [x] 进 `reports` 路由自动 list + get。空态区分「无闭合周期 / 补跑 / 失败」。
- [x] 列表点选；类型标记；手动「运行报告」不写档案。
- [x] zh / en 文案。
- [x] 导出仍走现有 token 路径。

**回滚点**：隐藏列表，保留手动查询。

### 6. 文档与质量门

- [x] 更新 `docs/reporting.md`：自动档案口径、保留期、与 spool token 的区别。
- [x] 实施结束后由 finish-work 更新 `.trellis/spec`（sqlite-contract、dto-and-decoding、modules-and-errors）。规划阶段只列，不先改 spec。
- [x] `just monitor-check`、`just ci`、`npm run check:secrets`。

## 验证命令

```
npm --prefix residential-monitor run typecheck
npm --prefix residential-monitor run lint
npm --prefix residential-monitor test
npm --prefix residential-monitor run build
cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check
cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib c3::
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace
just monitor-check
just ci
npm run check:secrets
```

浏览器：进分析报告页应看到最新档案（库里已有闭合小时数据时）；点「运行报告」仍能出手动结果；重启后再进页内容仍在。无浏览器工具时用上述测试 + 说明未做真机点击。

## 风险文件

- `residential-monitor/src-tauri/src/storage.rs`（migrate / SCHEMA_VERSION）
- `residential-monitor/src-tauri/src/c0_contract.rs`
- `residential-monitor/src-tauri/src/c3/schema.rs`（只追加 allowlist，不改 `C3_DDL`）
- `residential-monitor/src-tauri/src/lib.rs`（collector tick + commands）
- `residential-monitor/src-tauri/src/c2/facade.rs`
- `residential-monitor/src/main.ts`
- `residential-monitor/src/dto.ts`

## start 前检查

- `prd.md` 无未决 Open Questions。
- `design.md` / `implement.md` 已写。
- `implement.jsonl` / `check.jsonl` 有真实 spec 条目。
- 用户已明确批准规划摘要。

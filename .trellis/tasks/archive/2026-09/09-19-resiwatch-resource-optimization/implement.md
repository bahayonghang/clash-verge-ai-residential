# Implementation plan

状态：in_progress。2026-09-19 用户回复「审阅完毕，开始实施」，task.py start已完成。授权产品代码与隔离验证，不包含现有安装库写操作、部署、提交或发布。

2026-09-20 实施检查点：热路径、展示生命周期、守恒保留与后台调度代码已实施并通过独立复核；aux-series完整 `just ci` 为frontend300、Rust534+process3、root139，稀疏metadata修复后的完整gate为frontend300、Rust537+process3、root139。容量生成、A50/A250/A1000首日恢复、45天raw退出及隔离VACUUM已有实测；字典扫描完成信号已修复，同hash老化副本确认全部1024个孤立值退出。稀疏metadata UPDATE已通过SQLite索引写入证明，三对同fixture迭代分别减少candidate SQLite写33.38%/12.56%/17.64%，A1000 ingest p95由53.0364降至31.4212ms。10k/1Hz/1800秒峰值配对完成，CPU/SQLite写/native private p95/ingest p95分别下降36.51%/69.94%/4.89%/29.52%，candidate无frame overrun。11对旧变化矩阵仍保留失败，A250/A1000完整30天报告仍超时，WebView、安装态soak、真实worker队列与报告非空窗口阶段仍待证据。各身份与失败逐项见 `research/acceptance-status.md`，整体尚未验收，自动删除门仍关闭。

2026-09-20 续作检查点：专项 Rust 复核确认档案积压已使用紧凑 `ArchiveDescriptor`、有界队列、单在途与公平重试，已有22项档案回归；逐桶 raw series 已有3项等价性/取消回归。此次没有合理的新增产品代码；新增 `research/rust-continuation-20260920.md` 固化证据。29日 A=1000 series 从35,604.236ms降至26,297.359ms且摘要一致，但 totals/attribution 30,567.243ms、host ranking 53,544.078ms仍超过10秒门，后续SQL改动必须先取得阶段计划和语义等价证据。任务继续保持 `in_progress`，AC7/AC8未宣称通过。
2026-09-24 续作检查点：`raw_fold` 将家宽过滤下推到会话投影，并按会话只计一次 distinct。新 `monitor-db` SHA256 `2D77910A368C949C0E75C44CFFC2F828BA83A9A3E2972FC56EA8BFA1FBBB2D16`。同一 A250 30 天库 host 排名 21 次全部 exit 0，最慢墙钟 9166.963 ms；network 6430.812 ms；A50 1252.404 ms。未生成 A1000。自动删除仍关闭，未提交、未安装。AC7 矩阵/AB 与安装态门仍未完成。
安装态仍未授权：WebView、真实 collector/后台 worker、窗口隐藏恢复和 24 小时 soak 不能在当前授权下执行。`AUTO_DELETE_ENABLED` 保持 false，容量门与守恒门都通过之前不打开。
2026-09-24 当前二进制复测：`monitor-bench` SHA256 `4C17FA3F7A8A37FD461387B7068F428031D7B2939718D0AEC8B5A45682878EC3`。冻结基线 exe 已缺失，比值对照归档 JSON，不是同窗口 AB。三轮主场景 fixture hash 一致、300 次提交、守恒、无 overrun；相对归档 baseline，CPU 合计 15.40625→1.8125 秒，SQLite 写 70,595,256→32,718,048 B。11 对矩阵 hash 一致且守恒，但 110% 门仍失败；A1000 metadata ingest p95 为 284.837 ms，高于 2026-09-20 candidate 的 53.036 ms。非空 1 分钟报告生产首读 7690.685 ms，成本在 raw 查询，不在 plan/finalized/version/close。详见 `research/performance-20260924-current-binary/note.md`。AC7/AC8 仍未通过。

## Ordered work

1. **基线 AC1。** 确认源码SHA/安装哈希、保留用户dirt；monitor-bench加入真实facade/档案路径fixture。补表/索引B/row、WebView进程树、窗口状态、文件级I/O归属。冻结同源baseline；当前短样本不能替代AB基线。
2. **无损热路径 AC2–4。** 依次改档案due/backlog/retry、metadata变更提交、内部query绕过spool、展示可见性/单在途。每项先加入捕获实际重复行为的回归。storage/facade同一Rust负责人串行，前端在DTO约定后可独立写。
3. **保留回收 AC5–6。** 补分维/分类/coverage真实守恒、关闭chunk水位、完整raw上的整日精确聚合确认、首次物化、session关闭/旧epoch持久退役证明、receipt持久过期边界；必要时加新migration。先覆盖跨小时同session、最后小时失败仍保留整日raw、NULL-ended历史会话升级及活跃零流量保护、二次清理与过期重复bundle，再验证取消/空间/重放并接低频维护。完整门通过后才开自动DELETE。同步CLI预览、能力显示、规格与中英文对应说明。
4. **性能容量 AC7–8。** 同输入AB三轮各5分钟，完整30天A=50/250/1000，跨期限fixture，10k/1Hz/30分钟峰值，24h安装态soak。未达目标继续定位；短smoke不能代替长期或安装态门。
5. **审查交付。** 独立复核核算/删除/receipt重放及各AC；执行just ci，改站点文档才补docs-build；写验收结果并同步规范。保留`.gitignore`用户修改，提交/发布/换安装包/真实维护遵从后续授权。

## Expected file ownership

- Rust热路径：`residential-monitor/src-tauri/src/lib.rs`、`c2/facade.rs`、`accounting.rs`、`storage.rs`、`c3/archive.rs`、`c3/service.rs`、`c4/period.rs`。
- Rust保留：`c3/retention.rs`、`c3/query.rs`、`c3/schema.rs`、`dbcli/maint.rs`及真实migration/open/recovery调用点；不扩展到路由/凭据。
- 基准：`bench.rs`、`src/bin/monitor-bench.rs`及相关fixture，所有压力库写隔离临时目录。
- 前端：`residential-monitor/src/app.tsx`、`src/hooks/use-live-page.ts`、`use-report.ts`、`use-residential-share.ts`，必要lifecycle DTO/decoder和能力说明；组件不直接invoke。
- 规格/文档：`.trellis/spec/residential-monitor/`、`residential-monitor/docs/reporting.md`、`data-directory.md`；涉及站点时同步`docs/`与`docs/en/`。

## Validation commands

以下为实施计划，规划阶段未执行。先最小相关回归，最后完整gate；无需在无新变化时重复跑相同全量门。

```powershell
rtk cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib c3::
rtk cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib c4::
rtk cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib c5::
rtk cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib crash_commit_unknown_retry_same_bundle_once
rtk cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check
rtk cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings
rtk cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace
rtk just ci
```

保留已有unknown commit、重复bundle、facts/outbox原子性、host/chain/target/coverage回归。既有基准命令须执行前核对--help，全部使用隔离目录：

```text
monitor-bench replay --active 250 --hz 1 --duration 60s --profile c1 --synchronous full
monitor-bench generate --average-active 250 --days 30 --out <isolated-dir>
monitor-bench analyze --all-generated --dir <isolated-dir>
monitor-bench c5-concurrent --dir <isolated-dir>
monitor-bench c5-soak-smoke --dir <isolated-dir>
```

旧replay不是AC1/7的真实链路，新增基准完成后再给出精确命令；c5 smoke不是24h证据。预计验证本身包含至少24小时soak和30分钟峰值，代码工期待同源baseline后估计。

## Planning checks and start gate

本轮只运行任务结构/上下文路径校验及文本diff检查；它们不证明产品优化或数据安全门。implement.jsonl/check.jsonl只登记spec/research。

```powershell
rtk proxy python .trellis/scripts/task.py validate .trellis/tasks/09-19-resiwatch-resource-optimization
rtk git diff --check
```

无阻塞产品问题，用户已审阅本规划并批准实施。技术风险已进入AC5/6/8，不算已解决。

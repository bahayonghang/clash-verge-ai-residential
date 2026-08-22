# 实施：报告快照配额与一周保留

验证命令：`just monitor-check`。不打开自动 DELETE。不改已发布 migration 文本。

回滚点：每阶段保持 `just monitor-check` 可通过。阶段 B 未完成时不要只改前端解码 `manual`。

## 阶段 A：spool 复用与 LRU

1. `SnapshotRecord` 增加 `last_access_utc`。`get` 刷新该字段。
2. `insert`：过期清理 → 超 32 MiB 拒绝 → 同 fingerprint 未过期则原地替换并续 TTL → 否则 LRU 淘汰至可插入 → 仍失败才 `QuotaExceeded`。
3. 单测：第 9 个不同 fingerprint 插入成功且 `active_count()==8`；同 fingerprint 不增加 count；过期 token 在 insert 时被清掉；32 MiB 单 token 仍拒绝。
4. 风险文件：`c3/snapshot.rs`、`c3/query.rs`（常量不改值）。

## 阶段 B：前端释放 token

5. `useReport`：成功替换、effect cleanup、取消/过期响应均 `release_report`。取消后仍返回的 token 必须释放。
6. `useReportArchive` 换结果时释放旧 token。
7. 抽出可测 helper（持有 token 集合的 begin/finish/cancel），单测覆盖「第二次成功不累计」「取消释放」。
8. `RankBarCard`：能力说明只用 `drilldownCapability.noteZh`；`errorZh` 只进空态/alert。补测。
9. `loadArchives(true)`：列表失败才 `report.archive.unavailable`；水合失败保留 `archives`，结果区用原错误码文案。
10. 风险文件：`hooks/use-report.ts`、`hooks/use-report-archive.ts`、`dimension/rank-bar-card.tsx`。IPC 仍只在 hooks。

## 阶段 C：手动档案 7 天

11. `ArchiveKind::Manual`，`as_sql`/`parse`/`serde` 为 `manual`。`next_job` 不产生 manual。
12. `ARCHIVE_MANUAL_RETAIN_DAYS = 7`。`purge_expired` 按 `generated_utc` 删 manual。hour/day 仍按 `range_end_utc`。
13. `persist_manual`：成功结果清 `report_snapshot_token` 后 upsert；同唯一键覆盖；失败查询不写。写入后 purge。
14. `run_report` 增加可选 `persist_manual: bool`，缺省 false。`AppFacade` 成功后再 persist。C2 不写 SQL。
15. 单测：覆盖写入、再跑同窗覆盖、7 天后 purge、`next_job` 忽略 manual 行、失败不落库。
16. 风险文件：`c3/archive.rs`、`c2/facade.rs`、`lib.rs`。不改 `C3_ARCHIVE_DDL`。

## 阶段 D：列表、家宽、文案、文档

17. `ReportArchiveKind` 与解码接受 `manual`。`ArchiveKindFilter` 增加 `manual`。列表 kind 列「手动」；会话来源行仍「本次手动查询」。
18. `pickLatestArchive` 仍只选 day/hour。`runQuery` / 家宽生成报告传 `persistManual: true`。现查 `useReport` 不传或 false。运行成功后刷新列表，不改当前选中为自动档案，除非用户点选。
19. `docs/reporting.md`：手动 7 天、spool 复用/LRU、TTL 仍 10 分钟。
20. 规划收口时改 `.trellis/spec/residential-monitor/frontend/view-state.md`：删掉「手动不写 report_archive」，改为显式运行写 `kind=manual`、7 天过期、现查不写。
21. 风险文件：`dto.ts`、`format/report-view.ts`、`reports/archive-list.tsx`、`i18n/zh.ts`+`en.ts`（键成对）、`docs/reporting.md`、上述 spec。

## 阶段 E：门

22. `just monitor-check`。
23. 手测（实现后、归档前）：冷启动 概览 → 家宽 → 主机 → 规则 → 链路 → 进程 → 分析报告，三页无配额文案；运行报告、重启、筛选手动能打开同一结果并导出。

## `task.py start` 前

- [x] `prd.md` 已收敛，无阻塞 Open Questions
- [x] `design.md` / `implement.md` 已写
- [x] `implement.jsonl` / `check.jsonl` 已填真实 spec/research
- [ ] 用户批准本规划摘要

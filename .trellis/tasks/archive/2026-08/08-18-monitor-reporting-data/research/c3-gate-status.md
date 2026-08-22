# C3 Gate 状态

生成时间：2026-08-18。完整 30 天 `A=50/250/1000` 重跑、24 小时 soak、代码签名与 GitHub Release 未执行。

## Gate

| Gate | 结果 | 证据 |
|---|---|---|
| 1 查询契约 | 通过 | `c3::query::query_contract_tests`；能力过期返回 `capability_unsupported` |
| 2 前向 schema | 通过 | `storage_schema_creates_only_core_tables`；C1 checksum 仍为 `c1-core-v1`；无 alert/outbox 表 |
| 3 ReportService / SQL corpus | 通过 | golden totals；EQP 无 AUTOMATIC INDEX；命名 SQL 无 OFFSET |
| 4 Snapshot token | 通过 | token 返回前关闭事务后可 PASSIVE checkpoint；TTL / quota fail closed |
| 5 报告 UI | 通过 | `reports` 路由可用；图表与数据表读同一 `ReportResult`；TS `dto.test.ts` / `routes.test.ts` |
| 6 流式导出 | 通过 | 同一 token 的 CSV/JSON/HTML totals 一致；取消清理 partial；低空间 fail closed |
| 7 精确保留 | 通过 | 物化幂等；`AUTO_DELETE_ENABLED=false`；中断后续跑；freelist 不宣称为已释放空间 |
| 8 备份 / Recovery restore | 通过 | Online Backup；坏候选 / 未来 schema / 低空间 / 取消 fail closed；`restore_available=true` |
| 9 真实规模并发 | 部分 | fixture 级 writer + report + checkpoint 通过。完整 30 天三档库未重跑（暂停条件） |

## 暂停项

- 未重跑完整 30 天 `A=50 / 250 / 1000` 库、13 个月 rollup 与长期 core daily。
- 未跑 24 小时 soak、代码签名、GitHub Release。
- 未执行 `just tinstall`、本机登录自启动或 Credential Manager 真机写入。

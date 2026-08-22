# C5 跨层集成审查

审查链：

```text
controller frame
  → AccountingEngine
  → CommitBundle
  → SQLite facts + coverage + alerts + outbox
  → LiveProjection / ReportService / AlertEngine
  → AppFacade DTO
  → UI / CSV / JSON / HTML / Windows notification
```

## 结论

| 项 | 结果 |
|---|---|
| controller meter 与 attributed 分字段 | 保持。前端概览分开展示，缺口为「未知」 |
| 分类 + 其他守恒 | C1 核算未改 |
| coverage 不写零 | C1–C4 测试与 C5 故障矩阵均要求 `coverage_written_as_zero=false` |
| 周期告警复用 C3 | C4 `period` 仍只调用 ReportService |
| snapshot token 不持长事务 | C3 既有测试；C5 未改查询 |
| WebView / 托盘无第二套账本 | 前端 store 只缓存 DTO |
| secret 不进 DTO / 诊断 / 导出 | C4 诊断扫描与 C5 about / 文档保持 |

未发现需要退回 C1–C4 改写语义的分叉。C5 只增加删除、VACUUM、关于页和发布验证入口。

# 母任务设计：整改编排

## 边界

母任务只编排。产品改动发生在子任务。跨子任务冲突写在这里，不靠目录顺序暗示。

## 文件冲突

| 文件 | 先做 | 后做 |
| --- | --- | --- |
| `storage.rs` | 子任务 1 事务回滚 | 子任务 10 语句复用与物化事务 |
| `credential.rs` / `collector.rs` / `session.rs` | 子任务 2 HTTP/鉴权 | 子任务 12 清零与锁中毒 |
| `c2/facade.rs` 报告/备份路径 | 子任务 4 放锁与 cancel | 子任务 11 日志 |
| `clash-verge-ai-residential.js` | 子任务 5 | 无后续子任务改同一文件（9 只改 skill/CI/docs） |

## 数据流不变式

- 单 writer SQLite。失败必须 rollback，不得留下开启事务。
- `ReportService::run` 不得持 `Mutex<AppFacade>`。
- TCP 只 loopback。secret 不进日志、SQLite、Channel。
- 前端不重算 Top N；排序在 `LIMIT` 之前由后端完成。
- 公开模板与 example TOML 的 `server`/`username`/`password` 只能是 `""` 或 `"xxx"`。

## 回滚

每个子任务独立提交。母任务对照失败时只重开未验收子任务，不整批 revert。

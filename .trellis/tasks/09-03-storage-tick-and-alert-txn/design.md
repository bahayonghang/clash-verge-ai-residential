# 设计：物化事务与语句复用

## 边界

`storage.rs` persist/intern、`c3/retention.rs` 物化、`c4/store.rs` + `facade.rs` upsert_rule。依赖子任务 1 的 rollback 模式。

## 数据流

- 物化：hourly 已有 Immediate；daily/core/coverage 复用同一模式。
- 规则：`upsert_rule` SQL 移入 `commit_alert_bundle` 的 slice，先内存引擎，bundle 失败则引擎不暴露新规则（先写内存则失败时 reload from db）。
- statement：`StorageCoordinator` 持 `Option<Statement>` 若干，`open` 时 prepare，`persist_slice` reset。

## 回滚

bundle 失败 rollback 后 `load_rules` 覆盖内存引擎。

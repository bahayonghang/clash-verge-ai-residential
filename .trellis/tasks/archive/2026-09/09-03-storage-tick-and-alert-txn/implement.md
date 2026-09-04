# 实现：物化事务与语句复用

1. 确认子任务 1 已合并到当前分支。
2. retention daily/core 外包 Immediate。
3. `AlertCommitSlice` 增加可选 rule upsert；facade 不再单独 `upsert_rule` execute。
4. coordinator 缓存 intern/session/attr/chain statement。
5. 验证：`cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace` 中 storage/c3/c4。

# metadata UPSERT 索引写入证据

## 结论

当前 `SQL_ATTR_UPSERT` 在任意 metadata 变化时把七个字段全部列入 `UPDATE SET`。SQLite 3.53.2 会按 SET 字段集合打开并维护相关索引，即使字段值没有变化。对 host+chain 工作负载，改为一条只包含实际变化字段的固定内部列名 UPDATE，可减少四个不变字段索引写入；七字段同时变化时仍只执行一条 UPDATE，避免逐字段 UPDATE 的退化。

## 隔离实验

测试由 `storage::metadata_index_write_tests` 使用项目捆绑 SQLite 执行：250 条 attr、10 个 FULL/WAL 事务、每次更新全部250条；结果摘要在完整 UPSERT 和 sparse UPDATE 间相同。实验库由测试临时目录生成，没有使用安装库。

SQLite 的 `EXPLAIN` 也记录了 `OpenWrite`：完整 UPSERT 会打开七个 attr 索引；host+chain sparse SQL (`update ... set host_id=?,chain_key=?`) 只打开 `idx_session_attr_host`、`idx_session_attr_chain_identity` 和受 chain 表达式影响的 `idx_session_attr_rule_group`。

| 场景 | 写法 | WAL | 帧 | wall |
| --- | --- | ---: | ---: | ---: |
| host+chain | 完整 UPSERT | 374,952 B | 91 | 33.00 ms |
| host+chain | 一条 sparse UPDATE | 210,152 B | 51 | 31.64 ms |
| 七字段全变 | 完整 UPSERT | 374,952 B | 91 | 33.00 ms |
| 七字段全变 | 一条 sparse UPDATE | 374,952 B | 91 | 30.66 ms |

逐字段 UPDATE 也做过对照：host+chain降低到51帧，但七字段全变上升到约50.5ms。因此生产实现使用一条动态 SQL，字段名仅从七个内部常量选择；值仍全部绑定参数。不会引入用户可控标识符、表、依赖、配置或新长期状态。

## 实现边界

`StorageCoordinator::intern_and_attr` 复用一次 cached canonical session 查询，同时读取已有七字段。新会话执行固定 INSERT；已有会话按原规则做空值不降级合并（host/process/rule/network/chain）和 policy/category 权威更新，只把变化字段加入一条 UPDATE。没有变化时不写 attr。

事务、receipt、unknown commit 重试、chain 整组替换和 retention attr 失效 trigger 未改变。`PrepareCache` 的诊断集合改为 SQL 文本字符串，以覆盖有限的内部 sparse 形状；`prepare_cached` 仍由 rusqlite 控制容量，诊断计数表示见过的文本，不宣称实际未被淘汰的编译次数。

## 验证

- `cargo test --release --manifest-path residential-monitor/src-tauri/Cargo.toml --lib storage::storage_attribution_lifecycle_tests -- --nocapture`: 5 passed。
- `cargo test --release --manifest-path residential-monitor/src-tauri/Cargo.toml --lib storage:: -- --nocapture`: 57 passed。
- `metadata_index_write_proof` 与 `metadata_sparse_index_write_proof` 均通过，包含 host+chain、七字段全变、结果摘要和 WAL/index 证据。
- `rustfmt --check`、`git diff --check` 通过。

容量矩阵需由主任务用冻结的真实 facade benchmark 重建；上述隔离WAL结果不能替代 A50/A250 长期门。

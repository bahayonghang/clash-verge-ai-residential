# 设计：报告放锁与取消

## 边界

`lib.rs` 命令只做：短锁读路径与 snapshot store 句柄 → 放锁 → C3 纯函数/服务 → 短锁写档案或错误。`AppFacade` 不再在持锁时调用 `ReportService::run`。

`OperationRegistry` 成为 cancel flag 的唯一登记处。进度 DTO 不变。

## 数据流

1. `start_operation` 分配 `Arc<AtomicBool>`，插入 registry。
2. 长任务闭包捕获 path、`Arc<ReportSnapshotStore>` 需要变成可短锁取出的内部结构：store 已在 facade 上，报告 spool 按规格必须独立目录。实现时把 `snapshots` 的 insert 放到第二次短锁，或给 store 自己的 Mutex（不得与 AppFacade 同一把锁）。
3. `cancel_operation` 置位 flag。C3 已有 `cancel` 轮询点必须真正读到 true。

## 权衡

给 `ReportSnapshotStore` 单独 Mutex 比把整个 facade 拆模块更小。token LRU 与采集 commit 并发：store 只写 spool 文件，不写 SQLite writer。规格禁止 CLI 写 store；桌面只此一家。

## 回滚

若放锁后发现 Recovery-only，丢弃结果并返回 `recovery_only`，不写 `report_archive`。

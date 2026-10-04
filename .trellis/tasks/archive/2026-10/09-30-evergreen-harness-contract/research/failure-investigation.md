# T03 完整产品门的失败记录

日期：2026-09-30。源码起点：`d3a25b4164343f5cbeab8a51efe1a1d3974cdf4a`，工作树含已批准 T01/T03 变更。T03 未改 Rust 产品源码、数据库配置或测试阈值。

## 首次正式检查

`just ci` 于 12:50:20–12:51:36 UTC 运行，退出 1。见 `product.result.json`、`product.log` 和保留原始字节的 `product.log.gz`。Rust 单元测试 541 通过、1 失败、6 ignored。失败阻断了随后 root `npm run ci`；该次运行不能记为完整产品门通过。

失败测试：`storage::lifecycle::tests::session_fanout_yields_a_durable_prefix_before_hard_deadline`。`residential-monitor/src-tauri/src/storage_lifecycle.rs:646` 在重新打开数据库后的循环中对 `cleanup_ledger_with_cancel(...)` 的结果调用 `unwrap()`，收到 SQLite `OperationInterrupted`。

## 已核对的调用路径

`cleanup_ledger_with_cancel` 在同文件约 95–143 行设置 SQLite progress handler。取消标记为 true 或从调用开始经过 250ms 时，handler 返回 true。清理先给 session 操作设置 100ms 主动让出时点；小于 150ms 时再清理 receipts，receipts 的主动让出时点为 200ms。语句、提交及返回路径仍受 250ms 硬预算约束。错误返回后移除 handler，并在仍有事务时补充 rollback。

该测试使用隔离临时 SQLite 库，构造 1000 个 session，首次连接的临时 trigger 每次删除 sleep 3ms。首次 cleanup 后关闭并重新打开数据库，再循环 cleanup，要求每次推进并最终清空。失败发生在重新打开后的 cleanup `unwrap()`。现有函数允许返回硬预算中断，而该测试此次假定 cleanup 成功。

以上代码与错误支持“触发了 SQLite 中断路径”。该次墙钟预算为何耗尽的具体原因未查明。未采集到足以区分磁盘延迟、调度暂停或其它竞争的细粒度测量。不能把一次失败归因到 T03 文档改动，也不能把时间敏感测试记为已修复。

## 一次定向复测

经主会话确认，保持产品源码与阈值不变，只执行一次同版本测试：

`cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib storage::lifecycle::tests::session_fanout_yields_a_durable_prefix_before_hard_deadline -- --exact --nocapture`

12:53:40–12:53:42 UTC：退出 0，1 通过，547 filtered。首轮提交 30 行前缀，耗时 105.9169ms；reopen 后继续清空。见 `rust-targeted.*`。该结果不替代正式 `just ci`。主会话随后授权在无其它测试/构建竞争时重跑一次完整门；结果使用独立 `product-rerun.*` 收据，首次失败保持原名。

## 捕获与平台参数说明

- 单独 root `npm run ci` 于 12:52:43–12:52:49 UTC 退出 0。`root-ci.result.json` 已记录真实命令退出码。collector 在保存收据后打印末尾输出时触发 GBK `UnicodeEncodeError`，collector 外层退出 1。未把 collector 退出码当作 npm 失败。后续 collector 显式使用 UTF-8 stdout，并仅对子进程设置 `PYTHONIOENCODING=utf-8`；没有改全局环境。原始收据未覆盖。
- `get_context.py --mode phase --step 2.2 --platform kimi` 退出 0，但缺少 Kimi 角色块。`workflow_phase.py` 对平台标签做去空格/连字符后的名称匹配；`kimi` 不等于标签 `Kimi Code`。使用 `--platform kimi-code` 后获得 built-in coder、对应 role skill、首行及禁止递归的段落，收据为 `phase-kimi-code.*`。未修改 runtime 或平台映射。
- 初始 Python 子进程输出可能按宿主编码生成；`.log` 使用 UTF-8 解码时的替代字符不影响 `.log.gz` 保留的原始字节。修正 collector 后的新增 Python 收据使用显式 UTF-8。

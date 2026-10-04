# 原实现的 network 未知过滤差异

日期：2026-09-30。状态：原失败已保留；强模型确认既有合同后，一行修复及同一 fixture 已通过。适用五套 harness；该问题需要强模型审查，不能交由低成本模型自行改变过滤合同。

## 首个失败

新增投影缓存语义 fixture 后、修改生产算法前运行：

`cargo +1.98.0 test --locked --release --target x86_64-pc-windows-msvc --manifest-path residential-monitor/src-tauri/Cargo.toml --target-dir <独立 candidate-target> --lib c3::raw_fold::tests::repeated_projection_values_preserve_fallbacks_missingness_and_exit_keys -- --exact --nocapture --test-threads=1`

`projection-cache-20260930/original-semantics.receipt.json` 保存实际参数、起止时间和退出码。退出码 101；实际执行 1 test，1 failed。原始 stdout/stderr gzip 及 SHA256 保留。此时 `git diff` 只有 raw_fold.rs 的 166 行新增测试，没有生产语句变化。原生产文件字节快照为 `build-20260930/raw_fold-pre-candidate.rs`。

失败过滤：`ReportFilters { network: Some("__unknown__"), ..Default::default() }`，grouping=Host，分钟半开区间 [-2,121)。

- raw fold：upload=10，download=100，connections=5，active_duration_sec=120。
- 独立 grouped SQL：上述字段均为 0。

## 直接机制

- `raw_fold.rs` 的 `resolve_filters` 对 process/network 共用 `resolve_id`。`resolve_id` 将 `__unknown__` 解析为 `IdFilter::Missing`，从而匹配 NULL 或缺少字典行的 network ID。
- `sql.rs` 的 `filter_clause` 对 host/process 定义 unknown 特殊谓词；network 始终按 `dimension_dict` 中 network/value 等值查 ID。没有 `__unknown__` 字典行时匹配 0 行。
- 已读 storage spec 明确 host/process unknown 下钻，没有定义 network unknown 下钻。`ReportFilters.network` 目前为 `Option<String>`；仅看 DTO 不能决定该值是否受到全部公开入口支持。

## 初次处置边界

尚未改变 oracle、忽略测试、修复 filter 或实施投影缓存。已向主会话和独立审查代理报告。网络 unknown 语义需单独定性；第一项缓存若继续，必须保留既有差异证据，不把排除该用例后的局部门声明为全部过滤等价。

## 审查后的恢复

独立审查代理与主会话核对历史实现后，确认该差异由 raw-fold 的 process/network 共同 Missing 分支引入；原 SQL 合同只给 host/process 定义 unknown 特殊匹配。network 字符串仍合法，按 network 字典 value 等值查找；没有相应字典行时结果为空。审查依据见本任务 `network-unknown-review.md`（由独立审查代理维护）。

主会话确认该恢复属于已批准 raw_fold.rs 与 AC2 oracle 一致性范围。实施仅把 resolve_id 条件限定为 `value == UNKNOWN_IDENTITY && kind == "process"`。没有修改 SQL oracle、删除网络 unknown 用例或新增 InvalidQuery。

同一 fixture 重测退出 0，实际 1 test passed，见 `projection-cache-20260930/network-contract-restored.receipt.json`。修复后、投影缓存前的源码单独保存为 `projection-cache-20260930/raw_fold-contract-restored.rs`。随后缓存版本 raw-fold 回归 4 passed / 1 ignored；此处的 ignored 是显式隔离阶段探针，不是新增失败用例。

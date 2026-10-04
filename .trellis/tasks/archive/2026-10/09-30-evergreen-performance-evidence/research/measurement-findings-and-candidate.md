# T05 初轮测量与范围内候选

日期：2026-09-30。状态：新构建与阶段证据已取得；候选尚未修改产品代码。适用工具：Claude Code、Codex、Grok Build、Kimi Code、OMP；阶段判读与算法审查由强模型完成。

## 已验证身份

- `build-20260930/` 保存 baseline 原始/适配后 manifest、五文件 patch、candidate 前后 manifest、dist manifest、工具链及每条命令原始日志。
- Rust/Cargo 均为 1.98.0，target 为 x86_64-pc-windows-msvc；使用 --locked --release 和分离 target-dir。实际 Cargo 首选入口是本机 mbx cargo.exe。candidate 的 generated OUT_DIR input changed 缓存 warning 保留，构建退出 0；冻结源文件前后相同。
- baseline、candidate、test 构建分别退出 0，用时 119.971 / 74.746 / 19.541 秒。四个复制后 exe 哈希见 executable-identity.json。
- 双侧 A8 / 3 virtual tick smoke 均退出 0；3 commits、流量守恒、相同 fixture hash、WAL/FULL。smoke 不代替正式门。

## 首读执行器失败

预审中的 `c3::service::tests::isolated_nonempty_report_stage_proof` 没有匹配测试。初轮命令退出 0，但运行 0 tests、无 JSON；production-first-validation.json 正确记录 FAIL。驱动仍继续后续诊断，导致本轮未经主动预热的生产首读机会已经丢失。所有原始收据保留。

复制 test exe 的 --list 确认实际名称为 `c3::service::raw_stage_probe_tests::isolated_nonempty_report_stage_proof`。补测驱动要求精确名称、stdout 实际 1 test、JSON 存在及独立语义校验。补测明确标记此前阶段已经读取数据库。没有物理冷缓存证明。

## 当前测量

| 样本 | production | projection ms | scan ms | 状态或边界 |
| --- | ---: | ---: | ---: | --- |
| 补测非空一分钟 default-auto | 14.0227 ms | — | — | Raw；250 连接；upload 3497 / download 11725；固定 oracle 通过；已读页缓存 |
| 初轮 A250 30d all | 无生产 deadline | 15407.9465 | 13069.8577 | 本轮首次读取全量；不是物理冷缓存证明 |
| 初轮 A250 30d residential | 无生产 deadline | 2901.9163 | 4113.2073 | 在 all 之后，缓存与 query 定义不同 |
| 补测 A250 30d default-auto | 10018.5676 ms | — | — | DeadlineExceeded；native test 退出 0，生产门 FAIL |
| 补测 A250 30d all | 无生产 deadline | 3894.5494 | 5421.8366 | 已读页缓存；current 2160000 连接 |
| 补测 A250 30d residential | 无生产 deadline | 2754.6016 | 4403.7208 | 已读页缓存；current 1620000 连接 |

all query 使用 local 时区和 previous_equal_window=true；residential 使用 UTC、无 previous。default-auto production probe 的 now=end；不代表 CLI 真实 now。all 与 residential 不能仅按过滤器解释差值。阶段计时没有生产 deadline，阶段成功不关闭报告门。

all 30d totals 为 upload 151200035 / download 507599545；residential 为 upload 113399990 / download 380699625。两轮各自一致，但独立完整语料 SQL oracle 尚未运行。

已复制 release test exe 聚焦结果：raw_fold 3 passed / 1 ignored；c3::service 53 passed / 3 ignored；bench 12 passed / 1 ignored。ignored 包含隔离数据与显式探针，不表示自动运行。

## 第一项最小候选：投影派生值复用

限定文件：`residential-monitor/src-tauri/src/c3/raw_fold.rs`。不改 SQL、schema、投影阈值或生产 query owner。

源码事实：每个 session 都重新把 process/network/category 的字典字符串送入 pool；chain_key 已 intern 后仍复制 String，再重复调用 last_chain_hop 与 chain_identity 生成 String，rule fallback 也复制字典 String。30d 合成库存在大量重复维度/chain，但当前实现按 session 重复派生。阶段证明 projection 本身仍占 2.75–3.89 秒；没有声称上述分配是全部耗时来源。

候选机制：

1. 字典加载时为 process/network/category/rule 各值 intern 一次，保存 pool identity；保留 id 缺失、字典行不存在、存在但空字符串的区别。原 process/network/category 的空值返回 unknown identity 且 missing=false；未知 id 返回 missing=true。rule 的空字符串行为仍保留。
2. 按已经 intern 的完整 chain_key identity 缓存 chain identity 和 rule hop override；复用现有 last_chain_hop/chain_identity 函数，每个不同原始 key 只解析一次。缓存只活在同一 SessionIndex 构造期，不跨 reader/snapshot。
3. 原始 chain_key 字符串及其排序保留，exit 仍按原始 key 聚合和升序破同值；缺失/空白/单跳/尾空 hop 的行为保持。
4. SessionFact、window-scoped <=3120 分钟阈值、长窗口全表投影、residential EXISTS、分钟单次索引扫描、所有维度与过滤定义保持。

必须检查：补充重复链、相同 chain 不同 fallback rule、字典空值与悬空 id 的 grouped SQL oracle；现有全维/过滤/排序/负分钟/零字节/跨窗口 oracle；取消与整份 deadline；fmt/clippy；复制新身份 test/bin，重复相同已读缓存阶段和生产minute/30d。若 projection 未改善或核算回归，回退本候选而保留收据。

## 后续候选边界

minute scan 在已读缓存中仍占 4.40–5.42 秒。第一项候选不承诺关闭 A1000 门。待第一项同窗口证据形成后，再评审在 fold_window 中按 session 预计算过滤/分组/缺失/exit 归属，并使用已压缩 identity 的连续累加槽，减少每分钟重复哈希；不得删维度、扩大3120阈值、改变 distinct/previous 或跳过完整30天。该第二项尚未定稿，也未实施。

若仍需 schema、索引、物化层、调度器或其它未列文件，交主会话申请范围变更。F1–F12、原主场景三轮、21次容量门仍未运行，不能用当前阶段证据销项。全部应用文件写入仍为 null，AC7 不通过；安装态与 AC8 独立门仍未验证。

## 首项实施与交替测量

主会话和独立强模型审查批准首项。新增 fixture 先发现 network unknown 的既有 SQL 合同回归；原 exit 101 与一行恢复后 exit 0 单独保存，见 existing-network-unknown-mismatch.md。没有改变 oracle。

cache-v1 仅修改 raw_fold.rs，包含字典 identity 复用、按完整 chain identity 缓存两个派生值，以及语义 fixture。fmt、release workspace/all-targets clippy -D warnings、raw_fold 4 passed、service 53 passed、bench 12 passed、git diff --check 通过；显式 ignored 分别为 1/3/1。冻结源 manifest、三个新 exe 与 compiler-artifact 见 projection-cache-20260930/。旧四 exe 和源码身份保留。

projection-cache-comparison-20260930/ 对同一已读页缓存 A250 语料，按 before/cache、cache/before、before/cache 交替各三轮：

| 查询 | projection 中位 ms：before → cache | 比值 | scan 中位 ms：before → cache | 比值 |
| --- | --- | ---: | --- | ---: |
| all | 3176.7212 → 2729.5520 | 0.859236 | 3889.1742 → 3906.8561 | 1.004546 |
| residential | 2628.9666 → 2268.6029 | 0.862926 | 3495.5474 → 3757.0764 | 1.074818 |

全部阶段精确 totals 与独立 SQL 语料 oracle 一致。额外 production minute 为 4.8246 → 4.1297 ms；30d default-auto 为 7178.2956 → 6730.6508 ms，均为 Raw 且固定 totals/非空 series 一致。这些样本来自已读取的页缓存，不能覆盖初轮 10 秒失败，也不代替 21 次容量门。

独立审查指出，预先 intern 字典会使未参与本窗口的字典文本留在 SessionIndex.pool，短/空窗口存在新增驻留风险。目前没有该风险的运行内存证明。建议在同一文件改成只对实际使用的字典值进行 lazy memoization，再冻结新身份复测；该后续尚待主会话确认。分钟扫描仍占较大耗时，首项不证明 A1000 可通过 10 秒门。

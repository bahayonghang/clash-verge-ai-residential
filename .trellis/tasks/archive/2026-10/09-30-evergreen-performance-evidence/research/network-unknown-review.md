# T05 network unknown 过滤合同审查

日期：2026-09-30。审查代理：`/root/t02_review`，强模型 `trellis-check`。仅写本文件；未运行构建、测试或负载，未打开或 hash corpus，未修改产品。

## 结论与归属

**判定：现有 raw-fold 对 network 的缺失过滤特例违反既有 SQL 合同。** `resolve_id` 把 process 的 `__unknown__` 特例同时应用到了 network。该行为由 2026-09-24 的 raw-fold 实现引入；既有原始 SQL、当前 raw SQL、维度层 SQL、公开报告说明和 UI 均未定义 network 的同类缺失特例。

`filters.network="__unknown__"` 是 API 验证接受的字符串过滤值。按既有合同，该值走 network 字典的 value 等值查找。正常字典不保存该哨兵，因此此 fixture 的结果应为空。不能通过删除测试、修改 SQL oracle、把该输入改成 InvalidQuery，或扩展 network 缺失过滤来消除失败。

建议将 `raw_fold.rs::resolve_id` 的特殊分支限定为 `kind == "process"`。该修改恢复 T05 AC2 要求的 SQL 等价性，文件属于已批准范围，未引入新的查询语义、schema、权限或门槛。现有证据支持在主会话协调后继续实施，不需要新增产品语义决策。若改为给 network 增加缺失过滤能力，则需要另行界定公共合同、跨层实现与授权。

## 首个失败与未改算法证明

实施代理新增 `repeated_projection_values_preserve_fallbacks_missingness_and_exit_keys` fixture，尚未实施投影缓存。审查时 `git diff --stat` 显示 `raw_fold.rs` 仅新增 166 行；`#[cfg(test)]` 之前的生产代码与 HEAD 逐行相同，SQL 无工作树修改。

`projection-cache-20260930/original-semantics.receipt.json` 记录：2026-09-30 14:35:36–14:35:56 UTC，原始退出码 101；实际运行 1 test，0 passed / 1 failed / 0 ignored。失败处 grouping=Host，filters 只有 `network=Some("__unknown__")`：

| 字段 | 原 raw-fold | 既有 grouped SQL |
| --- | ---: | ---: |
| upload | 10 | 0 |
| download | 100 | 0 |
| connection_count | 5 | 0 |
| active_duration_sec | 120 | 0 |
| missing_upload / missing_download / missing_connections（Host） | 0 / 0 / 0 | 0 / 0 / 0 |

本代理只读核对了 stdout/stderr gzip 解压内容的 SHA256，两者均与 receipt 一致。receipt SHA256 为 `a81b5c1de068546a8bb7fcc83a4202639ef40c246f4b56b6b62abc8126d7e62b`。原命令、源码前提与 FAIL 必须保留；后续修复使用新的收据名。

## 权威合同追踪

| 层与证据 | 已确认行为 | 对判断的作用 |
| --- | --- | --- |
| `c3/query.rs:183–190,823–842` | network 是 `Option<String>`；validate_query 只检查窗口、时区、page、top_n、cursor，未拒绝该过滤值 | 不能将该 case 标为无效或不可调用输入 |
| `c3/sql.rs:598–616` | process unknown 使用 `PROCESS_MISSING_SQL`；network 始终绑定 value 并按 network 字典 ID 匹配 | raw SQL 对两种过滤的定义不同 |
| `c3/sql.rs:679–700` | 维度层只有 host/process 的 unknown 映射到 `dimension_id=0`；network 仍按字典 value 等值 | 当前多个查询层对 network 的原合同一致 |
| `c3/raw_fold.rs:660–708` | process/network 共用 resolve_id；该函数只检查 value==UNKNOWN_IDENTITY，不检查 kind | 新路径泛化了 process 的特殊分支，形成此次回归 |
| `.trellis/spec/residential-monitor/storage/sqlite-contract.md:30` | 字典禁止存放哨兵；显式给 host/process 定义 unknown 特殊过滤，没有 network 特例 | 支持沿用字面过滤，而非新增缺失匹配 |
| `residential-monitor/docs/reporting.md:9–12` | 分别解释 host/process unknown 下钻与 residential 哨兵；没有 network 缺失过滤承诺 | 公开说明与既有 SQL 一致 |
| `src/format/rank.ts:9–13,95–109` | 维度页仅 host/process/rule/chain；unknown 仅转换为 host/process 过滤 | UI 未提供 network unknown 下钻来源 |
| `dimension-page.tsx:76–79,132–136` | unknown 下钻入口与选择回调均限定 host/process | 没有 UI 行为要求 network 扩展为 Missing |
| `reports/query-form.tsx:17,77–91` | 报告页允许按 network 分组；表单不提供 network 值过滤输入 | network 分组能力不能推导为 unknown 特殊过滤能力 |
| `dbcli/mod.rs:70–90,119–129,368–393,539–550` | `--by network` 只设置 grouping；CLI 无 network 值过滤参数；内部 ReportQuery 走 run_uncached | CLI 未提供相反的 network 缺失过滤合同 |

表中 `c3/*` 路径相对 `residential-monitor/src-tauri/src/`；UI 路径相对 `residential-monitor/`。

排名输出的 `identity="__unknown__"` 与过滤值的匹配定义要分别判断。排名层可把空字典值与缺失行显示为同一 unknown identity；当前 process 的 missing 归因仍区分“字典空串存在”和“ID 为 NULL/悬空”。不能仅依据排名标签，给所有过滤字段添加缺失谓词。

## 历史定位

- `f681c1ef10c67cd8c1c1eec22c033b1ade1aac08`，2026-08-22，提交主题“进程页支持未知下钻与核算过滤”：历史 SQL 的 process 分支已特殊处理 unknown；network 分支仍使用字典 value 等值；维度层特殊分支仅 host/process。
- 固定 baseline `c278bb7b56603001e32e353d2ee589dccef0bfe9` 的 `sql.rs:488–490` 同样对 network 使用字典 value 等值。
- `b8a64a1d67540e7608e0b2bcab3c87b29837f719`，2026-09-24，提交主题“交付资源优化并登记未过门”：新 `raw_fold.rs:681–694` 已出现未检查 kind 的 resolve_id；当前生产分支由该提交引入。
- `e5d19b15e7e7cca78cca4c4adabd2d755560c724`，同日短窗口投影优化：当前 blame 仍将该 resolve_id 分支归到 b8a64a1。

未发现支持 network 缺失特例的相反合同。历史定位说明当前 SQL oracle 保留了既有语义，raw-fold 路径引入了差异。

### 文档时效边界

frontend 的 `dto-and-decoding.md:244` 与 `view-state.md:19` 仍写 Host unknown 可下钻、其它维不可。该文字尚未跟上 f681c1e 的 Process unknown 扩展；当前 UI、storage spec 和公开 reporting 文档已经包含 Process。此处是既有文档漂移，不能据此回退 Process，也不能据此扩展 Network。本审查仅记录，不修改不在本次 ownership 内的文档。

前一份 `review.md` 要求覆盖 process/network 的 unknown 测试，用于暴露过滤与 oracle 的差异。该要求按本报告收口：Process 保持 Missing 特例；Network 按字面字典值查找。不能将 raw-fold 当前的网络缺失特例当作需要保留的合同。

## 最小处置与回归顺序

1. 保留原始 fixture、grouped SQL oracle 和 `original-semantics.*` FAIL，不删除或跳过 `network="__unknown__"` 组合。
2. 仅在 `raw_fold.rs::resolve_id` 将特例限定到 Process：

   ```rust
   if value == UNKNOWN_IDENTITY && kind == "process" {
       return Ok(IdFilter::Missing);
   }
   ```

3. 在任何 cache 改造之前，以新收据名运行同一精确 fixture，要求实际 1 passed / 0 failed / 0 ignored。这样可以将合同修复与缓存优化区分。
4. fixture 同时验证：process unknown 仍选中 NULL/悬空 ID；process/network 的空字符串都保持精确字典值匹配；network 的普通值与不存在值、无过滤时 missing 归因仍与 SQL 一致。network unknown 的合法无匹配结果为空，不新增 InvalidQuery，也不放宽字典哨兵规则。
5. 原 fixture 通过后继续已审查的投影缓存方案，重跑 fixture、raw-fold/service 聚焦测试、fmt/clippy 和既定最终检查。新源码与 exe 重新冻结；正式性能门不变。
6. 将“特殊 unknown 过滤仅按既有字段定义，不从排名哨兵泛化”回写已批准的 storage 说明或本任务证据，注明适用 Claude Code、Codex、Grok Build、Kimi Code、OMP。frontend 的 Host-only 旧文档另行登记，按相应文档 ownership 处理。

本报告没有执行第 2–6 步。实施文件仍由 `/root/t02_skill_sync` 独占。

## 验证边界

- 只读检查 query、SQL、raw-fold、UI、CLI、spec、公开说明与相关 Git 历史。
- 原始测试日志哈希一致；原生产算法尚未改动的前提已核实。
- 本轮未运行 lint、type-check、测试或任何 corpus 负载，未宣称修复已经通过。
- 本文件保存后做 UTF-8/行尾空白与 scoped `git diff --check` 检查；检查不扩展为产品验收。

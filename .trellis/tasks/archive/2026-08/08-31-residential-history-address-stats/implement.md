# Implement：家宽历史地址累积统计恢复

## 1. 共享家宽 matcher

1. [x] 在 `src-tauri/src/residential.rs` 将归属收敛为一个共享 matcher：target=`家宽` 对节点做包含匹配，其它 target 精确匹配，空 target 集不匹配；保持 target 配置顺序决定 tags / primary。
2. [x] 让 `accounting::classify` 与 C2 实时 `residential_only` 都只调用共享 matcher，删除两套口径的重复分支与旧的“未配置也按家宽子串命中”行为。
3. [x] 增加单元回归：语义 target、精确自定义 target、多 target 顺序、重复命中、空 target、无关节点和中英文 / 特殊字符 target。

## 2. raw 历史恢复

4. [x] 在 `c3/sql.rs` 定义唯一 legacy-safe raw residential membership predicate：已分类行直接命中；仅对 category 为空的行通过 `connection_chain + target_item` 共享语义恢复，使用 `EXISTS` 防止倍增。
5. [x] 让 `filter_clause(filters.category=__residential__)` 与 `share_residential_raw` 注入同一 predicate；保持其它 category、host/process/rule/chain filters、绑定参数和 named SQL 契约不变。
6. [x] 增加 SQL corpus / EQP 测试：无未消解槽位、无用户值插值、使用 `connection_chain` 主键查找、多个链路节点 / target 只计一次。
7. [x] 增加 C3 service / share fixture：`primary_category_id` 全空但 chain 完整时 totals、series、Host ranking、上下行 Top 1 与份额立即恢复；非空历史 category 不被当前 target 覆写；raw totals / series / rankings 守恒。
8. [x] 以安装实例只读脱敏聚合查询验证当前规模在 10 秒 deadline 内；只记录行数、命中数、耗时和 PASS/FAIL，不输出 Host、IP、进程路径或节点值。若失败，回到设计评估最小索引，不直接加入后台迁移。

## 3. 未来写入与保留层

9. [x] 增加 AccountingEngine → `persist_live_facts` 集成回归，证明 `AI-家宽` 在 target=`家宽` 时写入 category，非匹配行仍 null，policy / primary 顺序稳定。
10. [x] 物化一个修复后的小时 fixture，证明新写入能进入 hourly / daily category 非零层；对 raw 期外 legacy-null 明确返回现有能力限制 / 未知，不声称已恢复。

## 4. 滚动时间窗口

11. [x] 在 `lib/time-range.ts` / `App` 建立单一滚动窗口状态：自动刷新每分钟重算当前 preset；切换 preset 和从暂停恢复立即重算；暂停保持绝对快照；today 跨午夜更新。
12. [x] 保持 `useReport` / `useResidentialShare` 的分钟 snap、memo query、sequence 与旧结果保留机制；清理 timer，避免隐藏页各自建时钟或重复秒级请求。
13. [x] 用 fake timers 覆盖 24h 长驻、暂停 / 恢复、today 跨午夜、预设切换和卸载；断言旧窗口响应不能覆盖新窗口。

## 5. 家宽页状态与文案

14. [x] 从现有 `queryEcho.rangeStartUtc/rangeEndUtc`、`generatedUtc` 与 `autoRefresh` 生成统计窗口 / 最近更新 / 暂停元信息，放在家宽历史聚合区标题附近；不新增第二个刷新按钮。
15. [x] 收敛空态：窗口内零命中、无 coverage、能力不支持、请求失败与刷新旧结果分别呈现；共享错误不在排名和趋势重复堆叠。
16. [x] 补齐中英文文案和组件测试；保留方向、Top N、`aria-sort`、趋势图升序 / 表格降序、窄屏横滚与主题 token。

## 6. Spec、验证与回滚门

17. [x] 更新 `.trellis/spec/residential-monitor/backend/modules-and-errors.md`、`storage/sqlite-contract.md`、`frontend/view-state.md`，替换旧的双口径禁止合并规则并记录新的共享契约。
18. [x] 聚焦验证：
    - `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml residential`
    - `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml accounting`
    - `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml c3::sql`
    - `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml c3::service`
    - `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml c3::share`
    - `npm --prefix residential-monitor test -- time-range use-report use-residential-share aggregate-section`
19. [x] 完整自动门：
    - `npm --prefix residential-monitor run check`
    - `cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check`
    - `cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings`
    - `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace`
    - `just ci`
    - `python ./.trellis/scripts/task.py validate 08-31-residential-history-address-stats`
    - `git diff --check`
20. [ ] 原生 WebView 检查：宽 / 窄窗口、中文 / 英文、至少一深一浅主题；验证自动刷新暂停 / 恢复、真实窗口时间、非空历史排名、趋势和五类状态。未执行组合保持 `UNVERIFIED`。
21. [x] 回滚检查：matcher、raw fallback、窗口节拍三段可独立还原；没有 schema / 本机数据回滚、安装、凭据或外部状态变更。

## 7. Start 前门

- [x] 用户审阅并明确批准本版 Goal / In Scope / Out of Scope / AC / Key Decisions。
- [x] `prd.md`、`design.md`、`implement.md` 无阻塞开放问题。
- [x] `implement.jsonl` / `check.jsonl` 只含真实 spec / research 条目。
- [x] `task.py validate` 通过后，才可运行 `task.py start`；本轮不得编辑产品代码。

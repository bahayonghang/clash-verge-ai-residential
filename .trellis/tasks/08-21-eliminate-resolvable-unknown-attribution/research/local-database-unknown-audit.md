# 本机 24 小时 Unknown 归因审计

## 方法与证据边界

- 2026-08-21 只读打开开发态数据库 `%TEMP%/io.github.bahayonghang.residential-monitor/monitor.sqlite3`，SQLite URI 使用 `mode=ro`。
- 窗口以库内最新 `connection_minute.utc_minute` 为结束，取 `[2026-08-20T08:24:00Z, 2026-08-21T08:24:00Z)` 共 1440 个分钟索引；这与上一任务 `unknown-host-24h.md` 的窗口和截图数值一致。
- 本审计证明数据库中保存了什么、当前 SQL 怎样解释它；不证明正在运行的安装包已包含分支最新代码，也不证明控制器在其它模式下一定提供相同字段。

## 复现结果

| 维度 | 已归因会话 | 未归因会话 | 未归因上传 | 未归因下载 | 未归因总量 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Host | 16,436 | 20,692 | 895,321,222 B | 3,807,849,948 B | 约 4.4 GiB |
| Process | 1 | 37,127 | 2,007,721,981 B | 12,341,013,583 B | 约 13.4 GiB |
| 原始 `chain_key` | 37,128 | 0 | 0 B | 0 B | 0 B |

窗口总量为上传 2,007,722,266 B、下载 12,341,023,221 B、37,128 个会话。

### Host

`s.host` 为空的总量为 4,703,171,170 B，正好对应截图 Top Hosts 的 `Unknown 4.4 GiB`。上一任务已证明这些旧会话没有保存 `destinationIP`，因此无法从现有库无损回填；新 `host -> sniffHost -> destinationIP` 逻辑只能改善重新采集后的会话。

### Process

37,128 个会话中只有 1 个有 `process_id`，其总量为 9,923 B；这对应截图唯一已知进程 `mihomo 9.7 KiB`。其余约 13.4 GiB 的进程元数据在库中真实缺失。用户提供的 Clash Verge 截图虽然有 `Process` 列，但可见行的该列也是空白；当前证据不支持把这些流量猜成某个进程。

### Chain：已定位的确定性缺陷

所有 37,128 个会话都有非空 `chain_key`。其中 8,628 个会话的 `chain_key` 恰为单跳 `DIRECT`，上传 69,846,745 B、下载 4,941,934,579 B，总量约 4.7 GiB，正好对应截图 Top Chains 的 `Unknown 4.7 GiB`。

根因不是控制器缺字段，而是链路排名复用了规则分组辅助函数：

- `c3/rule_name.rs:10-15` 的 `last_chain_hop` 对不含 `>` 的单跳值返回 `None`，这是为了让规则聚合回退到 rule / `DIRECT`。
- `c3/sql.rs:89-100` 的 `RANK_RAW_CHAIN` 却把该 `None` 直接映射为 `__unknown__`，所以合法单跳 `DIRECT` 被错误归为 Unknown。
- `c3/retention.rs:315-327` 的 chain 字典物化同样只 intern `last_chain_hop(...) is not null` 的值，单跳 `DIRECT` 在小时维度会落到 `dimension_id = 0`。
- `c3/sql.rs:336-338` 的 chain 过滤也只比较 `last_chain_hop(a.chain_key)`，因此即使 UI 改成显示 `DIRECT`，单跳链路下钻仍会得到空集。

链路维应拥有独立 identity 机制：多跳取当前产品约定的末级策略组，单跳保留 trim 后原值，空数组才是缺失。不能直接改变规则聚合用 `last_chain_hop` 的单跳语义，否则会把单跳代理节点错误提升为规则组。

## 结论

截图中的三个底部 Unknown 不是同一种问题：

1. Chain `Unknown 4.7 GiB` 是可修复的查询/物化契约错误，已用字节数精确复现。
2. Host `Unknown 4.4 GiB` 是旧采集信息丢失；未来样本可改善，现有历史无法从当前数据库无损恢复。
3. Process `Unknown 13.4 GiB` 是控制器元数据在本次观测中几乎完全未报告；应检测并表达“维度不可用/未报告”，不能伪归因。

上方五张 `Unknown` 计量卡还属于第四类：截图时 collector 正在连接且无当前样本。它们必须与历史维度桶分开建模和展示。

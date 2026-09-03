# 家宽历史累积统计读空：调查记录

## 1. 症状与数据流

```text
Controller connections
  -> AccountingEngine classify(targets, chains)
  -> LiveConnectionView.primary
  -> connection_session_attr.primary_category_id
  -> ReportQuery(category = __residential__, grouping = host)
  -> totals / series / rankings
  -> 家宽历史统计
```

实时筛选走另一条路径：`is_residential_filter` 接受“精确 target 或节点名包含家宽”；历史核算的 `residential_tags` 只接受精确 target。前端榜单本身已经查询历史 `ReportResult`，不是从实时 Top 1 推导。

## 2. 已确认根因

### RC1：默认 target 与精确核算语义不兼容

- 设置前端默认填入 `家宽`：`residential-monitor/src/hooks/use-settings.ts:25-26,109-112`。
- 历史核算仅精确匹配：`residential-monitor/src-tauri/src/residential.rs:13-27`。
- 实时筛选额外接受节点名包含「家宽」：`residential.rs:29-34`。
- 安装实例中，target 与链路节点完全相等的历史会话为 0；包含 target 的会话为 70,282。最近 24 小时包含匹配为 25,022 个会话，存在非零上下行增量。
- 所有 264,654 个 `connection_session_attr.primary_category_id` 均为空，故 `__residential__` 过滤必然读空；Host 和分钟流量并未丢失。

### RC2：相对时间预设被冻结为启动时快照

- `App` 只在初始 `defaultTimeRange()` 和用户切换预设时计算绝对起止时间：`residential-monitor/src/app.tsx:31-33,95-104`。
- `autoRefresh` 没有驱动 `timeRange` 推进；`useReport` / `useResidentialShare` 只在绝对范围改变时重新请求。
- 结果是长驻 / 托盘应用顶部仍写「近 24 小时」，实际查询却停留在启动时的旧 24 小时窗口。实时采样时间与历史统计窗口因此可以相差数十小时。

### RC3：当前空态掩盖“归属不可用”和“窗口陈旧”

- `RankBar` / `TrendArea` 收到成功但空的 `ReportResult` 后只显示通用无数据文案。
- 页面不展示 `queryEcho.rangeStartUtc/rangeEndUtc` 或 `generatedUtc`，用户无法判断结果对应哪个绝对窗口。
- 份额、榜单和趋势共享归属缺口，但展示上看起来像三个独立的“无数据”区域。

## 3. 可恢复性

- `connection_chain` 当前保留 265,054 个会话的节点事实；因此 raw 保留期内可从配置 target + 链路节点重新判定家宽归属。
- 不需要根据 Host / IP 猜测，也不需要访问 Clash 私有配置。
- 只修未来写入不能满足用户查看过去一段时间的要求；必须提供查询时恢复或一次性幂等重建。

## 4. 方案候选

### A. 共享语义选择器 + raw 查询时恢复（推荐 MVP）

- target=`家宽` 作为显式语义选择器，匹配链路节点名包含「家宽」；其它 target 保持完全相等。
- 把 matcher 收敛为单一后端契约，同时供实时筛选、AccountingEngine、raw 家宽 filter / share 使用。
- raw 查询对缺失分类键的旧会话通过 `connection_chain` 判定；不先大批量改写用户库，因此可立即恢复、回滚简单。
- 新采样正常写入分类键；物化层只消费已稳定分类。
- 风险：raw 查询增加 `EXISTS` / join 成本；必须以实际规模、索引和 10 秒报告 deadline 验证。raw 期外旧记录仍不能凭空恢复，需要诚实能力说明或后续重建。

### B. 后台幂等重建分类与派生层

- 按 session 分块重算 `primary_category_id` / policy version，再重建受影响 hourly / daily 分区。
- 优点：长期层一致，查询成本稳定。
- 风险：属于数据迁移；需进度、取消、失败恢复、watermark、并发写协调和较重验证，不应在没有长期层需求证据时作为默认首选。

### C. 保持精确匹配，只要求用户配置真实节点全名

- 代码改动最小，但无法自动恢复当前 target=`家宽` 下的历史；还需要节点发现 / 选择 UX。
- 与用户“修复当前读取不到”以及已有默认值的预期不符，不推荐。

## 5. 已确认边界

用户已选择 A 的产品语义：target=`家宽` 作为受控语义选择器，恢复已有 raw 历史；其它 target 继续精确匹配。实现同时修复滚动时间窗口与诚实空态。性能证据若证明查询时恢复不能在 deadline 内稳定完成，才回到设计评估最小索引或 B；不得预先加入迁移框架。

## 6. 明确不推断

raw 保留期外、没有 category 且没有可查询 raw / chain 证据的记录不可恢复，保持未知 / 能力限制。Host、IP、进程与流量大小均不是家宽归属依据。

## 7. 实施后只读性能验证

2026-08-31 在安装实例 `monitor.sqlite3` 上以 SQLite `mode=ro + query_only` 执行与产品相同的 raw membership 聚合：最近 24 小时窗口含 220,121 条分钟行、92,385 个会话，命中 25,096 个会话，target 数为 1，谓词耗时 301.4 ms；最大 30 天预设覆盖当前全部可用 raw 时含 385,436 条分钟行、162,283 个会话，命中 41,851 个会话，谓词耗时 607.0 ms。两次均低于 10 秒 deadline，结果 **PASS**。本次探针只输出行数、会话数、命中数、target 数和耗时，未读取到输出中的 Host、IP、进程路径或节点值，也未写数据库。

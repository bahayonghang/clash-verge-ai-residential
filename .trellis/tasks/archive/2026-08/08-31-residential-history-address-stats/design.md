# Design：家宽历史地址累积统计恢复

## 1. 设计目标与边界

本任务修复两个相互放大的断点：家宽历史归属因 target 精确匹配而全空，以及相对时间预设在长驻应用中冻结。目标不是再造报告系统，而是让既有 C3 `ReportResult` 在真实滚动窗口内返回可解释的家宽 totals、series 和 Host Top N。

保留：

- C3 是历史统计唯一权威，前端不聚合、不从实时列表累计。
- `filters.category="__residential__" + grouping="host"`、方向权威 Top N、`queryEcho`、Host → sniffHost → 目的 IP、趋势图升序 / 表格降序。
- 现有卡片、表格、主题、时间预设和顶部自动刷新控件。

不引入通用匹配 DSL、新 schema、无限 raw 保留、后台迁移框架或第二套历史存储。

## 2. 共享家宽归属契约

### 2.1 规则

`src-tauri/src/residential.rs` 继续是唯一 owner，但从“两套漂移口径”收敛为一个 matcher：

1. 按 target 配置顺序判断，保证 primary / tags 稳定。
2. target 名恰好为 `家宽` 时，匹配任一 `node.contains("家宽")`。
3. 其它 target 仅匹配 `node == target`。
4. 空 targets 不匹配任何节点；不再凭一个未配置的中文子串默认为家宽。
5. 返回 tag / primary 仍是配置 target 名，不把实际链路节点名复制成 category。

`AccountingEngine::classify`、实时 `residential_only`、未来历史写入及 raw 恢复必须消费这一个契约。前端和 C3 调用方不得复制字符串判断。

### 2.2 新采样

`AccountingEngine` 继续在 canonical snapshot 后分类。命中 `AI-家宽` 等节点时，target=`家宽` 产生 `primary=家宽`；`persist_live_facts` 由既有 `intern_and_attr` 写入 category dimension。零 delta 帧仍只丰富 metadata，不虚构流量。

## 3. 已有 raw 历史恢复

### 3.1 选择查询时 fallback，而非批量改库

在 raw 保留期内，旧记录已经拥有 `connection_chain` 和 `connection_minute`。对 `__residential__` 的 raw membership 使用：

```text
primary_category_id 非空
OR
primary_category_id 为空且 connection_chain 按当前共享 matcher 命中 target_item
```

该 fallback 只补 legacy missing category，不覆写已存在的历史 primary；用 `EXISTS` 表达布尔归属，避免多个链路节点 / target 造成流量倍增。`connection_chain(session_pk, position)` 主键支持按 session 查找，target_item 是小表。

同一权威 predicate 必须用于：

- `filter_clause(__residential__)`，从而覆盖 raw totals / series / Host rankings / manual report；
- `share_residential_raw` 的家宽分子；分母仍是全部可归因 raw 观测。

Rust 侧只定义一次 predicate / renderer，再注入命名 SQL；禁止在 report 与 share 两处复制两版 SQL。所有用户 target 值来自 SQLite 绑定 / 表值，不能字符串拼入 SQL。

### 3.2 保留层边界

- 5m–30d 等仍位于 raw 保留范围的查询可立即恢复，无需等待新采样。
- 新采样正常写入 category 后，后续 hourly / daily 按现有 materializer 得到稳定分类。
- 已经离开 raw 且当时没有 category 的旧记录没有足够现成派生证据；继续显示能力限制 / 未知，不做 Host 或流量启发式回填。
- 不修改 `ReportQuery.targetPolicy`、DTO schema 或历史非空 category。

### 3.3 性能与失败

- 使用实际规模 fixture / 本机脱敏只读查询验证 `EXISTS` 路径在 C3 10 秒 deadline 内；记录行数与耗时，不记录 Host / IP / 节点值。
- 如 query-time fallback 超出 deadline，规划回退点是仅增加一个内部 SQLite 索引或改为有界、幂等修复；不得先引入后台迁移框架。
- SQL / deadline 失败沿用 `ReportError`，前端显示错误态并保留上次结果，不降级为“没有排名”。

## 4. 滚动时间窗口

### 4.1 状态所有权

`App` 仍拥有 `TimeRange` 与 `autoRefresh`，但相对预设不能永久保存一组启动时绝对时间：

- 自动刷新开启时，以分钟为最小节拍重新运行 `timeRangeFromPreset(currentPreset, now)`；分钟对齐与 `useReport` 的 snap 契约一致，避免秒级请求风暴。
- 切换预设立即重算。
- 从暂停恢复自动刷新时先立即重算，再恢复节拍。
- 暂停时保留最后一组绝对起止时间，不继续发报告请求。
- `today` 每次重算使用本地当天 00:00 到当前时间；跨午夜自动进入新一天。

定时器只推进时间范围；`useReport` / `useResidentialShare` 继续基于 memoized query、request sequence 与旧响应丢弃发起请求。不要为每个页面创建独立时钟。

### 4.2 测试时钟

把“给定 preset、autoRefresh、now 得到下一窗口”的纯转换与定时触发分开测试。Vitest fake timers 覆盖：长驻 24h、暂停、恢复、跨午夜 today、组件卸载清 timer；不得用真实等待。

## 5. 家宽页 UX brief（Operate）

### 5.1 Job and outcome

用户在 Windows 本地工作台进入家宽页，要快速回答“所选时间段内哪些地址累计占用家宽最多，结果是否仍是当前窗口”。成功证据是非空且可排序的地址榜、同源趋势，以及可见的实际统计窗口 / 更新时间。

### 5.2 Interaction and layout

- 不新增首屏大卡或新的图表类型。在「家宽目的地址排名」标题附近增加一条低强调元信息：实际起止 / 截止时间、最近生成时间；暂停时明确写“已暂停于 …”。
- 顶部 Refresh 图标继续是自动刷新开关；恢复时立即刷新。绿色仅表示自动刷新开启，不表示数据健康。
- 排名、趋势沿用现有结构；方向与 Top N 控件不变。加载新窗口时保留旧结果并标正在刷新，避免整块闪空。
- 通用“无排名数据”拆成：窗口内无家宽命中、无采集覆盖、能力不支持、查询失败。共享原因只显示一次主提示，榜单 / 趋势不重复堆三份警告。
- 中文 / 英文、宽 / 窄和四主题均使用现有 token、`tabular-nums`、语义表格与键盘焦点；不另起视觉世界。

### 5.3 Data ranges and states

- Top N：10 / 20 / 50 / 100；零到 100 行。
- 时间：5m、30m、1h、24h、7d、30d、today；自动、暂停、恢复、跨午夜。
- 报告：首次加载、保留旧结果刷新、成功有数据、成功无命中、无 coverage、能力不支持、错误。

## 6. 安全、兼容与回滚

- 不读取 / 记录具体 Host、IP、进程路径、链路节点或 secret；测试 fixture 使用保留域名和虚构节点。
- 不新增 DTO 字段优先：窗口信息来自现有 `queryEcho` / `generatedUtc`，暂停来自既有 `autoRefresh`。只有现有 DTO 无法诚实表达原因时才回到规划增加最小字段。
- 默认 24h、下载方向、Top 20、报告 token 和导出行为保持。
- 回滚分三段：先还原前端窗口节拍 / 元信息，再还原 raw legacy fallback，最后还原共享 matcher。无 schema 或用户库回滚。

## 7. Spec changes required

- 后端 `modules-and-errors.md`：把“实时与核算不得合并”改为 D1 的共享 matcher、raw legacy fallback 和保留层边界。
- 存储 `sqlite-contract.md`：记录唯一 raw membership predicate、`EXISTS` 不倍增、索引 / deadline 与无批量改库边界。
- 前端 `view-state.md`：记录相对时间滚动、暂停快照、窗口元信息和家宽空态契约。

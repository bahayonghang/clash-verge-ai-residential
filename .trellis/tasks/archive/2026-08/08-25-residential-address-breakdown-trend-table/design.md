# Design：家宽地址流量拆解与趋势明细优化

## 1. 设计目标

保持现有采集、存储和归因模型不变，把家宽页查询从“全量流量按 target 分类”改成“先选家宽子集，再按实际目的主机拆解”，并让现有 `ReportQuery.sort` 在 SQLite `LIMIT` 之前真正生效。趋势图继续按时间升序，趋势表独立按时间降序。

## 2. 权威数据流

```text
Controller metadata
  -> host / sniffHost / destination IP
  -> connection_session.host
  -> primary_category_id (精确 target 核算)
  -> ReportQuery {
       filters.category = "__residential__",
       grouping = "host",
       sort = 当前方向 desc
     }
  -> C3 totals / series / exact Top N
  -> 家宽地址条形图 + 表格 + 趋势图 / 明细
```

`target` 只参与 `primary_category_id` 的分类；Host identity 只参与地址分组。两者不改名、不复用字段。

## 3. 后端：让 `SortSpec` 成为真实查询契约

### 3.1 安全 ORDER BY 渲染

在 `c3/sql.rs` 为排名模板增加内部 `{order_by}` 槽位，并提供只接收 Rust `SortSpec` / `SortField` 的渲染函数。调用方不能传 SQL 字符串。

映射规则：

| sort field | raw 表达式 | dimension 表达式 |
|---|---|---|
| upload | `sum(m.upload)` | `sum(h.upload)` |
| download | `sum(m.download)` | `sum(h.download)` |
| name | 投影列 `1` | 投影列 `1` |
| identity | 投影列 `1` | 投影列 `1` |

- 数值排序按请求方向，再以 identity / 投影列 1 升序稳定破同值。
- name / identity 当前投影相同，按请求方向排序；保持两个公开枚举值兼容，不删字段。
- `filters` 继续走现有绑定参数；字段和方向只来自反序列化枚举。
- raw host / attr / rule / chain、hourly normal / category、daily normal / category 全部走同一渲染入口，避免某一层仍固定下载排序。
- named SQL 名称保持不变；实际执行前断言 `{filters}` / `{order_by}` 均已消解。

`fill_raw_rank` 与 `fill_dimension_layer` 使用 `query.sort` 渲染排名 SQL，然后才绑定 `top_n`。默认 `download desc` 的结果保持兼容。

### 3.2 正确性回归

构造三个地址：

- A：上传最大、下载很小；
- B：下载最大、上传很小；
- C：非家宽，数值最大。

以 `topN=1` 分别执行 upload desc / download desc，断言 A / B 分别为首行且 C 永不进入家宽结果。raw 与至少一个 dimension tier 都要覆盖，证明排序发生在 LIMIT 前且过滤发生在分组前。

## 4. 前端：地址排名

### 4.1 查询形态

`AggregateSection` 改为：

```ts
grouping: "host"
filters: { ...emptyReportFilters(), category: RESIDENTIAL_ACCOUNTING_FILTER }
sort: { field: direction, descending: true }
```

方向为 session-only `"download" | "upload"`，默认 download。保留 10 / 20 / 50 / 100 Top N 控件。方向或 Top N 改变时由 `useReport` 发出新权威查询；不得从旧 rankings 推导另一个方向的 Top N。

切换期间以 `queryEcho.grouping`、`queryEcho.filters.category`、`queryEcho.sort` 和 `queryEcho.topN` 校验排名结果。旧结果不满足当前选择时，排名区显示 loading，不用错误方向的数据填充新控件状态。趋势序列与排序无关，可继续显示最后一个同窗口结果。

### 4.2 展示语义

- 标题改为“家宽目的地址排名” / “Residential destination ranking”。
- 方向控件明确“下行”/“上行”，具有 `aria-pressed` 或等价选中语义。
- 条形图 value 使用当前方向字段。
- 表格同时保留名称、上行、下行；当前方向表头 `aria-sort="descending"`。
- 份额使用当前方向值 / 对应 totals，不再始终使用下载分母；份额列标题随方向说明口径。
- 地址标签使用 `formatRankLabel`：域名正常显示，IP 带 `IP` 标记，`__unknown__` 保留缺失语义。
- 零家宽结果显示空态；不得回退到未分类的全局流量。

家宽手动报告改为同一 `grouping=host + residential filter`。它仍默认 download desc，导出行包含上下行两列；不扩展全局报告 UI。

## 5. 前端：趋势明细

后端 series 和图表输入保持升序。表格组件通过非原地复制生成：

```ts
const newestFirst = [...series].sort((a, b) => b.bucketUtc - a.bucketUtc);
```

不调用原地 `reverse()`，不改变 `ReportResult`。

表格视觉采用现有项目模式：

- `rounded-md border` 的有界滚动容器；
- `thead` sticky，使用不透明 / 高透明 card 背景与底部分隔，滚动后列名仍可见；
- 时间左对齐且不换行；上行 / 下行右对齐、`tabular-nums`；
- 行分隔与 `hover:bg-muted/40`；
- 窄屏保留最小表宽并横向滚动，不压缩数值到不可读；
- `<table>`、`<thead>`、`<th>` 语义及双语文案不变。

不在本任务给家宽趋势表新增图表联动、点击钉住或键盘选择；这些不是用户请求。

## 6. 文件边界

主要修改：

- `residential-monitor/src-tauri/src/c3/sql.rs`
- `residential-monitor/src-tauri/src/c3/service.rs`
- `residential-monitor/src/components/features/residential/aggregate-section.tsx`
- `residential-monitor/src/components/features/residential/report-section.tsx`
- `residential-monitor/src/components/features/residential/index.test.tsx`
- 新增聚焦的家宽排名 / 趋势组件测试；如 `aggregate-section.tsx` 继续增长，可在同目录抽取 `address-rank.tsx`、`trend-table.tsx`，不创建通用表格框架。
- `residential-monitor/src/i18n/{zh,en}.ts`
- `.trellis/spec/residential-monitor/backend/modules-and-errors.md`
- `.trellis/spec/residential-monitor/frontend/view-state.md`

不改 schema、migration、`session_host.rs`、实时筛选、用户配置与数据库。

## 7. 兼容、性能与回滚

- 默认 sort 仍为 download desc；家宽以外现有调用的默认结果不变。
- 只改变 ORDER BY 表达式，不增加全表候选回传或前端聚合；精确 Top N 的成本形态与现有查询一致。
- category + host 是既有受支持组合，raw 和 13 个月 dimension 层均可回答；能力过期继续使用现有 `drilldownCapability`。
- 回滚可分两段：先还原家宽查询 / 展示，再还原排序渲染。无数据迁移、无本地数据回滚。

## 8. 明确不处理

- 不把 target 从精确匹配改为子串 / 正则，不回填历史分类。
- 不把进程名或链路名伪装成目的地址。
- 不解析加密隧道内域名。
- 不修改全局报告页面布局、其它维度页样式或实时热点算法。

# 家宽地址拆解与趋势明细现状分析

日期：2026-08-25。分析范围仅包含用户截图、当前 `dev` 分支代码、既有 Trellis 任务与规范；未修改产品代码、用户配置或本地数据库。

## 结论摘要

当前现象由两个独立问题组成：

1. 家宽聚合卡把 `category`（配置 target / 未分类）当排名维度，且没有先过滤家宽子集。因此只有一个 target 时天然只有一个桶；target 未精确命中时，全部流量还会折叠进“未知”。
2. `ReportQuery.sort` 虽已存在于 Rust / TypeScript 契约并进入 query echo，但六类排名 SQL 全部硬编码为“下行降序后 LIMIT”。如果只在前端把返回行改排成上行降序，会漏掉“上行很高、下行不在下载 Top N”的地址，无法得到精确上行热点。

趋势查询本来按时间升序返回，适合折线图；家宽页表格直接复用了同一数组，所以也从旧到新。正确修复是只对表格创建降序副本，不能反转 `ReportResult.series` 或传给图表的数组。

## 1. 排名维度用错

- `residential-monitor/src/components/features/residential/aggregate-section.tsx:30-35` 构造 `useReport` 时使用 `grouping: "category"`，没有 category 过滤。
- 同文件 `:73-78` 把返回排名画成条形图，`:132-142` 把 category 行直接画成“名称 / 上行 / 下行 / 份额”表。份额固定使用下行分母。
- 文案仍明确写成“按 target 节点排名”（`residential-monitor/src/i18n/zh.ts:656`、`en.ts:656`）。
- `ReportSection` 也用 `grouping: "category"` + 空过滤（`residential-monitor/src/components/features/residential/report-section.tsx:24-31`），因此家宽手动报告 / 导出仍是 target 桶，不是地址拆解。

这不是数据采集缺少地址。实时快照已经显示 `agentn.api5.cursor.sh`、`cli-chat-proxy.grok.com`；历史存储也有统一 Host identity。

## 2. 已有能力足以按家宽目的主机拆解

### Host identity

`residential-monitor/src-tauri/src/session_host.rs:18-26` 的权威顺序是：

```text
metadata.host -> sniffHost -> destination IP -> 缺失
```

`prefer_host_identity`（`:29-47`）还会用后续域名升级先前 IP，同时避免空字段或临时 IP 覆盖可信域名。`c3::service` 已有 `host_rank_uses_destination_ip_and_keeps_empty_unknown` 回归测试（`residential-monitor/src-tauri/src/c3/service.rs:1045`）。因此新任务不需要增加地址字段、修改 schema 或另造身份算法。

### 家宽过滤

- 前后端已经共享 `__residential__` 哨兵（`residential-monitor/src/format/rank.ts:7`、`residential-monitor/src-tauri/src/c3/sql.rs:13`）。
- raw 层把它翻译为 `a.primary_category_id is not null`（`sql.rs:353-356`）。
- hourly / daily dimension 层把它翻译为 `h.category_id != 0`（`sql.rs:371-374`）。
- `residential_accounting_filter_keeps_tagged_sessions` 已证明该过滤会排除未标记会话，并保持 totals / series 守恒（`c3/service.rs:1264-1287`）。

所以正确查询形态是：

```text
家宽筛选：filters.category = "__residential__"
地址分组：grouping = "host"
```

这显式区分了“哪些连接属于家宽”和“按什么拆解”。未命中精确 target 的流量应被排除并显示空态，不能再以全局“未知”流量冒充家宽。

## 3. `sort` 契约目前是无效字段

- Rust `SortField` 已允许 `upload | download | name | identity`（`residential-monitor/src-tauri/src/c3/query.rs:165-171`），`ReportQuery` 保存 `SortSpec`（`:239-251`）。
- TypeScript DTO 同样严格声明并解码上述字段（`residential-monitor/src/dto.ts:322-334`、`:559-607`）。
- `buildReportQuery` 会把调用方排序写入 query（`residential-monitor/src/hooks/use-report.ts:83-104`）。
- 但 `c3::service` 没有读取 `query.sort`；raw host / attr / rule / chain 与 hourly / daily 排名 SQL 全部硬编码 `order by sum(download) desc ... limit ?`（`residential-monitor/src-tauri/src/c3/sql.rs:47-105,159-185,208-234`）。

因此当前 `sort` 只影响 fingerprint / echo，不影响排名。前端客户端排序只能重排“下载 Top N 候选集”，不可能证明精确上传 Top N。修复必须在 SQLite `LIMIT` 之前应用经过枚举白名单生成的 ORDER BY，并在 raw、hourly、daily 三层保持一致。

## 4. 趋势顺序和表格样式

- raw、hourly、daily series SQL 都按 bucket 升序，供趋势图从左到右展示时间；这是正确契约。
- `AggregateSection` 把 `series` 原样同时传给 `TrendArea`（`:167-175`）和表格 `series.map`（`:194-201`），所以表格也是旧→新。
- 当前表格滚动容器没有边框 / 圆角，表头不 sticky，数值列没有右对齐，时间与数值缺少稳定 padding（`:177-201`）。截图中滚动到中段后表头不可见，三列只剩散落文本。
- 分析报告的 `TrendCard` 已提供 rounded border、左右对齐与 hover 行样式，可作为项目内视觉依据；实时连接表已有 sticky header 模式。无需新建通用设计系统。

## 5. 任务边界

### 需要修改

- C3 排名 SQL 的安全排序渲染及 raw / hourly / daily 回归测试。
- 家宽聚合查询：host grouping + residential category filter + 上/下行方向控件。
- 家宽手动报告查询：host grouping + residential category filter。
- 家宽地址排名的方向数值、份额、IP 标签和可访问排序状态。
- 家宽趋势表：新→旧的表格投影，以及 sticky header / 边框 / 对齐 / hover / 窄屏滚动。
- 双语文案、语义测试、规范同步。

### 不需要修改

- SQLite schema、migration、Host identity、DTO 字段或实时采集协议。
- `target` 精确匹配语义与历史回填。
- SOCKS5 / TLS 隧道内部域名解析。
- 全局分析报告页面或其它排名表的视觉重构。

## 6. 风险与验证重点

1. **全局排序行为**：修复 `ReportQuery.sort` 会作用于所有 C3 排名，但默认仍是 download descending，现有页面行为不变。必须对四个 sort field、两个方向与稳定 identity tie-break 做单测。
2. **截断正确性**：fixture 必须构造“上传第一不在下载 Top N”的样本，并令 `topN=1`；否则客户端重排伪修复也会通过。
3. **切换期间陈旧结果**：`useReport` 请求中保留上次结果。方向 / Top N 切换时，排名区必须用 `queryEcho` 校验结果属于当前选择，或明确标记旧方向，不能让控件显示“上行”却仍画下载排名。
4. **时间数组不变性**：测试同时断言图表输入仍升序、表格行严格降序，避免一次 `reverse()` 原地修改污染图表与其它消费者。
5. **配置未命中**：category filter 后零条是诚实空态；不得回退到全量 Unknown。原生 WebView 对真实 target 配置与视觉状态的验证必须单独记录，未执行时为 `UNVERIFIED`。

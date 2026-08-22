# 设计：进程页能力

## 1. 未知进程过滤

与主机维相同的哨兵模式。

`c3/sql.rs` `filter_clause`：`filters.process == "__unknown__"` 时使用与字段归因相同的缺失谓词（`a.process_id is null or not exists (select 1 from dimension_dict q where q.dimension_kind='process' and q.dimension_id=a.process_id)`），不绑定哨兵。

`append_dim_identity`：`kind == "process"` 且值为 `__unknown__` 时 `h.dimension_id = 0`。

`filtersForDrilldown`：`kind === "process" && isUnknownIdentity` 时设置 `process: "__unknown__"`。规则 / 链路未知仍返回空过滤。

`RankTable` / `DimensionPage`：`canDrill` 在 `kind === "host" || kind === "process"` 时允许未知行。

## 2. 核算口径开关

不新增 `ReportFilters` 字段。打开开关时 `filters.category = "__residential__"`。

`filter_clause`：该哨兵 → `a.primary_category_id is not null`，不绑定。精确层 → `h.category_id != 0`。

进程页本地 state：`residentialOnly` 布尔，默认 false。写入 `useReport` 的 filters 时与下钻 filters 合并：下钻 identity 优先，核算开关仍 AND。

前端不枚举重点目标名称，避免多目标漏计。

## 3. 缺字段空态

`RankBarCard`：`kind === "process"` 且 `attributionQuality.status === "unavailable"` 时不把 rankings 送进 `RankBar`。在归因说明下增加进程专用说明（i18n 中/英）和下一步。`RankTable` 照常渲染，便于未知行下钻。

`partial` / `complete` 仍画条形图；未知行留在图中。

## 4. 当前帧覆盖

`MetadataCoverage` 已在 `AppFacade` 每帧计算并进入诊断快照。把同一结构挂到 `LiveOverview`（Channel 载荷）。前端 decoder 必填互斥计数。进程页从 App 已有 overview 读取，不新开 IPC、不把 `processPath` 送进界面。

诊断面板可不改；进程页是展示面。

## 5. 兼容

- 不升 SQLite `SCHEMA_VERSION`。
- `__residential__` / `__unknown__` 不得 intern 进 `dimension_dict`（`intern_dim` 已拒 `__unknown__`；核算哨兵只出现在查询 filters）。
- 旧档案 `queryEcho.filters.category` 不会是 `__residential__`。

## 6. 回滚

去掉未知进程过滤与核算哨兵后，旧查询不再匹配空进程集合。LiveOverview 去掉 coverage 字段时 decoder 测试一并回退。

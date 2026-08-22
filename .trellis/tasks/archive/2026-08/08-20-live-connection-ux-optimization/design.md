# 设计：实时连接界面优化

## 设计目标与边界

实时连接仍是一个高密度 Operate 表面：原生 HTML table、现有侧栏/主题、Tauri Channel 和后端查询继续是权威边界。此次只重组筛选交互、稳定列宽拖动和新增两个方向热点摘要，不改变 collector、核算、coverage、关闭单条连接或本地 secret 处理。

## 组件与数据流

```text
Monitor Channel / snapshot
          │
          ├── health / pause / gap → 状态条与摘要状态
          │
          └── queryLiveConnections(appliedQuery)
                 │
                 ├── rows + nextCursor → 原生 table
                 └── matchedCount + topDownload + topUpload + sampleUtc
                                      → 两张热点摘要卡
```

### 前端状态分层

- `liveQuery`：唯一已应用查询，仍只留当前会话；每次应用筛选或排序都把 cursor 置空。
- `liveFilterDraft`：筛选面板的未提交输入，保留原始单位和文本，不触发 query。条件行只在字段/模式/单位变更后更新 draft；输入框按键不重绘整页。
- `liveQueryRequestSeq`：每次应用查询递增 token；`refreshLivePage` 捕获 token，过期响应不得覆盖较新的 rows/summary。应用态显示短暂 loading，失败保留上一份已应用结果并显示可恢复错误。
- `liveTableLayout`：保持现有持久化模型；拖动期间只更新 DOM 的目标 `<col>` 与 table width，拖动结束才写设置并重新 paint。
- `liveSummary`：随 `ConnectionPage` 同步保存，绝不从当前页 rows 二次聚合。summary 的 `sampleUtc` 与 query 响应同一 hub 快照；状态/gap 仍由 monitor snapshot/collectorRunning 决定。

### 筛选拓扑

1. 快速筛选区：突出「只看家宽」，显示当前命中数/最后应用时间，以及“添加条件”“清空全部”。
2. 已应用条件区：每条条件以可读 chip/紧凑行表达“字段 操作 值 [单位]”，提供单条删除；编辑入口打开同一 draft 控件。
3. 条件编辑区：字段选择决定模式和单位；回车或显式“应用”提交，Escape/取消恢复 draft；失焦可作为提交但不能在每个 input 事件提交。最多 8 条由 UI 与后端共同限制。
4. 列管理区独立于筛选区，保留显隐与恢复默认，不把列宽/列显隐混入筛选条件。

保留 `toQueryClause` 的前端展示单位到原始字节/毫秒转换；后端仍执行 AND、空值忽略和字段匹配。若使用 chip 展示，必须 escape 连接字段文本，不能把原始 mihomo JSON 插入 DOM。

### 固定列宽与拖动

- `<colgroup>` 是唯一尺寸源；table 使用显式像素 `width`、`table-layout: fixed`，wrapper 使用 `overflow: auto`。容器始终占可用宽度，只有 table 内容宽度随可见列宽总和变化。
- resize handle 使用 `pointerdown` + `setPointerCapture(pointerId)`；状态记录 `col/startX/startW/pointerId/dirty`。pointermove 只调用 `setColumnWidth` 的 clamp 结果并更新目标 col/table 的 style，不调用 paint。
- `pointerup`、`pointercancel`、`lostpointercapture` 和窗口失焦统一走结束函数：释放状态、只持久化一次；保存失败保持内存布局并给出非阻断提示。重新渲染前恢复 wrapper scrollTop/scrollLeft。
- handle 暴露 focusable separator 语义（或旁边提供“调整列宽”键盘操作），文案说明最小/最大值；排序按钮继续设置 header 的 `aria-sort`。不把整个 table 改为 ARIA grid。
- 持久化只写现有 `live_table_layout` 设置键，非法 payload 仍经 Rust/TS sanitize 回退默认，至少保留一列可见。

### 两个方向热点摘要

产品口径已确认：当前筛选结果中的最高下载累计连接 + 最高上传累计连接。

推荐扩展 `ConnectionPage`（不改 route/command 名称）为：

```text
ConnectionPage {
  rows: LiveConnectionView[]
  nextCursor: ConnectionCursor | null
  matchedCount: number
  sampleUtc: number | null
  summary: {
    topDownload: ConnectionHotspot | null
    topUpload: ConnectionHotspot | null
  }
}
ConnectionHotspot {
  identity: string
  label: string | null
  processName: string | null
  destination: string | null
  value: number
}
```

Rust `query_connections_with_targets` 先建立完整 matched 集合，再在同一集合上以数值降序、identity 作为稳定 tie-break 计算 summary，最后按用户 sort/cursor 截取 rows；这样 `limit=200` 不会把截断页误写成全量 Top 1。`AppFacade::query` 需要在同一 hub 锁定快照下取得 rows 与 `sampleUtc`，避免 summary 与 rows 跨采样。

若 backend 评估后决定不新增字段，必须证明当前 query 返回的是完整匹配集；否则该方案不可接受。前端 decoder 对 summary/matchedCount/sampleUtc 做显式校验，旧/缺字段响应进入能力未知状态而不是假造 0。

卡片只显示方向值和可识别标签：主机优先，缺失时进程/目标，再缺失显示「未知」；不显示 secret、完整原始 payload 或不必要的身份细节。无匹配显示“当前筛选无连接”，暂停/缺口/未连接显示对应状态而不是 0。卡片若使用图标或微型视觉条，必须同时提供文本值与同口径说明，不引入图表库/远程资源。

## 兼容性与回滚

- 不动 SQLite、C2 collector、Monitor Channel 消息和现有命令名；ConnectionPage 新字段只由实时查询消费。若 DTO 扩展验证或 Rust 编译出现边界问题，可先回滚 summary 字段/卡片，保留筛选和列宽子任务。
- 筛选子任务失败时可回退到既有 `liveQuery.filter`，不改变后端过滤语义；列宽失败时保留现有 clamp/persist 实现；摘要失败时卡片显示“能力未知/无可用摘要”，不回退为前端 Top 1。
- 任何手动 Windows/Tauri 实机验证都单独标为 `UNVERIFIED`，不能由 Vite/单元测试冒充。

## 可观测性与安全

- 查询请求只携带已存在的过滤字段、sort/cursor/limit；不把 secret、原始连接 JSON 或 SQL 暴露到前端。
- 研究和测试夹具使用脱敏主机/进程；summary label 做长度截断/HTML escape，保持未知语义。
- 研究报告引用的 Elastic/Grafana/W3C/MDN 仅作为交互与平台依据，不改变本产品数据口径。

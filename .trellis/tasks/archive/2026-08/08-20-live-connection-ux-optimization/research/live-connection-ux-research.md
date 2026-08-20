# 实时连接界面 UX 研究

## 仓库证据

### 筛选

- `residential-monitor/src/main.ts:311-371` 将所有字段条件直接展开在实时工具条下方；条件行同时承载字段、模式、值、单位和删除，页面一旦 `paint()` 会重新生成整段 HTML。
- `residential-monitor/src/main.ts:1270-1347` 对条件值有 input 状态更新和 change 请求两条路径。当前没有应用/取消模型，也没有请求序号或 AbortController；这解释了输入时容易重绘、焦点/光标不稳定的风险。
- `residential-monitor/src-tauri/src/c2/query.rs:74-163` 是真实过滤语义：只看家宽与最多 8 个 AND 条件，空值忽略；数值条件在后端按原始字节/毫秒比较。

### 表格宽度

- `residential-monitor/src/main.ts:374-396` 已显式写入 table width 与每个 `<col>` 的像素宽度；`src/live-table-layout.ts:104-159` 负责默认值、clamp、隐藏列与总宽度。
- `residential-monitor/src/main.ts:1437-1463` 在 pointermove 期间同时修改目标 `<col>` 和整张表的 width，松手后持久化并触发整页重绘。需要补齐 pointer capture/取消和只提交一次的状态机，并验证重绘不会把未拖动列重新分配。
- MDN 的 `table-layout: fixed` 说明：只有明确的 table width 与列宽时固定算法才生效；列宽不依赖内容，溢出由单元格 overflow 处理。来源：[MDN table-layout](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/table-layout)。

### 摘要数据边界

- `residential-monitor/src/ipc/live-session.ts:90-105` 目前只有 `query_live_connections` 返回当前页 rows/nextCursor；`src/dto.ts:41-64` 的行 DTO 已含 download/upload/rate/host/process/chains 等事实。
- `.trellis/spec/residential-monitor/frontend/view-state.md:7-9` 禁止前端实现分类、守恒、Top N 或导出统计，并要求图表有对应数据表；`.trellis/spec/residential-monitor/frontend/dto-and-decoding.md:23-27,49-51` 要求列表走后端查询、缺口不猜测。
- 因此“最高占用”若要代表全量当前连接，不能仅对 `limit=200` 的 UI rows 做未经标注的前端排名；应优先评估后端把摘要作为同一 query response 的可校验字段，或明确摘要只代表当前返回页并在文案中写清。

## 外部研究与可迁移结论

1. Elastic Kibana 的官方 controls 文档把筛选拆成可命名的 options list、数值 range slider，并支持自动应用、手动 Apply、清空和链式依赖。这支持把“快速筛选”和“已应用条件”分层，而不是把所有控件挤在同一行。来源：[Elastic Add filter controls](https://www.elastic.co/guide/en/kibana/current/add-controls.html)。
2. Grafana 官方变量/过滤器文档把变量控件放在 dashboard 顶部，变更后统一刷新相关面板；过滤器支持字段、操作符和值，并可选择自动应用或先 Apply。来源：[Grafana Variables](https://grafana.com/docs/grafana/latest/visualizations/dashboards/variables/)、[Grafana Dashboard controls](https://grafana.com/docs/grafana/latest/visualizations/dashboards/build-dashboards/create-dashboard/dashboard-controls/)。
3. Grafana 官方 Table 文档支持列标题过滤图标、列值搜索、contains/表达式/比较操作、清除过滤与排序；这说明“从表头或筛选条快速定位、已应用过滤可见且可清除”比长串裸 select 更可扫读。来源：[Grafana Table visualization](https://grafana.com/docs/grafana/latest/visualizations/panels-visualizations/visualizations/table/)。本项目仍受“原生 HTML、无 UI 框架”约束，借鉴信息层级而不是复制组件。
4. W3C APG 建议原生 HTML table 优先，排序列在 header 上提供 `aria-sort`；当前项目已有排序按钮，列宽把手需要补可达替代。来源：[WAI-ARIA Table Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/table/)、[Grid and Table Properties](https://www.w3.org/WAI/ARIA/apg/practices/grid-and-table-properties/)。

## 研究结论

- 推荐采用“快速开关 + 已应用条件 chips/行 + 显式添加/清空”的筛选拓扑，默认不把每次键入都变成查询请求；只在用户确认条件后更新 query，并用 query token 丢弃过期响应。
- 推荐保留固定像素表宽度与横向滚动，使用表格/colgroup 作为唯一尺寸源；拖动状态以 pointer capture 管理，重绘只恢复同一布局，不让内容测量决定列宽。
- 两张摘要卡应共享一次后端 query 的 `summary` 口径和状态字段。产品口径未确认前，不应在任务规划中擅自选择“最高下载/最高上传”还是“最高下载/活跃数”。

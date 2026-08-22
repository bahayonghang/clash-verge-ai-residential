# 实时表列宽跳动与表头排序：证据与方案

截图：用户 Image #1（红框列）、Image #2（换行挤扁）。对照 Clash Verge Rev `connection-table.tsx`（dev，2026-08-19 拉取）。

## 问题（一句话）

用户在实时表里要按列比较行，同时列宽不能随 1 Hz 刷新和单元格字长一起晃。

## 基本事实

- 窗口默认 1200×800，左侧栏占宽。主区大约 1000px 要放下 13 列。
- 采集 1 Hz。`connectionDelta` 后 `refreshLivePage()` 再 `paint()`，整页 `innerHTML` 重写。
- 表格权威是 `query_live_connections`，默认 `sortField=identity`、`limit=200`。不以 Channel upsert 排序。
- 已有筛选：「只看家宽」+ 最多 8 条「添加条件」（字段 + 精确/包含 + 文本，AND）。字段：`host` / `chain` / `rule` / `process` / `source` / `destination` / `type`。
- `sort_key` 已支持：`host`、`process`、`rule`、`network`、`upload`、`download`，其余回落到 `identity`。前端从不改 `sortField`。
- 归档任务 `08-19-live-table-filter` / `08-19-live-filter-toolbar` 把「表头排序」「列宽拖动」写在范围外。
- 禁止 UI 框架、远程资源。Vanilla TS + CSS。
- 未知不得画成零。

## 截图对照

Image #1：红框盖住「下载、上传、下载速度、上传速度、链路、规则、进程、时间、来源、目标、类型」。未框「主机」「操作」。表头单行，单元格大多单行。

Image #2：同一页在较窄可用宽度下：

- 「进程」竖排成「进 / 程」
- 「时间」拆成「34 分 / 钟前」
- 「链路」拆成「家宽-SOCKS5 / AI-家 / 宽」
- 主机域名中段换行
- 数字列宽随当前页最长字串收缩

根因是自动列宽 + 允许换行，不是主题色或间距。

## 列宽为什么会晃

按影响从大到小：

1. `residential-monitor/src/styles.css`：`.live-table { display: block; overflow-x: auto; }`。`display: block` 拆掉表格网格，列不再共用一套宽度。
2. 没有 `table-layout: fixed`，没有 `colgroup` / `col` 宽。浏览器按当前页内容算列宽。
3. `th` / `td` 允许换行。两字表头（进程、时间）被挤成竖排。
4. 1 Hz 全页重绘。字长变化例子：`0 B/s` ↔ `309.6 KiB/s`，`未知` ↔ `160.79.104.10:443`。自动布局每秒重算。
5. `.live-table-wrap` 与 `.live-table` 都设了横向滚动，换行和滚动互相抢。
6. `paint()` 只按 `id` 回焦点，不保存 `.live-table-wrap` 的 `scrollTop` / `scrollLeft`。重绘后滚动条回到原点，观感也是「表在跳」。

Clash Verge Rev 的做法：每列写死默认像素宽（下载/上传/速度 76，链路 280，规则 220，进程 180，时间 100，来源/目标 160，类型 120，主机 180），`table-layout` 等价于固定列，超出横向滚。另有 localStorage 记用户拖宽、显隐、列序；表头点击排序；行虚拟化。本仓库不能搬 React Table。

13 列按 Clash 默认宽相加约 1780px，再加「操作」约 72px，合计约 1850px，超过默认窗口主区。固定像素列宽必然出现横向滚动。这是稳定列宽的代价。若改成各列百分比撑满视口，列宽不再随字长跳，但链路/规则仍会在窄列里省略或换行。

## 排序合同缺口

`c2/query.rs` `sort_key`：

| 列 | 现有 key | 缺什么 |
|---|---|---|
| 主机 | `host` | 前端未接线 |
| 下载 | `download` 二十位补零 | 前端未接线 |
| 上传 | `upload` 二十位补零 | 前端未接线 |
| 下载速度 | 无 | `rate_download: Option<u64>` |
| 上传速度 | 无 | `rate_upload: Option<u64>` |
| 链路 | 无 | `chains` 连接串 |
| 规则 | `rule` | 未含 `rule(payload)` 展示串 |
| 进程 | `process` | 前端未接线 |
| 时间 | 无 | `duration_ms: Option<u64>` |
| 来源 | 无 | `source_ip` + 端口 |
| 目标 | 无 | `destination_ip` + 端口 |
| 类型 | 仅 `network` | 展示是 `Tun(tcp)` |
| 默认 | `identity` | 稳定平局 |

`Option` 字段不能把 `None` 写成 `0` 再比大小，否则「未知」与 `0 B/s` 混在一起。升序、降序都要把未知放到有值集合的另一侧（建议：未知永远在有值之后）。平局继续用 `identity`。

keyset 游标用 `sort_key` 字符串。新字段必须与比较函数使用同一编码。当前页仍是第一页 `limit=200`，没有翻页 UI。改排序必须清 `cursor` 再查第一页。

`dto-and-decoding.md` 写死每次 delta 后查「默认第一页 `sortField=identity`」。实施时要改成「当前 `liveQuery` 的第一页」，默认仍是 identity。会话内用户排序在 delta 刷新后保持。

Clash 排序在浏览器里对全量连接数组做。本产品列表权威在 Rust 查询层，不能改成前端重排当前 200 行而假装全表顺序。

## 「排序筛选」三种读法

1. 表头点击排序；筛选继续用工具条「添加条件」。Clash 表头就是这种：点表头排序，没有 Excel 式列菜单。
2. 红框列补进筛选。文本列已经在 `FILTER_FIELDS` 里。缺的是下载 / 上传 / 速度 / 时间的数值比较。对展示串做「包含」会误伤（`38.6 KiB` 包含 `38`）。
3. 每列表头弹出排序+筛选菜单，与「添加条件」并行，交互重复。

## 规划决定（2026-08-19）

用户选择 **B+C**：

- 表头点击排序（查询层）。
- 「添加条件」增加下载 / 上传 / 速度 / 时间的数值比较。
- 可拖数据列宽，列显隐，写入 `machine_setting`，重启保持。
- 列重排不做。排序与筛选仍只留会话。
- 操作列不可隐藏、不可排序、不拖宽。
- 数值输入：数字框 + 单位下拉。前端换成整数（字节或毫秒）再交给查询。

## 机制（B+C）

列宽：

- 去掉 `.live-table { display: block }`。滚动只留在 `.live-table-wrap`。
- `table-layout: fixed` + 默认像素宽（Clash 模板，速度列略宽以容纳 `309.6 KiB/s`）。
- `th`/`td` 单行 + 省略号。数字列 `tabular-nums`，右对齐。
- 数据列右缘可拖，最小宽约束。显隐开关 + 恢复默认。操作列不拖、不藏。
- 列宽和显隐写入 `machine_setting`，非法回落默认。Recovery 无库只留内存。
- `thead th` sticky。`paint()` 保存并恢复表格容器滚动位置。
- 列重排不做。

排序：

- 十二个数据列可点。操作列不排。
- 点击循环：降序 → 升序 → 回到 `identity`。
- `liveQuery.sortField` / `descending` 只留当前会话。每次改排序查第一页。
- 补齐 `sort_key`：`rateDownload`、`rateUpload`、`chain`、`duration`、`source`、`destination`、`type`；`rule` 与展示串对齐。
- 未知排在有值之后。表头 `aria-sort` + 可见箭头。文案跟 `uiLocale`。

筛选：

- 工具条「添加条件」增加数值字段。不在表头做 Excel 菜单。
- 比较原始字节或毫秒。未知不命中。数字 + 单位下拉，前端换算。

## 不做这些的理由

- 列重排与虚拟化：C 未包含列序；当前页上限 200。
- 前端按当前页排序：200 行之外的顺序是假的。
- 关闭全部、详情抽屉、改核算、改 Channel schema。

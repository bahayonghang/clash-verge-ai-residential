# 实施计划：侧栏色井、图表浮层、排名表头

## 启动前

- [ ] 用户明确批准本规划摘要后，再 `python ./.trellis/scripts/task.py start 08-21-neko-shell-readability`。
- [ ] 读 `.trellis/spec/residential-monitor/frontend/index.md`、`view-state.md`、`DESIGN.md`、Impeccable `craft-floor`。
- [ ] 记录基线：`git status --short`。工作区应干净；不要夹带无关文件。

顺序：token → 侧栏色井/间距 → 图表浮层 → 表头/归因 → DESIGN.md → 测试与实拍。token 必须先于色井和浮层，否则四主题对不上。

## 有序清单

### 1. Token

- [ ] `globals.css` 四主题：抬升暗色 `--popover`（见 `design.md` 表）；Latte 不动底，靠阴影。
- [ ] 声明 `--nav-overview` … `--nav-alerts`；`--nav-item-gap` 写入 comfortable / compact。
- [ ] `prefers-contrast: more` 块不删除；不必为九色各写一套。

### 2. 侧栏

- [ ] 业务项包色井；`--nav-tint` 来自单一映射表。选中行白图标浅白井。关于 / 设置无井。
- [ ] `nav` 使用 `gap-[var(--nav-item-gap)]`。底栏维持紧凑 gap。
- [ ] 更新 `sidebar.test.tsx`：九段 `data-nav-tint`（或等价）、关于无 tint、recovery 仍无业务导航、英文 220px 结构断言仍过。

### 3. 图表浮层

- [ ] 新增 `ChartHover` 外壳。`RankBar` 自定义 `content`，标题=label，数值=`valueFormatter`。
- [ ] Y 轴 `fill` 改为 `var(--muted-foreground)`。
- [ ] `TrendTooltip` 改用同一外壳（边框/底/阴影不三套）。
- [ ] `ShareDonut`：无 `onHover` 时挂 `ChartHover`；有 `onHover` 仍不渲染 Recharts `Tooltip`。
- [ ] 测试：RankBar 静态标记或 DOM 断言不含 `value :`；空态/加载态不变。

### 4. 表头与归因

- [ ] 新增 `SortableTh`。`RankTable` 与 `RankingTable` 接入。
- [ ] `RankTable` thead 加背景与字重；默认下行 `aria-sort="descending"` 且降序图标在 DOM。
- [ ] 删除 `RankTable` 的 `AttributionQualityNote`。`RankBarCard` 保留。
- [ ] `rank-table.test.tsx`：默认降序图标、不可排序列无 button/图标、渲染结果无「字段归因」。

### 5. DESIGN.md

- [ ] 修订 One Blue Rule 与 Navigation / Data table 段，与落地一致。

### 6. 检查

- [ ] `npm --prefix residential-monitor run typecheck`
- [ ] `npm --prefix residential-monitor run lint`
- [ ] `npm --prefix residential-monitor test`
- [ ] `npm --prefix residential-monitor run build`
- [ ] `git diff --check`
- [ ] 1200×800 Mocha 规则页：色井、悬停浮层、表头图标；切 Latte；compact 底栏仍在。无浏览器工具时写明 UNVERIFIED 项。

## 风险与回滚

- compact + 英文长标签 + 色井占宽：井必须 `shrink-0`，标题 `min-w-0 truncate`。220px 断言失败时先减井尺寸，不缩短英文。
- 800px 高窗口：只加 `--nav-item-gap`，不要同时加大 `--nav-item-py`。
- `color-mix` 与 hex+`15` 混用会让井底在 CSS 变量上失效。一律 `color-mix`。
- 回滚：还原本清单文件。无数据迁移。

## 不在本任务改

- `live-table-sort.ts` 的 Unicode 三角。
- Rust / DTO / 路由列表。
- 报告扇形图 inspect 钉住。

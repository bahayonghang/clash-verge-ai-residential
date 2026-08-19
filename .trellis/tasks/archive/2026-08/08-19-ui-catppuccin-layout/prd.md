# 概览布局、Catppuccin 主题与实时筛选 UI

## Goal

用户打开「家宽流量监控」后，概览按口径成对读上下行，实时连接的筛选是紧凑工具条，设置里可在 Catppuccin 四口味之间切换。窗口仍是专家工具。

## Task Map

父任务拥有需求源和跨子任务验收。实施从子任务开始，不在父任务上 `task.py start`。

| 子任务 | 交付 | 顺序 |
|---|---|---|
| `08-19-ui-catppuccin-theme` | 四口味 token、设置切换、持久化、Recovery 与五页换肤、主/次按钮角色 | 先做 |
| `08-19-overview-caliber-layout` | 成对口径分组、补齐已有下行字段、分类表、消掉孤儿卡与灰底空洞 | 主题之后 |
| `08-19-live-filter-toolbar` | 筛选工具条重构；查询语义不变 | 主题之后，可与概览并行 |

## Background

- 壳层：左侧栏产品名 + 五条 route。主区不重复页面标题。
- `styles.css` `:root` 硬编码海军 / 浅灰 / `#3b82f6`。html `color-scheme: dark`，`.workspace` `color-scheme: light`。无主题入口。
- 语言经 `save_ui_locale` + 设置键 `ui_locale` 持久化。主题没有对等键。
- `LiveOverview` 已有成对上下行与 `categoryDownload`。`renderOverview` 只画 8 张等权卡，缺 `otherDownload` / `gapDownload` / `overDownload` / `categoryDownload`。网格 `auto-fit minmax(11.5rem, 1fr)` 在宽窗变成 7+1 孤儿卡，其下空分类浅卡，再下整块灰底。机械 layout 扫描无命中。
- 实时筛选语义已由 `08-19-live-table-filter` 交付。全局 `label { flex-direction: column; min-width: 12rem }` 把「只看家宽」拉成全宽竖排；全部 `button` 共用主色，导致「添加条件」通栏蓝条。
- 用户已确认概览策略：成对口径分组，使用已有 DTO，不把实时表搬到概览。
- 前端 Vanilla TypeScript + Vite。禁止 UI 框架、远程 URL、CDN。

## Requirements

### R1 Catppuccin 明暗主题

由 `08-19-ui-catppuccin-theme` 交付。

- 设置页在语言控件旁提供外观选择：Latte、Frappé、Macchiato、Mocha。存储键 `frappe` 无音调符号。默认 Mocha。
- 切换后立即重绘壳层、五页与 Recovery Shell。本机设置持久化，重启后保持。非法或缺失值回落 Mocha。Recovery 无业务库时只改当前进程内存，不阻塞恢复。
- 每个口味是完整构图：侧栏、主区、卡片、表格同属该口味。禁止把 Mocha 反相得到 Latte，禁止保留「深侧栏 + 浅主区」双色壳再涂 Catppuccin 蓝。
- Token 以 CSS 自定义属性本地提供。禁止 CDN、远程字体、运行时 npm 主题包。
- 语义角色：画布、抬升面、正文、次要文字、主操作、次要操作、焦点、边框、成功、危险、表头/表行。口味只重映射角色。
- 蓝只用于当前导航和主按钮。连接成功用绿、失败用红、Recovery 提示用黄。不把 Catppuccin 全色板撒到徽章上。
- `prefers-contrast: more` 覆盖当前口味。`:focus-visible` 可用。
- 本地导航图标在四口味侧栏底上可辨认。不能辨认则加对比底板或重生成，不引入远程图标。
- 中英文都有外观标签。口味专有名词保持 Latte / Frappé / Macchiato / Mocha。事实性文案、口号、删除确认短语不因换肤改写。

### R2 实时筛选工具条

由 `08-19-live-filter-toolbar` 交付。

- 筛选区改为紧凑工具条：健康与最后采样一行；「只看家宽」横向开关（文字与勾选同一行）；「添加条件」为次要按钮，不是通栏主色条。
- 条件行：字段、匹配方式、文本、删除。删除为次要操作。添加后表格仍在默认 1200×800 首屏。
- 纵向 `label` 规则只作用于设置 / 报告 / 告警表单，不再污染工具条与勾选。
- 主区纵向：工具条按内容高度，表格吃剩余高度并可横向滚动。
- 查询语义不改：默认只看家宽、最多 8 条、AND、空值忽略、只留当前会话、走 `query_live_connections`。单条关闭与五类空态不改。
- 不新增搜索框、关闭全部、列宽拖动、表头排序。

### R3 概览成对口径

由 `08-19-overview-caliber-layout` 交付。

- 用已有 DTO 成对展示：控制器 meter、可归因观测、其他连接、未归因 gap、over-attributed。每组同时给出上行与下行。`null` 显示「未知」，不画成零。
- 口径不得合并成「总流量」。meter / 可归因 / 残余分组可扫。
- 活跃连接、覆盖句、健康句放在同一状态区，不单独占一行孤儿卡。
- 重点分类为表：名称、上行、下行。键为 `categoryUpload` 与 `categoryDownload` 的并集。空态是表内一行「无」，不是整幅空浅卡。
- 默认 1200×800 与更宽窗都不出现「一行七卡 + 一张孤儿卡 + 大块灰底」。不为填空加营销卡、假图表或实时连接预览。
- 不新增核算，不在前端做 Top N。

### R4 既有合同

不改 route id、采集、核算、报告公式、告警引擎、备份恢复。主题只新增外观设置键与 bootstrap 可选字段，不改连接查询形状。前端仍只保存视图选择与 DTO 缓存。实时筛选条件仍只留当前会话。

## Out of Scope

- 跟随 Windows 应用模式自动换肤。
- 把实时表搬到概览；新增 route。
- 关闭全部连接、Clash CLOSED 页、虚拟化、翻页 UI、连接详情抽屉。
- 远程 Catppuccin 资源、UI 框架、新字体文件。
- 发布、签名、NSIS。

## Acceptance Criteria

- [ ] **AC1** 设置页可选 Latte / Frappé / Macchiato / Mocha；切换后五页与 Recovery 立即换肤；重启后仍是所选口味；非法值回落 Mocha。
- [ ] **AC2** 构建产物不请求远程 CSS / 字体 / 主题包。`prefers-contrast: more` 与 `:focus-visible` 在当前口味下可辨认。
- [ ] **AC3** 实时页「只看家宽」为横向开关；「添加条件」不是通栏主按钮；有条件时表格仍在默认窗口首屏。筛选查询结果与改 UI 前一致。
- [ ] **AC4** 概览成对展示已有上下行口径；分类表含下行；宽窗不再出现 7+1 孤儿卡加大块灰底。口径仍分开。
- [ ] **AC5** `npm --prefix residential-monitor` 的 typecheck、lint、test、build 通过。主题相关 Rust 测试通过。既有语言、删除确认、关于页签名、实时空态与筛选测试不被这次改版绕过。

## Key Decisions

- 概览：成对口径分组，只用已有 DTO，不拉实时表。
- 主题：Catppuccin 官方四口味，默认 Mocha，每个口味完整构图。
- 持久化：复用 `ui_locale` 的 `put_setting` 模式，键 `ui_theme`。
- 筛选：只重构 UI，不改 `query_live_connections` 合同。

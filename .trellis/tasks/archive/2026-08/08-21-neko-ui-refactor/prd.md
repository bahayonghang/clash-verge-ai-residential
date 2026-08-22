# residential-monitor 前端按 neko-master 重构

## Goal

把 residential-monitor 的整套界面替换为 `ref/neko-master/apps/web` 的视觉与信息架构：深/浅双模、圆角卡片、侧栏图标导航、Recharts 面积图与横向条形图、Top 列表进度条、分页数据表。功能范围仍然只覆盖 Clash Verge Rev / mihomo 控制器的连接事实，不引入 neko 的多后端、GeoIP、鉴权与 PWA 能力。「只看家宽」从实时页的一个复选框升级为独立的家宽页，包含实时监控、聚合统计与专用报告三段。

口径纪律不变：controller meter 与可归因观测各自成项，缺口与未知显示「未知」而不是 0。

## Background / Confirmed facts

### neko-master 与本项目是同一数据源

neko-master 不只是一套界面，它自带 Clash / Mihomo 接入与统计分析，吃的是同一份控制器数据，所以它的聚合契约同样可移植。

- neko 的 gateway collector 直连 Clash / Mihomo 控制器（`ref/neko-master/apps/collector/src/modules/collector/gateway.collector.ts`，694 行），配置项是名称 / 类型 `Clash / Mihomo` / 地址 / 端口 / Token，与 residential-monitor 的 `settings.address` + secret 等价。
- 连接载荷结构一致：`ref/neko-master/packages/shared/src/index.ts:4-42` 的 `ConnectionMetadata` / `Connection` 与 residential-monitor 的 `LiveConnectionView`（`residential-monitor/src-tauri/src/c2/hub.rs:14-38`）覆盖同一批字段（host、sourceIP、destinationIP、process、processPath、network、chains、rule、rulePayload、upload、download、start）。
- neko 的统计表按维度分层：`ref/neko-master/apps/collector/src/database/schema.ts` 有 `domain_stats`、`ip_stats`、`proxy_stats`、`rule_stats`、`device_stats`、`country_stats`，分钟层 `minute_stats` / `minute_dim_stats`、小时层 `hourly_dim_stats`，以及跨维表 `rule_chain_traffic`、`rule_domain_traffic`、`rule_ip_traffic`、`domain_proxy_stats`、`ip_proxy_stats`。截图里的「Rule Chain Flow」「Associated Domains」「Top IP Addresses 带 DOMAINS 列」就来自这批跨维表。
- 维度对应关系：neko Domains ↔ 本项目 `DimensionKind::Host`；Rules ↔ `Rule`；Proxies ↔ `Chain`；Devices（按 sourceIP 分设备）↔ 本项目更有意义的是 `Process`，`sourceIp` 字段也在，两者都可用。

### 可移植的聚合契约：规则名归并（重要）

- neko 的 `ref/neko-master/apps/collector/src/shared/utils/rule-name.ts:7-29` 定义 `buildRuleName`：**当链路多于一跳时，规则统计归并到最后一跳（顶层策略组），mihomo 在 `rule` 字段上报的原始规则类型（RuleSet / IPCIDR / Match…）不得作为多跳链路的聚合键**；只有链路无策略组跳（如直连 DIRECT / REJECT）时才用 `rule(rulePayload)`。`ref/neko-master/AGENTS.md:70` 把这条列为「违反过就出过真实回归」的第一条契约，v1.3.9 曾因此回归。
- residential-monitor 当前的写入与之不同：`residential-monitor/src-tauri/src/storage.rs:637` 把 `rule_id` 直接 intern 成 `row.rule`（mihomo 原始规则类型），`:640-644` 把 `chain_key` 存成 `row.chains.join(">")` 的完整链路串。
- 后果：直接按现有 `rule` 维度做「规则」聚合页，排行首项会是 `RuleSet` / `Match` 这类规则类型而不是策略组；按完整链路串做「链路」聚合页，会把同一个顶层策略组拆散到多行。这两页需要按 `buildRuleName` 的等价语义定义聚合键。做法已定为**查询期派生**（SQLite 标量函数 `last_chain_hop`，不改写入、不迁移），归 `08-21-c3-dimension-capability`；理由与 SQL 侧的一处语义差异见该子任务 `design.md` 第 2 节。

### 跨维下钻的能力边界已有契约

- `residential-monitor/src-tauri/src/c3/query.rs:615-627`：30 天 raw 期内 `sessions: true`、`current_policy: true`、`cross_dimension: true`、`exact_top_n: true`。
- 同文件 `:629-661`：超出 raw 期进入精确维度层，`cross_dimension: false`、`sessions: false`，只支持「历史主分类 + 单一分析维度」。
- 同文件 `:668-681`：更久的 daily core 层 `exact_top_n: false`，只有可归因总量与 coverage。
- 结论：neko 截图里的跨维下钻（规则 → 链路 → 域名、域名 → IP）在 30 天内可诚实提供，超出后必须按 `drilldownCapability` 降级，不得继续渲染下钻入口。

### Regions 世界地图为什么不做

- neko 的地理数据不来自控制器，而是自带 MaxMind：`ref/neko-master/apps/collector/src/modules/geo/geo.service.ts:86,394-479` 从本地 `geoip/` 目录加载 `GeoLite2-City.mmdb` / `GeoLite2-ASN.mmdb` / `GeoLite2-Country.mmdb`，`:265` 还有一条远程 `fetch(lookupUrl)` 兜底。
- mihomo 的连接元数据本身带 `destinationGeoIP: string[] | null` 与 `destinationIPASN`（`ref/neko-master/packages/shared/src/index.ts:8-9`，形状见 `packages/shared/src/geo-ip-utils.ts:19-33`：`[countryCode, countryName, city, asOrganization]`），但 residential-monitor 在采集边界就丢掉了这两个字段——`c2/hub.rs:14-38` 的 `LiveConnectionView` 没有地理字段。
- 因此本次不做 Regions：neko 的实现方式依赖 MMDB 文件与远程兜底，与本项目「数据只留本机、无遥测、禁止远程 URL」冲突；唯一合规路径是改采集层捕获 mihomo 自己解析出的 `destinationGeoIP`，但那取决于用户是否开启 mihomo 的 geoip，属于独立的采集能力扩展，不在界面重构范围内。

### 目标视觉来源

- `ref/` 已被 `.gitignore` 忽略，neko-master 只作参考，不进仓库、不作为依赖。
- neko 设计令牌在 `ref/neko-master/apps/web/app/globals.css:71-140`：`--radius: 0.75rem`；light `--background:#f5f7fa` / `--card:#ffffff` / `--primary:#0063ff`；dark `--background:#0b0f19` / `--card:rgba(23,28,43,0.8)` / `--primary:#3b82f6`；`--chart-1..5` 五色序列；body 叠三层 `radial-gradient`（`:149-165`）。
- 侧栏在 `ref/neko-master/apps/web/components/layout/navigation.tsx:109-281`：`w-64` sticky，顶部 logo + 状态圆点 + 双 ping 动画，导航项 `rounded-xl` 选中态整块 `bg-primary`，底部固定「关于 / 设置」。
- 顶栏在 `ref/neko-master/apps/web/app/[locale]/dashboard/components/header/index.tsx:158-596`：`h-14` sticky + `backdrop-blur-md`，右侧自动刷新旋转按钮、时间范围选择器、语言、主题，底部一条 `animate-progress-indeterminate` 进度条。
- 统计卡在 `ref/neko-master/apps/web/components/features/stats/stats-cards.tsx:68-108`：`rounded-xl p-3.5 border bg-card`，`w-8 h-8` 图标底色为主色 15% 透明度，标签 `text-[11px] uppercase tracking-[0.14em]`，数值 `tabular-nums` 且用 framer-motion 做数字过渡；同文件 `:126-163` 定义 shimmer 占位卡与不可用卡。
- 趋势图在 `ref/neko-master/apps/web/components/features/stats/charts/trend-chart.tsx:353-422`：Recharts `AreaChart`，下载 `#3b82f6` / 上传 `#a855f7`，两条 `linearGradient` 填充，`CartesianGrid strokeDasharray="3 3"` 且 `vertical={false}`，`isAnimationActive={false}`，高度固定 200px；`:177-247` 定义 loading 骨架与空态两个分支。
- Top 列表项在 `ref/neko-master/apps/web/components/common/top-list-item.tsx:46-98`：序号徽章（前三名金/银/铜）+ 图标 + 标题 + 右侧数值 + `h-1.5` 占比进度条，进度条透明度随占比从 0.7 升到 1.0。
- 卡片容器在 `ref/neko-master/apps/web/components/common/overview-card.tsx:22-43`：标题 `text-sm font-semibold uppercase tracking-wider`，`bg-gradient-to-b from-card to-card/50`。
- 页面装配在 `ref/neko-master/apps/web/app/[locale]/dashboard/components/content/index.tsx:332-405`：单个 `switch (activeTab)` 分发，每页外层统一 `space-y-6`。

### 当前实现

- 前端是单文件模板字符串渲染：`residential-monitor/src/main.ts` 3088 行，`renderOverview` 在 `:330`、`renderLive` 在 `:371`、`renderReports` 在 `:727`、`renderSettings` 在 `:1016`、`renderAlerts` 在 `:1145`、`navHtml` 在 `:1241`、`renderApp` 在 `:1345`。样式是手写 `residential-monitor/src/styles.css` 2234 行。零 UI 运行时依赖（`residential-monitor/package.json:18-20` 只有 `@tauri-apps/api`）。
- 路由契约是五段：`residential-monitor/src/dto.ts:5` 的 `RouteId = "overview" | "live" | "reports" | "alerts" | "settings-data"`，Rust 侧由 `list_routes` 提供，`residential-monitor/src/ipc/routes.test.ts:14-31` 断言五段顺序与图标齐备。
- 主题是四款 Catppuccin：`residential-monitor/src/theme.ts:1` 的 `UiTheme = "latte" | "frappe" | "macchiato" | "mocha"`，`:77-80` 只切 `data-theme` 与 `color-scheme`。字体、字号、密度、侧栏宽度各有独立持久化命令。
- 文案双语在 `residential-monitor/src/i18n/zh.ts`（22.7K）与 `en.ts`（24.0K），键必须成对存在。
- 家宽判定在 `residential-monitor/src-tauri/src/c2/query.rs:124-128`：链路节点等于 `save_targets` 配置的某个 target，或节点名包含「家宽」。过滤入口是 `ConnectionFilter.residential_only`（`:17`）。
- 报告维度已齐备：`residential-monitor/src-tauri/src/c3/query.rs:116-124` 的 `DimensionKind = Category | Host | Process | Rule | Chain | Network`。
- 分钟级序列的数据基础已存在：`connection_minute` 表在 `residential-monitor/src-tauri/src/storage.rs:168-174`，索引 `idx_connection_minute_utc` 在 `c3/schema.rs:105-106`；`c3/service.rs:227-238` 的 `SERIES_RAW` 已把 bucket 作为 `?3` 参数传入。限制只在 `c3/query.rs:108-113` 的 `Granularity` 枚举只有 `Hour | Day | Month`，以及 `c3/service.rs:222-226` 把它们硬映射成 60 / 1440 / 43200 分钟。neko 同样按分钟层建表（`minute_stats` / `minute_dim_stats`），路线一致。
- IPC 命令面共 59 个（`residential-monitor/src-tauri/src/lib.rs:1237-1296` 的 `generate_handler!`，registered 与 declared 均为 59）。本次重构复用现有命令，新增命令只在 `08-21-c3-dimension-capability` 与 `08-21-residential-page` 内，且只增不改。

### C3 查询与物化的五处既有缺口

审阅暴露出维度查询链路上五处缺陷。它们不是本次重构引入的，但会让新的聚合页显示错误或空白数据，因此必须在本任务内修掉。全部归 `08-21-c3-dimension-capability`。

1. **精确维度层只物化 host**。全仓对 `traffic_hourly_dimension` 的生产写入只有 `c3/retention.rs:103-119` 一处，`dimension_kind` 写死字符串 `'host'`；日层从小时层复制（`:136-155`）。读取端按 `dimension_kind = ?3` 过滤（`c3/sql.rs:64-121` 的六条 SQL）。所以 process / rule / chain / network 在超出 30 天 raw 期后是**空排名**。且 `needs_exact_dimension`（`c3/query.rs:576-584`）不含 `Category`，家宽长区间会掉到 DailyCore，而 `fill_core` 在 `c3/service.rs:456` 直接 `result.rankings.clear()`。
2. **能力报告谎报**。`plan_capability` 在维度层仍返回 `exact_top_n: true`（`c3/query.rs:629-661`），即「没有物化」被报告成「精确可用」。
3. **filters 不进排名与序列**。`RANK_RAW`（`c3/sql.rs:32-43`）与 `fill_raw_attr_rank`（`c3/service.rs:266-309`）都只有时间窗与 `top_n`，无任何 `ReportFilters`；`SERIES_RAW`（`c3/sql.rs:18-30`）的 where 只有 `(?4 = 0 or s.host = ?5)`。所以跨维下钻的排名与趋势会返回全局数据。`TOTALS_RAW`（`:3-16`）支持五个维度但**不支持 category**，且 `ReportFilters::dimension_filter_count()`（`c3/query.rs:162-173`）不数 category，`needs_raw` 不会因 category 过滤而触发——category 在整条链路上是二等过滤器。
4. **规则与链路的聚合键不可用**。`storage.rs:637` 把 `rule_id` intern 成 mihomo 上报的原始规则类型（RuleSet / IPCIDR / Match…），`:640-644` 把 `chain_key` 存成完整 `a>b>c` 串。`c3/service.rs:279-288` 的 `case ?3` 漏了 `chain` 分支，落到 `else a.host_id`，`DimensionKind::Chain` 取排名返回主机排名；`c3/service.rs:240` 把 `Category` 与 `Host` 一起路由到 `RANK_RAW`，而 `RANK_RAW` 无条件 `group by s.host`，所以 Category 排名也返回主机排名。
5. **排名之和与合计不闭合**。`RANK_HOURLY` / `RANK_DAILY_DIM`（`c3/sql.rs:82-92,111-121`）用 INNER JOIN `dimension_dict`，而物化时缺失维度值写成 `coalesce(a.X_id, 0)`，`dimension_dict` 无 `dimension_id = 0` 的行（`intern_dim` 在 `storage.rs:671-679` 对 None 返回 None）。缺失值的流量计入合计但被排名丢弃，差额无处可见，与「未知保持未知」冲突。

### 已排除的顾虑

`query_fingerprint`（`c3/query.rs:565-568`）只对 `ReportQuery` 的 JSON 做 SHA-256，与 SQL 文本无关。重构 SQL 或重排参数**不会**让 `report_snapshot_meta` 的已归档报告失效。

### 已确认的产品决策

用户在本次规划中确认了六项：

1. 前端栈引入 React 19 + Tailwind v4 + Recharts + lucide-react + Radix 基元，按 neko 组件近 1:1 移植。
2. 概览页保留成对口径（meter / 可归因 / 其他 / gap / over + 活跃连接），只套 neko 卡片外观，不改成单一总量卡。
3. 导航页面集：概览、实时连接、家宽、主机、规则、链路、进程；现有分析报告、告警、设置 / 数据管理三页保留并同步移植。不做 Regions 世界地图。
4. 家宽页交付家宽视角特有的读数与导出，**不新建与 `traffic_*_dimension` 平行的家宽统计表**。按 target 的历史聚合复用 `DimensionKind::Category`。
5. 精确维度层的物化从只写 `'host'` 扩展到 host / process / rule / chain / network 五个维度，并把 `Category` 纳入精确维度判定，使长区间排名可用。
6. 家宽判定保留两种语义（核算侧精确 target；实时筛选侧精确 target 或含「家宽」），但收敛到一个模块的两个具名函数，以消除两处实现各自漂移。实时页行为不变。


### 需要同步修改的既有约束

以下四处现行文字与本次决策直接冲突，属于本任务范围内必须一并更新的产物：

- `PRODUCT.md:33`「固定五页」与新的十段路由（九个业务页 + 设置 / 数据管理）冲突。
- `PRODUCT.md:37`「前端是 Vanilla TypeScript + Vite，不引入 UI 框架」与决策 1 冲突。禁止远程 URL 与 CDN 的部分保留。
- `DESIGN.md` 整份是 Catppuccin 深侧栏工作台的令牌与规则，`DESIGN.md:167` 明确「Don't 引入 UI 框架、远程字体或 CDN」。需要按 neko 令牌重写，保留禁止远程字体与 CDN。
- `.trellis/spec/residential-monitor/frontend/index.md`「不引入 UI 框架」需改写为 React + Tailwind 的约定，并保留禁止 `window.__TAURI__`、eval、远程 URL 与 CDN。

`PRODUCT.md:38`「前端只保存视图选择和 DTO 缓存，不在浏览器里重做核算或 Top N」**不修改**，继续约束全部聚合页：排名与合计必须来自 Rust 查询。

## Requirements

### R1. 一套设计系统，一次落地

- Tailwind v4 令牌层按 neko `globals.css` 移植：`--radius`、`--background`/`--foreground`/`--card`/`--popover`/`--primary`/`--secondary`/`--muted`/`--accent`/`--destructive`/`--border`/`--input`/`--ring`、`--chart-1..5`、`--sidebar-*` 全套。
- 现有四款 `UiTheme` 值保持不变（不改 Rust 契约、不迁移已持久化的值），重新定义为 neko 令牌变体。字体、字号、密度、侧栏宽度四项偏好继续生效。
- 字体全部走本机字体栈，不引入 webfont、不引入 Geist、不引入 emoji flag 字体，不触碰禁止远程资源的约束。
- 全部资源经 Vite 本地打包，产物中不得出现外部 URL。

### R2. 十段路由应用壳

- 导航顺序：概览、实时连接、家宽、主机、规则、链路、进程、分析报告、告警；底部固定「关于」「设置 / 数据管理」入口。
- `RouteId` 与 Rust `list_routes` 扩展保持一致，recovery-only 分支仍不渲染业务页。
- 顶栏承载连接状态、自动刷新开关、时间范围选择、语言、主题；不恢复顶栏横导航作为主认页方式。

### R3. 聚合数据只从 Rust 来，且必须是子集

- 主机 / 规则 / 链路 / 进程四页与概览页的 Top 列表，全部由 `run_report` 按 `DimensionKind` 取 `rankings`，前端不在 JS 里做分组、求和、Top N 或守恒。
- 趋势图需要的分钟级序列由 Rust 新增粒度提供，前端不用逐分钟连接快照自行累加。
- **规则聚合键按 `buildRuleName` 的等价语义定义**：链路多于一跳时归并到最后一跳（顶层策略组）。派生键的实现只有一处（SQLite 标量函数 `last_chain_hop`），规则维度与链路维度共用，前端不复制。注意这与 R6 的家宽判定不同：家宽有意保留两种语义。
- **链路聚合键取顶层策略组**，不用完整 `a>b>c` 串——后者会把同一策略组拆散到多行。
- 修复 `DimensionKind::Chain` 与 `DimensionKind::Category` 的排名回落到主机排名的缺陷。
- `ReportFilters` 必须在 totals、series、rankings 三处都生效，`filters.rule` / `filters.chain` 的匹配语义与派生键一致。**下钻返回全局数据视为缺陷，不视为近似。**
- 精确维度层必须物化排名所需的五个维度，并把 `Category` 纳入精确维度判定。
- 维度值缺失的流量不得被静默丢弃：排名里以「未知」一行出现，或在 `note_zh` 显式声明差额。
- 跨维下钻按 `drilldownCapability` 诚实降级；`exact_top_n` 必须反映该 grouping 是否真有物化数据，不得谎报。

### R4. 口径与未知不被视觉重构损坏

- 概览页顶部保留六格：controller meter、可归因观测、其他连接、未归因 gap、over-attributed、活跃连接。**前五格为上/下行成对读数**，第六格是活跃连接计数加覆盖与健康状态。
- 「可归因观测」的既有定义不得改写：它是每条连接 delta 之和，**包含未分类连接**（`accounting.rs:229-231`），对照 controller meter 的全局计数。不得把它重新解释为「已分类流量」。
- 任何位置的缺失值显示「未知」，不显示 0、不显示 `--` 之外的伪默认。coverage 缺口、暂停、未配置、未连接、订阅缺口继续各自成态。
- 「有覆盖且实测为 0」与「无覆盖」必须分开表达：`coalesce(sum, 0)` 的 0 只在有覆盖时可用作真实读数。
- 图表必须有同口径的文本或数据表兜底。

### R5. 现有功能零回退

- 实时连接页保留：后端筛选语义（字段 / contains / exact / 数值比较 / 单位换算 / 最多 8 条 AND / 空值忽略）、列宽与列显隐持久化、表头排序、关闭单条连接、方向热点摘要、专门空态。「只看家宽」的选中集合与改造前完全一致（见 R6）。
- 分析报告页保留：查询构造、series、rankings、coverage、drilldownCapability、policyMetadata、归档列表、导出预览与导出。
- 告警页保留：规则列表与编辑、告警中心分页、通知能力检测、outbox 积压、诊断快照与导出。
- 设置 / 数据管理页保留：控制器地址与 secret（含回填与显示切换）、targets、外观四项、保留预览与执行、备份 / 恢复 / 校验、数据目录、关于（未签名候选不得标成 signed）、删除本地数据的预览与二次确认短语、user vacuum。
- 不改动 C2 采集生命周期、Monitor Channel、托盘、凭据边界与存储 schema。C3 的物化范围扩展与新增 named SQL / 命令是本任务批准的例外，**不新增业务表**。
- 允许删除的渲染实现见 `design.md` 第 2 节；数据与解码纯函数不得重写。

### R6. 家宽独立页与判定收敛

- 三段结构：实时监控（当前家宽链路上的连接与速率）、聚合统计（按 target 节点分组的排名、占比与趋势）、生成报告（家宽报告与导出）。
- 家宽判定**保留两种语义**并收敛到一个模块的两个具名函数：
  - 核算侧：链路节点精确等于某个已配置 target。写入 `primary_category_id`，决定 `DimensionKind::Category` 的分类值。
  - 实时筛选侧：精确 target 匹配**或**节点名含「家宽」。保持「只看家宽」行为不变。
  - 两者差异必须写在模块文档注释、`docs/known-limits.md` 与家宽页界面上。合并任一方向都会改变行为或改变历史分类归属，因此不合并。
- 前端不复制任何家宽字符串匹配。
- 家宽占比的分母是可归因观测；分母未知或无覆盖时显示「未知」，不显示 0%。
- 未配置 targets 时给出中文下一步，不显示 0。

### R7. 双语与可访问性

- 每个新增用户可见字符串必须同时进 `zh.ts` 与 `en.ts`。键集合一致性由 `src/i18n/index.test.ts:7-8` 的既有断言保证，各子任务沿用，不新增重复测试。
- 保留 skip link、`:focus-visible`、`prefers-contrast: more`、`prefers-reduced-motion`；侧栏与主操作键盘可达；数据表保留 `<table>` 语义与 `aria-sort`。
- 默认 1200×800 与窄窗口下无横向溢出、无重叠。

## Task Map

| 子任务 | 目录 | 交付物 | 依赖 |
|---|---|---|---|
| 前端基座 | `08-21-neko-shell-foundation` | React + Tailwind + Recharts 工具链、设计令牌、侧栏 / 顶栏 / 十段路由壳、六项外观偏好接线、`list_routes` 扩展 | 无 |
| C3 维度查询与物化能力（Rust only） | `08-21-c3-dimension-capability` | 分钟粒度、规则/链路派生键、`ReportFilters` 注入全路径、五维物化 + category 排名、能力报告诚实、未知维度值可见 | 无 |
| 概览页与四个聚合页（前端） | `08-21-neko-overview-aggregation` | 成对口径卡、分钟级趋势图、Top 三列、四个聚合页骨架、`use-report`、`trend-area` / `rank-bar` / `stat-card` / `overview-card` / `top-list-item` | 基座；C3 能力（AC2/5/6/7） |
| 实时连接页移植（前端） | `08-21-neko-live-page` | 连接表、筛选工作区、列宽与显隐、排序、关闭连接、热点卡、`use-live-page` | 基座 |
| 家宽独立页与专用家宽读数 | `08-21-residential-page` | 判定收敛到一个模块（两个具名函数）、`share_residential_raw` + `residential_share`、家宽页三段 | 基座；概览聚合的图表与基元；C3 能力（AC5） |
| 报告 / 告警 / 设置页移植（前端） | `08-21-reports-alerts-settings-port` | 三页移植 + `share-donut` + `PRODUCT.md` / `DESIGN.md` / frontend spec 同步更新 | 基座 |

**顺序**：前端基座与 C3 能力子任务无前置依赖，可并行起步。基座落地后实时页与报告告警设置两页可并行推进。概览聚合依赖基座；其 AC2 / AC5 / AC6 / AC7 需等 C3 能力落地才能验收。家宽页依赖基座 + 概览聚合的基元；其 AC5 需等 C3 能力落地。

共享组件的新增一律落在 `components/ui/`、`components/common/`、`components/charts/`，由 `design.md` 第 1 节指定的建立方负责并登记 API，后续者复用而不重造。

## Out of scope

- 不引入 neko 的多后端切换、鉴权 / showcase 模式、PWA、Service Worker、版本检查与 GitHub star 拉取。
- 不做 Regions 世界地图，不引入 MaxMind MMDB 文件、不加远程地理查询、不改采集层去捕获 `destinationGeoIP`。若日后要做，是独立的采集能力任务。
- 不引入 `@xyflow/react` 链路 DAG；链路页用条形图与表格表达链路占比。
- 不引入 neko 的 ClickHouse 读写分层、agent 探针协议与 Surge 适配。
- 不改为 Next.js、不引入 SSR、不引入路由库；单窗口内部路由继续用应用自己的 `RouteId` 状态。
- 不引入 React Query、WebSocket 客户端；数据仍走 Tauri command 与既有 Monitor Channel。
- 不新增 neko 那套按维度独立建表的统计 schema。本项目的 `dimension_dict` + `traffic_*_dimension` 是通用分层，`dimension_kind` 列可承载任意维度；当前问题是只物化了 `'host'`，扩物化范围即可（依据见 `design.md` 第 8 节）。
- 不新增业务表、不改 schema 版本、不写迁移。
- 不改 v1 的平台范围（Windows 11 NSIS current-user）、不加应用内自动更新、不加遥测。
- 不复制 neko 的品牌名、logo、文案与 MIT 头部声明到本仓库产物。

## Acceptance Criteria

- [ ] AC1 (R1/R2)：十段路由在 `latte` 与三款深色下均可扫读；侧栏、顶栏、卡片、表格、图表使用同一套 Tailwind 令牌，无遗留 Catppuccin 硬编码色值。
- [ ] AC2 (R4)：概览页六格口径与 `LiveOverview` 字段一一对应，前五格成对；把 `meterUpload` / `gapUpload` / `overUpload` 置为 `null` 时界面显示「未知」，不显示 0。
- [ ] AC3 (R3)：概览 Top 列表与四个聚合页的排名数据来自 `run_report` 返回的 `rankings`；前端源码中不存在对连接数组做分组求和产出排名的代码路径。
- [ ] AC4 (R3)：`DimensionKind::Rule` 在多跳链路下按顶层策略组归并；`Chain` 与 `Category` 的排名不再返回主机排名。三者各有单测。
- [ ] AC5 (R3)：`ReportFilters` 的六个字段在 totals、series、rankings 三处均生效；以排名行标签回填 `filters.chain` / `filters.rule` 能取到子集。下钻不返回全局数据（实测确认）。
- [ ] AC6 (R3)：`DimensionKind::Process` / `Rule` / `Chain` / `Network` / `Category` 在超出 raw 期的区间返回非空排名；无物化的维度 `exact_top_n` 为 false 且有中文原因。
- [ ] AC7 (R3)：维度值缺失的流量在排名里以「未知」一行出现或在 `note_zh` 声明差额；「排名之和 + 未知 == 合计」或差额有说明。
- [ ] AC8 (R3)：趋势图三档向后端请求分钟档粒度；`Granularity` 的既有 `Hour | Day | Month` 序列化值不变（有断言）。
- [ ] AC9 (R5)：实时页、报告页、告警页、设置页的 R5 全部功能在新界面下可用；四页原有的自动化测试保持通过或按等价断言迁移，不删除覆盖点。批准删除的渲染实现见 `design.md` 第 2 节。
- [ ] AC10 (R6)：家宽页三段可用；未配置 targets 时显示中文下一步而非 0；两个判定函数集中在一个模块，且「只看家宽」的选中集合与改造前一致（有测试）。
- [ ] AC11 (R6/R4)：家宽占比在无覆盖时显示「未知」，在有覆盖且实测 0 时显示 0 并注明；分母在界面上标为「可归因观测」。
- [ ] AC12 (R7)：`zh.ts` 与 `en.ts` 键集合一致（沿用 `src/i18n/index.test.ts` 的既有断言）；构建产物中不含外部 URL、webfont 或 CDN 引用。
- [ ] AC13：`just monitor-check` 通过（typecheck + lint + vitest + build），`cargo fmt --check`、`cargo clippy -D warnings`、`cargo test --workspace` 通过。
- [ ] AC14：`PRODUCT.md`、`DESIGN.md`、`.trellis/spec/residential-monitor/frontend/index.md` 已按新栈与十段路由更新；`PRODUCT.md:38`「前端不重做核算或 Top N」保留未改（有 diff 确认）。
- [ ] AC15 (R3)：五维物化对 30 天库体积与物化耗时的影响有 `monitor-bench` 实测数字，不是估算；`retention_preview` 的行数口径变化已在 `docs/data-directory.md` 说明。
- [ ] AC16 (R7)：1200×800 与窄窗口实拍无横向溢出；skip link、`:focus-visible`、`prefers-contrast: more`、`prefers-reduced-motion`、`aria-sort` 与键盘可达性通过手工检查。

## Notes

- Windows 安装态验证、真实控制器长跑、30 天库容量与 C5 硬化门仍是手工证据，不由本任务的 CI 短样本冒充。
- 本任务在 `refactor/neko-ui-port` 分支上完成后**一次性合入 `main`**，不分批合入（理由见 `design.md` 第 11 节）。

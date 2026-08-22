# 前端基座：React + Tailwind 工具链与应用壳

## Goal

在 residential-monitor 里建立 React 19 + Tailwind v4 + Recharts 的前端基座，落地 neko 设计令牌与应用壳（侧栏 + 顶栏 + 十段路由），并把现有的语言、主题、字体、字号、密度、侧栏宽度六项偏好接到新壳上。本子任务不移植任何业务页内容：业务页在壳内渲染 `<PagePending />` 占位。不做「降级到 `main.ts` 旧渲染函数」（理由见父任务 `design.md` 第 11 节）。

## 父任务

`.trellis/tasks/08-21-neko-ui-refactor`。跨子任务的目录约定、主题映射、数据流约束与回滚形状见父任务 `design.md`，本文件不重复。

## Confirmed facts

- 当前渲染是整页 `innerHTML` 替换：`residential-monitor/src/main.ts:1416-1434`。为了不丢焦点与滚动，`:1369-1381` 手工保存 `activeElement.id`、`selectionStart/End/Direction` 与 `.live-table-wrap` 的 `scrollTop/scrollLeft`，`:1435-1457` 再逐个写回。这套手工状态保存在 React 下由 DOM diff 自然解决，是本次换栈的直接收益。
- 应用壳结构：`aside.shell`（brand + nav + `#shell-resize` 分隔条）+ `main.workspace#workspace`（`#view` + 错误行）。`#shell-resize` 带完整 ARIA slider 语义（`role="separator"`、`aria-valuemin/max/now/valuetext`、`aria-keyshortcuts="ArrowLeft ArrowRight Home End"`），必须原样保留。
- 侧栏宽度状态在 `src/shell-width.ts`，clamp 常量 `SHELL_WIDTH_MIN` / `SHELL_WIDTH_MAX` / `SHELL_WIDTH_DEFAULT`，持久化命令 `save_ui_sidebar_width`。
- 导航项由 `navHtml`（`src/main.ts:1241-1250`）生成，图标来自 `src/nav-icons.ts` 的 `ROUTE_ICONS`，产品标记是 `BRAND_MARK`。`src/ipc/routes.test.ts:29-31` 断言每个 route 都有本地图标且 `BRAND_MARK` 非空。
- recovery-only 分支不渲染业务页导航：`src/main.ts:1423-1427`，改为 `shell.recovery` 文案。
- 主题四值与 `applyTheme` 在 `src/theme.ts:1,77-80`；`data-theme` + `color-scheme`，`latte` 为 light。字体/字号/密度分别写 `data-font` / `data-fontSize` / `data-density`（`:82-93`）。
- `index.html` 现有 skip link `<a class="skip" href="#workspace">` 与 `<div id="app" tabindex="-1">`，样式表以 `<link rel="stylesheet" href="/src/styles.css">` 引入。
- 现有依赖只有 `@tauri-apps/api`（`residential-monitor/package.json:18-20`）；`check` 脚本是 `typecheck && lint && test && build`。
- neko 侧栏参照 `ref/neko-master/apps/web/components/layout/navigation.tsx:109-281`，顶栏参照 `.../app/[locale]/dashboard/components/header/index.tsx:158-596`，令牌参照 `.../app/globals.css:71-140`。

## Requirements

### R1. 工具链

- 新增依赖：`react`、`react-dom`、`@types/react`、`@types/react-dom`、`@vitejs/plugin-react`、`tailwindcss`、`@tailwindcss/postcss`、`clsx`、`tailwind-merge`、`class-variance-authority`、`lucide-react`、`recharts`，以及本子任务实际用到的 Radix 基元（至少 `@radix-ui/react-tooltip`、`@radix-ui/react-dropdown-menu`）。
- `vite.config.ts` 加 React 插件；`tsconfig.json` 打开 `jsx: "react-jsx"`；ESLint 配置加 React 与 Hooks 规则并保持 `npm run lint` 零告警。
- 不引入 Next.js、next-intl、next-themes、React Query、framer-motion 之外的动画库。数字过渡若需要动画，本子任务先用 CSS transition，framer-motion 由概览页子任务在确有需要时再评估。
- 不引入任何 webfont、远程图标、CDN 或外部 URL。

### R2. 设计令牌

- `src/styles/globals.css` 作为 Tailwind v4 入口，按父任务 `design.md` 第 3 节以 `:root[data-theme="..."]` 四段声明 neko 令牌全集。
- 保留 `data-font` / `data-fontSize` / `data-density` 三个维度对排版与间距的影响；密度 `compact` 至少作用于卡片内边距、表格行高与导航项高度。
- `prefers-contrast: more` 与 `prefers-reduced-motion` 的覆盖块必须存在。
- 保留 skip link 与 `:focus-visible` 可见焦点环。

### R3. 应用壳

- 侧栏：产品标记 + 产品名 + 连接状态圆点（复用 `healthOf` 的既有状态文案）+ 九段导航 + 底部「关于」「设置 / 数据管理」。导航项为 neko 的 `rounded-xl` 选中整块底色形态，图标 + 文字。
- `#shell-resize` 分隔条保留：ARIA 属性、键盘 `ArrowLeft/ArrowRight/Home/End`、指针拖动、clamp 与松手一次性持久化。
- 顶栏：连接状态、自动刷新开关、时间范围选择器占位、语言切换、主题切换。时间范围选择器的取值模型在本子任务只需给出组件与状态，不接报告查询。
- 主区独立滚动；不在主区重复当前页标题。
- recovery-only 分支只渲染 recovery 内容，不渲染九段导航。

### R4. 路由与降级

- `RouteId` 扩展为十值：侧栏导航九段 `overview | live | residential | host | rule | chain | process | reports | alerts`，加底部 `settings-data`。Rust `list_routes` 与 `src/ipc/routes.test.ts` 同步更新。
- 本子任务只交付壳。九个业务页与设置页在壳内渲染 `<PagePending />`（「本页移植中」）。**不降级到 `src/main.ts` 的旧渲染函数**：旧渲染依赖 `main.ts` 内约 20 个模块级可变状态，暴露给 React 壳会造成两套状态源同时可写。代价是中间态不可日常使用，因此整个重构一次性合入 `main`。
- `index.html` 的 script 指向 `/src/main.tsx`；`src/main.ts` 暂不删除，作为回滚点。

### R5. 偏好接线

- 语言、主题、字体、字号、密度、侧栏宽度六项在新壳内均可改、可持久化、重启后生效；调用的仍是现有六个 command。
- 字体选择器的系统字体枚举（`list_ui_fonts`）继续可用。

### R6. 双语与检查

- 新增字符串同时进 `zh.ts` 与 `en.ts`。
- 键集合一致性由 `src/i18n/index.test.ts:7-8` 的既有断言保证。**不新增重复测试**，跑既有测试即可。

## Out of scope

- 不移植任何业务页内容（概览、实时、家宽、四个聚合页、报告、告警、设置）。
- 不删除 `src/main.ts` 与 `src/styles.css`。
- 不改 Rust，除 `list_routes` 的路由列表扩展外。
- 不改任何数据 DTO、查询或采集逻辑。
- 不做 neko 的移动端底部导航（本产品是固定窗口桌面应用）。

## Acceptance Criteria

- [ ] AC1 (R1)：`npm --prefix residential-monitor run build` 产出 React 应用；`index.html` 只引用 `/src/main.tsx`；产物中无外部 URL、webfont 或 CDN 引用（有检查命令或测试断言）。
- [ ] AC2 (R2/R3)：九段导航 + 底部两项在四款主题（`latte` / `frappe` / `macchiato` / `mocha`）与中英文下均可扫读，选中态为整块主色圆角条，无 Catppuccin 硬编码色值残留在新增文件中。
- [ ] AC3 (R3)：`#shell-resize` 的 `role`、`aria-valuemin/max/now/valuetext`、`aria-keyshortcuts` 与键盘 `ArrowLeft/ArrowRight/Home/End` 行为与改造前一致；clamp 与松手一次性持久化有测试覆盖。
- [ ] AC4 (R5)：六项偏好均可改并在重启后生效；`list_ui_fonts` 的系统字体枚举可用。
- [ ] AC5 (R3)：recovery-only 分支不渲染业务页导航。
- [ ] AC6 (R6)：`src/i18n/index.test.ts` 的既有键集合断言通过；源码中无新增的重复键集合测试。
- [ ] AC7 (R4)：`npm --prefix residential-monitor run typecheck`、`lint`、`test`、`build` 全部通过；`cargo fmt --check`、`cargo clippy -D warnings`、`cargo test --workspace` 通过（路由扩展涉及 Rust）。
- [ ] AC8 (R2/R3)：1200×800 与窄窗口实拍无横向溢出；skip link 与 `:focus-visible` 可用；`prefers-reduced-motion` 下 ping 与进度条动画停止。

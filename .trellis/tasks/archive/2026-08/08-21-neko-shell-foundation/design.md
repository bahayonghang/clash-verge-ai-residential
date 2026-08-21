# 设计：前端基座

## 状态所有权

`app.tsx` 持有全部跨页状态，通过 props 下发；不引入状态库。

```ts
type ShellState = {
  boot: BootstrapDto | null; // get_bootstrap 一次性
  route: RouteId; // 本地视图选择，不持久化
  stream: MonitorState; // src/ipc/reducer.ts 归约结果
  prefs: {
    // 六项外观偏好，来自 boot，改动即持久化
    locale: UiLocale;
    theme: UiTheme;
    font: UiFont;
    fontSize: UiFontSize;
    density: UiDensity;
    sidebarWidth: number;
  };
  autoRefresh: boolean; // 顶栏开关，本地
  timeRange: TimeRange; // 顶栏选择，本地；本子任务不消费
};
```

`prefs` 的六个字段各自对应一个既有 command。写入路径统一为「乐观更新本地 → 调用 command → 失败时回滚并显示中文下一步」，不做静默失败。

## Hooks

- `useBootstrap()`：`get_bootstrap` + `decodeBootstrap`，返回 `{ boot, error }`。
- `useMonitorStream(boot)`：`subscribe_monitor` + Channel 监听，内部调用现有 `src/ipc/reducer.ts` 与 `src/ipc/live-session.ts`，不重写归约。卸载时取消订阅。
- `usePreferences(boot)`：六项偏好的读写，副作用是调用 `theme.ts` 的 `applyTheme` / `applyFont` / `applyFontSize` / `applyDensity` 与 `shell-width.ts` 的 `applyShellWidth`。这些函数写 `documentElement` 的 data 属性，React 不接管它们。
- `useSidebarResize(width, onCommit)`：指针与键盘两条路径，clamp 走 `shell-width.ts` 的既有函数，`onCommit` 在 pointerup / keyup 各调一次。

## 业务页降级策略

选择**占位**而非降级到旧渲染函数。理由：旧渲染函数依赖 `main.ts` 内的模块级可变状态（`liveFilterDraft`、`reportInspectPinned`、`liveTableLayout` 等约 20 个），把它们暴露给 React 壳会造成两套状态源同时可写，比占位的过渡期成本更高。

代价：基座合入后到各页移植完成前，应用不可用于日常使用。缓解：基座与四个页面子任务在同一分支 `refactor/neko-ui-port` 上推进，只在全部页面移植完成后才合入 `main`。

占位组件 `<PagePending route={...} />` 显示页名与中文「本页移植中」，并保留侧栏与顶栏可用，便于逐页验收。

## 令牌分层

```css
/* globals.css 结构 */
@import "tailwindcss";

@theme inline {
  /* 令牌 → Tailwind 工具类映射，照抄 neko globals.css:20-69 的结构 */
}

:root[data-theme="latte"] {
  /* neko light: globals.css:71-105 */
}
:root[data-theme="frappe"] {
  /* neko dark + background/card/sidebar 提亮 6% */
}
:root[data-theme="macchiato"] {
  /* neko dark + 提亮 3% */
}
:root[data-theme="mocha"] {
  /* neko dark 原值: globals.css:107-140 */
}

:root[data-font-size="sm"|"md"|"lg"] {
  --ui-scale: ...;
}
:root[data-density="compact"] {
  --pad-card: ...;
  --row-h: ...;
}

@media (prefers-contrast: more) {
  /* 提高 --border / --muted-foreground 对比 */
}
@media (prefers-reduced-motion: reduce) {
  /* 关闭 ping / shimmer / 进度条 / transition */
}
```

`--primary` 与 `--chart-1..5` 在三档深色下取同一组值，避免出现三套图表配色。

`body` 的三层 `radial-gradient`（neko `globals.css:149-165`）只在深色三档启用；`latte` 用纯 `--background`，避免浅底上的紫蓝晕影压低对比度。

## 基元清单（本子任务建立）

`components/ui/`：`card`、`button`、`badge`、`separator`、`tooltip`、`dropdown-menu`、`skeleton`。
`components/common/`：`theme-toggle`、`language-switcher`、`time-range-picker`、`status-dot`。
`components/layout/`：`sidebar`、`header`、`shell`、`page-pending`。

`cn()` 放 `src/lib/utils.ts`（`clsx` + `tailwind-merge`），与 neko 一致。

按父任务规则，后续子任务新增基元落在同目录并在各自 `design.md` 登记。

### 已定稿 API（后续子任务复用，不重造）

`src/lib/utils.ts`

- `cn(...inputs: ClassValue[]): string` — `clsx` + `tailwind-merge`
- `formatTemplate(template, vars)` — 替换 `{name}`
- `invokeErrorZh(error, fallback)` — 读取 command 的 `messageZh`

`src/lib/time-range.ts`

- `TimeRangePreset = "5m" | "30m" | "1h" | "24h" | "7d" | "30d" | "today"`
- `TimeRange = { preset, startUtc, endUtc }` — UTC 整数毫秒
- `timeRangeFromPreset(preset, now?)` / `defaultTimeRange(now?)` — 默认 `24h`

`src/lib/health.ts`

- `healthOf(locale, session)` → `{ title, action }`
- `healthTone(session)` → `"ok" | "warn" | "bad"`

`components/ui/`

- `Button` — `variant`: default | destructive | outline | secondary | ghost | link；`size`: default | sm | lg | icon；`asChild?`
- `Badge` — `variant`: default | secondary | destructive | outline
- `Card` / `CardHeader` / `CardTitle` / `CardDescription` / `CardContent` / `CardFooter`
- `Separator` — Radix，`orientation` horizontal | vertical
- `Skeleton` — `div` 脉冲占位
- `Tooltip` / `TooltipTrigger` / `TooltipContent` / `TooltipProvider`
- `DropdownMenu` / `DropdownMenuTrigger` / `DropdownMenuContent` / `DropdownMenuItem` / `DropdownMenuLabel` / `DropdownMenuSeparator`

`components/common/`

- `StatusDot({ session, label, ping? })`
- `ThemeToggle({ locale, theme, onThemeChange })` — 四值 `latte|frappe|macchiato|mocha`
- `LanguageSwitcher({ locale, onLocaleChange })`
- `TimeRangePicker({ locale, value, onChange, className? })` — `onChange(preset)`，本子任务不接报告查询
- `AppearanceMenu({ locale, font, fontSize, density, fonts, fontsError, onFontChange, onFontSizeChange, onDensityChange })`

`components/layout/`

- `Shell` — 侧栏 + 顶栏槽 + `#workspace` 主区；`errorZh` 用 `role="alert"`
- `Sidebar` — brand、状态圆点、九段导航、底部关于/设置、`#shell-resize`
- `ShellResizeHandle` — ARIA slider 语义
- `Header` — 状态、自动刷新、时间范围、语言、主题、字体与密度
- `PagePending({ locale, route })` — 「本页移植中」
- `RecoveryPane({ locale, boot })`

`hooks/`

- `useBootstrap()` → `{ boot, error }`，边界 `decodeBootstrap`
- `useMonitorStream(boot, locale)` → `MonitorState`，复用 `reducer` / `live-session`
- `usePreferences(boot)` → 六项偏好读写 + `list_ui_fonts`；失败回滚并给出中文下一步
- `useSidebarResize(width, onCommit)` → `{ displayWidth, ...handlers }`；clamp 走 `shell-width.ts`，pointerup / keyup 各持久化一次

`RouteId` 十值（顺序稳定）：

`overview | live | residential | host | rule | chain | process | reports | alerts | settings-data`

## 路由扩展

`RouteId` 从五值扩到十值。Rust 侧 `list_routes` 需要给出新增五页（residential / host / rule / chain / process）的 `RouteDescriptor`（`titleZh` 由前端 `localizeRoutes` 覆盖，Rust 只需 `id` / `available` / `unavailableUntil`）。

`src/ipc/routes.test.ts` 的断言从「五段」改为「十段且顺序稳定」，并继续断言每个 route 有本地图标。新增五个图标由本子任务在 `src/assets/icons/` 生成，风格对齐现有图标，不引入 lucide 的远程资源（lucide-react 是打包进产物的 React 组件，可用；侧栏图标是否改用 lucide 由实施时决定，若改用则 `ROUTE_ICONS` 与其测试同步调整）。

## 兼容

- 不改 `MonitorStreamMessage`、不改任何 DTO、不改 59 个 command 的签名。
- `src/format/**` 与 `src/ipc/**` 只被调用，不被修改。
- `src/main.ts` 与 `src/styles.css` 保留在仓库中直到父任务收口。

## 回滚

把 `index.html` 的 `<script type="module" src="/src/main.tsx">` 改回 `/src/main.ts`，并恢复 `<link rel="stylesheet" href="/src/styles.css">`。Rust 的路由扩展是纯增量，可单独保留。

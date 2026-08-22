# 实施：前端基座

1. [x] 装依赖并接工具链：`vite.config.ts` 加 `@vitejs/plugin-react`，`postcss.config.mjs` 加 `@tailwindcss/postcss`，`tsconfig.json` 开 `jsx: "react-jsx"`，ESLint 加 React / Hooks 规则。跑 `npm run typecheck && npm run lint` 确认零告警。
2. [x] 写 `src/styles/globals.css`：Tailwind 入口 + `@theme inline` 映射 + 四段 `data-theme` 令牌 + 字号/密度 + `prefers-contrast` / `prefers-reduced-motion` + skip link + `:focus-visible`。
3. [x] 建基元：`src/lib/utils.ts` 的 `cn()`，`components/ui/{card,button,badge,separator,tooltip,dropdown-menu,skeleton}`。
4. [x] 扩路由：Rust `list_routes` 增加 residential / host / rule / chain / process 五页描述符；`src/dto.ts` 的 `RouteId` 扩到十值；`src/i18n/{zh,en}.ts` 加 `route.*` 文案；新增侧栏图标；`src/ipc/routes.test.ts` 改为十段断言。
5. [x] 写 hooks：`useBootstrap`、`useMonitorStream`、`usePreferences`、`useSidebarResize`。`useMonitorStream` 直接复用 `src/ipc/reducer.ts` 与 `live-session.ts`，不重写归约。
6. [x] 写壳：`components/layout/{shell,sidebar,header,page-pending}` + `src/app.tsx` + `src/main.tsx`。侧栏含 brand、状态圆点、九段导航、底部两项、`#shell-resize`；顶栏含状态、自动刷新、时间范围、语言、主题。
7. [x] 切换入口：`index.html` 的 script 指向 `/src/main.tsx`，移除 `styles.css` 的 link。`src/main.ts` 与 `src/styles.css` 保留。
8. [x] 补测试：`#shell-resize` 的 clamp / 键盘 / 一次性持久化；`zh.ts` 与 `en.ts` 键集合一致性；构建产物无外部 URL。
9. [ ] 跑全量检查并实拍：`npm --prefix residential-monitor run typecheck && lint && test && build`；`cargo fmt --check`、`cargo clippy -D warnings`、`cargo test --workspace`；四款主题 × 中英文 × 1200×800 / 窄窗口。
   - 已跑：typecheck / lint / test / build 通过；`cargo fmt --check` 通过；`cargo test --workspace` 通过（247 + 3，1 ignored）。
   - 未完成：`cargo clippy -D warnings` 在既有 `credential.rs` 的 `manual_slice_fill` 失败（非本子任务文件，未改）；无 GUI，未做四主题 × 中英文实拍。

## 回滚点

第 7 步之前，应用仍走旧渲染路径，可随时中止。第 7 步之后的回滚是把 `index.html` 的 script 与 stylesheet 改回 `/src/main.ts` 与 `/src/styles.css`；第 4 步的 Rust 路由扩展是纯增量，可单独保留。

## 交接给后续子任务

- `components/ui/` 与 `components/common/` 的 API 定稿后写入本 `design.md`，后续子任务复用不重造。
- `<PagePending />` 的九个占位由各页面子任务逐个替换。
- 时间范围选择器只交付组件与状态，接入报告查询由概览页子任务完成。

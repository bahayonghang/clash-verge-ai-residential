# 实施：neko 界面移植总体排期

## 阶段顺序

1. [x] **两条并行起步**（无前置依赖）：
   - [x] **前端基座**（`08-21-neko-shell-foundation`）— 工具链、令牌、侧栏、顶栏、十段路由壳。
   - [x] **C3 维度查询与物化能力**（`08-21-c3-dimension-capability`）— Rust only。分钟粒度、派生键、filters 注入、五维物化、能力报告诚实。
2. [x] 基座落地后，以下三项可并行：
   - [x] **实时连接页**（`08-21-neko-live-page`）— 纯前端，不依赖 C3 子任务。
   - [x] **报告 / 告警 / 设置页**（`08-21-reports-alerts-settings-port`）— 纯前端，含 `PRODUCT.md` / `DESIGN.md` / frontend spec 更新。
   - [x] **概览页与四个聚合页**（`08-21-neko-overview-aggregation`）— 纯前端。界面可先做；AC2 / AC5 / AC6 / AC7 需等 C3 子任务落地才能验收。
3. [x] 概览聚合的基元落地后：
   - [x] **家宽独立页**（`08-21-residential-page`）— 判定收敛 + 家宽读数 + 三段页。AC5 需等 C3 子任务落地。
4. [ ] **父任务收口**：删除 `src/main.ts` 与 `src/styles.css`、移除全部 `<PagePending />`、跨页一致性复查、实拍证据、一次性合入 `main`。
   - 已完成：删除旧入口、移除 PagePending、源码搜索、CHANGELOG/docs、验证命令。
   - 未完成：十段路由实拍与能力降级窗口检查（本轮无 Tauri 窗口）；`monitor-bench` 未测；未合入 `main`（本轮不 commit）。

## 共享组件的建立方

按 `design.md` 第 1 节的目录约定，各基元与图表封装有指定建立方：

| 目录 | 内容 | 建立方 |
|---|---|---|
| `components/ui/` | card button badge separator tooltip dropdown-menu skeleton | 基座 |
| `components/ui/` | input select switch popover table tabs scroll-area | 实时页（先落地者） |
| `components/common/` | theme-toggle language-switcher time-range-picker status-dot | 基座 |
| `components/common/` | stat-card overview-card top-list-item | 概览聚合 |
| `components/charts/` | trend-area rank-bar | 概览聚合 |
| `components/charts/` | share-donut | 报告告警设置 |

建立方必须在自己的 `design.md` 里写出 API。后续子任务复用，不得在 `features/**` 内部复制同名组件。冲突时以建立方的 API 为准。

## 每页至少一个 hook

`components/**` 不得直接 `invoke`（`design.md` 第 5 节）。各页的 hook 归属见该节表格。每个 hook 必须实现请求序号递增与过期响应丢弃、失败保留上次结果并单独暴露 `errorZh`。

## 父任务收口检查项

- [x] `index.html` 只引用 `/src/main.tsx`；`src/main.ts` 与 `src/styles.css` 已删除。
- [x] 全仓搜索无 Catppuccin 色值残留（`#1e1e2e`、`#cdd6f4` 等）与无 `styles.css` 引用。
- [x] `app.tsx` 内无 `<PagePending />` 残留。
- [x] `components/**` 内无直接 `invoke` 调用（源码级搜索确认）。
- [ ] 十段路由逐页在四款主题 × 中英文下扫读，1200×800 与窄窗口实拍。本轮无 Tauri 窗口，未实拍。
- [ ] 概览六格口径的 `null` → 「未知」实测；关闭控制器后各页空态互不混淆。本轮无 Tauri 窗口，未实测。
- [ ] 把查询区间拉到超出 raw 期与 daily core 层，逐页确认能力降级文案正确、无空表伪装成「无流量」。本轮无 Tauri 窗口，未实测。
- [ ] 家宽页两种口径的说明可见；「只看家宽」选中集合与改造前一致。代码与单测已由子任务交付；GUI 可见性本轮无窗口，未确认。
- [x] `zh.ts` / `en.ts` 键集合一致性测试通过（`src/i18n/index.test.ts` 既有断言）。
- [x] 构建产物扫描：无外部 URL、无 webfont、无 CDN。
- [x] `just monitor-check` 通过；`cargo fmt --check`、`cargo clippy -D warnings`、`cargo test --workspace` 通过。
- [ ] 五维物化的 `monitor-bench` 实测数字已填入 `08-21-c3-dimension-capability/design.md` 第 5 节。blocked：本轮未测，design.md 第 5 节保持 not measured。
- [x] `PRODUCT.md` 第 38 行未改（`git diff PRODUCT.md` 确认）。
- [x] CHANGELOG.md 增加 English 条目；子项目 docs 中文同步（至少 `data-directory.md`、`known-limits.md`、`first-run.md`、`reporting.md`、`alerts.md`）。

## 验证命令

```bash
npm --prefix residential-monitor run typecheck
npm --prefix residential-monitor run lint
npm --prefix residential-monitor test
npm --prefix residential-monitor run build
just monitor-check
cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check
cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace
```

本轮上述命令均已通过。

## 中间态说明

基座合入后到四个页面子任务完成前，应用不可用于日常使用（业务页是 `<PagePending />` 占位）。**因此整个重构在 `refactor/neko-ui-port` 上完成后一次性合入 `main`，不分批合入。** 理由与被拒的替代方案见 `design.md` 第 11 节。

父任务收口已删除 `src/main.ts`、`src/styles.css` 与 `<PagePending />`。未合入 `main`。

## 回滚点

见 `design.md` 第 11 节的回滚形状表。

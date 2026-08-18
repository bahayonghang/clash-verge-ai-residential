# 家宽监控应用壳侧栏与界面重构 — 设计

## Architecture

工作只发生在 `residential-monitor` 前端壳。Rust Command、Channel、DTO 和核算保持不变。

```text
index.html
  #app
    aside.shell-nav          产品标记 + 口号 + 五条 route
    main#view                当前页内容，无重复页面标题
    [role=alert]             既有 errorZh
```

- `renderApp` 拥有壳层 DOM。`navHtml` 改为侧栏项：每项是 `button.nav-item`，含本地图标、`titleZh`、`data-route`、`aria-current` / `data-disabled`。
- 页面选择仍只写在前端 `route: RouteId`。图标表是前端常量 `Record<RouteId, string>`，指向 Vite 打包的本地资源，不进入 bootstrap。
- 各 `render*` 只返回主区。删除与当前 route 重复的页面级 `h1`/`h2`。区块 `h2` 保留。
- Recovery-only：同一 `aside` 只渲染产品身份和「Recovery Shell」状态，不渲染可点击的五页按钮。

## Data flow and contracts

```text
get_bootstrap.routes[] ── titleZh / available ──► 侧栏文字与禁用态
RouteId 前端图标表     ── 本地 URL            ──► <img> 或 CSS image
subscribe_monitor      ── 既有 reducer        ──► 概览 / 连接表（字段不变）
```

- 解码、reducer、Command 名称和 payload 不改。
- secret 仍不回显。缺口与未知仍走现有 `unknownOr` / 独立 coverage 文案。
- 动态重绘后仍按元素 `id` 恢复焦点。

## Visual world

规划不锁定色板、字体或材质。`task.py start` 之后走 Impeccable new-work（Operate 模式、替换视觉世界）。方向合同必须满足：

- 侧栏在第一视口左侧，产品名与五条标题同时可见。
- 专家工具密度：表格是表格，指标是紧凑网格，表单是紧凑字段。
- 无远程字体、无 CDN、无 UI 框架。
- 图标按选定世界生成，深色或浅色底上都要可辨认，并带高对比回退。

`.impeccable/config.json` 尚未记录 `buildPath`。实施进入方向选择时再问一次：先出效果图再写代码（comp-first），或直接写代码（code-first）。未问之前不把默认值写入配置。

## Assets

- 位置：`residential-monitor/src/assets/`，由 Vite 打进 `dist`，满足 `img-src 'self'`。
- 需要 6 张图：产品标记 + 五条 route。可用 `data:` 仅作生成中间态，提交物是仓库内文件。
- 不改 `src-tauri` 窗口图标和 NSIS 图标，除非生成过程顺便产出且不扩大验收。

## Compatibility

- 窗口默认 1200×800 不改。侧栏固定宽度，主区滚动。
- `@media print` 隐藏侧栏与按钮，与现网隐藏 `.nav` 同职责。
- `prefers-contrast: more` 覆盖新 token，不只覆盖旧色。
- CSP、无 `window.__TAURI__`、无远程 URL 保持。

## Trade-offs

- 图标不放进 DTO：换图标不必改 Rust。副作用是预览态 `previewBootstrap` 与正式 bootstrap 共用前端图标表。
- 全页换视觉、不换信息架构：用户认页靠侧栏，认数靠原字段。风险是新世界把密度做松；验收以 AC7 卡住。
- 不做窄窗汉堡菜单：产品是桌面工作台。窗口过窄时主区横向滚动，不把标题收进图标。

## Rollback

回滚 `residential-monitor/src/main.ts`、`styles.css`、新增 `src/assets/` 与相关测试即可。不回退 migration，不改数据库，不改 Credential Manager。

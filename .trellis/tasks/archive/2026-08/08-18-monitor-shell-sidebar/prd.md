# 家宽监控应用壳侧栏与界面重构

## Goal

把 `residential-monitor` 桌面壳改成左侧栏承载产品名、页面标题和本地图标，并用新的视觉世界重画五个页面。用户在 1200×800 窗口里先认页、再读数、再操作。

用户价值：导航不再和主内容抢垂直空间。视觉替换后仍是同一套观测工具。

## Confirmed Facts

- 产品是 Windows 11 Tauri 桌面应用「家宽流量监控」。默认窗口 1200×800，可缩放。平台设计语言是 WebView 里的 HTML/CSS。
- 当前壳在 `residential-monitor/src/main.ts` `renderApp`：顶部 `header.top` 写死产品名和口号「观测下界，不是账单。secret 不会出现在此页面。」；`nav.nav` 横向渲染 `boot.routes[].titleZh`。
- 固定 route：`overview`、`live`、`reports`、`alerts`、`settings-data`。标题只来自 bootstrap。
- 各页主内容再写一遍页面级 `h2`，与导航标题重复。区块标题（重点分类、规则、数据管理）是内容结构，不是页面标题。
- 现有 `styles.css` 深色 token 只作现状证据和反例，不作为新视觉权威。
- 前端是 Vanilla TypeScript + Vite。禁止 UI 框架、`window.__TAURI__`、eval、远程 URL 和 CDN。CSP `img-src 'self' data:`。
- 前端只保存导航、筛选、分页和 DTO 缓存。缺口、未知、未归因差额不得画成零。
- `boot.branch === "recovery-only"` 只进入 Recovery Shell。
- 已决：全页视觉重做。已决：主区保持当前专家工具密度。
- 本任务保持 `planning`，直到用户批准本规划摘要后才能 `task.py start`。

## Requirements

### R1 左侧栏承载页面标题

- 应用壳为左侧栏 + 主内容。产品名「家宽流量监控」和五条 `boot.routes[].titleZh` 只出现在左侧栏。
- 当前 route 有 `aria-current="page"` 与可见选中态。不可用 route 保持 `data-disabled`，不伪造数据。
- 主区去掉与当前 route 标题重复的页面级 `h1`/`h2`。区块标题保留。
- 口号放在侧栏产品名下方，不占顶部整行标题带。
- Recovery-only：侧栏只保留产品身份和恢复状态，不提供普通五页假入口。打印时隐藏侧栏与按钮。

### R2 生成本地导航图标

- 五条 route 各有一枚本地图标；产品名旁有一枚产品标记。图标由本机 image 生成，写入 `residential-monitor` 本地资源。
- 图标在 CSP `img-src 'self' data:` 下工作。新侧栏底色、选中、禁用、`prefers-contrast: more` 下可辨认。
- 侧栏按钮同时有图标和 `titleZh`。图标不替代文字。

### R3 全页视觉重做，保持专家工具密度

- 替换色、材质、字体和控件语言。五个业务页与 Recovery Shell 都按新视觉世界重画。
- 重做对象：应用壳、指标块、表格、表单、按钮、状态色、空状态与告警提示。不改业务字段、按钮动作和事实性中文文案。
- 默认 1200×800 下，指标、连接表、报告表、告警表和设置表单仍尽量一屏可扫。表格保持表格，表单保持紧凑标签字段，不拆成大留白营销卡片。
- 侧栏固定宽度并显示图标 + 文字。主区独立滚动。不引入汉堡菜单。
- 不引入 UI 框架、远程字体或 CDN。`:focus-visible`、`.skip`、`prefers-contrast: more` 仍可用。动态重绘后按元素 `id` 恢复焦点。
- 视觉世界在实施阶段由 Impeccable new-work 选定，规划不写死色板或字体。

### R4 行为与契约不变

- 不改 route id、bootstrap 路由表形状、Command / Channel DTO、核算口径、关闭连接语义。
- 不改事实性文案：观测下界不是账单、secret 不出现在页面、缺口/未知不写成零、关于页不得把未签名标成 `signed`、删除部分失败不得显示「已全部删除」。
- 前端仍只保存视图选择和 DTO 缓存。图标映射只存在前端，不进入 DTO。

## Acceptance Criteria

- [ ] **AC1** 默认窗口下，产品名和五条页面标题只出现在左侧栏；顶部不再有横向 `.nav` 标题带。
- [ ] **AC2** 点击侧栏标题切换 `overview` / `live` / `reports` / `alerts` / `settings-data`，当前项有 `aria-current="page"`；不可用项不可进入且不显示伪数据。
- [ ] **AC3** 主区不再重复当前页面标题。区块标题仍在。
- [ ] **AC4** 五条导航各有一枚仓库内本地图标，与文字标题同时可见；构建产物不请求远程图片。
- [ ] **AC5** Recovery-only 仍只显示恢复界面，侧栏不提供普通五页假入口。
- [ ] **AC6** 五个业务页与 Recovery Shell 均使用新视觉世界，不再沿用旧 `--bg` / `--panel` / `--accent` token。
- [ ] **AC7** 默认 1200×800 下，概览指标与当前页主表或主表单首屏可见，不因大留白卡片把主任务推到折页下。
- [ ] **AC8** `npm --prefix residential-monitor run typecheck`、`lint`、`test`、`build` 通过。关于页与删除结果的既有断言不被这次改版绕过。
- [ ] **AC9** 键盘可到达全部侧栏项；`:focus-visible` 与 skip link 仍可用；`prefers-contrast: more` 下标题与选中态可辨认。

## Out of Scope

- 不改 Rust 采集、核算、报告、告警、备份、恢复业务逻辑。
- 不新增 route，不拆分「设置 / 数据管理」。
- 不改窗口默认尺寸、托盘、单实例、登录自启动。
- 不引入 React / Vue 或其他 UI 框架。
- 不发布新版本、不改签名或 NSIS 安装流程。
- PRODUCT.md / DESIGN.md 是 Impeccable 记录，不是本任务验收门。

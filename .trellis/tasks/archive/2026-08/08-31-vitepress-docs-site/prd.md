# VitePress 重构仓库文档

## Goal

用 VitePress 把根 `docs/` 做成可本地预览的中英双语文档站。默认中文：GitHub 上现有 `docs/*.md` 写成中文，站点 `/` 打开中文。英文在 `docs/en/`，站点 `/en/`。覆盖 6 篇用户文档和 4 篇 Agent 文档；两套语言事实一致，导航可切换。既有 10 篇路径不改。`docs/adr/` 三篇不进站点、不改写、不翻译。

## Background and Confirmed Facts

- 根 `docs/` 现有 6 篇用户文档：`routing-scope.md`、`configuration.md`、`local-configuration.md`、`dns-and-leak-model.md`、`multi-profile.md`、`troubleshooting.md`。`README.md:161-168` 把它们当作文档入口。
- Agent 文档 4 篇：`docs/agents/{domain,issue-tracker,triage-labels,residential-rule-tuning}.md`，由 `CLAUDE.md` / `AGENTS.md` 引用。
- ADR 3 篇：`docs/adr/0001-process-lookup-vs-process-routing.md`、`0002-unknown-process-drilldown.md`、`0003-controller-only-process-identity.md`。`docs/agents/domain.md` 要求改代码前读这些文件。
- `residential-monitor/docs/` 另有 14 篇桌面文档，不在根 `docs/`。
- 仓库没有 VitePress、没有 GitHub Pages workflow。根 `package.json` 零依赖、无 lockfile；`.github/workflows/ci.yml` 直接跑 `npm run ci`，不先 `npm install`。
- Trellis spec 硬链仓库路径，例如 `.trellis/spec/guides/index.md:50` → `docs/routing-scope.md`，`.trellis/spec/frontend/index.md:48` → `docs/local-configuration.md`。
- `CLAUDE.md` 约定 `docs/` 用中文。现状不整齐：仅 `local-configuration.md` 是中文标题，其余用户文档是英文标题，正文中英混排。
- 开关真源是 `scripts/sync-local-config.js` 的 `SWITCH_CONFIG_FIELDS`（`routing` 21 项 + `runtime` 7 项）。
- 当前 VitePress 官方安装要求 Node.js 22+。扩展脚本与根 `engines` 仍是 Node 18+。
- VitePress 内置 i18n：默认语言页面在 `docs/*.md`，另一语言在 `docs/<locale>/`。根路径 `/` 是默认语言。本任务无 Pages，不做服务器语言重定向。

## Product Decisions

- D1（用户于 2026-08-31 确认）：站点覆盖根 `docs/` 全部——6 篇用户文档 + 4 篇 `docs/agents/`。不含 `residential-monitor/docs/`。
- D2（用户于 2026-08-31 确认，D5 修订）：进站文档要理顺信息架构并重写表述。不得借重写扩大或缩小家宽范围，不得改开关默认值或排障步骤的事实。
- D3（用户于 2026-08-31 确认）：只提供本地预览与构建。不新增 GitHub Pages workflow，不改仓库 Pages 设置，不配自定义域名。
- D4（用户于 2026-08-31 确认）：ADR 不进侧栏、不进站点路由。文件只留在 `docs/adr/`。Agent 仍按仓库路径阅读。
- D5（用户于 2026-08-31 确认）：站点中英双语。10 篇进站文档与首页均有中文页和英文页；导航可切换语言。ADR 仍不翻译、不进站。
- D6（用户于 2026-08-31 确认）：默认中文。`docs/*.md` 与 `docs/agents/*.md` 写中文；英文在 `docs/en/` 与 `docs/en/agents/`。站点 `/` 为中文，`/en/` 为英文。

## Constraints

- C1：`docs/adr/**` 的路径、文件名和正文冻结。禁止改写、合并、重命名、加 frontmatter、复制进 `docs/en/`。VitePress 不得扫描或输出这些文件。
- C2：不改扩展脚本运行时、`HOME_PROXY_TEMPLATE` 凭据占位、`*.local.toml` / `*.local.js`。
- C3：不把真实凭据、订阅 URL、未脱敏 Connections 写进文档站。
- C4：重写不得改变产品合同。纳入/排除类别、负向测试要求、官方证据句、开关表必须与扩展脚本和 `SWITCH_CONFIG_FIELDS` 一致；中英文两套事实相同。
- C5：10 篇既有 Markdown 的仓库相对路径不改名、不拆文件。它们是中文真源。英文是新增文件，不替换这些路径。
- C6：根 `package.json` 保持扩展脚本的零运行时依赖与 Node 18+ `npm run ci`（无需先 install）。文档站依赖不得进入 Clash Verge 粘贴脚本，也不得让根 CI 矩阵变成必须安装 VitePress。

## Requirements

- R1：用 VitePress 默认主题与内置 `locales` 提供中英双语文档站。中文 `lang` 为 `zh-CN`，英文为 `en-US`。每种语言有首页；侧栏分成「使用与配置」/「Usage」和「Agent」两组，能打开 D1 的 10 篇。页内链接留在同一语言。本地搜索可用。导航提供语言切换。
- R2：中文 10 篇写中文，英文 10 篇写英文。主机名、TOML 键、JS 常量、命令、产品名、保留节点名（`AI-家宽`、`家宽-SOCKS5`）两语都保持原样。两套正文的产品事实一致。ADR 不翻译。
- R3：中文首页为 `docs/index.md`，英文首页为 `docs/en/index.md`。说明扩展脚本做什么、文档站怎么本地打开、两组文档入口。不替代根 `README.md`。
- R4：提供本地预览与构建命令（`just` 配方，根 `package.json` 可做无依赖转发）。构建输出不进 git。文档命令标明需要 Node 22+。
- R5：根 `README.md` 文档节保留指向中文 `docs/*.md` 的 GitHub 文件链接，并补充本地预览命令与英文目录 `docs/en/`。`CLAUDE.md` 增加文档站命令，并写明 `docs/en/` 是英文。`CHANGELOG.md` 的 Unreleased 记录此事。
- R6：中英文的 `domain` 页都指向仓库路径 `docs/adr/`，不链成站点路由。
- R7：中文 `docs/local-configuration.md` 的截图 `../assets/clash-verge-rev-global-extend-script.png` 在两种语言的站点页里都能显示。英文页使用能解析到同一文件的相对路径。

## Acceptance Criteria

- [ ] AC1（R1、D1、D4、D5、D6）：本地预览 `/` 为中文、`/en/` 为英文，导航可切换。每种语言的侧栏能打开 6 篇用户文档、4 篇 Agent 文档和首页。侧栏与路由都没有 ADR。不含 ResiWatch 桌面文档。
- [ ] AC2（R2、C4、D5）：中文 10 篇为中文正文，英文 10 篇为英文正文；主机名、TOML 键、命令、产品名未被翻译。两套开关表都与 `SWITCH_CONFIG_FIELDS` 的 28 行一致；两套 routing-scope 的纳入/排除与扩展脚本当前行为一致。
- [ ] AC3（R4、C6、D3）：文档构建命令退出码 0；构建产物被 gitignore。仓库没有 GitHub Pages workflow。根 `npm run ci` 仍无需安装第三方包即可在 Node 18 上通过。`just ci` 通过。
- [ ] AC4（C1、R6）：对 `docs/adr/` 的 git diff 为空。不存在 `docs/en/adr/`。中英文 domain 页都写仓库路径 `docs/adr/`。
- [ ] AC5（C5、R5、R7）：10 篇既有文档的仓库路径未改，内容为中文。README 能在 GitHub 上点开这 6 篇中文用户文档，并写明本地预览与 `docs/en/`。两种语言的本地配置页都能显示既有截图。
- [ ] AC6（C2、C3）：本任务不改扩展脚本行为，文档与构建输出不含真实凭据、订阅 URL、未脱敏 Connections。

## Out of Scope

- 修改或翻译三篇 ADR，或把 ADR 复制进 `docs/en/`。
- 把 `residential-monitor/docs/` 迁入本站点。
- 部署 GitHub Pages、自定义域名、Algolia、自绘主题、Vue 组件定制、按浏览器语言自动跳转。
- 改 Clash 扩展脚本或 ResiWatch 桌面功能。
- 把根 `engines.node` 从 `>=18` 抬到 22。
- 把英文 `CHANGELOG.md` / CI / GitHub 模板改成中文。
- 把根 `README.md` 改成完整双语页（只在文档节标明英文目录即可）。
- 第三种语言。

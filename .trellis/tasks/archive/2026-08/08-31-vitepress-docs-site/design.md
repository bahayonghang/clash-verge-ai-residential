# Design：VitePress 重构仓库文档

## 1. 设计目标与边界

- 文档站只服务根 `docs/`：终端用户读「使用与配置 / Usage」，Agent 读「Agent」。
- 默认中文。`docs/*.md` 仍是 GitHub、README、Trellis 的中文真源。英文是平行副本，在 `docs/en/`。
- VitePress 是本地阅读层，不是第三份正文。
- 根仓库扩展脚本合同不变：零运行时依赖、Node 18+、`npm run ci` 无需 install。
- ADR 不是站点内容，也不进入英文树。

## 2. 工具落点

根 `package.json` 不添加 `vitepress`。文档工具放在 `docs/package.json`：

- `devDependencies.vitepress`：官方当前发行线（Node 22+），锁定 `docs/package-lock.json`。
- `"type": "module"` 只写在 `docs/package.json`。
- scripts：`dev` / `build` / `preview`。

根层只做转发：`docs:dev` / `docs:build` / `docs:preview` → `npm --prefix docs run …`；`justfile` 同样转发。

`just ci` 与 `npm run ci` **不**跑文档构建。

## 3. 站点与语言结构

```text
docs/
  package.json
  package-lock.json
  .vitepress/
    config.mjs
    cache/                     # gitignore
    dist/                      # gitignore
  index.md                     # 中文首页
  local-configuration.md       # 中文，原地重写
  configuration.md
  routing-scope.md
  multi-profile.md
  dns-and-leak-model.md
  troubleshooting.md
  agents/*.md                  # 中文，原地重写
  adr/*.md                     # 冻结，srcExclude
  en/
    index.md
    local-configuration.md
    configuration.md
    routing-scope.md
    multi-profile.md
    dns-and-leak-model.md
    troubleshooting.md
    agents/*.md
```

`docs/.vitepress/config.mjs`：

```js
locales: {
  root: { label: "简体中文", lang: "zh-CN" },
  en: { label: "English", lang: "en-US" }
}
```

- `srcExclude: ['adr/**']`
- 每种语言手写侧栏，不自动扫目录
- `themeConfig.search.provider: 'local'`
- 默认主题；不用 VitePress 容器语法（GitHub 会原样显示）
- `vite.server.fs.allow` 包含仓库根，以便 `../assets/` 与英文页 `../../assets/` 读到同一截图

中文侧栏：`/`、`/local-configuration`、`/configuration`、`/routing-scope`、`/multi-profile`、`/dns-and-leak-model`、`/troubleshooting`，以及 `/agents/*`。

英文侧栏同一组，前缀 `/en/`。

## 4. 正文规则

- 先定中文真源，再写英文，使事实从中文复制而不是两套各写各的。
- 开关表两语都对齐 `SWITCH_CONFIG_FIELDS` 28 行（`routing` 21 + `runtime` 7）。
- `routing-scope` 两语都对照扩展脚本当前纳入/排除，不增不删域名合同。
- 专有名不译。
- 页内相对链接只指向同语言文件（中文 `troubleshooting.md`，英文同名相对路径）。
- `domain` 页两语都写仓库文件 `docs/adr/…`，禁止 `/adr/` 或 `/en/adr/`。
- 首页只做导读与本地启动说明，不复制 README 完整路由边界。

## 5. 资源与忽略

- 截图仍在 `assets/clash-verge-rev-global-extend-script.png`。
- `.gitignore` 增加 `docs/.vitepress/cache/` 与 `docs/.vitepress/dist/`。

## 6. 兼容与回退

- 根 `engines.node` 保持 `>=18`。文档命令写明 Node 22+。
- `CLAUDE.md` 语言约定改为：`docs/` 默认中文，`docs/en/` 为英文站点源。
- 回退：删除 `docs/package.json`、`docs/package-lock.json`、`docs/.vitepress/`、`docs/index.md`、`docs/en/`，还原 10 篇中文 Markdown 与根脚本/justfile/README/CHANGELOG/CLAUDE.md。`docs/adr/` 不应出现在 diff 里。

## 7. 风险

- Node 22 文档工具与 Node 18 根 CI 并存：依赖隔离在 `docs/`。
- 双语漂移：英文必须跟中文真源，验收对照脚本与开关表，不对照旧英文混排稿。
- 无 Pages 时 `/` 不会按浏览器语言跳转。这是 D3 的后果，不在本任务补。

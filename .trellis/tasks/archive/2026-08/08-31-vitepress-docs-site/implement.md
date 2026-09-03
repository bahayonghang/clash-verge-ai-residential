# Implement：VitePress 重构仓库文档

## 前置

- 用户批准本规划摘要之后才能 `task.py start`。
- 不改 `docs/adr/**`，不创建 `docs/en/adr/`。
- 新依赖只出现在 `docs/package.json`。
- 跳过项目级 formatter / 与文档无关的全量测试；验证命令见文末。

## 顺序

1. **脚手架**
   - 新增 `docs/package.json`（`type: module`，仅 vitepress）和 `docs/package-lock.json`。
   - `docs/.vitepress/config.mjs`：`locales.root` 中文、`locales.en` 英文；手写两侧栏；`srcExclude: ['adr/**']`；本地搜索；`vite.server.fs.allow` 含仓库根。
   - `.gitignore`：`docs/.vitepress/cache/`、`docs/.vitepress/dist/`。
   - 根 `package.json` 只加无依赖转发；`justfile` 加 `docs-dev` / `docs-build` / `docs-preview`。
   - 不把文档构建塞进 `npm run ci` 或 `just ci`。

2. **中文首页与 10 篇原地重写**
   - `docs/index.md`
   - `local-configuration.md` → `configuration.md` → `routing-scope.md` → `multi-profile.md` → `dns-and-leak-model.md` → `troubleshooting.md`
   - `docs/agents/` 四篇
   - 中文标题与正文；相对链接留在中文树；专有名不译；开关表 28 行；routing-scope 对照扩展脚本。
   - 截图路径保持 `../assets/clash-verge-rev-global-extend-script.png`。
   - 不用 VitePress 容器语法。

3. **英文树**
   - 新增 `docs/en/index.md` 与 10 篇同名英文页（含 `docs/en/agents/`）。
   - 事实从已定稿的中文页翻译，不从旧英文混排稿恢复。
   - 相对链接留在英文树；截图用能到达同一 `assets/` 文件的相对路径。
   - `domain` 仍写仓库 `docs/adr/`。

4. **仓库入口**
   - `README.md`：保留 6 篇中文 GitHub 链接；补本地预览、Node 22+、`docs/en/`。
   - `CLAUDE.md`：文档站命令；`docs/` 中文、`docs/en/` 英文。
   - `CHANGELOG.md` Unreleased 英文条目。

5. **验收**
   - `npm --prefix docs run build` 退出码 0。
   - `git diff -- docs/adr` 为空；无 `docs/en/adr/`。
   - `just ci` 通过。
   - 本地预览：`/` 中文、`/en/` 英文、语言切换、各一组用户页与 Agent 页、无 ADR、截图可见。

## 风险文件

| 文件 | 风险 | 回退 |
|---|---|---|
| `docs/adr/**` | 误改即违反 C1 | 立刻还原 |
| `docs/en/adr/` | 误把 ADR 送进英文树 | 删除该目录 |
| `docs/routing-scope.md` 与 `docs/en/routing-scope.md` | 两语口径不一致或漂移 | 对照扩展脚本 |
| 开关表中英文 | 漏行 | 对照 `SWITCH_CONFIG_FIELDS` 28 行 |
| 根 `package.json` | 误加 vitepress 或改 `engines` | 只允许无依赖转发 |
| `.github/workflows/ci.yml` | 误加 Pages 或 docs-build | 本任务不改 |

## 验证命令

```text
git diff -- docs/adr
npm --prefix docs run build
just ci
```

预览：`just docs-dev`。不要把 `docs-build` 并进 `just ci`。

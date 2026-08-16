# 执行计划：Cursor/Grok 家宽路由 + TOML 键值自动补全

前置：阅读 `.trellis/spec/guides/index.md` 及 frontend 层相关规范；
遵循现有代码风格（中文注释、零依赖、严格校验）。

## 步骤

1. [ ] clash-verge-ai-residential.js — Cursor 清单
   - `CURSOR_SUFFIX_DOMAINS` 增加 `authenticate.cursor.sh`、`cursorvm.com`
   - `CURSOR_DOMAIN_REGEXES` 增加 `^adminportal[0-9]+\.cursor\.sh$`
   - 注释中记录未纳入的排除域（marketplace/CDN/downloads/anysphere）
2. [ ] clash-verge-ai-residential.js — Grok 清单与开关
   - 新增 `ROUTE_GROK_CORE = true` 常量（放在 ROUTE_CURSOR_CORE 附近）
   - 新增 `GROK_SUFFIX_DOMAINS = ["grok.com"]`（含证据注释）
   - `activeSuffixDomains()` / `allPossibleSuffixDomains()` 接线
   - `ROUTE_CURSOR_CORE` 默认值 `false` → `true`
   - module.exports constants 增加 `ROUTE_GROK_CORE`、`GROK_SUFFIX_DOMAINS`
   - `SCRIPT_VERSION` → 5.6.0，头部注释补 v5.6 要点
3. [ ] clash-verge-ai-residential.local.toml.example
   - `[routing]`：`cursor_core = true`；新增 `grok_core = true`（插在
     cursor_process_fallback 之前，与 SWITCH_CONFIG_FIELDS 顺序一致）
4. [ ] scripts/sync-local-config.js — 键值补全（design.md B1-B3）
   - 新增 `DEFAULT_EXAMPLE_PATH`、`formatSwitchLine()`、
     `completeLocalToml()`（文本级插入 + EOL 检测 + 幂等）
   - `syncLocalConfig()` 集成：example 解析 → 补全 → 原子写回 → 重新
     parse/validate → 渲染
   - `SWITCH_CONFIG_FIELDS` 新增 grok_core 映射
   - main() 输出补充的键列表
5. [ ] justfile — 更新 `render-local` 注释（说明自动补全；逻辑不变）
6. [ ] tests/regression.test.js
   - 正向：authenticate.cursor.sh / adminportal42.cursor.sh /
     vm.cursorvm.com / grok.com / cli-chat-proxy.grok.com 命中规则与
     RESIDENTIAL_DOH DNS policy（cursor_core/grok_core 默认开）
   - 负向：marketplace.cursorapi.com、cursor-cdn.com、
     downloads.cursor.com、anysphere-binaries.s3.us-east-1.amazonaws.com、
     api.mixpanel.com、x.ai 不产生 AI-家宽 规则
   - 开关关闭（注入 false 后）上述域名规则与 DNS policy 消失；
     再开启不残留重复
7. [ ] tests/sync-local-config.test.js（design.md B4 六个用例）
8. [ ] package.json version → 5.6.0；CHANGELOG.md 新增 5.6.0 条目
9. [ ] docs：routing-scope.md（Cursor 行更新默认开 + Grok 行新增 +
   排除清单补充）、local-configuration.md（补全行为）、README.md
   （开关表/特性说明）

## 验证命令

- `npm run ci`（check + test + check:secrets）
- 手动冒烟（可选，不提交产物）：
  `cp example /tmp local.toml` 删键 → `node scripts/sync-local-config.js`
  → 确认补全与渲染输出

## 回滚点

- 步骤 1-3 为纯清单/默认值变更，git revert 即可；重渲染自动清理。
- 步骤 4 若补全器有缺陷：移除 completeLocalToml 调用即可回到旧行为，
  已写入本地 TOML 的行合法且与模板默认一致，无需清理。

## 审查门

- 实现完成后走 trellis-check / `npm run ci`，通过后再进入 spec 更新
  与提交。

# Design：5 个 ChatGPT exact 主机

## 边界

恢复并填充 `OPENAI_CORE_EXACT_DOMAINS`，改版本号，修正 GPT DNS 关闭断言的键名，同步测试与文档。不改 DNS 构建算法、规则注入顺序、上游解析、`sync-local-config` 开关映射。无新 TOML 键。

## 匹配形态

不用 `DOMAIN-SUFFIX,chat.openai.com`。该规则会匹配官方未列出的任意子域。

恢复常量（v5.7 已删除）：

```js
const OPENAI_CORE_EXACT_DOMAINS = [
  "chat.openai.com",
  "android.chat.openai.com",
  "desktop.chat.openai.com",
  "ios.chat.openai.com",
  "tcr9i.chat.openai.com"
];
```

接线：

- `activeExactDomains()`：`...(ROUTE_OPENAI_CORE ? OPENAI_CORE_EXACT_DOMAINS : [])`
- `allPossibleExactDomains()`：展开 `OPENAI_CORE_EXACT_DOMAINS`（开关关闭时仍能清理已生成的 exact 规则）
- `allPossibleSuffixDomains()`：额外列入 `"chat.openai.com"`，注释写明仅清理误注入的 `DOMAIN-SUFFIX,chat.openai.com` 与 `+.chat.openai.com`。不要把它放进 `OPENAI_CORE_SUFFIX_DOMAINS`，否则会重新注入 suffix。
- `module.exports.constants` 增加 `OPENAI_CORE_EXACT_DOMAINS`

生成物：

- 规则：`DOMAIN,<host>,AI-家宽` × 5
- DNS：`policy[host] = RESIDENTIAL_DOH` × 5
- 不生成 `DOMAIN-SUFFIX,chat.openai.com`，不生成 `+.chat.openai.com`

`ROUTE_OPENAI_CORE === false` 时 `activeExactDomains` 不含这 5 个主机；`allPossible*` 仍含它们，因此旧规则与旧 DNS 键会被清掉。

## 为何仍纳入 `tcr9i.chat.openai.com`

官方 9247338 域名表有该主机，无用途说明。官方 Voice 写的是 UDP 3478，未点名该主机。按「只补官方明文」纳入 exact；不打开 Livekit 或 UDP 3478。若后续官方或脱敏 Connections 证明它只承载 Voice 媒体，再单独立项删除。

## 版本

`SCRIPT_VERSION` 与 `package.json`：`5.7.0` → `5.8.0`。文件头补 v5.8 一句：5 个 `chat.openai.com` 家族 exact 主机。

## 测试

`tests/regression.test.js`：

- 版本 `5.8.0`；constants 解构增加 `OPENAI_CORE_EXACT_DOMAINS`，断言数组等于上述 5 项。
- 正向 `assertAiRoute` 覆盖 5 个主机。
- 负向保持 `oaistatic.com`、`oaistatsig.com`、`auth.openai.com`、`www.openai.com`。
- 生成规则不含 `DOMAIN-SUFFIX,chat.openai.com` 与 `DOMAIN-SUFFIX,openai.com`。
- 「AI DNS policy…」追加：5 个裸键等于 `RESIDENTIAL_DOH`；`+.chatgpt.com` / `+.api.openai.com` 仍为家宽；`+.chat.openai.com` 不存在。
- 托管清理输入同时放入 `DOMAIN,chat.openai.com,AI-家宽` 与 `DOMAIN-SUFFIX,chat.openai.com,AI-家宽`。断言 exact 清理后重注一次，suffix 被清理且不重注。
- 现有幂等测试已比较两次 `nameserver-policy`；补 5 个 exact 规则各一次。

`tests/sync-local-config.test.js`：

- 修正现有伪测试：`openai_core = false` 时断言 `!("+.chatgpt.com" in policy)` 等 suffix 真键，而不是裸 `chatgpt.com`。
- 同期断言 5 个 exact 裸键不存在；`ruleMatchesHost` 对 5 个主机为 false。

## 文档

- `docs/routing-scope.md`：ChatGPT 行写 5 个 exact；证据句引用 9247338；写明 `tcr9i` 用途不明；写明原生应用 Connections 为 `UNVERIFIED`。
- README 版本号与 ChatGPT 摘要。
- CHANGELOG `[5.8.0]`。

## 取舍

| 候选 | 决定 | 原因 |
|---|---|---|
| `DOMAIN-SUFFIX,chat.openai.com` | 不加 | 宽于官方 5 主机 |
| `oaistatic.com` | 不加 | 静态 CDN |
| `chatgpt.livekit.cloud` / UDP 3478 | 不加 | Voice 基础设施 |
| `sora.com`、`chat.com`、`ai.com` | 不加 | 其它产品或短链 |
| `antigravity-pa.googleapis.com` | 不加 | 仅社区线索 |
| 新开关 | 不加 | `openai_core` 已足够 |
| 把 CI 写成已验证原生应用 | 不做 | Node 不能模拟宿主 |

## 回滚

单一提交。Revert 即去掉 5 个 exact、清理用 suffix 键、5.8.0 元数据与测试修正。

## 提交白名单

允许：`clash-verge-ai-residential.js`、`tests/regression.test.js`、`tests/sync-local-config.test.js`、`package.json`、`docs/routing-scope.md`、`README.md`、`CHANGELOG.md`、本任务目录。禁止：`.gitignore`、`.trellis/.template-hashes.json`、`*.local.toml`、`*.local.js`。

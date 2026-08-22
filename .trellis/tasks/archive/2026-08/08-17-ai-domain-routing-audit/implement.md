# Implement：5.8.0 ChatGPT exact 主机

前置：实现前跑一次 `npm run ci` 确认基线。不要编辑或提交 `*.local.toml` / `*.local.js`。不要 `task.py start`，除非用户已批准本修订规划。

## 代码

- [x] 1. `clash-verge-ai-residential.js`
  - [x] 1.1 恢复 `OPENAI_CORE_EXACT_DOMAINS`，仅 5 个官方主机，注释引用 9247338
  - [x] 1.2 `activeExactDomains` / `allPossibleExactDomains` 接入该常量；后者不受开关影响
  - [x] 1.3 `allPossibleSuffixDomains` 额外列入 `"chat.openai.com"`（仅清理，不进 `OPENAI_CORE_SUFFIX_DOMAINS`）
  - [x] 1.4 `module.exports.constants` 导出 `OPENAI_CORE_EXACT_DOMAINS`
  - [x] 1.5 `SCRIPT_VERSION = "5.8.0"`；文件头补 v5.8 一句
- [x] 2. `tests/regression.test.js`
  - [x] 2.1 版本 `5.8.0`；断言 exact 数组
  - [x] 2.2 正向 5 主机；负向保持；生成输出无 `DOMAIN-SUFFIX,chat.openai.com` / `openai.com`
  - [x] 2.3 DNS：5 个裸键 = `RESIDENTIAL_DOH`；`+.chatgpt.com` / `+.api.openai.com` 仍在；无 `+.chat.openai.com`
  - [x] 2.4 托管清理：exact 重注一次，suffix 清理不重注；两次 `main` 后规则与 policy 幂等
- [x] 3. `tests/sync-local-config.test.js`
  - [x] 3.1 修正 `openai_core = false` 的 DNS 断言：测 `+.chatgpt.com` / `+.api.openai.com` / `+.oaiusercontent.com`，不要测裸 suffix 名
  - [x] 3.2 同期测 5 个 exact 主机规则与裸 DNS 键均不存在
- [x] 4. `package.json` version `5.8.0`

## 文档

- [x] 5. `docs/routing-scope.md`：5 个 exact、9247338、`tcr9i` 用途不明、原生应用 `UNVERIFIED`
- [x] 6. `README.md`：版本号与 ChatGPT 摘要
- [x] 7. `CHANGELOG.md`：`[5.8.0]`

## 验证

- [x] 8. `npm run ci`
- [x] 9. 需要本地凭据时再跑 `node scripts/sync-local-config.js`；不提交生成物
- [ ] 10. 真实 ChatGPT 桌面/iOS Connections：若本机可做，脱敏后写入任务笔记并去掉 `UNVERIFIED`。做不到则保持 `UNVERIFIED`，不把 CI 绿写成客户端已验证

## 回滚点

一个提交；revert 即回滚。

## 审查门

- 生成输出只有 5 个 exact，没有 `chat.openai.com` suffix
- DNS 关闭测试使用 `+.` / 裸键真键
- 官方摘录仍在 `research/openai-9247338-allowlist-excerpt.md`
- 提交不含 `.gitignore`、`.trellis/.template-hashes.json`、本地凭据文件

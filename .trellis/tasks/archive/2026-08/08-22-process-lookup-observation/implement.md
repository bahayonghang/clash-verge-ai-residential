# 实施：顶层查找进程

## 顺序

1. 改 `clash-verge-ai-residential.js` 的 `ensureProcessLookup`。
2. 改 `tests/regression.test.js` 断言与嵌套键用例。
3. 更新 `docs/configuration.md`、`docs/local-configuration.md`、`CHANGELOG.md`。
4. `npm test`、`just ci`、`npm run check:secrets`。

## 风险

- 生成脚本会覆盖用户有意设为 `off` 的顶层键。这是本任务的产品决定。
- 不要改 `clash-verge-ai-residential.local.js`；由 `just render-local` 再生。

## 回滚

恢复 `ensureProcessLookup` 在 fallback 关闭时 return，以及「输入 off 则输出 off」的回归。

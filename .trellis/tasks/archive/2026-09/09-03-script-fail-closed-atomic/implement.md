# 实现：脚本失败原子性

1. 抽出 `cloneConfigForEdit(config)`。
2. `main` 使用 working 副本；成功返回 working。
3. 统一 `fail(message)`：`console.error` + `throw`。
4. `tests/regression.test.js` 增加：占位凭据 throw 后原 config 组未加 exclude-filter。
5. 验证：`npm test`。

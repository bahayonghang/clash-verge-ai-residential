# 设计：顶层查找进程

## 机制

`ensureProcessLookup(config)`：

1. 不再在 `ENABLE_AI_PROCESS_FALLBACK` 为 false 时直接 return。
2. `config["find-process-mode"] = "always"`。
3. 不删除 `profile.find-process-mode`（Verge 可能继续写嵌套键）；顶层键决定内核行为。

`main` 已有的 `ensureProcessLookup(config)` 调用保持一处。进程规则数组仍只在 fallback 开关为 true 时生成。

## 测试

- 替换 `tests/regression.test.js` 中「AI-only 模式不强制进程匹配」对 `find-process-mode === "off"` 的断言。
- 增加嵌套 `profile.find-process-mode` fixture，断言顶层 always。
- 现有 `ENABLE_AI_PROCESS_FALLBACK` 为 true 的用例保持：有进程规则，查找进程不为 `off`。

## 文档

`docs/configuration.md` 与 `docs/local-configuration.md` 在 `routing.ai_process_fallback` 旁写：脚本会把顶层查找进程设为 always；这不等于进程路由。

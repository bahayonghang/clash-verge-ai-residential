# 设计：脚本失败原子性

## 边界

只改 `clash-verge-ai-residential.js` 的 `main` 与测试。不改 TOML 渲染协议。

## 数据流

`main` 开头：若 `config` 非对象，改为 throw（与 fail-closed 一致，可在本任务做；若担心 Verge 传入空配置，保持早退并在测试注明）。推荐 throw。

浅拷贝：

```
working = {
  ...config,
  proxies: [...(config.proxies || [])],
  "proxy-groups": (config["proxy-groups"] || []).map(g => ({ ...g, proxies: Array.isArray(g.proxies) ? [...g.proxies] : g.proxies })),
  rules: [...(config.rules || [])],
  dns: { ...(config.dns || {}) },
}
```

harden / upsert 只打 working。成功 `return working` 或把字段写回（若 Verge 要求同引用：仅在全部成功后 `Object.assign` 回 host）。优先返回 working，回归断言返回值。

throw 前 `console.error(message)`。

## 回滚

失败路径不写回 host。测试保存进入前 `JSON.stringify` 关键字段。

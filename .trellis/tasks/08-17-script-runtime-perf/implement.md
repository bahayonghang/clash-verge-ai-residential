# Implement：v5.8.1 生成期索引

前置：实现前跑 `npm run ci` 记录基线。用户批准**本修订**规划摘要之前不要 `task.py start`，不要改产品代码。

## 代码

- [x] 1. `clash-verge-ai-residential.js`
  - [x] 1.1 `buildOutboundIndex(config)`：`groups` / `proxies` 两个 `Map`，值为 `{ count, value }`；键为真值 `item.name` 原值，不丢空串
  - [x] 1.2 `findOutbound(outboundIndex, name)`：校验两个 `Map`，否则抛「需要 outbound 索引」；只用 `Map.get`；歧义条件与文案不变；不读 `config`、不建表
  - [x] 1.3 `resolveCandidate` / `resolveFromCandidates` / `resolveUpstreamName` 必填 `outboundIndex` 并下传；缺则抛错
  - [x] 1.4 `hardenReachableUpstreamGraph`：必填索引；`Set` + 计数 + 最多 8 样本；结束后一条汇总 `warn`；开关为 false 时不查叶子
  - [x] 1.5 `validateTopLevelUpstream` 必填同一索引
  - [x] 1.6 `main`：保留名校验之后建一次索引并下传
  - [x] 1.7 `module.exports` 增加 `buildOutboundIndex`、`findOutbound`
  - [x] 1.8 `SCRIPT_VERSION = "5.8.1"`；文件头 v5.8.1 摘要
- [x] 2. `tests/regression.test.js`
  - [x] 2.1 版本断言 `5.8.1`；解构新导出
  - [x] 2.2 缺索引 / 非法索引抛错；合法索引可解析唯一节点
  - [x] 2.3 普通名：双节点、双组、组与节点同名、归一化后歧义，四条拒绝
  - [x] 2.4 单叶子 `udp: false` 汇总（名 + 路径）
  - [x] 2.5 同名节点挂两个组：总数 1、首次路径
  - [x] 2.6 9 个不同名 `udp: false`：前 8 个在、第 9 个不在、含总数
  - [x] 2.7 2000 叶子 / 1000 个不同名 `udp: false`：成功、UDP warn 1 条、样本名 ≤ 8
  - [x] 2.8 大订阅连续两次 `main`：规则、policy、家宽 `dialer-proxy` 一致
- [x] 3. `package.json` `"version": "5.8.1"`

## 文档

- [x] 4. `CHANGELOG.md` 增加 `[5.8.1]` Changed
- [x] 5. `README.md` 当前版本行改为 `v5.8.1`
- [x] 6. `docs/configuration.md`、`docs/local-configuration.md`：`runtime.warn_on_reachable_udp_disabled` 改为汇总一条、最多 8 个样本

## 验证

- [x] 7. `just ci`（或 `npm run ci`）
- [x] 8. 不提交 `*.local.toml` / `*.local.js`

## 回滚点

单提交可整体 revert。风险文件：`clash-verge-ai-residential.js`、`tests/regression.test.js`。

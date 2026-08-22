# Design：生成期 outbound 索引与 UDP 警告汇总（v5.8.1）

## 边界

只改配置生成期的查找与日志。`main` 在校验保留名之后、解析上游之前建一次索引，并把它传给 `resolveUpstreamName`、`hardenReachableUpstreamGraph`、`validateTopLevelUpstream`。

不改规则字符串、DNS 对象形状、嗅探、TUN、进程开关、上游解析优先级。不在模块顶层缓存索引。

线性复杂度的验收门是「`findOutbound` 不能在缺索引时工作」，不是耗时阈值。2000 节点的现网平方级实现仍能在约 25 ms 内成功跑完，耗时门会假绿。

## 索引

新增 `buildOutboundIndex(config)`，一次扫描两组数组：

```js
{
  groups: Map,   // name -> { count, value }  value 为第一次出现的组对象
  proxies: Map   // name -> { count, value }
}
```

收录规则与 `namedItems`（`:572`）对齐：只跳过假值 `item`；键为 `item.name` 原值（含 `""`、数字等）；查找用 `Map.get(name)`，即 `===`。不要跳过空字符串，也不要把键 `String()` 化。重复名只增加 `count`，建索引时不抛错。`validateReservedNameCollisions` / `buildGroupMap` 仍负责保留名与「配置里任意两组同名」的既有拒绝。

`findOutbound(outboundIndex, name)`：

1. 若 `outboundIndex` 不是对象，或 `groups` / `proxies` 不是 `Map`，抛错：`[AI-家宽] findOutbound 需要 outbound 索引`。
2. 禁止读取 `config`，禁止调用 `namedItems` / `buildOutboundIndex`。
3. 歧义条件与现网相同：`groupCount > 1` 或 `proxyCount > 1` 或两者均为 1。错误文案保持「outbound 名称…存在歧义」。
4. 返回 `{ kind, value }` 或 `null`，`value` 为第一次出现的对象。

`resolveCandidate` / `resolveFromCandidates` / `resolveUpstreamName` / `hardenReachableUpstreamGraph` / `validateTopLevelUpstream` 的索引参数为**必填**。缺索引时在函数入口抛与上同类的错误，不要自行建表。`resolveUpstreamName` 虽被导出，测试与 `main` 都必须传入索引。

`main`：

```js
const outboundIndex = buildOutboundIndex(config);
const upstreamName = resolveUpstreamName(config, profileName, outboundIndex);
hardenReachableUpstreamGraph(config, upstreamName, outboundIndex);
validateTopLevelUpstream(config, upstreamName, outboundIndex);
```

`allOutboundNames` 仍用于 `formatAvailableOutbounds` 与归一化候选扫描。不必为归一化再建第二张表。

`upsertNamedItem` 之后不重建索引：其后没有按名全表热路径。

`module.exports` 增加 `buildOutboundIndex`、`findOutbound`，供缺索引与歧义单测直接调用。

## UDP 警告汇总

`WARN_ON_REACHABLE_UDP_DISABLED === false` 时不调用 `findOutbound` 查叶子、不分配路径数组。

开关为 true 时，图遍历内使用：

```js
const udpDisabledNames = new Set();
const udpDisabledSamples = []; // { name, path }，长度 ≤ 8
let udpDisabledCount = 0;
```

叶子为代理且 `udp === false` 时：

1. `udpDisabledNames.has(name)` 为真则跳过。
2. 否则 `add`、`count += 1`；仅当 `samples.length < 8` 时 `push({ name, path: [...stack, name] })`。

禁止 `array.find` / `some` 按名去重。不要保存第 9 个及以后的路径字符串。

遍历结束后若 `udpDisabledCount > 0`，调用一次 `warn`：

- 前缀：`[AI-家宽] N 个可达节点显式关闭 UDP：`
- 样本：`“name”（路径：a -> b -> name）`，顿号分隔，最多 8 条
- `N > 8` 时追加 `……（共 N 个）`
- 后缀：上游组选中这些节点时 WebRTC/STUN 可能失败或改走其他路径

顶层 / 可达组 `disable-udp: true` 仍 `throw`。`removeInjectedReferencesFromGroup` 的每组一次 warn 不变。

## 版本与文档

- `SCRIPT_VERSION` / `package.json` / `README.md` 当前版本行 → `5.8.1`
- 文件头补一行 v5.8.1：大订阅 outbound 索引；UDP 叶子警告改为汇总
- `docs/configuration.md`、`docs/local-configuration.md` 中 `runtime.warn_on_reachable_udp_disabled` 改为汇总一条警告（最多 8 个样本）
- `CHANGELOG.md` 增加 5.8.1 Changed

## 测试

`tests/regression.test.js` 新增。不设耗时断言。

1. `findOutbound()` / `findOutbound({}, "x")` / `findOutbound({ groups: new Map() }, "x")` 抛「需要 outbound 索引」。`buildOutboundIndex` 后 `findOutbound(index, "HK")` 返回 `kind: "proxy"`。
2. 两个代理都叫 `HK`，组 `🚀节点选择` 含 `HK`：`main` 抛歧义。
3. 两个组都叫 `🚀节点选择`：`main` 抛歧义（在 `buildGroupMap` 之前由 `findOutbound` 拒绝）。
4. 一个组与一个节点都叫 `HK`，且不存在唯一的 `🚀节点选择` 组、候选会查到 `HK`（例如仅有名为 `HK` 的组与节点，Profile 走通用候选或 MATCH 指向 `HK`）：`main` 抛歧义。
5. 精确名 `🚀节点选择` 不存在；名为 `🚀 节点选择` 的组与节点各一条：归一化命中该唯一字符串后 `findOutbound` 仍抛歧义。
6. 单节点 `udp: false`：恰好一条汇总，含该名与路径。
7. 同一 `udp: false` 节点出现在两个可达子组：总数 1；路径是先 DFS 到的那条。
8. 9 个不同名的 `udp: false` 叶子：正文含前 8 个名，不含第 9 个，含 `9`。
9. 2000 叶子、1000 个不同名 `udp: false`：成功；UDP warn 1 条；样本名 ≤ 8。
10. 同一 config 连续两次 `main`：规则、policy、家宽 `dialer-proxy` 一致。

夹具 4 的构造必须让 `validateReservedNameCollisions` 先通过，冲突名不能是 `AI-家宽` / `家宽-SOCKS5`。

## 回滚

还原查找辅助函数、`main` 传参与导出即可回到 v5.8.0。歧义检测与 fail-closed 校验保持原抛错条件。

## 风险

| 风险 | 处理 |
|---|---|
| 索引在 `upsert` 家宽节点前建立 | 与现网一致：热路径都在 upsert 之前 |
| 两个同名组在未被选中时仍由 `buildGroupMap` 拒绝 | 既有行为；AC5 的「两个同名组」夹具把该名当作被选上游，走 `findOutbound` |
| boa 5 秒未实测 | 标 `UNVERIFIED` |

# FlClash 0.9.1 脚本报错与“代理”入口消失：根因调查

调查日期：2026-10-19（America/New_York）。本轮只研究，未修复、未提交。

后续进展：用户现已批准实施，修复和本机应用结果另见 [validation.md](validation.md)；本文保留初次调查时的证据状态。

## 结论

最符合现有证据的根因是：FlClash 将缺省的 `rule-providers` 映射经 Go JSON 序列化为 `null`，本仓库脚本却把 `null` 当成非法已有配置并抛错；FlClash 的失败处理继续应用空运行配置，正常代理分组消失，在规则模式下仅剩的隐藏内置 GLOBAL 也不会显示，因而侧栏“代理”入口被隐藏。

这是“宿主空值表达与脚本校验不兼容”加上“宿主失败时应用空配置”的连锁结果。不能将此现象解释为脚本删除了订阅，也不能从报错直接推断家宽节点不可用。

证据等级：本地类型错误已确定性复现；官方 0.9.1 输入、失败与导航机制已追踪；用户这次实际传入的字段值、运行模式及原生端到端流程没有现场采集，因此个案的 `null` 值和规则模式仍属于高可信推断。

## 用户现象与观察边界

- 第一张截图：选择“家宽”扩展脚本后报 `rule-providers 必须是对象，拒绝覆盖已有配置`，栈含 `validateRegexDnsProviderOwnership`；侧栏无“代理”入口。
- 第二张截图：扩展模式为“脚本”，选中的脚本名为“家宽”。这证明脚本被选中，不证明它与当前仓库文件完全相同。
- 后补截图：没有该脚本时侧栏有“代理”；“配置”入口在前后截图均存在。用户称“配置那个 tab”消失，图像差异实际指“代理”。
- 用户确认客户端版本 `0.9.1`。截图未展示运行模式、实际配置对象或内核日志。
- 前后截图的已选订阅不同，不能作为同一个订阅严格控制变量的 A/B 运行实验；但不影响源码说明错误链如何产生该现象。

## 官方来源与版本固定

只采用项目官方源码和 Go 官方文档。搜索结果中的第三方“Clash Official”类站点没有用作依据。

- [FlClash v0.9.1 发布页](https://github.com/chen08209/FlClash/releases/tag/v0.9.1)
- GitHub API 查得标签指向提交 `8d4f9744948751f1273fe871e1d05276fac09b0e`。
- 该提交 `core/Clash.Meta` 子模块指向 `8380d0e7462f061a964323e9bf3129a236fcc5d5`；`.gitmodules` 指向官方维护者的 `chen08209/Clash.Meta`。
- 以下源码行号来自按上述 SHA 下载的原始文件，以 LF 划分；网页搜索缓存的行号/排版可能不同。未将 main 分支当作 0.9.1 的实现证据。

## 从输入到错误

1. [内核配置读取](https://github.com/chen08209/FlClash/blob/8d4f9744948751f1273fe871e1d05276fac09b0e/core/config.go#L65)：`handleGetConfig` 读取订阅文件并返回 Mihomo `RawConfig`；`core/short_id.go:12-25` 负责反序列化。
2. [RawConfig 定义和默认值](https://github.com/chen08209/Clash.Meta/blob/8380d0e7462f061a964323e9bf3129a236fcc5d5/config/config.go#L451)：`:452` 的 `RuleProvider` 是 Go map，JSON tag 没有 `omitempty`；`:483-605` 的默认构造没有初始化此 map；`:607-623` 在默认对象上解码 YAML。缺省字段可因此留下 nil map。
3. [Go JSON 文档](https://pkg.go.dev/encoding/json#Marshal)：nil map 编码为 JSON `null`。`core/method.go:85-94` 用 `json.Marshal` 序列化 RPC 响应。
4. [Dart 配置读取](https://github.com/chen08209/FlClash/blob/8d4f9744948751f1273fe871e1d05276fac09b0e/lib/core/controller.dart#L290)：`:294-299` 仅将 `rule` 改名为 `rules`，没有删除 null 规则集字段。
5. [脚本适配层](https://github.com/chen08209/FlClash/blob/8d4f9744948751f1273fe871e1d05276fac09b0e/lib/common/javascript.dart#L13)：`:17-19` 仅对 `proxy-providers` 做 null → 空映射兼容，未对 `rule-providers` 做同样处理；`:22-27` 将配置 JSON 传给脚本求值器并向外传播错误。
6. [脚本调用](https://github.com/chen08209/FlClash/blob/8d4f9744948751f1273fe871e1d05276fac09b0e/plugins/rust_api/rust/src/script/mod.rs#L37)：`:37-41` 将 JSON 解析后传给 `main(config)`，没有第二个 profileName 参数。
7. 本地 `clash-verge-ai-residential.js:1645-1648` 的判断允许 undefined、拒绝 null；`:1939-1943` 克隆后无条件调用。错误发生在上游解析和代理注入之前。

关键区别：YAML 未声明规则集不等于 JS 收到字段缺省。经过有类型结构体和 JSON 桥接，脚本可能收到显式 null。这是本次兼容边界漏掉的语义。

## 为什么“代理”入口也消失

- [异常包装](https://github.com/chen08209/FlClash/blob/8d4f9744948751f1273fe871e1d05276fac09b0e/lib/state.dart#L104)：`:104-135` 的 `safeRun` 显示错误并返回 null。
- [配置应用](https://github.com/chen08209/FlClash/blob/8d4f9744948751f1273fe871e1d05276fac09b0e/lib/providers/actions/setup.dart#L695)：`:695-712` 的生成结果为 null 时，运行 YAML 取空字符串；`:731-743` 仍写入运行配置并调用内核 setup。`:380-384` 的注释明确说明失败时仍应用空配置，避免继续运行旧配置；这不是保留上一份可用配置的回滚。
- [内核处理](https://github.com/chen08209/FlClash/blob/8d4f9744948751f1273fe871e1d05276fac09b0e/core/config.go#L124)：`:124-154` 应用解析结果；如果解析报错则改用默认配置。空输入的正常默认路径和解析失败的回退路径都不会恢复机场节点。不能把本次脚本错误误说成一定发生了第二次内核解析错误。
- [刷新代理组](https://github.com/chen08209/FlClash/blob/8d4f9744948751f1273fe871e1d05276fac09b0e/lib/providers/actions/setup.dart#L408)：`:408-409` 在应用后刷新代理组。`lib/providers/actions/proxies.dart:114-145` 从运行内核读取，界面反映的是已应用配置，不是订阅原文件。
- [内置 GLOBAL](https://github.com/chen08209/FlClash/blob/8d4f9744948751f1273fe871e1d05276fac09b0e/core/proxy.go#L118)：`:118-138` 将内置 GLOBAL 标记 hidden。`lib/providers/state/proxies.dart:4-14` 在规则模式过滤 hidden 分组；全局模式不过滤，直接模式始终没有可显示组。因此“代理入口消失”有模式条件，不能写成任何模式都会消失。
- [导航状态](https://github.com/chen08209/FlClash/blob/8d4f9744948751f1273fe871e1d05276fac09b0e/lib/providers/state/navigation.dart#L4)：初始化后 `hasProxies` 来自当前可显示组是否非空。
- [导航项目](https://github.com/chen08209/FlClash/blob/8d4f9744948751f1273fe871e1d05276fac09b0e/lib/views/navigation.dart#L23)：`:23-30` 让“代理”在 hasProxies 为 false 时没有显示模式；“配置”不受此条件约束。

所以应区分：订阅仍存在、运行配置失效、代理组被过滤、导航入口被隐藏。截图符合这条链，没有显示订阅文件被删除。

## 可复现验证

从仓库根目录运行：

```powershell
node .trellis/tasks/10-20-flclash-script-provider-failure/research/repro.cjs
go run .trellis/tasks/10-20-flclash-script-provider-failure/research/nil-map-json.go
```

| 检查 | 结果 | 能证明什么 |
| --- | --- | --- |
| 脱敏 Node 复现 | 16/16 PASS | 缺省/undefined/空映射通过；null、数组、字符串、数字、布尔值报同文错误；开关全关仍报；输入不变 |
| 既有正则住宅 DNS 测试 | 12/12 PASS | 当前拒绝 null 的行为被测试明确锁定；并非测试漏跑或偶发 |
| Go 标准库映射编码探针 | PASS | nil map → `{"rule-providers":null}`；空 map → `{"rule-providers":{}}` |
| FlClash 0.9.1 / 真实订阅运行 | NOT_RUN | 未现场读取输入、未运行或改变用户客户端 |
| 网络连通、住宅 SOCKS5、Mihomo 真实应用 | NOT_RUN | 本错误在脚本类型校验处已中止，无法从截图推断这些项目 |

Node 复现不是 QuickJS 运行验收；Go 探针复制了有关字段形状，仅验证标准编码语义，没有编译完整 FlClash/Mihomo。细节见 [本地研究](local-provider-analysis.md)。本次没有产品修改，不运行 monitor/docs/full CI，也不将源码追踪标成原生运行 PASS。

## 恢复建议

1. 在出错订阅的“扩展”页切回“标准”模式，暂时停止使用“家宽”脚本。
2. 返回配置页重新选中该订阅，使 FlClash 重新生成并应用原订阅配置；必要时在取消脚本后停止再启动代理。
3. 检查“代理”入口、原分组和基础联网是否恢复。保留订阅和脚本，避免清数据、重装或删除订阅。

这些是基于失败机制的恢复建议，尚未在用户客户端执行或验证。它们会暂时停止住宅分流。只重启但保留有问题的脚本，仍会重新触发错误；关闭 Vertex/Cursor 开关也绕不过无条件校验。

## 后续修复建议与边界

- 仓库最小修复候选：仅将 `rule-providers: null` 视作“没有规则集”，在工作副本上规范化；保留数组/字符串/数字/布尔值拒绝，保留托管名称所有权校验、用户 provider 保留及输入不变。
- 同步调整 `tests/regression.test.js:624-644` 与 `.trellis/spec/frontend/state-management.md` 的显式 null 拒绝合同，并加入宿主 JSON null 正例、错误类型负例和开关/幂等检查。不能简单移除整段校验或用 `value || {}` 吞掉其它非法值。
- FlClash 侧的改进候选是让 rule-providers 与 proxy-providers 的空值适配一致；是否保留旧运行配置则涉及路由安全语义，不是本仓库脚本能单独保证的行为。
- 独立兼容风险：FlClash 调用只传 config，依赖 profileName 的精确订阅覆盖在该宿主可能不生效；当前错误发生得更早，不能混成此次直接原因。空值修复后应核对上游解析与住宅 DNS provider 的 Mihomo 应用。
- 引入线索：主会话 `git blame -L 1645,1649` 指向 `6dfa7fa0` 的正则 DNS 校验，不据此声称用户此前版本曾成功。
- 后续实现需有明确文件范围、回归用例与 FlClash 实机验收；本轮不扩展到修改 local TOML/JS、代理凭据或路由算法。

## 尚未证实的个案信息

用户运行时 rule-providers 的实际类型、当前模式、粘贴脚本内容/版本及取消脚本后的恢复结果未采集。源码和复现足以解释已报告现象，但未完成用户客户端端到端确认。

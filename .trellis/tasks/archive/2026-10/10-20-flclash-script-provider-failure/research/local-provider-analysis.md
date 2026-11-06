# Research: FlClash 扩展脚本 rule-providers 校验失败的本地链路

- Query: 定位截图中的 `validateRegexDnsProviderOwnership` 报错，复现缺省、null、空映射与错误类型，检查输入不变性与开关关系。
- Scope: internal；官方 FlClash 0.9.1 源码与网络资料由主会话研究，不在此重复。
- Date: 2026-10-19（用户时区 America/New_York；任务目录由主会话指定）

## Findings

### 结论与证据等级

已确认截图报错来自本仓库脚本自身的类型校验。`null` 能稳定触发完全相同的中文消息；数组、字符串、数字、布尔值也能触发，因此截图本身不能单独证明实参一定是 null。需要将 FlClash 0.9.1 的宿主序列化行为与此条件连接，才能完成端到端根因判断。

报错发生在上游解析、家宽节点构建、规则注入、DNS 重建之前。脚本没有返回空代理列表，也没有在该失败路径删除原始代理。用户补充截图确认消失的是侧栏“代理”入口，“配置”仍在；该现象应由宿主错误处理与导航状态解释，不能把脚本抛错直接等同于订阅被删除。

### Files found

| 文件 | 作用 |
| --- | --- |
| `clash-verge-ai-residential.js` | 公开扩展模板、类型判断、变换流程与正则 DNS provider 管理。 |
| `tests/regression.test.js` | 覆盖 provider 类型拒绝、保留名称所有权、四组开关组合、输入不变性及幂等。 |
| `CHANGELOG.md` | 未发布记录包含正则住宅 DNS provider 功能，可作为有限的历史线索。 |
| `.trellis/spec/frontend/state-management.md` | 明确 provider 所有权、非对象拒绝及输入不变合同。 |
| `.trellis/spec/frontend/hook-guidelines.md` | 描述 Clash Verge Rev 宿主错误回退；不能直接套用到 FlClash。 |
| `research/repro.cjs`（相对本任务） | 可运行的 16 场景脱敏复现；仅从公开模板读取，在 VM 内改变开关，无本地文件、真实凭据或网络依赖。 |

### Code patterns

1. `clash-verge-ai-residential.js:574-575`：`isPlainObject(value)` 明确排除 null 和数组，接受非 null 的对象。这里不是严格原型检查；本次仅讨论 JSON 能表达的值。
2. `clash-verge-ai-residential.js:582-599`：`cloneConfigForEdit` 用顶层展开拷贝配置，独立复制代理/规则数组与组对象；`rule-providers` 的值先原样保留，包括 null。
3. `clash-verge-ai-residential.js:1645-1648`：条件是 `providers !== undefined && !isPlainObject(providers)`。因此字段缺省或显式 undefined 允许通过；null 不被视作缺省。
4. `clash-verge-ai-residential.js:732-737`：`fail` 打印消息并抛出 `Error`。当前公开模板的错误行号与截图生成脚本存在偏移，不能据此断定用户脚本具体提交。
5. `clash-verge-ai-residential.js:1939-1947`：`main` 克隆后无条件运行 provider 校验；没有按 Vertex 或 Cursor 正则开关跳过。它在解析可用上游前中断。
6. `clash-verge-ai-residential.js:1955-1974`：实际节点、代理组、规则与 DNS 的注入都在校验之后。失败路径没有执行这些步骤，更没有返回一个删空代理的配置。
7. `clash-verge-ai-residential.js:133,139`：公开默认 Vertex 开启，Cursor 索引关闭。`clash-verge-ai-residential.js:1401` 的 `activeDomainRegexes` 从开关派生正则；`clash-verge-ai-residential.js:1701-1713` 重建 inline/classical provider，在关闭全部正则时移除旧托管对象。
8. `tests/regression.test.js:624-644`：开关开/关两组测试都把顶层 null、数组、字符串、42 列为必须拒绝，且断言输入不变、错误不泄露凭据。这是显式既有行为，不是没有测试的偶然路径。
9. `tests/regression.test.js:543-586`：四组开关组合验证用户 provider 保留、托管 provider 内容、DNS 范围、输入不变、重复运行一致。
10. `tests/regression.test.js:612-620`：关闭全部正则时缺省映射不会凭空新建，显式空对象得以保留。

### 脱敏复现结果

命令：

```powershell
node .trellis/tasks/10-20-flclash-script-provider-failure/research/repro.cjs
```

- 环境：Node.js `v26.7.0`，不是 FlClash 自带 JavaScript 引擎。
- 输入脚本 SHA-256：`cc6d7f85376c21bab02924b6d0ff34451f505751263d158df81969fdfdf116ba`。
- 场景总数：8 种输入 × 2 组开关 = 16，全部符合当前脚本合同，退出码 0。
- 两组开关分别是公开默认和两个正则开关同时关闭；关闭开关只在 VM 源码字符串中进行，未写入公开模板。
- 使用保留节点 `家宽-SOCKS5` 搭配 `home.example.test` 与虚构凭据，走公开模板已支持的现有节点复用机制。机场节点也完全虚构，不进行真实连接。

| `rule-providers` 输入 | 公开默认 | 正则开关全关 | 原输入 |
| --- | --- | --- | --- |
| 字段不存在 | 正常返回 | 正常返回 | 不变 |
| 显式 undefined | 正常返回 | 正常返回 | 不变 |
| null | 截图同文报错 | 截图同文报错 | 不变 |
| `{}` | 正常返回 | 正常返回 | 不变 |
| `[]` | 截图同文报错 | 截图同文报错 | 不变 |
| 字符串 | 截图同文报错 | 截图同文报错 | 不变 |
| 数字 42 | 截图同文报错 | 截图同文报错 | 不变 |
| false | 截图同文报错 | 截图同文报错 | 不变 |

每个正常返回场景仍有 2 个节点与 2 个代理组（原机场组 + 新家宽组）；默认生成托管 DNS provider，全关不生成。每个报错场景抛出 `[AI-家宽] rule-providers 必须是对象，拒绝覆盖已有配置`，未返回配置。

### 既有窄范围测试

```powershell
$env:TEMP = (Resolve-Path -LiteralPath .trellis/tasks/10-20-flclash-script-provider-failure/research).Path
$env:TMP = $env:TEMP
node --test --test-name-pattern='正则住宅 DNS' tests/regression.test.js
```

- PASS：12 tests，0 fail，0 skip，退出码 0，runner duration 220.8702 ms。
- 包括正则 DNS 默认投影、四组开关、开→关→开、全关清理、开/关类型拒绝和用户引用拒绝。
- `withPatchedSwitches`（`tests/regression.test.js:211-230`）会创建临时脚本；仅为该测试子进程把 TEMP/TMP 指向当前 task research，使临时写入及清理保持研究允许范围。未改变全局环境或产品文件。
- 该 PASS 证明现有拒绝 null 的合同生效，不证明 FlClash 兼容性正确，也不替代宿主或 Mihomo 验证。

### Related specs

- `.trellis/spec/frontend/type-safety.md:7`、`:26`、`:46`：运行时边界校验与失败抛错合同。
- `.trellis/spec/frontend/state-management.md:19`：克隆后变换、先校验再覆盖。
- `.trellis/spec/frontend/state-management.md:78`、`:109`：正则住宅 DNS 的所有权与非对象映射拒绝。
- `.trellis/spec/frontend/hook-guidelines.md:27-37`：明确标注为 Clash Verge Rev dev 2026-08 的异常回退行为；不能据此声称 FlClash 出错后也一定回退原配置。

### 历史线索与后续设计建议

- `CHANGELOG.md:5,27` 将正则 DNS provider 功能列在“未发布”，而当前模板 `SCRIPT_VERSION` 仍为 `5.11.0`（`clash-verge-ai-residential.js:27`）。单凭版本文字不足以确认用户使用了哪个源码状态。
- 如果官方 FlClash 0.9.1 源码证明空 provider 被序列化为 null，最小兼容设计应把该“无提供方”表达与真正错误类型分开；数组/字符串和同名所有权冲突仍应拒绝。不能笼统删除所有 provider 校验。
- 这将变更当前测试和 spec 的显式合同，须回到主会话规划及后续实现授权；本次没有修改合同、代码或配置。
- 关闭 Vertex/索引开关不是该错误的可行规避方式，因为校验不受开关控制。

## External references

本文件未独立检索外部资料。FlClash 0.9.1 官方源码、发布标签、宿主错误处理和核心版本见同目录的 [root-cause-report.md](./root-cause-report.md)（由主会话编写）；这里的 Node 复现不构成真实宿主验证。

## Caveats / Not Found

- 未读取用户的 `*.local.js`、`*.local.toml`、真实订阅或凭据；无法确认截图时传入值的具体类型。
- 用户已补充截图，确认 FlClash `0.9.1` 消失的是侧栏“代理”入口，“配置”入口仍在。该视觉证据不等于订阅文件被删除，也不单独证明系统代理断网。
- 未运行真实 FlClash、Mihomo、公网 DoH 或 UDP 链路测试。
- 未运行 `just ci` / monitor / docs 构建：这是未改产品文件的定向原因研究，仅记录窄测试 PASS。
- 研究角色禁止任何 git 操作，因此未运行 blame/log；CHANGELOG 只能提供变更类别线索，不能确定引入提交。
- 轻量检索 memory registry 未找到相关项目/FlClash/provider 条目；未使用记忆事实作为结论。
- 所有本次写入均限制在主会话指定任务的 `research/` 内，未修复产品代码、测试、spec 或本地配置。

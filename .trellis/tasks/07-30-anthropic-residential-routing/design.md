# Anthropic 核心 API 精确分流设计

## Problem

脚本当前仅生成 `DOMAIN,api.anthropic.com,AI-家宽`。用户实际连接中的
`a-api.anthropic.com` 因没有命中注入规则，继续匹配原 Profile 的
`DOMAIN-SUFFIX,anthropic.com,GPT`。同一域名也不在脚本生成的
`nameserver-policy` 中，导致连接和 DNS 均未绑定住宅链路。

## Design

### Single-source domain catalog

在 `CORE_EXACT_DOMAINS` 中紧邻 `api.anthropic.com` 添加
`a-api.anthropic.com`。不增加新的数组、开关或专用 helper。

现有数据流保持不变：

```text
CORE_EXACT_DOMAINS
  -> activeExactDomains()
     -> buildDomainRules()       -> DOMAIN,...,AI-家宽
     -> buildNameserverPolicy()  -> RESIDENTIAL_DOH
  -> allPossibleExactDomains()
     -> buildManagedRuleSet()    -> current-version cleanup
     -> managed DNS key set
```

### Rule precedence

`main()` 先放入 `buildInjectedRules()`，再追加清理后的 Profile 规则。因此
新生成的精确规则会位于用户现有的
`DOMAIN-SUFFIX,anthropic.com,GPT` 之前，Mihomo 首条匹配语义会把
`a-api.anthropic.com` 送入 `AI-家宽`，同时保留用户的宽泛规则供其他
Anthropic 域名使用。

### Scope boundary

不添加 `anthropic.com` 后缀规则。`www.anthropic.com`、
`docs.anthropic.com`、`status.anthropic.com` 等相邻站点不会因本次变更
进入家宽链路。已有 Claude 产品域、共享依赖开关和进程级兜底均不变。

### Documentation

在 `CHANGELOG.md` 的 `Unreleased` 下记录该缺陷修复。现有 README 和
`docs/routing-scope.md` 已把 Anthropic 模型 API 定义为默认范围，无需
扩大或重写产品合同。

## Compatibility And Migration

- 公共脚本仍兼容 Clash Verge Rev 和 Node.js 18+ CommonJS 测试边界。
- TOML schema、renderer 和所有公开开关不变。
- 新规则属于当前版本托管输出；重复运行会先精确清理再重建。
- 用户订阅或 Merge 中的 `DOMAIN-SUFFIX,anthropic.com,...` 保留原样，
  但因位于注入的精确规则之后，不再接管 `a-api.anthropic.com`。

## Risks And Rollback

- 风险：`a-api.anthropic.com` 的用途未来发生变化。当前依据是用户提供
  的脱敏 Connections 记录；精确规则把影响限制在该主机。
- 自动测试无法模拟 Clash Verge Rev JavaScript 宿主或 Mihomo 的真实
  匹配显示，仍需要脱敏真实 Profile 手工验证。
- 回滚只需删除该精确域名、对应测试断言和 changelog 条目；没有配置
  迁移或持久数据回滚。

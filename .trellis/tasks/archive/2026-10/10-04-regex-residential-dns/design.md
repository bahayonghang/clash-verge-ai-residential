# 设计：现有正则的住宅 DNS

## 1. 机制与边界

业务规则仍由 `activeDomainRegexes()` → `buildDomainRules()` 生成。DNS 读取同一份 active regex，不复制端点名单。

新增一个保留的本地规则集名称 `AI-家宽-DNS-REGEX`。非空时生成：

```yaml
rule-providers:
  AI-家宽-DNS-REGEX:
    type: inline
    behavior: classical
    payload:
      - 'DOMAIN-REGEX,^[a-z0-9-]+-aiplatform\.googleapis\.com$'
nameserver-policy:
  "rule-set:AI-家宽-DNS-REGEX":
    - "https://1.1.1.1/dns-query#AI-家宽&disable-ipv6=true"
    - "https://8.8.8.8/dns-query#AI-家宽&disable-ipv6=true"
```

上例 `nameserver-policy` 在实际输出中位于 `dns` 下。Cursor 开启时把其现有 regex 加入 payload；Vertex 关闭则去掉对应 regex。规则集不带网络 URL、缓存路径或更新周期；不改业务 `rules`。

官方来源和限制见父任务 `research/external-evidence.md`。文档未给可靠最低支持版本；必须使用实际内核确认 inline/classical + DNS rule-set + DOMAIN-REGEX 的组合，不能仅以 YAML/JSON 结构合法作为通过。

## 2. 构造顺序

1. `main` 克隆配置后，在覆盖前检查新增保留名称是否冲突。
2. 现有上游、住宅节点、业务规则构建保持顺序。
3. 克隆 `rule-providers` 映射，构造/清理本任务 provider，不改用户 provider 的内部对象。
4. `buildNameserverPolicy` 将保留 policy 键加入托管键全集；exact/suffix/私网策略保持，正则规则集 policy 在宽泛 geosite 策略之前加入。
5. 两个 regex 开关均关闭时，不生成 policy，并删除已确认归本脚本管理的 provider。若输入原本没有 `rule-providers`，不为了空结果留下空对象。

`buildNameserverPolicy()` 的测试导出仍可单独观察预期键；完整 provider 引用有效性由 `main()` 测试覆盖。

## 3. 所有权与错误

- 新名称与 key 为固定常量，不能嵌入动态上游名称。
- 已有同名 provider 只有在字段正好为 `type/behavior/payload`、type 为 inline、behavior 为 classical、payload 为本任务已知模式的合法非空开关组合时才认定为托管对象；未知字段/额外模式/远程 provider 均拒绝，不静默覆盖。
- 当前所有已知 regex 及未来显式退休模式用于识别旧托管 payload；清理不能只看当前 active 值。
- `rule-providers` 为非对象时，不把异常值静默转换为空映射。
- provider 名称仅供本任务 DNS 使用。已有用户 `RULE-SET` 规则、其他 policy、`dns.fake-ip-filter`、`sniffer.skip-domain` 或 `sniffer.force-domain` 对保留 provider 的引用属于冲突；保持输入并报错，不能在关开关时删掉 provider 却留下引用。只识别该保留名称的引用，不实现通用规则改写器。
- 所有失败发生在返回新配置前，原输入保持不变。错误只包含配置名称/字段，不包含节点凭据。

## 4. 基线和测试

`tests/fixtures/routing-default-v5.11.json` 保持原样。修改默认投影测试时，基于旧 fixture 手工声明唯一允许的 DNS policy 增量；另断言完整 provider 内容。不得从新实现生成 expected。

原测试中“无裸正则键/无宽后缀”的断言保留；“没有任何对应 DNS 路径”的解释需随新机制修订。Node 使用生成的 regex payload 验证模式集合和正反主机，不模拟完整 Mihomo 优先级。

## 5. 文件范围

- `clash-verge-ai-residential.js`
- `tests/regression.test.js`
- `tests/sync-local-config.test.js`
- `docs/dns-and-leak-model.md`
- `docs/configuration.md`
- `docs/local-configuration.md`
- `.trellis/spec/frontend/state-management.md`
- `.trellis/spec/frontend/quality-guidelines.md`
- `CHANGELOG.md` 的 Unreleased
- 本子任务规划与验证记录

不修改 `scripts/sync-local-config.js`、开关登记表、公共 TOML 示例或旧 fixture；两个现有开关足够表达行为。若发现必须增加范围，先回到规划。

## 6. 兼容与回滚

真实验证使用脱敏 Profile、现有内核和明确版本。关闭 Verge DNS 覆盖的效果必须在最终配置中确认；不得修改用户设置来制造通过。运行中的 controller/端口/凭据不由测试修改。

回滚产品更改及文档后，从原始 Profile 重新运行旧脚本。直接在新输出上使用旧脚本会留下未知 provider/policy，不能宣称该方式完成回滚。若只允许在新输出上恢复，需先得到用户对清理这两个托管资源的授权。

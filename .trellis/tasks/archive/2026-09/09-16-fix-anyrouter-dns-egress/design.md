# Design: extra 分类、站点开关与 DNS 解耦

## Architecture and boundaries

扩展脚本继续只生成单例 `AI-家宽` 组。extra 不是新的 Mihomo 组，而是路由开关分类。

配置面保持扁平布尔键，与 `SWITCH_CONFIG_FIELDS` 一致，不引入 `[routing.extra]` 嵌套表。

```text
routing.extra              → ROUTE_EXTRA              分类总开关，默认 false
routing.extra_anyrouter    → ROUTE_EXTRA_ANYROUTER    站点开关，默认 true
```

站点生效条件：`ROUTE_EXTRA && ROUTE_EXTRA_ANYROUTER`。

脚本内用站点登记表作为 extra 的唯一真源，避免再维护一份「全部 extra 后缀」与开关脱节的数组。

```js
const EXTRA_SITES = [
  {
    id: "anyrouter",
    constant: "ROUTE_EXTRA_ANYROUTER",
    suffixDomains: ["anyrouter.top"],
    exactDomains: [],
    residentialDns: false
  }
];
```

- `activeExtraSuffixDomains()` / `activeExtraExactDomains()`：分类开且对应常量开时才返回该站点域名。
- `allPossibleExtraSuffixDomains()`：登记表全部后缀，供 `allPossibleSuffixDomains` 与 DNS 托管键清理，与开关无关。
- `residentialDnsSuffixSet()`：`activeSuffixDomains()` 里 `residentialDns !== false` 的后缀。当前 extra 站点全部为 `false`。
- 现有 `EXTRA_SUFFIX_DOMAINS` 改为由登记表派生，或删除并由测试改读登记表导出。

`buildNameserverPolicy` 只对需要住宅 DNS 的活动后缀写 `+.${domain}`。extra 站点即使路由开启也不写。默认 `nameserver` 仍是机场上游 DoH（`buildUpstreamDoh`），不得改 `DIRECT_DOH`。

## Data flow and contracts

```text
TOML extra / extra_* 
  → 渲染器注入常量
  → EXTRA_SITES + 分类门闩
  → 活动域名 → DOMAIN / DOMAIN-SUFFIX
  → 活动且 residentialDns 的域名 → nameserver-policy
  → 登记表全集 → managed 规则与 DNS 键清理
```

渲染器：`SWITCH_CONFIG_FIELDS` 增加一行 `extra_anyrouter`。示例 TOML 写 `extra = false`、`extra_anyrouter = true`。缺失键按示例补全。已有 `extra = true` 的本地文件只补新键，不改旧值。禁止写 `*.local.toml` / `*.local.js`。

调优生成器：`SUPPORTED_SWITCH_BUILDERS` 删除 `extra`，新增 `extra_anyrouter → ["anyrouter.top"]`。分类键 `extra` 落入 `unsupported`（无独立 host）。`supported + unsupported` 仍等于 routing 键数（26）。

## Compatibility and migration

| 输入 | 结果 |
|---|---|
| 公开模板 / 新示例 | extra 关，默认投影回到 v5.11（无 AnyRouter 规则、无 `+.anyrouter.top`） |
| 本地已有 `extra = true`，缺 `extra_anyrouter` | 补 `true` → AnyRouter 仍路由，但去掉住宅 DNS |
| 本地 `extra = false` | 所有 extra 站点关闭并清理 |
| 旧配置残留 `+.anyrouter.top` | 托管清理删除 |

不把旧 `routing.extra` 重解释为站点键。

## Trade-offs

- 分类默认关、站点默认开：打开 extra 即启用当前唯一站点；以后加站时可把新站默认关，而不必先关 AnyRouter。双关需要两次翻转，比「只有一个 extra_anyrouter」多一个键，但符合「分类 + 有的开有的关」。
- extra 站点一律不写住宅 DNS：换来避开 ESA GeoDNS，代价是 extra 的 Mihomo 自解析走机场 DoH。SOCKS5 仍可能在家宽侧解析主机名，这是既有模型，不在本任务改变。
- 第二站点只用测试补丁证明，避免无证据的生产域名。

## Rollback

回退公开模板的 extra 常量和 `SWITCH_CONFIG_FIELDS` 即可。用户本地 `extra = true` 不受回退脚本改写。不要用 hosts 钉 IP 作为回退。

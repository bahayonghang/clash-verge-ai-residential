# 多 Profile 上游解析

Mihomo 的 `dialer-proxy` 只接受一个名字。脚本在运行时为当前 Clash Verge Rev Profile 解析出一个合法名称。

## 解析顺序

1. `PROFILE_UPSTREAM_OVERRIDES[profileName]` 里的候选。
2. `HOME_PROXY_TEMPLATE["dialer-proxy"]`，通常是 `🚀节点选择`。
3. `UPSTREAM_CANDIDATES` 里的名字。
4. 启用 `ALLOW_FINAL_RULE_UPSTREAM_FALLBACK` 时，最后一条 `MATCH` 或 `FINAL` 规则的目标。
5. 启用 `ALLOW_HEURISTIC_UPSTREAM_FALLBACK` 时，按组名语义猜测。该开关默认关闭。

第一个存在且结构合法的代理或组会被选中。解析从不把数组写进 `dialer-proxy`，也从不静默回落到 `DIRECT`。

上游名可以含空格和 emoji。不能含 `#` 或 `&`：Mihomo 把它们当作绑定到上游的 DoH URL 分隔符，脚本在构建 DNS 之前会拒绝这样的名字。

## 递归保护

注入家宽节点之前，脚本会：

- 从每个 `include-all` 或 `include-all-proxies` 组排除 `家宽-SOCKS5`；
- 从选中的上游图里去掉脚本托管的组引用；
- 拒绝直接和间接的组循环；
- 拒绝保留名冲突；
- 拒绝顶层显式关闭 UDP 的上游；
- 拒绝解析结果为 `DIRECT`、`REJECT`、家宽节点或 `AI-家宽` 组的上游。

## 运行时限制

静态配置只能证明选择器存在，不能可靠读出它当前选中的项。不要把 `DIRECT`、`REJECT`、`家宽-SOCKS5` 或 `AI-家宽` 放进当作 `dialer-proxy` 的选择器里。

## 诊断

转换成功后脚本打一行日志，版本号来自脚本常量 `SCRIPT_VERSION`，不要把某个发行号抄进文档：

```text
[AI-家宽 v<SCRIPT_VERSION>] Profile“<name>”：dialer-proxy -> <resolved group>
```

解析失败时，用脱敏后的代理组名和最后的 `MATCH` / `FINAL` 规则更新候选列表。不要公开节点 endpoint 或订阅 URL。

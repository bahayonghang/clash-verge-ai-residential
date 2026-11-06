# 外部方案与 Mihomo DNS 证据

## 查证范围

- 日期：2026-10-04。
- 用户来源：https://x.com/wquguru/status/2106041666922783181 。
- 通过临时 Chrome 后台页读取了文章正文与项目链接；读取后已关闭该页。
- 参考库：https://github.com/wquguru/one-exit ，本地快照 `151e1e4fb2c39334bcbce97c72d577b6802dabc2`。
- 当前公共脚本版本 `5.11.0`。本轮未读取私人配置或 ResiWatch 数据库，未观测真实流量。

## 文章与本仓库目标的差异

文章标题为「普通人的 Claude 订阅方案（从 0 到 1 完整版，含支付和身份认证）」，页面显示发表于 2026-10-02。文章明确提出 Claude、Gmail、Google Pay 共用一个静态住宅出口，并分别链接 Clash Party、Shadowrocket、FlClash 配置。

本仓库承诺严格 AI-only。把邮件、支付和共享 Google 服务整体送入住宅出口会扩大现有产品范围。文章可证明参考方案的意图，不能作为服务商官方域名清单或账号风控效果的证明。

文章中的支付、身份、账号使用建议不属于本次路由任务。本次不实施此类操作，不提出避免账号限制的承诺。住宅代理地址固定不代表账号状态或服务可用性获得保证。

## DNS 策略的官方依据

### 来源

1. Mihomo DNS 文档：https://wiki.metacubex.one/config/dns/ 。
2. 域名通配语法：https://wiki.metacubex.one/handbook/syntax/#domain-wildcards 。
3. 规则集文档：https://wiki.metacubex.one/config/rule-providers/ 。
4. 配置解析源码：https://github.com/MetaCubeX/mihomo/blob/Meta/config/config.go ，`parseNameServerPolicy`。
5. 标签校验源码：https://github.com/MetaCubeX/mihomo/blob/Meta/component/trie/domain.go ，`ValidAndSplitDomain`。
6. 规则集匹配源码：https://github.com/MetaCubeX/mihomo/blob/Meta/rules/provider/rule_set.go ，`NewRuleSet` / `Match` / 域名匹配包装。
7. classical 匹配源码：https://github.com/MetaCubeX/mihomo/blob/Meta/rules/provider/classical_strategy.go 。

源码 `Meta` 分支由 WebFetch 读取；随后 GitHub API 返回分支快照 `88dcbf7f1614a67c3b36b848ee3592dfa92ada36`（2026-09-30T14:35:49Z）。对该提交的 raw URL 用本机 curl 复取时遇到 TLS 握手错误，未获得固定提交的逐行快照。因此以上结论标注为分支源码查证，不伪称完成了固定版本构建验证。未来实施需记录实际内核版本并验证。

### 已确认的语义

- `nameserver-policy` 支持精确主机、域名通配、`geosite:` 与 `rule-set:`。
- `parseNameServerPolicy` 对普通键执行 `trie.ValidAndSplitDomain`；没有直接把 `DOMAIN-REGEX` 路由表达式当成 DNS policy 键的分支。
- `ValidAndSplitDomain` 拒绝某个标签含 `*` 但不等于单个 `*` 的输入。因此 `*-aiplatform.googleapis.com` 和 `repo*.cursor.sh` 不能用于解决本项目的正则 DNS 覆盖。
- `+.googleapis.com` / `+.cursor.sh` 会扩大到共享或非 AI 主机，不符合本仓库范围。
- DNS 策略可引用 classical 规则集。官方解析器拒绝 `ipcidr` 行为；classical 行为允许并提示只适用于域名规则。
- 官方规则集文档支持 `type: inline`、`behavior: classical` 和 `payload`，不需要远程规则订阅。
- classical matcher 将有效规则交给规则解析器，再逐个匹配。由此可设计仅含 `DOMAIN-REGEX` 的 inline provider，并通过 DNS policy 的 `rule-set:<name>` 引用。该设计仍需真实 Mihomo 验证，不能仅凭 Node 对生成对象的断言宣称可用。
- DNS URL 的 `#<proxy>` 指定代理。`respect-rules` 涉及 DNS 连接路由；代理服务器域名必须有独立 bootstrap 解析路径以避免循环。

### 兼容性和验证边界

文档示例中的 `mihomo/1.18.3` 属于 User-Agent 示例，不能作为 inline provider 或 DNS classical 支持的最低版本。

Node 测试可以证明规则/策略/provider 的生成与清理关系，不能证明 DNS 实际经指定出站、HTTP/3 回退成功、SOCKS5 UDP 可用或所有流量同出口。后续设计必须保留脱敏真实 Profile 与实际内核 gate。缺少内核或用户可用验证环境应记为 BLOCKED，不改为 PASS。

## 不足以支持的结论

- 没有实际流量或故障证据，不能给出节省字节/百分比。
- 没有服务商官方资料证明 one-exit 所有宽泛后缀均为 AI 必需。
- 没有完成 Google 当前 locations 页正文提取；页面在本次 WebFetch 返回中只有导航内容。Vertex 主机的官方来源应沿用仓库已有审计证据或由后续实施重新核实，不能用这次访问作为新的域名收录依据。

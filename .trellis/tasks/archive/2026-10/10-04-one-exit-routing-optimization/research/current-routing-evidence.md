# 当前路由与回退的配置证据

## 基线

- 公共脚本 `clash-verge-ai-residential.js` 版本 5.11.0。
- 不读取 `*.local.toml` / `*.local.js`、实际 Profile、控制器或 ResiWatch 数据库。
- 2026-10-04 在 Node 内存中加载公开脚本，以 `home.example.test` / `airport.example.test` 虚构节点调用导出函数；无网络拨号、无产品文件写入。

## F1 / P2：正则域名缺少对应住宅 DNS 策略

### 证据

- `clash-verge-ai-residential.js:1392-1399`：区域 Vertex 与可选 Cursor 索引为 active regex。
- `:1483-1488`：业务路由注入 `DOMAIN-REGEX`。
- `:1649-1679`：DNS 构造只遍历 exact/suffix，不生成 regex 规则集。
- `docs/dns-and-leak-model.md:28-34` 已明确此例外；`.trellis/spec/frontend/quality-guidelines.md:164-171` 禁止以宽后缀伪补齐。
- 已归档 `.trellis/tasks/archive/2026-09/09-07-js-routing-egress-optimization/prd.md:50` 明确当时不增加 regex→DNS provider；本次是后续优化提案，不把先前范围选择改写为失误。
- `tests/regression.test.js:475-493` 保持窄边界和无裸正则 policy；`:300-319` 固定默认投影，不能用新实现重新生成旧基线掩盖差异。

### 复现

内存中将 `ROUTE_CURSOR_REPOSITORY_INDEXING` 改为 true（不修改文件），分别调用 `buildInjectedRules()` 与 `buildNameserverPolicy()`：

| 主机                                  | 业务规则命中                                 | 精确 DNS policy | rule-set DNS policy |
| ------------------------------------- | -------------------------------------------- | --------------- | ------------------- |
| us-central1-aiplatform.googleapis.com | `^[a-z0-9-]+-aiplatform\\.googleapis\\.com$` | 无              | 无                  |
| repo42.cursor.sh                      | `^repo[0-9]+\\.cursor\\.sh$`                 | 无              | 无                  |

断言全部通过。该结果证明生成配置的覆盖差异；不证明用户发生真实 DNS 泄漏。fake-IP 应答通常不查询公网，SOCKS 域名转发也可能把解析留给住宅服务端。需要本地真实查询时才涉及当前默认非 AI DoH 路径。

### 建议

沿用 `activeDomainRegexes()` 的模式和开关，以 Mihomo 本地 inline/classical provider 建立 DNS policy 引用；不增加第三方依赖、远程规则源、主机或宽后缀。先在实际内核验证该机制，再实现 provider 所有权、幂等与开关清理。保留 `extra.residentialDns=false` 的有意解耦。

正向主机：us-central1-aiplatform.googleapis.com、europe-west4-aiplatform.googleapis.com、repo42.cursor.sh。负向：maps.googleapis.com、storage.googleapis.com、fonts.googleapis.com、repofoo.cursor.sh、repo42.cursor.sh.example.test、foo.us-central1-aiplatform.googleapis.com。

## F2 / P2：可达上游 empty-fallback 可重新引用住宅节点

### 证据

- `ref/one-exit/clash-party/override.yaml:29` 使用 empty-fallback；该字段触发了本次覆盖检查。
- `clash-verge-ai-residential.js:1093-1108` 只清理显式 proxies 中的脚本注入名称。
- `:1110-1130` 为 include-all 追加住宅节点 exclude-filter。
- `:1147-1224` 只沿显式 proxies 遍历组图；`empty-fallback` 未进入检查。
- `:1032-1043` 为住宅节点设置 `dialer-proxy=upstreamName`。
- `tests/regression.test.js:618-683` 覆盖 include-all 排除、组循环和显式住宅引用，`:322-339` 拒绝 **AI-家宽 本身** 的额外字段；都没有验证普通可达上游的 empty-fallback。

### 官方语义

https://wiki.metacubex.one/config/proxy-groups/ ：`empty-fallback` 指定组无成员时使用的代理名称，默认 COMPATIBLE；不能指向其他代理组。本例使用合法的 SOCKS5 节点名“家宽-SOCKS5”。不以“empty-fallback 指向另一个组”的无效配置证明缺陷。

### 复现输入

```json
{
  "proxies": [
    {
      "name": "家宽-SOCKS5",
      "type": "socks5",
      "server": "home.example.test",
      "port": 1080,
      "udp": true
    },
    {
      "name": "Airport",
      "type": "socks5",
      "server": "airport.example.test",
      "port": 1080,
      "udp": true
    }
  ],
  "proxy-groups": [
    {
      "name": "Proxy",
      "type": "url-test",
      "proxies": [],
      "include-all-proxies": true,
      "filter": "^NoAirportMatches$",
      "empty-fallback": "家宽-SOCKS5"
    }
  ],
  "rules": ["MATCH,Proxy"]
}
```

`main(input,"synthetic")` 成功返回。输出 `Proxy.empty-fallback` 仍为 `家宽-SOCKS5`，同时住宅节点的 dialer-proxy 为 Proxy。新增的 `^家宽-SOCKS5$` exclude-filter 未移除显式 fallback 引用。

第二组实验把该 url-test 组改名 Nested，通过 `Proxy.proxies=["Nested"]` 连接；同样成功返回上述 fallback，住宅 dialer 仍为 Proxy。两个实验均证明输入未被修改。

当过滤后无普通节点时，配置引用形成 `家宽-SOCKS5 → Proxy [→ Nested] → empty-fallback 家宽-SOCKS5`。本轮未启动真实内核，不能将结果写成已实测运行时堆栈溢出或流量外泄；已确认脚本输出保留了递归关系。

### 建议

在现有可达上游遍历中拒绝 `empty-fallback` 等于脚本保留住宅出站名称。抛出包含组名、字段、可达路径的中文错误，保持输入不变；不静默改成 DIRECT、COMPATIBLE 或其他机场节点。

仅处理当前选定上游的可达组；不检查用户所有未使用组，不重写未知字段，不增加地区筛选、不把缺省 fallback 的运行时行为纳入本轮重构。

## 最小文件范围

- 两项共有：`clash-verge-ai-residential.js`、`tests/regression.test.js`、`CHANGELOG.md`，必要的对应 frontend spec。
- DNS：`tests/sync-local-config.test.js`（现有两个开关的生成输出验证）、`docs/dns-and-leak-model.md`、`docs/configuration.md`、`docs/local-configuration.md`。
- 回退：`docs/configuration.md`、`docs/troubleshooting.md`。
- 不改版本号、依赖、旧固定投影 fixture、渲染器开关表、ResiWatch、实际 local 配置或参考项目。

## 检查记录

只执行了上述公开函数的小型内存实验，退出码 0。没有运行 `just ci` 或 `just docs-build`，因为本轮只交付研究与规划。真实 Mihomo、Clash Verge、DNS 和 UDP 验证尚未执行，应在获准实施后执行并记录结果。

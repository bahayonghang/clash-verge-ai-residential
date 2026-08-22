# Mihomo 内核 DNS/规则语义调研（2026-08-16）

来源：mihomo Alpha 分支源码 + metacubex wiki。已核对的关键源码：
`config/config.go`（parseNameServer L1195-1215 / parseNameServerPolicy L1326 /
respect-rules 校验 L1407）、`component/trie/domain.go`、`dns/util.go`、
`dns/middleware.go`、`tunnel/dns_dialer.go`、`component/geodata/init.go`。

## 对本脚本的影响判定

### 1. `+.domain` 通配键 → 匹配裸域本身（脚本现状正确，无需修复）

wiki 语法页 + `component/trie/domain.go` 源码双重证实：`+.example.com` 拆成
`example.com`（裸域）与 `.example.com`（任意深度子域）两条插入，语义等同 DOMAIN-SUFFIX。
因此 `buildNameserverPolicy` 只写 `+.suffix` 键即可覆盖裸域，静态分析中的 P1 撤销。
可补一条回归测试断言该设计依据。

### 2. DoH fragment `#代理名&参数` → 语法与参数全部有效

- fragment 以 `&` 分隔；无 `=` 段为代理名（支持策略组名），有 `=` 段为参数。
- `disable-ipv6=true` 是真实参数（`dns/util.go` wrapClientWithDisableTypes 丢弃 AAAA）。
- 中文/emoji 组名无解析障碍（源码推断，wiki 未明示）。
- 脚本的 `buildUpstreamDoh` 对 `#`/`&` 的拒绝逻辑与 mihomo 语义一致，保留。

### 3. respect-rules 与 fragment 的优先级 → fragment 优先

`respect-rules: true` 仅对"未写 fragment 代理"的 nameserver 填特殊值 `RULES`。
脚本所有 nameserver 均带 fragment，因此行为完全确定，不存在"DNS 意外按规则走错出口"
的歧义。前提条件 `proxy-server-nameserver` 非空已满足（DIRECT_DOH）。
结论：当前 DNS 架构语义正确，无需改动。

### 4. `geosite:cn` / `geosite:private` 键 → geosite.dat 硬依赖（真实运维风险）

`parseNameServerPolicy` 遇 `geosite:` 键调用 `NewGEOSITE` → geosite.dat 缺失时
自动下载（默认 GitHub releases）→ 下载失败 = 配置解析失败 = mihomo 拒绝启动。
在 Clash Verge Rev 中表现为 `mihomo -t` 校验不通过 → 首次启动回退"默认最小配置"
（所有代理不可用），运行中则保留旧配置并提示验证失败。
缓解：文档化（troubleshooting 新增条目：GeoData 初始化失败的处理）；
订阅 Profile 本身几乎总带 GEOIP/geosite 规则，风险集中在全新裸安装 + 离线场景。
不建议为它引入开关（默认值两难：关了破坏国内域名解析路径，开了保留风险）。

### 5. fake-ip 下 nameserver-policy 的实际生效时机（设计澄清，非缺陷）

`dns/middleware.go` withFakeIP：A/AAAA 查询未命中 fake-ip-filter 时直接分配
fake-ip 返回，不向上游查询。真实查询只发生在：fake-ip-filter 命中的域名、
非 A/AAAA 类型、L3 出口建连需要真实 IP（ipExchange → matchPolicy）、
proxy-server-nameserver 通道。
另外：socks5 出口支持域名直传（RFC1928 ATYP=0x03），AI 域名的真实解析大多
发生在家宽 SOCKS5 服务器侧 —— nameserver-policy 是兜底而非主路径。
结论：现有"AI 域名 DNS 经家宽"的文档表述偏强，可在 dns-and-leak-model.md 精确化。

### 6. sniffer 键名 → 全部合法

顶层 `override-destination` 未废弃；协议级 override-destination 覆盖全局；
`force-dns-mapping`、`parse-pure-ip` 合法；旧 `sniffing` 数组才是废弃写法。
脚本 `hardenSniffer` 产出的结构合法，无需改动。

### 7. dialer-proxy + socks5 udp → 行为确认

UDP 经 dialer-proxy 所指代理的 ListenPacketContext 封装；家宽 socks5 的 UDP
依赖服务端 UDP ASSOCIATE（TCP 连接生命周期绑定）。wiki 警告"被中转的落地节点
不要选 hy2/tuic/wg"—— 本脚本落地固定 socks5，符合建议；机场侧仅作第一跳
dialer，协议不限。风险面（家宽服务端不支持 UDP、机场节点 udp 默认 false）
已在 dns-and-leak-model.md 有部分记录，可补"机场节点未显式 udp: true 时
mihomo 默认关闭 UDP"的说明。

### 8. exclude-filter → 正则（`|` 分隔），脚本生成的 `(?:A)|(?:B)` 兼容 Go RE2

`include-all-proxies` 引入全部出站代理（不含策略组）且按名称排序 ——
脚本的 exclude-filter 注入命中正确集合。

### 9. 规则类型 → IP-CIDR6 / DOMAIN-REGEX / PROCESS-NAME-REGEX / PROCESS-PATH-REGEX / AND 全部受支持

`no-resolve` 仅用于 IP 类规则且位于策略之后 —— 脚本现有规则串全部合规。

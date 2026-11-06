# one-exit 与当前脚本对照

## 来源与边界

参考仓库 `ref/one-exit`，commit `151e1e4fb2c39334bcbce97c72d577b6802dabc2`。已阅读 README、三份平台配置与检查器，未修改或运行参考项目。本轮研究子代理因 400/429 服务错误未完成；下列结果由主会话直接阅读形成，不计为独立复核。

文章读取情况与官方 DNS 依据见 `external-evidence.md`。

## 逐项比较

| 维度            | one-exit                                                                                                             | 当前脚本                                                                              | 结论                                                                         |
| --------------- | -------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| 产品范围        | Claude、Google（含 Gmail/Pay）、Persona；`ref/one-exit/README.md:3`                                                  | 显式 AI 产品，其余保留原 Profile                                                      | 不整体导入参考清单                                                           |
| 链式拨号        | HTTP 住宅代理经 `Dialer-Res`；`clash-party/override.yaml:8-29`                                                       | SOCKS5 经当前 Profile 的真实上游，`clash-verge-ai-residential.js:1014-1047`           | 链路结构已实现，无需改为 HTTP                                                |
| 中转选择        | 地区名称正则 + url-test，300 秒、50ms tolerance、empty-fallback；`override.yaml:19-29`                               | Profile override、通用候选、最终规则回退；`:955-994`                                  | 不新增地区猜测；当前兼容不同订阅的机制更适合本仓库                           |
| 固定出口        | README 说明中转切换不改变住宅代理；`README.md:5-9`                                                                   | 单成员 `AI-家宽`；`:1758-1769`                                                        | 结构已具备，实际 IP 仍需供应商与连接证据                                     |
| Claude 域名     | Party/SR 用 `anthropic.com` 宽后缀、`clau.de`、MCP 官网；`override.yaml:32-40`                                       | API 用 exact；`clau.de` 已退休；`claudemcpcontent.com` 已覆盖；`:194-227`             | 当前更窄；不恢复退休域名或 MCP 文档站                                        |
| FlClash 差异    | `DOMAIN-KEYWORD,anthropic/claude/modelcontextprotocol`；`flclash/claude-google.js:43-50`                             | 明确产品后缀/主机                                                                     | 关键词可能命中 `not-claude.example.test` 等无关域，拒绝采用                  |
| Anthropic IP    | 三份模板均 `/21`；Party/FlClash 另有 ASN 399358                                                                      | 官方 inbound `/23` + IPv6 `/48`；`:503-507`                                           | 保留当前更窄范围；/21 含 2048 个 IPv4 地址，/23 含 512 个，前者为后者 4 倍   |
| Google          | 整体 google.com、googleapis.com、gstatic.com、gmail.com、googleusercontent.com、recaptcha.net；`override.yaml:42-47` | Gemini/Vertex/Antigravity 明确端点，可选共享认证；`:229-299`、`:417-425`              | 不把邮件、地图、字体、支付、共享存储送入住宅出口                             |
| Persona         | withpersona.com；`override.yaml:49`                                                                                  | 未默认收录                                                                            | 共享身份验证服务；没有本地证据，不新增                                       |
| 进程路由        | Android Claude、Play Store、Wallet 进程；`flclash/claude-google.js:51-62`                                            | `ENABLE_AI_PROCESS_FALLBACK=false`；查找进程独立为 always；`:1531-1539`、`:1842-1846` | 不引入 Android 进程规则或全量进程路由                                        |
| DNS             | 三份模板均未设置 DNS 策略                                                                                            | exact/suffix 指定住宅 DoH；区域 Vertex/可选 Cursor regex 有已知例外；`:1649-1679`     | 可优化当前已知正则例外；参考方案不提供现成 DNS 修复                          |
| UDP/QUIC/WebRTC | HTTP 节点没有本仓库的 SOCKS5 UDP 契约；README:62 建议手动关闭 Chrome QUIC，无对应规则                                | udp:true、上游 UDP 校验、保守 sniffer，共享实时捕获默认关闭                           | 不推导“切 HTTP 或全局拒绝 UDP 可解决泄漏”；实际路径待实测                    |
| TUN/IPv6        | 无设置/告警实现                                                                                                      | 只读检查宿主所有字段；`:1772-1806`                                                    | 保留当前宿主所有权边界                                                       |
| 验证            | webshare.io 后缀被固定走住宅，随后访问 ipv4.webshare.io；`override.yaml:51`、`README.md:52-68`                       | 普通非 AI 测试域不默认走住宅；`docs/troubleshooting.md:71`                            | 只证明探测连接的出口，不能推断 AI/DNS/UDP 全部同出口；不增加整个 webshare.io |
| 幂等/所有权     | FlClash 原地 push 代理/组，规则前置；重复调用没有清理；`flclash/claude-google.js:16-29,64`                           | 克隆、保留名称校验、精确托管清理、去重；`:580-612`、`:754-804`、`:1599-1629`          | 保留当前机制                                                                 |
| 空组回退        | 显式 `empty-fallback`；`override.yaml:29`                                                                            | 上游遍历只检查 proxies，未检查该字段；`:1147-1224`                                    | 发现可复现的配置校验缺口，独立任务处理                                       |

## 不能混淆的两个 DIRECT

参考 README:7 与模板注释允许手动将住宅节点的 `dialer-proxy` 改为 DIRECT。该配置改变“本机到住宅代理”的传输路径，住宅代理仍然是最终目标；不等价于把业务规则改为 DIRECT。

本仓库要求住宅端口通过机场上游建立连接，且拒绝把 DIRECT 用作顶层上游。该手动建议不纳入本次任务。现有 `AI-家宽` 不含 DIRECT 成员，也不能证明宿主在脚本失败时会阻断流量；宿主可能回到原 Profile。

## 验证能力

`ref/one-exit/scripts/check_templates.py:36-81` 校验 Party 的部分模板结构和规则类型；`:91-103` 只对 FlClash 文本中的 dialer/filter/fallback 做有限检查；`:106-123` 检查 Shadowrocket 规则。检查器不启动三种客户端，不验证 DNS/UDP/同出口，不证明占位节点可连接，不检查 FlClash 重复运行行为。

## 术语

`CONTEXT.md:23-25` 定义“进程路由”。当前脚本多处注释仍用“进程兜底”描述同一控制项。本研究统一使用“进程路由”，保留代码标识符原样；不把术语清理加入本次改动范围。

## 筛选结果

1. **纳入规划**：当前正则 AI 路由的住宅 DNS 一致性，作为已有明确例外的后续能力，不标记为已实测泄漏。
2. **纳入规划**：可达上游 `empty-fallback` 指向住宅节点的配置校验缺口。
3. **保持现状**：精确/有界域名、现有认证开关、单成员出口、SOCKS5、宿主字段只读、公开模板与本地凭据隔离。
4. **不纳入**：宽泛 Google/Anthropic 后缀、关键词、旧 /21、ASN、Persona、Play/Wallet 进程、webshare 全域、地区猜测、HTTP 迁移、DIRECT 传输回退、全局 QUIC/UDP 禁用。

# 分流架构比较与优化依据

调研日期：2026-09-07。当前仓库基线：`dev@063c5561b9e81e42f89d7966a5d1a9772bdc44b3`，脚本 v5.11.0；开始时工作区干净。本报告只依据公开源码、官方文档和内存合成配置，不读取本地凭据、不连接控制器、不改变网络。

## 1. 参考项目实际做了什么

参考版本通过 GitHub API 固定为 `yding-git/personal-edge-proxy@a6f4f75f268567656b4b2412cbcef4d715365d97`。阅读了 README、Xray 示例、WARP 与 static SOCKS 文档；示例源码通过固定提交再次读取。

| 层 | 参考项目的职责 | 对本项目的启发 |
|---|---|---|
| 入站 | HY2 日常使用，REALITY 为备用入口 | 改入口不等于改变目标服务看到的出口 |
| 普通出站 | VPS 原生 direct 是默认出口 | 普通流量与高成本出口分开 |
| AI 出站 | OpenAI/Gemini 等指定目标经独立 WARP local proxy | 按服务选择出口，不应全系统改默认路由 |
| 固定出站 | Claude/Anthropic 可选 static SOCKS | 固定出口绑定独立于入口，故障不应偷偷切换身份 |

参考项目使用客户端粗分流、服务端细分流。本项目则在客户端 Mihomo 配置生成阶段完成细分流，服务端通常是用户无管理权限的机场。两者部署边界不同。

来源：[README](https://github.com/yding-git/personal-edge-proxy/blob/a6f4f75f268567656b4b2412cbcef4d715365d97/README.md)、[固定出口说明](https://github.com/yding-git/personal-edge-proxy/blob/a6f4f75f268567656b4b2412cbcef4d715365d97/docs/static-socks.md)、[WARP 说明](https://github.com/yding-git/personal-edge-proxy/blob/a6f4f75f268567656b4b2412cbcef4d715365d97/docs/warp-outbound.md)。WARP 是独立 Cloudflare 出口，不能当作住宅 IP 或服务可用性保证。

## 2. 不能直接照搬的示例细节

固定版本的 [Xray 配置](https://github.com/yding-git/personal-edge-proxy/blob/a6f4f75f268567656b4b2412cbcef4d715365d97/examples/xray-server.example.jsonc) 使用 `IPOnDemand`；规则依次是私网 block、Claude/Anthropic static SOCKS、指定 AI 的 WARP。WARP 规则限定 TCP，static SOCKS 规则声明 TCP/UDP。未匹配流量使用默认出站。

- `anthropic.com` / `openai.com` 后缀比本项目清单更宽，照抄会把文档、平台或其他非核心主机纳入。本项目既有 exact/suffix 划分更适合节约家宽。
- VPS 入站访问私网应阻断；客户端访问 localhost/MCP/LAN 应保持 DIRECT。本地脚本不能复制服务端私网 block。
- 示例的 TCP-only WARP 不是可直接迁移的完整 UDP 策略。本项目不能因此加全局 UDP REJECT、强制 HTTP/1.1 或 Voice 端口规则。
- `127.0.0.1:40000` 位于参考 VPS，不是本机现成出口。接入 WARP 需要独立部署/已有节点与能力验证，用户已选择本任务不做。
- 本项目不替代机场、不部署 Xray/HY2/REALITY，不增加多档模式、服务出口注册表或自动故障切换。

## 3. 当前 JS 的真实数据路径

```text
本机请求
  ├─ 私网/localhost                  → DIRECT
  ├─ 启用的核心域名/精确主机/区域正则 → AI-家宽 → 家宽-SOCKS5
  │                                              ↑
  │                           经已选 Profile 上游连接家宽 endpoint
  ├─ Anthropic IP 回退（默认开）      → AI-家宽
  ├─ 可选端口/共享 DNS/进程兜底       → AI-家宽
  └─ 剩余请求                        → 原 Profile 规则按原顺序处理
```

`dialer-proxy` 决定连接家宽 endpoint 时的传输上游，最终对外出口仍由家宽 SOCKS 决定。`ALLOW_FINAL_RULE_UPSTREAM_FALLBACK` 在生成配置时借用 MATCH/FINAL 的目标寻找上游，不是运行时家宽故障回退。依据：`clash-verge-ai-residential.js:950`、`:1009`、`:1479`、`:1726`；[Mihomo dialer-proxy 文档](https://wiki.metacubex.one/config/proxies/dialer-proxy/)。

原 Profile 的剩余规则可能指向机场、DIRECT、特定服务组或用户自定义家宽规则。因此“关闭开关必然改走机场”比代码实际保证更强。

这里的“流量分配”是按目标的策略选择，不是流量配额、比例分摊或负载均衡。JS 在 Profile 配置生成时运行，实际连接匹配由 Mihomo 完成；没有性能证据支持把新增缓存、并发重构或规则微优化列为本任务目标。优先减少误捕获和补齐控制粒度。

## 4. 已有优点，不应重做

1. 私网规则优先于进程捕获，保护 localhost/MCP；注入规则优先于原有宽泛规则（`:1368`、`:1479`、`:1756`）。
2. AI 和普通 DNS 已分流；节点域名 bootstrap 独立，避免家宽递归（`:1593`）。
3. 进程、通用 STUN/TURN、公共 DoH 捕获默认关闭；Cursor 仓库索引独立且默认关闭（`:169`、`:179`、`:185`、`:191`）。
4. 清理使用全可能清单，开关关闭后可删除旧托管规则/DNS；未知规则原样保留（`:1312`、`:1494`、`:1531`）。不能按 `AI-家宽` 目标批量删除用户规则。
5. 核心产品域保留会话完整性；登录、第三方依赖和网页资源有独立开关。不能靠全收窄 exact 或全放大 suffix 解决所有问题。

## 5. 发现与优先级

### F1 / P1：固定家宽组对动态来源的约束需要补齐

`validateReservedNameCollisions()` 在 `clash-verge-ai-residential.js:782` 只检查 `type=select` 与显式 `proxies=[家宽-SOCKS5]`；`buildAiGroup()` 在 `:1652` 展开旧组对象。旧组的 `use`、`include-all*`、筛选字段可能继续影响实际成员，显式列表只有一个节点不能证明实际只有一个出口。官方 [代理组通用字段](https://wiki.metacubex.one/config/proxy-groups/) 明确额外来源、排除条件和空组回退的语义。

这是配置结构问题，不代表已观测到用户真实流量绕行。最小修复应在已有保留名边界拒绝非托管形状，保护单一出口；不能把机场/DIRECT 加进家宽组充当可用性优化。合成复现和建议见 validation.md。

### F2 / P2：核心流量的开关覆盖不完整

`CORE_SUFFIX_DOMAINS` (`:218`) 含 4 个 Claude 产品后缀；`CORE_EXACT_DOMAINS` (`:247`) 混合 3 个 Anthropic exact、4 个 Antigravity/Code Assist exact 与 1 个 Gemini Developer API exact。它们无条件进入 active 列表（`:1266`、`:1283`）。

关闭 `gemini_web_core` 和 `vertex_ai_endpoints` 后仍产生 5 条 Google 家宽规则，已内存复现。不是这两个旧开关实现错误，而是缺少独立控制核心 API 的入口。Claude 核心也没有关闭入口。

建议只补三个默认 true 的 boolean：`anthropic_core`、`gemini_api_core`、`antigravity_core`。域名集本身不扩大，默认覆盖范围和目标不改变。与已有 OpenAI/Cursor/Grok 开关共同覆盖现有核心服务。用户已选择沿用两条路径，不创建服务选择器组。

### F3 / P2：进程与 IP 回退需要明确服从关系

当前 `openai_core=false`、`ai_process_fallback=true` 时仍产生 ChatGPT/Codex/OpenAI 进程规则，已内存复现（`:1398`、`:1428`）。Cursor 的进程开关也不依赖 `cursor_core`。这源于旧开关只控制域名的设计；规划将核心关闭含义扩展到其专属进程兜底，并明确这个非默认组合的行为变更。

新 `anthropic_core=false` 应同时禁止脚本生成 Anthropic IP 回退，避免以 IP 规则重新捕获刚关闭的核心服务。`anthropic_ip_fallback` 仍保留独立控制，默认值不改变。辅助域、认证、静态资源及全局实时/DNS开关继续独立，不把 core 解释为任何情况下的全域禁用。

### F4 / P2：正则路由的 DNS 保证表述过强

`activeDomainRegexes()` 和 `buildDomainRules()` 生成区域 Vertex/可选 Cursor 索引规则（`:1303`、`:1390`），而 `buildNameserverPolicy()` 只处理 exact/suffix（`:1544`）。Node 输出证实没有区域正则的等价 DNS policy。现有 `docs/dns-and-leak-model.md:27`、`:29` 对“AI 域名全部同住宅 DNS”说得过满。

官方 [DNS 配置](https://wiki.metacubex.one/config/dns/) 的 policy 键采用 [Clash 域名通配语法](https://wiki.metacubex.one/handbook/syntax/)，不能直接假定支持 `DOMAIN-REGEX` 的正则文本。`respect-rules` 管的是 DNS 连接，不会自动把业务主机的正则转换为 policy。fake-IP 应答、SOCKS 域名转发也不能代替本地真实解析路径的证据。

本任务选择记录已有例外、修正文档、保持窄规则；不新增 provider/规则下载器，也不扩大为 `+.googleapis.com` 或 `+.cursor.sh`。正则专属本地 DNS 同出口暂不纳入实施验收，真实解析与 UDP 行为仍需宿主证据。它们不是已被证实的 DNS 泄漏。

### F5 / P2：配置与观测映射必须一起更新

`scripts/sync-local-config.js:24` 的 routing 表有 21 个键；`skills/residential-rule-tuning/scripts/build-inputs.js:8` 支持 9 个域名开关，其余 12 个列为 unsupported。新增三个开关后应为 24 / 12 / 12，并把原始核心域映射到对应开关。`tests/install-agent-skills.test.js:151` 和 skill 文本写死了 21，须同步。

生成器当前从公开模板导出规则，不代表用户实际运行的本地 TOML。统计数据库也不保存 rulePayload；按 host 映射不是实际开关因果归因。本任务不改数据库/CLI schema，不新增收益计算器。

## 6. 预期收益与证据边界

确定可获得的是更完整的控制能力、更明确的异常配置拒绝和文档合同。默认三个开关都为 true，因此默认家宽字节不会因新增开关自动下降。

本轮没有读取 monitor.sqlite3、没有实时 Connections、没有测试用户业务登录/推理，不能承诺节省百分比、实际固定 IP、延迟改善或风控改善。若以后选择关闭某项，应在相同窗口与相近工作负载下用 ResiWatch share/audit、完整性状态及业务验证评估；不能用 Top-N、历史一次峰值或测试 fixture 代替当前收益证据。

## 7. 与既有任务和规范的关系

已归档的 `08-18-cursor-repository-upload-routing`、`08-19-ai-route-domain-audit`、`08-21-openai-auth-routing` 已体现在当前清单及 docs/routing-scope.md；本任务不重新扩展这些域名。特别保留 `daily-cloudcode-pa.googleapis.com`。

局部规范有陈旧描述：frontend/index.md、state-management.md 仍称 main 原地修改，但当前实现 clone 后返回；旧清理描述也与现有 allPossible/retired 集合不完全一致。quality-guidelines/CLAUDE.md 的早期 `just ci = npm run ci` 描述不再完整，实际 justfile:16 还依赖 monitor-check。实施时只同步本任务触及的合同；以当前源码、测试和 justfile 命令为准。

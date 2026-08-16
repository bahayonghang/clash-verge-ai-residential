# 静态代码分析：clash-verge-ai-residential.js v5.6.0

本文件记录不依赖外部资料即可确认的代码层发现。Mihomo 语义与域名时效问题见同目录其他研究文件。

## 仓库执行流程全景

```text
用户编辑 clash-verge-ai-residential.local.toml（gitignored，真实凭据）
        |
        v
just render-local / node scripts/sync-local-config.js
  1. 解析 example TOML，校验其开关键与 SWITCH_CONFIG_FIELDS 一致（防漂移）
  2. 解析 local TOML；缺失开关键按 example 默认值做文本级补全并写回 local TOML
     （保留用户注释/行尾；home_proxy 凭据绝不自动补全）
  3. 用 local TOML 的 home_proxy 渲染 HOME_PROXY_TEMPLATE 代码块
  4. 按 SWITCH_CONFIG_FIELDS 逐个替换模板中的布尔常量（正则锚定 `const X = true;` 行）
  5. 原子写入 clash-verge-ai-residential.local.js（gitignored）
        |
        v
用户将 local.js 全文粘贴进 Clash Verge Rev 的 Global Extend Script
        |
        v
Clash Verge Rev 在每次 Profile 激活/刷新时调用 main(config, profileName)
  1. 校验保留名（AI-家宽 / 家宽-SOCKS5）冲突 -> 抛错 fail closed
  2. resolveUpstreamName: Profile 覆盖 -> 模板默认 -> UPSTREAM_CANDIDATES
     -> MATCH/FINAL 目标 -> (可选启发式) -> 否则抛错
  3. 递归链防护: include-all 组加 exclude-filter；从上游出发 DFS
     清除可达组中对家宽/AI 组的引用、检测环、检测空组、检测 disable-udp
  4. buildHomeProxy + validateHomeProxy -> upsert 家宽-SOCKS5（dialer-proxy=已解析上游）
  5. upsert AI-家宽 select 组（仅含家宽节点，fail closed）
  6. cleanExistingManagedRules（仅清当前版本可生成规则）+ 前置注入
     私有 DIRECT -> AI 域名 -> Anthropic IP -> (可选 realtime/DoH/process)
  7. buildDnsConfig: 重建 dns（fake-ip + 严格 nameserver-policy + respect-rules）
  8. hardenTun / hardenSniffer / ensureProcessLookup / ipv6=false
        |
        v
CI（GitHub Actions + 本地 just ci）:
  node --check 语法 -> node --test 回归（regression/sync/check-template-safety）
  -> check-template-safety 扫描公开模板占位符 + 全仓 token 模式
```

## 确认无问题的部分（核对过）

- `completeLocalToml` 的同 index 插入排序：比较器确实让 appendTable 先 splice，
  键追加落在新表头之前，与注释一致（手推验证）。
- 两个整表同 index 追加时后 splice 的表会插到先插入表之前，表顺序颠倒但语义合法。
- `upsertNamedItem` 拒绝重名、`findOutbound` 拒绝组/节点同名歧义。
- 托管规则清理只删与当前版本生成串完全相同的行，用户规则原样保留（回归测试覆盖）。
- `check-template-safety` 正确排除 local 文件与 `*.local.js`。
- `writeFileAtomically` 在 Windows 上 renameSync 覆盖行为可用（libuv MOVEFILE_REPLACE_EXISTING）。
- 私钥/token 扫描模式覆盖常见泄漏形态。

## 发现的问题（按严重度排序）

### P1 — `buildNameserverPolicy` 只为后缀域生成 `+.` 键，裸域键缺失（待研究确认 mihomo 语义）

`activeSuffixDomains()` 生成 `policy["+.domain"]`。若 mihomo 的 `+.` 不匹配裸域本身，
则 `chatgpt.com`、`claude.ai`、`gemini.google.com`、`grok.com` 等裸域查询走
`nameserver`（机场 DoH），流量却走家宽 —— 与 dns-and-leak-model.md 宣称的不变量
"AI domain resolution and AI application connections must use the intended residential path" 不符。
规则层 DOMAIN-SUFFIX 匹配裸域不受影响；fake-ip 模式下真实查询少，但 TUN dns-hijack
的明文查询、`direct-nameserver-follow-policy` 等路径仍可能触发。
修复方向：为后缀域同时写 `domain` 与 `+.domain` 两个键（幂等、无副作用）。
若研究确认 `+.` 已含裸域则改为文档澄清 + 测试断言。

### P2 — `hardenReachableUpstreamGraph` 静默删除用户组中的 AI-家宽/家宽-SOCKS5 引用

防递归所必需，但用户在 Profile 中自建引用 AI-家宽 的组（合法用法：
把 AI-家宽 放进自己的 selector）会被无提示清除，用户只会在 Connections 里发现
规则不生效。至少应 `warn` 说明哪个组的哪个引用被移除、为什么。

### P3 — DNS 策略硬编码 `geosite:cn` / `geosite:private` 依赖 geosite.dat 成功下载

首次安装 + 无代理冷启动时 geosite.dat 下载失败会导致 mihomo DNS 模块初始化失败
（待研究确认具体行为），整条链路不可用且报错对用户不直观。至少在 troubleshooting
文档中写明；可考虑提供开关或改用 `rule-set`。

### P4 — `buildPrivateDirectRules` 前置会覆盖用户对私网段的代理意图

公司 VPN 场景用户故意让 10/8 走代理组时，注入的 `IP-CIDR,10.0.0.0/8,DIRECT,no-resolve`
排在用户规则之前，私网代理失效。这是 fail-closed 设计的副作用，但进程兜底默认关闭时
私有 DIRECT 规则并不需要排最前（它们只需先于 PROCESS 规则）。可降风险：仅当
ENABLE_AI_PROCESS_FALLBACK 开启时才前置私有 DIRECT 规则，否则保持注入块内顺序不变
（仍在用户规则前，行为不变）——真正修复需把私有规则插到用户规则尾部 MATCH 之前，复杂；
先记录取舍，倾向保守不动。

### P5 — 本地 TOML 无法配置 PROFILE_UPSTREAM_OVERRIDES（数组值）

sync-local-config 的 parseValue 只支持 str/int/bool，多 Profile 用户只能手改
生成的 local.js，而下一次 render 会被覆盖。功能缺口，非缺陷。

### P6 — `warnForUdpDisabledLeaf` 只警告显式 `udp === false`

机场订阅普遍不写 udp 字段（mihomo 默认 false），此时链路上 UDP 静默不可用，
WebRTC/STUN 走家宽会失败且无提示。设计上避免噪音，可考虑一次性 info 汇总。

### P7 — 轻微：`buildHomeProxy` 中 `port`/`server` 无 per-field 占位检测

模板填了 server 但 port 仍是默认 443 时静默用 443（文档已说明 TODO）；
username 显式 `""` + password `""`（无认证）时行为正确（已有测试）。

### P8 — `.trellis/spec` frontend 层（误判，已修正）

经核对 `.trellis/spec/frontend/index.md`，该层内容已针对本仓库适配
（明确声明"不是浏览器前端"，描述的是 Clash 扩展脚本与渲染 CLI 的运行模型），
不是未处理的模板占位。Phase 3.3 已在该层补充宿主执行契约与托管清理模型知识。

## 引擎兼容性风险（待 Verge 引擎研究确认）

脚本使用：`Object spread`、`Set`/`Map`、`String.prototype.includes/padStart?`、
`Object.hasOwn`（sync-local-config.js）、`Array.prototype.flat?`（未用）、
`replaceAll`（sync-local-config.js parseValue）、structuredClone（仅测试用）。
主脚本本身未用 hasOwn/replaceAll —— 需确认 Clash Verge Rev 的 JS 引擎
（Rust boa_engine？）对这些特性的支持，避免宿主运行时 SyntaxError 导致整个
全局扩展脚本失效。

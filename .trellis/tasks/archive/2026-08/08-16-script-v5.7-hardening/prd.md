# PRD：脚本漏洞与健壮性优化（v5.7.0）

## Goal

结合网络调研全面分析 `clash-verge-ai-residential.js` v5.6.0 的问题与漏洞，
梳理仓库执行流程后执行优化。调研结论持久化在本任务 `research/` 四个文件
（静态分析、mihomo 语义、Verge 宿主行为、AI 端点域名）。

## Requirements（按优先级）

### R1 域名清单对齐官方文档（有官方证据的缺口）

1. Claude：新增 exact 域 `mcp-proxy.anthropic.com`（MCP connector 代理）、
   `assets-proxy.anthropic.com`（官方警告缺失会白屏）。
2. OpenAI：`api.openai.com` 从 exact 提升为 suffix，覆盖官方 Codex 数据驻留前缀
   `us.api.openai.com` / `eu.api.openai.com`（exact 形态漏匹配）。
3. Grok：新增 exact 域 `auth.x.ai`（官方 enterprise must-allow OAuth 域）、
   `api.x.ai`（官方 API 直连推理端点）；继续排除安装域 `x.ai`。

约束：遵守仓库"Domain 变更规则"——每个新增域有官方资料；补充负向测试；
不引入宽泛 provider suffix。

### R2 递归清理的可观测性

`removeInjectedReferencesFromGroup` 静默删除用户组中对 `AI-家宽` / `家宽-SOCKS5`
的引用（防递归所必需）。需输出 warn 日志说明哪个组删了什么、为什么，
否则用户只会发现规则不生效。

### R3 宿主行为差异的文档化（Clash Verge Rev 权威字段）

当前 Clash Verge Rev 会在全局脚本执行后强制还原 `tun` / `ipv6` 等权威字段：
脚本内 `hardenTun` 与 `config.ipv6 = false` 在新版宿主上无效。代码保留
（兼容旧版宿主），但必须：在脚本 info 日志中提示这些字段由 Verge 设置页控制；
docs 同步更新。

### R4 运维风险文档化（geosite.dat 硬依赖 + fake-ip 语义）

nameserver-policy 的 `geosite:cn` / `geosite:private` 键使 mihomo 对 geosite.dat
形成硬依赖：全新安装 + 离线首启会导致配置解析失败（Verge 表现为校验不通过、
首次启动回退默认最小配置）。在 troubleshooting.md 增加定位条目；
dns-and-leak-model.md 精确化 fake-ip 下 nameserver-policy 的真实生效时机
（多数 AI 域解析实际发生在家宽 SOCKS5 服务器侧，socks5 支持域名直传）。

### R5 版本与文档元数据同步

- `SCRIPT_VERSION` / package.json version 升至 5.7.0；
- 文件头注释补 v5.7 变更摘要；
- README.md、docs/routing-scope.md 反映域名变更与证据。

## 明确不做（记录取舍）

- 不为 `downloads.claude.ai` 做域内下载排除：托管规则清理模型是精确串匹配，
  引用动态 upstream 名的规则会破坏该模型或误删用户规则；更新下载频率低。
  在 routing-scope.md 记录取舍。
- 不改 DNS 架构（`+.` 通配、DoH fragment、respect-rules 语义均经源码核实正确）。
- 不为 geosite 引入开关（默认值两难，风险场景集中且可文档化）。
- 本地 TOML 数组值（PROFILE_UPSTREAM_OVERRIDES）支持属功能增强，建议另开任务。
- 私有 DIRECT 规则前置覆盖用户私网代理意图：fail-closed 设计取舍，
  在 troubleshooting.md 记录。

## Acceptance Criteria

- [ ] 新增/变更域名各有正向路由测试；`x.ai`、`marketplace.cursorapi.com`、
      `www.anthropic.com` 等仍不走家宽（负向）。
- [ ] 旧版本生成的 `DOMAIN,api.openai.com,AI-家宽` 在升级后仍被托管清理
      （幂等迁移），`DOMAIN-SUFFIX,api.openai.com,AI-家宽` 只注入一次。
- [ ] 用户组引用被清除时输出 warn（含组名与被删引用）。
- [ ] `npm run ci` 全绿（check + 全部测试 + secrets 扫描）。
- [ ] README / routing-scope / dns-and-leak-model / troubleshooting /
      configuration 与实现一致；版本号 5.7.0。
- [ ] 公开模板仍只含 `xxx` 占位符，真实凭据不进入任何可提交文件。

## Notes

- 域名证据与"不加"清单见 `research/ai-endpoint-domains.md`；
  Mihomo/Verge 语义结论见 `research/mihomo-dns-semantics.md`、
  `research/clash-verge-rev-host-behavior.md`；
  代码层发现见 `research/static-code-analysis.md`。

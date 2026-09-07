# JS 分流可控性与固定出口约束优化

## Goal

在现有“家宽 / 原 Profile”两条路径内，让用户能够完整控制各核心 AI 服务是否进入家宽，并确保脚本管理的家宽组不能混入其他出口。借鉴 personal-edge-proxy 的职责分离与分服务策略，保留本项目已有窄域名边界。

## Authority and scope

- 用户原始要求：学习参考 GitHub 项目的流量分配方式，深入分析当前 JS，并创建优化 Trellis 任务。
- 用户本轮明确选择：“先优化现有两条路径（推荐）：改动小，不要求新增节点或部署 WARP”。
- 2026-09-07 初始轮完成研究和规划；随后用户明确要求“开始按照trellis任务规划实施”，现已授权按本规划实施、验证和正常 Trellis 收尾。真实 Profile、凭据、部署和远程发布仍不在本次范围内。
- 一个任务完成：出口组约束、核心开关及必要的 renderer/审计映射/文档联动。无需父子任务或独立平台。

## Confirmed background

| 发现 | 当前证据 | 问题性质 |
|---|---|---|
| F1：已有家宽组的 use/include-all* 会被继承 | clash-verge-ai-residential.js:782、:1652；合成 main 输出已复现 | P1，固定出口配置约束缺口 |
| F2：Claude、Gemini Developer API、Antigravity 部分核心域没有开关 | clash-verge-ai-residential.js:218、:247、:1266、:1283 | P2，能力缺口，不是旧 Web 开关失效 |
| F3：关闭核心域名后，启用的进程/IP兜底可重新捕获 | clash-verge-ai-residential.js:1398、:1428、:1436 | P2，需要明确专属兜底服从关系 |
| F4：正则业务路由没有等价 nameserver-policy | clash-verge-ai-residential.js:1303、:1544；docs/dns-and-leak-model.md:27 | P2，已有保证表述超过配置证据 |
| F5：开关与统计映射分散 | scripts/sync-local-config.js:24；skills/residential-rule-tuning/scripts/build-inputs.js:8；tests/install-agent-skills.test.js:151 | 必须同步接线，避免新开关缺失归属 |

外部固定版本、比较结论、完整证据与不采纳项见 [研究报告](research/routing-analysis.md)；复现与测试矩阵见 [验证记录](research/validation.md)。

## Requirements

- **R1 / 固定出口**：已有同名组包含可改变单一出口的额外配置时，应在保留名校验处清楚拒绝，保持原输入不变；成功输出的家宽组只能选择家宽 SOCKS5，不能继承额外节点来源、排除条件或备用出口。保留合法的 icon/hidden 展示信息。
- **R2 / 核心控制**：新增 `routing.anthropic_core`、`routing.gemini_api_core`、`routing.antigravity_core`，均默认 true；只对当前已有核心域分组，不新增/删除域名。开关关闭时移除相应脚本托管域名规则和 exact/suffix DNS，剩余流量交回原 Profile。
- **R3 / 专属兜底**：服务 core=false 时不生成该服务专属进程兜底；`anthropic_core=false` 时不生成 Anthropic CIDR 回退。已有认证、辅助、资源和全局捕获开关继续独立；core 不是绕过所有显式规则的禁用总闸。
- **R4 / 联动完整**：公开模板、TOML 示例、渲染映射、Node 导出、规则审计输入、相关测试和中英文文档保持一致，保留现有用户值与托管清理语义。
- **R5 / 证据一致**：修正文档对关闭开关、正则 DNS、固定出口和收益的表述；保留正则 DNS 现状为显式例外。默认不改变家宽覆盖和出口，无实际数据不承诺节省量。

## Acceptance Criteria

以下为实施验收；只根据已完成的验证勾选结果。

- [x] **AC1 → R1**：V2/V3 对 use、三种 include-all、排除/替代选择字段产生明确异常，原输入不变；合法组与 icon/hidden 被正确保留，V1/V10 二次执行通过。
- [x] **AC2 → R2**：三个新开关分别 true→false→true 的 V4 通过；当前旧版本生成的对应规则/DNS在关闭后被清理，其他服务不受影响；不增加宽泛域名或 DNS 后缀。
- [x] **AC3 → R3**：V5/V6/V7/V8 通过，验证 IP、各产品进程、Cursor 双开关、独立辅助开关以及私网优先的完整矩阵。
- [x] **AC4 → R2/R4**：V9/V10 证明原 Profile 规则顺序、未知用户规则与输入对象所有权保留；默认域名集合/目标、DNS和优先级层次保持一致。允许同目标且不重叠条目的内部顺序变化。
- [x] **AC5 → R4**：V11/V12 通过；三个新 boolean 的解析、缺失默认补全、重复键/类型错误均有测试；routing 总数 24，supported 12，unsupported 12，新域名可正确归属。
- [x] **AC6 → R5**：V13 和文档审查通过，清楚区分“回原 Profile”与“强制机场”、exact/suffix 与正则 DNS 覆盖、节点结构与真实运行保证；禁止虚构收益。变更记入 CHANGELOG Unreleased，版本/发布边界按 design.md 第8节执行。
- [x] **AC7 → R1–R5**：相关 Node 测试、`npm run ci`、实际 `just ci`、`git diff --check` 通过；文档修改按 implement.md 检查。真实宿主验收另列状态，不以 Node 通过代替。

## Out of scope and deferred evidence

- 不接入 WARP/Xray/HY2/REALITY、不新增服务器/依赖/服务选择器组，不改变机场上游解析算法，不加故障自动切换或全局 UDP 阻断。
- 不扩大或重新审计全部域名，不默认关闭现有服务，不删除用户自定义家宽规则，不改 ResiWatch Rust/数据库/API/UI。
- 不新增 regex→DNS provider 机制。正则专属同出口 DNS、UDP 全路径、真实固定 IP及节省字节暂为 UNVERIFIED；本任务不宣称已解决。
- 不读写本地凭据，不自动运行 render-local、更换当前 Profile 或进行现场故障注入。全部关闭核心后仍保留现有家宽节点配置要求，避免另加全局禁用模式和破坏用户规则引用。

## Planning status

技术方案按用户选定范围收敛，独立审查无阻塞；用户已在后续消息明确批准实施。

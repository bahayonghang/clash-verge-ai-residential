# PRD：家宽地址流量归因分析与按服务细分

## 目标与用户价值

概览页 TOP 主机第 1 名是家宽地址 `89.42.81.110`（83.2 MiB），用户认为「所有流量统计都打到家宽地址里」，希望把 grok、cursor 及脚本覆盖的其他服务（Claude / ChatGPT / Gemini / Antigravity 等）的流量细分出来。同时家宽页聚合区（累积用量、占比、TARGET 排名、趋势）全部显示「未知」/「无排名数据」，用户判断两组问题相关。本任务产出一份有证据的归因分析报告，回答三件事：

1. `89.42.81.110` 这一行里装的是什么流量。
2. 脚本覆盖的各服务流量如何细分，今天怎么做到、做不到的边界在哪。
3. 家宽页聚合区全「未知」的根因与修复路径。

## 症状（2026-08-25 09:57 概览截图）

| 维度 | 现象 |
|---|---|
| TOP 主机 | #1 `89.42.81.110` 83.2 MiB；#3 `cli-chat-proxy.grok.com` 47.1 MiB；#6 `openrouter.ai` 15.4 MiB |
| TOP 链路 | #3 `AI-家宽` 57.2 MiB（DIRECT 99.5、Proxy 88.6） |
| TOP 进程 | `Tabbit Browser.exe` 63.3 MiB、`grok.exe` 49.2 MiB、`qodersec.exe` 56.0 MiB |

## 已确认事实（代码证据）

1. 主机 identity 回退链为 `metadata.host` → `sniffHost` → `destination_ip`（`residential-monitor/src-tauri/src/session_host.rs:18-27`；`residential-monitor/docs/reporting.md` 报告口径）。连接无域名（host 空、嗅探未命中）时，主机维直接显示目的 IP。这是家宽地址以「主机行」出现的机制原因。
2. `89.42.81.110` 行 83.2 MiB > `AI-家宽` 链路 57.2 MiB，所以该行不可能是规则路由进 AI-家宽 的流量的聚合；它只能是「目的地址就是家宽 IP 且无域名」的连接。
3. 规则路由的 AI 流量在主机维天然可见域名（截图 #3 `cli-chat-proxy.grok.com`、#6 `openrouter.ai`），与事实 2 相互印证。
4. 链路维下钻支持 chain → rule / host（`residential-monitor/src/format/rank.ts:112-123`）：链路页选中 `AI-家宽` → 按主机细分，规则路由部分的按服务（域名）细分今天就能做。
5. 家宽页实时段用筛选口径（`residentialOnly`），聚合段与报告段 `grouping: "category"`（家宽/机场两桶），家宽页不提供按服务细分（`residential/aggregate-section.tsx:31`、`residential/report-section.tsx:26`）。
6. 目的地为家宽 IP 的隧道流量，真实目标域名在隧道内加密，Mihomo 层不可知；该部分最多按进程维细分。

## 现场验证结论（2026-08-25）

- H1 证实：`89.42.81.110` 行 = AdsPower SunBrowser（指纹浏览器）profile 直连家宽 SOCKS5 `:12324` 的隧道流量。netstat 反查 PID 65384 = SunBrowser.exe；24 小时库内 25,529 条会话、6.34 GB。H2 未观察到独立直连服务。
- 家宽页「未知」为双缺陷叠加：target 配置 `家宽` 与核算口径精确匹配零命中；断连期逐秒写入的未闭合 gap 行令覆盖计算归零。
- 详细证据、数字与修复路径见 `research/findings.md`。

## 需求

- R1 判定 `89.42.81.110` 主机行的流量构成（进程 × 规则 × 链路），每条结论附脱敏连接证据。
- R2 给出按服务细分的可复现操作结论：
  - 规则路由（AI-家宽）部分：现有「链路 → 按主机下钻」的 UI 操作路径，实测一次 grok / cursor 的输出。
  - 直连家宽 IP 部分：列出涉及的进程；说明域名级细分在 Mihomo 层不可行的原因（事实 6），给出替代口径（进程维）与根治方向（改由 Mihomo 域名规则接管，脚本已覆盖 grok.com / Cursor 官方主机，见 `docs/routing-scope.md`）。
- R3 产品改进建议单列成节（如家宽页按服务视图、家宽 IP 主机行标注），只立项不实施。
- R4 判定家宽页聚合区全「未知」的根因，量化两个缺陷各自的影响，并给出用户侧与代码侧修复路径。

## 验收标准

- A1 任务 `research/` 下有分析报告，回答「83.2 MiB 是什么流量」，结论逐条附脱敏证据（连接记录或下钻结果）。
- A2 报告含 grok / cursor 细分的 UI 操作步骤，且在本机实测通过一次。
- A3 对直连家宽 IP、无域名部分，明确 Mihomo 层可细分边界：进程级可行、域名级不可行及原因。
- A4 产品改进建议（若有）独立成节，不与分析结论混写。
- A5 报告给出家宽页两个缺陷的精确证据（核算口径 332,731 条会话 `primary_category_id` 全 NULL；29,187 条未闭合 gap 行，29,187 × 1,800 = 52,536,600 与概览缺口显示一致）。
- A6 报告覆盖脚本全部默认激活服务的 24 小时细分表，且各服务合计与 AI-家宽 链路总量闭合。

## Out of Scope

- 不改 `residential-monitor` 产品代码与 Clash 扩展脚本规则。
- 缺陷 B（coverage gap 行未闭合/不去重）的代码修复、数据目录迁移（`lib.rs:307-309` 默认 %TEMP%）另立任务。
- 不在本任务内实现任何新视图 / 新维度；确有需要另立任务。

## Open Questions

- 无阻塞项。

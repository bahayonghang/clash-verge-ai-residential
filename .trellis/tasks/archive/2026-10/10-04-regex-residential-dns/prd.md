# 补齐正则 AI 域名的住宅 DNS 策略

## Goal

使现有区域 Vertex 与可选 Cursor 仓库索引规则在 Mihomo 需要真实 DNS 查询时，具有与业务路由一致的住宅解析策略。保持业务域名范围和现有开关默认值不变。

## Background

父任务：`../10-04-one-exit-routing-optimization/`，对应父需求 R1/R3/R4。

`clash-verge-ai-residential.js:1392-1399,1483-1488` 注入正则业务规则，`:1649-1679` 只建立 exact/suffix DNS。`docs/dns-and-leak-model.md:32` 和 `.trellis/spec/frontend/quality-guidelines.md:170-171` 已承认该例外。父研究 `research/current-routing-evidence.md` 的 F1 复现了输出差异，严重性 P2；尚无用户环境 DNS 泄漏证据。

## Requirements

- **R1**：仅为现有 `VERTEX_AI_DOMAIN_REGEXES` 和 `CURSOR_REPOSITORY_INDEXING_DOMAIN_REGEXES` 建立住宅 DNS 覆盖。继续遵循 `vertex_ai_endpoints` / `cursor_repository_indexing`，不新增路由或开关。
- **R2**：保持严格 AI-only、私网与 bootstrap DNS、非 AI DNS、extra DNS 豁免及 exact/suffix 规则语义。
- **R3**：新增配置遵循保留名称、输入不变、重复执行幂等和开关撤销契约；不静默覆盖用户同名配置，不留下不可解析引用。
- **R4**：说明旧显式例外如何改变，并用真实内核验证正则 DNS 策略；无实际路径证据时不宣称全部 DNS 同出口。

## Acceptance Criteria

- [x] **AC1 → R1**：Vertex 开启时，us-central1-aiplatform.googleapis.com、europe-west4-aiplatform.googleapis.com 命中住宅 DNS；关闭后撤销。Cursor repo42.cursor.sh 仅在索引开关开启时命中；默认仍关闭。
- [x] **AC2 → R1/R2**：maps.googleapis.com、fonts.googleapis.com、storage.googleapis.com、repofoo.cursor.sh、repo42.cursor.sh.example.test、foo.us-central1-aiplatform.googleapis.com 均不被新增策略匹配。不存在 `+.googleapis.com`、`+.cursor.sh`、标签内星号或裸正则 policy 键。
- [x] **AC3 → R2**：业务规则逐项/顺序与原基线保持；AI exact/suffix、私网、国内、非 AI、bootstrap resolver 的语义与顺序保持。AnyRouter 的住宅 DNS 豁免保持；查找进程、进程路由、TUN/IPv6 不变。
- [x] **AC4 → R3**：测试 Vertex/Cursor 四种布尔组合、true→false→true、重复 main、原对象不变、托管旧输出清理、未知 provider/规则保留及同名冲突拒绝；保留未托管 DNS policy 的开关开启时也不保留旧托管 DNS 引用。
- [x] **AC5 → R1/R3**：本地渲染器针对现有两个开关的生成脚本，在虚构配置下输出相同门控结果；不编辑用户 local 文件或新增配置键。
- [x] **AC6 → R4**：记录实际 Mihomo 版本、配置检查与真实解析选路证据；至少覆盖 Vertex 正向/负向和 Cursor 开/关。fake-IP 缓存应答和普通出口探测不能替代真实查询。缺少环境记为 BLOCKED，停止完成声明。
- [x] **AC7 → R2/R4**：`just ci`、`just docs-build` 通过；只更新本任务命名文件和 CHANGELOG Unreleased，不变更版本、依赖、旧固定投影 fixture。

## Out of Scope

不收录新端点、不调整 regex 范围、不扩共享 DNS，不增加远程规则订阅或第三方依赖，不迁移 DNS 架构，不改账号认证策略、UDP/QUIC、ResiWatch、实际 local 配置或宿主设置。

## Sequencing and Approval

可独立实施；在回退防护子任务之后串行执行，避免共享文件冲突。2026-10-04 用户已批准按该顺序实施。内核验证是实施前置技术 gate；若机制不受支持，回到规划，不自动改用宽后缀或降低验收。

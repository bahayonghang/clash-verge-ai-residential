# 校验可达上游的住宅节点空组回退

## Goal

阻止脚本生成可达上游通过 `empty-fallback` 回指住宅出站的递归配置，避免过滤为空时住宅拨号再次选择自身。

## Background

父任务 `../10-04-one-exit-routing-optimization/`，对应父需求 R2/R3/R4。

参考配置 `ref/one-exit/clash-party/override.yaml:29` 使用 empty-fallback。当前脚本只清理 proxies 引用并过滤 include-all（`clash-verge-ai-residential.js:1093-1130`），可达遍历 `:1147-1224` 不检查 empty-fallback。父研究 F2 的顶层/嵌套虚构配置均成功返回住宅回指关系；严重性 P2。运行时异常未在真实内核复现，不宣称用户已经遭遇中断或泄漏。

## Requirements

- **R1**：当前选定上游的所有显式可达组，若 empty-fallback 引用脚本保留住宅出站，应明确拒绝生成配置。
- **R2**：不自动选择替代代理，不写 DIRECT/COMPATIBLE 回退；合法普通节点回退及无关配置保持。
- **R3**：沿用输入不变、错误不含凭据、幂等和现有组图校验契约；文档说明宿主在脚本失败后的边界。

## Acceptance Criteria

- [x] **AC1 → R1**：顶层 Proxy 与 Proxy→Nested 两种结构，在 empty-fallback 为 `家宽-SOCKS5` 时抛中文错误，错误含字段名、所在组及可达路径；输入对象与执行前完全相同。
- [x] **AC2 → R1**：同一检查同时拒绝 `AI-家宽` 保留名称，防止把脚本出站当作上游回退。该用例属于保留名称防护，不用其证明内核支持 fallback 指向代理组。
- [x] **AC3 → R2**：empty-fallback 为普通机场节点时保持原值；缺省字段不新增；同名字段位于不可达组时不因本检查失败；未知字段保留。
- [x] **AC4 → R2/R3**：无 fallback 的默认规则/DNS/组投影不变；既有 include-all 排除、显式引用清理、组循环、UDP 检查和重复执行测试通过。不扩展为全代理图审计。
- [x] **AC5 → R3**：`just ci`、`just docs-build` 通过，并记录脱敏实际 Profile/内核的空组行为核对。若缺少环境记 BLOCKED；不得把脚本抛错描述为操作系统级阻断。

## Out of Scope

不增加地区/延迟筛选、不更换上游解析优先级、不删除用户回退字段，不处理所有 provider 动态成员或任意代理 dialer 链，不改变内核缺省 COMPATIBLE 语义，不拒绝当前未选中的所有订阅组。

不变更域名规则、DNS 路径、UDP 策略、版本、依赖、真实 local 配置或宿主设置。

## Sequencing and Approval

无功能依赖；先实施本任务，随后执行 DNS 子任务。2026-10-04 用户已批准按该顺序实施，本任务已启动。提交、推送和真实 local 配置修改不在授权内。

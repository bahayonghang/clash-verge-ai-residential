# 扩展脚本失败不半改配置

## Goal

`main` 在全部校验通过之前不修改调用方传入的 `config`。失败时 Clash Verge 控制台能看到 `[AI-家宽]` 原因。

## Background

证据：`clash-verge-ai-residential.js:1684-1705` 先 `hardenAllIncludeAllGroups` / `hardenReachableUpstreamGraph`（就地改 `exclude-filter` / `proxies`），再 `validateHomeProxy` throw。`warn`/`info` 有包装，throw 不经 `console.error`。Verge UI 只显示 `Script execution failed`。

## Requirements

- R1 入口对 `proxies`、`proxy-groups`、`rules`、`dns`、`tun`、`sniffer` 做浅拷贝（组/规则数组浅拷贝到新数组；需要改的 group 对象再拷贝）。只返回校验全部通过后的对象。
- R2 校验失败 throw 前 `console.error` 同一条 `[AI-家宽]` 消息。
- R3 无安全上游、占位凭据、保留名冲突的既有 fail-closed 测试仍通过，并新增：throw 后调用方原 `config.proxy-groups` 与进入前 JSON 相等。

## Acceptance Criteria

- [ ] AC1 占位 `xxx` 凭据：throw，且输入 `config` 的 `proxy-groups`/`proxies` 与调用前 deep equal（相对被测字段）。
- [ ] AC2 成功路径仍注入 `AI-家宽` 与家宽节点；回归正/负域名样本不变。
- [ ] AC3 throw 路径在测试捕获的 `console.error` 中含 `[AI-家宽]`。
- [ ] AC4 `npm test` 中根脚本套件退出 0。

## Out of scope

- 把脚本拆成多文件。
- TOML 表达 `PROFILE_UPSTREAM_OVERRIDES`。
- `find-process-mode` 开关化。
- nameserver-policy 覆盖 DOMAIN-REGEX（记入母任务延期，不在本任务）。

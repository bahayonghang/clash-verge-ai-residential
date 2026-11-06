# 设计：仅兼容顶层 null，继续完整校验

状态：用户已批准并完成实现，未提交。需求以 [PRD](prd.md) 为准，实际验证见 [validation.md](research/validation.md)。

## 数据流与最小修改

```text
main(config[, profileName])
 → cloneConfigForEdit：在副本中将顶层 rule-providers === null 归为字段缺省
 → 原 validateReservedNameCollisions
 → 原 validateRegexDnsProviderOwnership
 → 原上游解析、节点/组/规则生成
 → 原 rebuildRegexDnsProvider / DNS 构建
 → 返回 working
```

在 `clash-verge-ai-residential.js:582-613` 的 cloneConfigForEdit 构造 working 后，严格判断 `working["rule-providers"] === null` 并删除副本顶层键，附中文注释说明宿主空值来源。不改变 config、全局 isPlainObject，不增加函数/导出/开关/依赖。

保留 validateRegexDnsProviderOwnership 的原判断和全部所有权/引用循环（当前 `:1645-1698`）。null 进入校验前已成为 undefined，其它非法值不变；不提前返回 main，不 catch 后返回原配置，不跳过校验。

## 输入合同

| 输入 | 预期 |
| --- | --- |
| 字段缺省/undefined | 既有允许行为不变 |
| 顶层 null | 副本移除该键，完整输出等于缺省输入 |
| 显式 {} | 原语义不变，全关仍保留显式空对象 |
| 普通用户映射 | 原校验和克隆重建流程保留用户项 |
| 顶层 []、字符串、数字、布尔值 | 原类型错误，含 false/0/空串 |
| 保留名称下的 null 或非规范 provider | 原所有权错误；嵌套 null 不能被顶层例外吞掉 |
| 顶层 null + 专用 provider 非法引用 | 仍经过引用校验并拒绝 |

规范托管 provider 的字段、inline/classical、已知唯一非空 payload 不放宽；rules、逻辑嵌套、sub-rules、复合 policy、filter、sniffer 检查保留。canonical 托管 DNS policy rerun 仍允许。

## 取舍

选择删除副本键，使全关不残留 null/新增空映射，无需更改 rebuildRegexDnsProvider（`:1701-1713`）。null → {} 会在全关时留下显式空对象；只放宽校验会被 builder 的提前返回留下 null。两者都需额外处理。

不使用 `value || {}`，避免吞掉 false/0/空串；不按客户端版本检测，相同空值输入应有相同语义。不改其它字段的 null 处理，不扩大为通用规范化框架。

## 五文件边界

| 文件 | 计划变化 | 需求 |
| --- | --- | --- |
| clash-verge-ai-residential.js | clone 阶段严格 null 归一、中文注释；原校验/构建保留 | R1–R4 |
| tests/regression.test.js | null 正例、完整等价/不变/幂等；外层拒绝矩阵移走 null、补 falsy；内层 null 保留；null + 引用冲突 | R1–R4 |
| tests/sync-local-config.test.js | 现有正则 DNS 子进程 probe 用 null 和单参数 main，断言输入不变；不改 renderer 源码 | R5 |
| .trellis/spec/frontend/state-management.md | 原非对象拒绝条款的顶层 null 窄例外及不变/冲突边界，英文编写 | R1–R4 |
| CHANGELOG.md | 未发布区中文修复条目；不升版本、不声称完整客户端支持 | R5 |

现有拒绝矩阵中的 invalidProviders 首项 null 是“同名 provider 内容为 null”，不能随外层 null 一起删掉。renderer 使用虚构临时配置，不运行真实 render-local。

## 验收机制映射

| 验收 | 机制与验证 |
| --- | --- |
| F-AC1 | clone 归一；四开关组合完整输出等价，含单参数 main、原代理保留及家宽生成 |
| F-AC2 | 原判断不改；falsy/其它非法值负例与内层 null/非规范 provider 原矩阵 |
| F-AC3 | 无提前返回；null 顶层映射重用冲突矩阵，保留用户项正例、规范 rerun |
| F-AC4 | 只改 working；成功/失败快照、幂等、provider/policy 与全关输出断言 |
| F-AC5 | 现有五组 renderer 开关/default 场景，生成脚本 null + 单参数，保留模板/TOML 不变 |
| F-AC6 | CLASH_SCRIPT_PATH 选择独立旧模板做红测，不回退工作区；窄测试/根 CI/just ci 分层记录 |
| F-AC7 | 同一脱敏订阅、同一模式在 0.9.1 应用，实际无目标错误且配置/代理组有效；无实机条件则 NOT_RUN |

## 风险、交付与回退

null 修复仅消除这一处错误；出现后续上游/凭据/Mihomo 错误时保留真实失败，不能吞错制造“无红框”。其它脚本失败时 FlClash 应用空配置的机制不变。实际输入仍未采集，本方案针对已获准的 null 兼容；profileName 覆盖差异排除，但 main(config) 调用必须测试。

无 schema/依赖迁移。客户端已粘贴的旧脚本不会自动更新：实现交付时提供更新说明；额外明确授权前，不替换用户客户端脚本或重新生成真实 local 文件。

新脚本应用失败时取消测试订阅脚本扩展、重新应用原订阅恢复基线并记录错误。代码只回退本任务五文件自己的改动，保留他人编辑；不自动 reset、清数据或删订阅。

# FlClash null 修复：独立复核

日期：2026-10-19（America/New_York）。范围为已批准的五个产品/合同文件及本任务证据；不提交、不操作私有配置。已读取 PRD、design、implement、check.jsonl 所列 spec/研究和实现证据，并核对最终 diff 与实际调用链。

## Findings (fixed)

无。未发现需要修复的产品或测试问题；本角色未修改五文件实现。

## Findings (not fixed)

无代码阻断项。完整 `just ci` 尚在主会话运行，不能把根 `npm run ci` 当作完整产品门已通过。

## 审阅依据

- `cloneConfigForEdit` 仅对工作副本的顶层 `rule-providers === null` 删除该键；原输入字段保留。`isPlainObject`、provider 校验器、重建器、主流程校验顺序均无变化，没有吞错、提前返回或新依赖。
- null 正例覆盖四种 Vertex/Cursor 组合、实际单参数及双参数入口、完整输出与缺省输入等价、原节点/组/规则保留、家宽节点/组和 DNS payload/policy、成功输入不变及重复执行稳定；全关无映射，显式空对象既有测试保留。
- 类型负例只撤掉顶层 null；数组、空/非空字符串、零/非零数字、两种布尔值和保留 provider 名称下的 null/非法字段/payload 继续拒绝。13 种引用冲突分别在 null/规范映射和开关全开/全关下拒绝，检查失败输入不变及虚构凭据不泄露。普通用户 provider 与相似名称正例保留。
- renderer 五种开关/default 场景在子进程以 null 和单参数 main 执行，保留规则/DNS 断言，并检查输入、执行前后临时 TOML 和公开模板不变。未改 renderer 源码。
- state-management 合同与实现一致；CHANGELOG 仅描述本处兼容，不声明完整客户端支持。产品 diff 仅为批准的五文件，无依赖/锁文件/路由变化。

## Verification

- Lint / syntax：PASS，依据实现角色最终 `npm run ci` 的语法/agent-contract/secret 检查；根扩展无独立 linter。主会话报告 monitor frontend check 和 Rust fmt 已通过。
- TypeCheck：PASS，依据主会话 `just ci` 内 monitor frontend check；根扩展为普通 JavaScript，无静态 TypeScript 门。
- Tests：PASS，依据持久化实施证据：定向 16/16、renderer 26/26、最终根 CI 259/259；独立旧公开模板对新增四项 null 正例两次 RED，失败均为目标类型错误；新模板 GREEN。未为独立审阅重复启动测试。
- `git diff --check`：PASS，本角色直接执行，退出码 0。
- 完整产品门：PENDING，主会话负责最终结果；本复核时 `just ci` 正在 Rust clippy 依赖编译阶段，无已知错误。
- 原生：主会话报告实际 FlClash 新脚本应用成功、原组与 AI-家宽 可见，运行配置原节点/组保留并新增家宽节点/组。本角色未操作客户端。实际失败时的 null 输入尚未现场采集；原生成功应用与确定性 null 回归应分别表述，不以旧订阅 raw JSON 快照证明本次目标运行输入。

结论：五文件实现与批准方案一致，未发现校验退化。完整质量门结果由主会话补充后才可宣称全部门禁通过；本结论不证明住宅连通、公网 DoH 或实际出口。

## 主会话最终门禁补充

独立复核后 just ci 已 exit 0：monitor 前端 300 tests、Rust 553 passed/6 ignored、root 259 passed，fmt/clippy/secret scan PASS。本机 0.9.1+2026101702 应用通过，见 validation.md。独立 source-map-js high 审计告警仍在，monitor npm audit exit 1，未混入本任务依赖修改。

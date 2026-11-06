# 执行计划与结果（用户已批准，未提交）

用户已批准本轮 PRD/design/implement，已激活任务并派发实现和 check。下列原计划保留验收要求，实际完成结果见 research/validation.md。本任务是一个兼容修复交付，无需父子拆分。

## 1. 实施前

- [ ] 核对最新批准、任务状态、Git 状态；重核设计行号及夹具，保护他人改动。
- [ ] 五文件白名单以 design.md 为准；额外产品文件/行为先回规划，不由 implement/check 扩权。
- [ ] 激活后按 Trellis 路由派发实现，提示首行含 active task，不递归派发 implement/check。两个 JSONL 加载 spec/research，PRD/design/本计划随任务加载。

## 2. 先建立失败回归

- [ ] tests/regression.test.js 正则 DNS 区域增加名称包含 `FlClash null 规则集` 的正例，复用 baselineProfile/quietMainWith/withPatchedSwitches。
- [ ] 四组 Vertex/Cursor 开关分别验证 null 与缺省完整输出相等、原节点/组/普通规则保留、家宽节点/组生成、payload/policy 正确、输入不变、幂等、全关无映射；含单参数 main(config)。
- [ ] 产品修复前将公开模板复制到系统临时目录保留旧快照；不复制 local 脚本，不 reset/stash/revert 共享工作区。独立进程设置 CLASH_SCRIPT_PATH 指向旧快照，对新 null 正例运行红测，记录非零退出及目标类型错误。
- [ ] 若失败来自路径、凭据或上游，修正脱敏夹具，不能算 null 红测；不能依赖其它旧拒绝断言的失败凑红。

红测命令（独立进程设置 CLASH_SCRIPT_PATH 后，结束恢复环境）：

```powershell
node --test --test-name-pattern='FlClash null 规则集' tests/regression.test.js
```

## 3. 实现与校验保留

- [ ] clone 构造 working 后严格 === null 删除该副本键，写中文原因注释；不更改 isPlainObject 或校验器。
- [ ] 外层非法矩阵仅移除顶层 null，补 false/true/0/空串；保留数组/非空串/非零数和所有同名 provider 错误，尤其内层 null。
- [ ] 现有 rules/sub-rules/policy/filter/sniffer 冲突夹具补 null 顶层输入；开关全关仍拒绝、输入不变、错误不泄露。保留普通 provider/相似名正例和规范托管 rerun。
- [ ] renderer 现有正则 DNS 集成 probe 使用 null 与单参数 main，保留已有五组开关/default；补子进程输入不变及临时 TOML 内容不变断言，保留公开模板不变检查。不改 renderer 源码，不运行真实 render-local。
- [ ] 清除旧快照环境覆盖，对修复模板运行同一正例为绿；再对旧快照运行仍红。记录旧/新模板 SHA 与结果。

## 4. 合同同步

- [ ] state-management.md 原条款补顶层 null 在 clone 中归为缺省；明确内层 null/其它非法类型、所有权/引用校验和输入不变，不改其它合同。
- [ ] CHANGELOG.md 未发布区中文修复条目，不升版本或扩大客户端支持声明。
- [ ] research/repro.cjs 是旧行为诊断记录，不用作本次绿测；它读取当前模板，修复后旧 null 拒绝断言预期会失败。永久回归以 tests/ 新用例为准，历史研究不改写成修复证据。

## 5. 实现后验证

```powershell
node --test --test-name-pattern='FlClash null 规则集|正则住宅 DNS' tests/regression.test.js
node --test tests/sync-local-config.test.js
npm run ci
just ci
git diff --check
```

- [ ] Node 18+、零依赖、项目语言/风格与 R1–R5/F-AC1–F-AC6 全部检查。
- [ ] just ci 为仓库质量门，含 monitor-check 和根 CI，npm run ci 不能替代；前一次根 CI 可作快速失败定位。工具/网络/缓存问题单列阻塞，不降门槛或无关修复。不改 docs/依赖/锁文件，无新增 docs-build/dependency-audit 要求。
- [ ] 按工作流派发独立 check，只在获批范围自修；最终 diff/Git 状态仅五文件与任务资料。

## 6. FlClash 0.9.1 正常窗口验收

- [ ] 优先隔离、脱敏测试订阅和虚构节点；先发现实际环境，不假定安装位置。替换真实客户端脚本须额外明确操作授权，不能自动改用户当前订阅。
- [ ] 同一订阅、同一模式对比：测试输入 null 能复现旧类型错误；新脚本不再报目标错，配置生成/应用成功、原分组和家宽组存在。只记录版本、字段类型与脱敏结果，不保存订阅 URL/凭据。
- [ ] 规则模式下“代理”入口恢复是配置成功的附带观察，不改导航，不用切全局模式掩盖问题，不以只无 toast 作为通过。
- [ ] 测试后恢复原测试状态。缺少实机环境或操作授权时 F-AC7=NOT_RUN，提供人工步骤，结论为代码验证完成、实机待验。虚构节点不证明住宅连通、公网 DoH 或实际出口。

## 7. 收尾

- [ ] 报告修改、红绿回归、各质量门、原生状态及独立剩余错误；实现批准不自动授权 commit/push/替换已安装脚本。
- [ ] 校验退化则暂停交付，在五文件内修正后重跑相关检查；不删除断言、不吞错。
- [ ] 回退只撤销本任务改动，保留他人工作和研究；客户端取消测试脚本后重应用原订阅，不清数据。

## 本轮规划自检

- [x] 核心目标为红框 null 失败；代理入口仅附带验证。
- [x] 顶层 null 与内层 null/其它 falsy 值区别明确。
- [x] 每项需求有设计机制和验收，原调查 AC 已保留。
- [x] 文件范围、质量门、实机 NOT_RUN、部署/提交授权边界明确。
- [x] PRD 已整理，真实上下文清单已准备。
- [x] 用户批准本轮方案，已激活并实施验证；明确不提交。

## 实际完成记录

- [x] 五文件实现及合同同步完成；独立 check 未发现问题。
- [x] 独立旧模板 4 项 RED、新模板窄测试 16 项 GREEN、renderer 26 项、root 259 项通过；完整 just ci 前端 300/Rust 553 通过（6 ignored），fmt/clippy/secret scan PASS。
- [x] 最新用户授权允许处理本机：备份原脚本后只补同一 3 行，原报错订阅实际应用成功；规则模式原组和 AI-家宽 可见，73/11 原节点/组保留并新增各一。
- [x] 未故意在旧脚本下再次打断正常代理，旧失败依据用户截图和独立红测。实际失败参数未现场采集，不将遗留另一订阅 config.json 作为该参数证据。
- [x] 当前保留修复后的目标订阅和代理页供使用；原脚本及先前选择有本地回退记录，未清数据或删除订阅。
- [x] 无需故障回退；未执行公网 DoH、住宅出口/UDP、Node 多版本验证。原计划6/7的条件项以这些实际结果为准。
- [x] 独立 source-map-js high 告警已记录，未升级依赖；未提交、未推送、未自动归档。会话记录脚本被策略拦截，未运行，结果保存于本任务资料。

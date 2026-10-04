# T04 / T06 基本运行恢复独立审查

日期：2026-10-02，America/Chicago。状态：**REVIEW_PASS / PREPARED_NOT_RUN / STOP_WRITING**。

本轮未发现需要修复的代码问题。T04 公开模板路线已通过一次真实隔离初始化及独立合同检查。T06 捕获与 ACP 准备通过独立合成回归，具备主会话冻结新候选的条件。五客户端实际基本读取由主会话冻结后执行；本报告没有启动模型客户端。

## Findings (fixed)

无代码缺陷修复。检查代理仅新增本目录下审查脚本与收据，没有修改产品、模板、本机覆盖或任务元数据。

## Findings (not fixed)

无本轮代码审查遗留问题。T05 原正式性能门、T06 完整模型/权限/hook/extension/child 证据、hosted 同 SHA、Windows WebView、托盘、真实控制器、凭据及长期运行证据仍独立待验。基本恢复结果不补全上述验收。

## Verification

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| T04 三组 Node 定向 fixture | 79/79 PASS | focused-fixtures.stdout.raw.log、receipt.json |
| T06 合成回归 | 51/51 PASS，native exit 0；模型客户端与候选冻结各 0 | t06-synthetic.stdout.raw.log、receipt.json |
| Lint | PASS，monitor eslint 与 Rust clippy 在实际 just ci 中通过 | just-ci.stdout.raw.log、stderr.raw.log、receipt.json |
| TypeCheck | PASS，monitor TypeScript 检查在实际 just ci 中通过 | just-ci 收据 |
| 全产品 just ci | native exit 0 | 前端 73 个测试文件、300 项；Rust 546 项及独立进程 3 项通过，6 项 ignored；根 227/227 |
| 最终 docs-build | native exit 0 | docs-build.receipt.json |
| 独立秘密扫描 | native exit 0 | secret-scan.receipt.json |
| 安装态 skill 校验 | native exit 0 | skills-check.receipt.json |
| 父任务、T04、T06 task validate | 各 native exit 0 | validate-*.receipt.json |
| 已跟踪 diff 空白 | native exit 0 | diff-check.receipt.json |
| 10 份新增源码空白 | PASS | new-source-whitespace.json 保留首轮 LF→CRLF 提示；new-source-whitespace-retest.json 仅对该 Git 进程关闭换行转换，检查规则与文件字节未变 |
| 四本机覆盖 ignored | native exit 0 | local-overrides-ignore.receipt.json |

dependency-audit 本轮未运行：没有修改依赖、锁文件或审计门。原 T01 安全验收不迁移为本轮新证据。首次准备阶段的断言错误与 no-index 判读错误仍保留在 T06 原准备目录，本轮没有覆盖。

全产品 driver 与 outer 实际退出均为 0，见 full-outer-observation.json；没有以最后一条命令覆盖前面失败。docs-build 在主会话最后文档回写后启动，使用最终内容。

## 实现核对

T04 核对 scripts/bootstrap-harnesses.js、四个 scripts/harness-templates/ 模板、tests/bootstrap-harnesses.test.js、package.json、justfile、harnesses 文档和质量规范。目标限定系统 Temp 的 trellis- 目录，拒绝现工作树、祖先、非白名单旧资产、junction、符号链接及硬链接。固定 JS/EXE 入口通过 argv 执行，shell=false；版本与 init 使用同一入口，版本失败在文件部署前退出。67 项公开最低合同白名单复制，现有四覆盖逐字保留，缺失项使用公开模板。原 checker 没有放宽，部署前后均运行。模板包含实际授权、task 路径冲突处理、Kimi built-in coder、禁止递归与 Codex V1/V2 适用说明，没有默认无条件自修。

T06 核对 runtime-capture-operational.py、operational-kimi-acp.py、prompt、verification、新绑定及同版本公开源码证据。原捕获器、旧绑定和历史候选不修改。新 wrapper 在模块内切换到 operational-bindings.json 并纳入 ASSETS；Codex 当前入口的版本记录为 0.160.0，旧 0.159.2 证据保持历史身份。secret/credentialId 识别、敏感停止、UTF-8、prompt 单参数、无 GO 不启动、独立退出与自有 Job 清理边界保留。ACP 权限请求始终取消并停止，文件回调只允许 AGENTS.md/CONTEXT.md。

Kimi 2.0.0 同一 exe 的 setMode 源码进入 plan、设置 manual、更新 mode 并发 current_mode_update，setSessionMode 等待 setMode。driver 在该响应与本次 plan 通知均出现后才发送 prompt，支持两种到达顺序。另核对公开 bundle 的 emit(notification) byte offset 105701276 和 activateSession byte offset 105724867：mode 通知为 best-effort；available_commands_update 用 setTimeout(0) 安排在新会话响应后。传输或确认缺失仍记录阻断，不降低 plan 条件。原生权限与协议是否实际成功尚待运行。

## 一次隔离 native 结果

已批准新目标：C:/Users/lyh/AppData/Local/Temp/trellis-t04-operational-review-_1wrp0je/candidate。执行前确认绝对路径在系统 Temp 内，候选不存在。所选入口为既有固定工具目录的 @mindfoldhq/trellis/bin/trellis.js，未获取或安装工具。

- CLI 版本 0.7.0-beta.3，version native exit 0。
- 同入口五平台 init native exit 0；bootstrap driver/outer 各 0。
- 原 checker 部署前后均 ok；候选自身 scripts/check-agent-contract.js 独立 native exit 0。
- 67 个公开文件、4 个新增默认覆盖 hash 前后一致，5 个必要平台资产均存在。
- 未复制本机覆盖。现有覆盖 CRLF 字节保留及混合场景由真实公开模板 fixture 验证；未为第二场景重复真实 init。

结果见 native-bootstrap-result.json、native-paths.json、native-bootstrap.receipt.json、candidate-contract-native.receipt.json 与 t04-outer-observation.json。该候选是最低合同复制范围，非完整 checkout，不证明客户端发现、hook 执行、原生子代理或产品安装态。首失败候选没有修改。

## 保护及冻结准入

T04 检查与 init 期间，四本机覆盖、当前 .template-hashes.json、Git index/HEAD/branch 及保护源码 hash 全部保持一致，见 preservation.json。全产品门开始后主会话完成了事先声明的 docs/agents/harnesses.md 与 .trellis/spec/frontend/quality-guidelines.md 最后回写；两者是 full-protected-before 与 final-protected 间仅有的保护文件变化。其余保护文件及 index/HEAD/branch 未变。未读取 local 配置、数据库、凭据或原生历史内容；本机覆盖仅计算已批准 hash。全局配置与 trust 只声明本轮未执行写命令，不声明全盘 hash 证明。

最终源码与绑定 hash 见 final-protected.json。新 capture SHA256 为 333242eb6af1ccc9ad89f0d1e3d81aff1ba77e2de01326312b19b700d8bb236c；ACP driver 为 bc0aad615491b106c3f26dfab66648a1da9170eb9079b2f35e2026db47544a8b。冻结必须使用最终内容，不能使用首次准备的 wrapper hash。

检查代理已完成授权检查并停止写入。主会话可冻结新 T06 基本读取候选，按精确 GO 逐工具一次执行。实际失败、敏感停止和超时保留各自收据；不自动重试、换模型、换账户或扩大权限。

# T04 隔离初始化入口实施收据

日期：2026-10-02，America/Chicago。状态：**IMPLEMENTED_FIXTURE_PASS / NATIVE_INIT_NOT_RUN**。

本轮派发明确指定 T04。原生 hook 注入 T06；主会话确认以 T04 fallback 实施。`task.py current --source` 返回 none。未修改任务指针。本轮直接执行已派发 implement 角色，未递归委派。

## 交付

- `scripts/bootstrap-harnesses.js`：零依赖 Node 18+ CommonJS 入口。调用必须显式提供 `--root` 与 `--entry`。候选须位于系统临时目录内、首个路径组件以 `trellis-` 开头；拒绝当前工作树及其祖先/子目录、链接、硬链接、私有/额外文件与已生成候选。拒绝 shell 包装器，固定 JS 使用当前 Node，EXE 使用 argv，`shell: false`。
- 版本与 init 使用同一个显式入口。不安装、升级、发现备用入口或更改 PATH。版本失败、不明确或不匹配时，目标目录不建立、模板不部署。
- 静态列出已审 67 项公开最低合同；只补齐缺失副本，已有合同须与公开来源一致。名单不包含本机覆盖、数据库、私有配置、runtime、workspace 或用户历史。该目录不是完整 fresh checkout，不用于执行全产品门。
- 四个完整公开模板在 `scripts/harness-templates/`。缺失覆盖独占创建；已有覆盖逐字保留。部署前后、init 后记录 SHA256。原 `checkContract` 部署后及 init 后分别执行；无效已有覆盖保持失败，不补 marker 或替换。
- Kimi 三角色说明原生支持 custom agents，项目固定采用 built-in coder + role skill。三个角色均核对非空 dispatch/current/injected 路径，冲突回主会话确认并 pull；不改共享指针。check 明确 read-only 禁止产品修复，self-fix 仅限已批准任务/文件。implement/check 不递归。research 仅写已确认任务 research，不读 implement/check manifest。
- Codex 保留真实默认 `project_doc_fallback_filenames` 与 `[agents] max_depth = 1`，说明 V1/V2 和提示词保证边界。默认模板没有 agent registry，未新增固定角色注册、模型、机器路径或用户 feature/trust 设置。
- `package.json` 仅接入新脚本/测试的语法和 test 列表。`justfile` 新增两个显式位置参数的 bootstrap recipe，以 Node script + argv 执行。

现有环境检查器、合同检查器、四个 ignored 本机覆盖未编辑。docs/spec 由主会话独立处理。共享文件中的其他既有脏改动保持原状。

## 检查

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| 两个新增 JS 的 node --check | PASS / exit 0 | 初次语法检查及 npm-check.log 的全语法接线 |
| bootstrap / environment / agent-contract 三套定向 node:test | PASS / 79，失败 0，跳过 0，exit 0 | tests.log |
| npm run check | PASS / exit 0 | npm-check.log |
| just 参数字面值与拒绝路径 | PASS / 预期 native exit 1 | just-safe-argv.log；保留空格、`;`、`&`、`$()`，relative root 拒绝，无 init |
| package.json / justfile git diff --check | PASS / exit 0 | scoped-diff-check.log |
| 8 个本轮所有权文件尾部空白与 SHA256 | PASS | owned-file-hashes.json |

Fixture 包含缺失/全部已有/混合覆盖、CRLF 字节保留、无效覆盖、不匹配/未知/失败版本、missing/invalid 项目版本、原生 init 非零/信号/超时/执行器异常、init 改写覆盖/公开合同、缺必要资产、完成候选二次调用无写入拒绝、非隔离/私有/已生成文件拒绝、junction/硬链接拒绝以及 argv 特殊字符。

初次 just PowerShell recipe 的实际失败保留：`Unexpected token 'bootstrap-harnesses' in expression or statement.`，native exit 1。首次改用 Node script 时未设置 positional-arguments，返回 `参数缺失或重复：--root`，exit 1。最终增加 `[positional-arguments]` 后传参通过；失败没有作为 bootstrap 成功。旧缺失覆盖初始化首失败证据不变。

`aggregate-wiring.diff` 是 package.json/justfile 相对 HEAD 的聚合 diff，包含本轮前已存在的改动；该文件不声称全部改动由本轮产生。

## 后续与边界

按照派发要求，**未运行真实 native init、just ci、just docs-build 或完整秘密扫描**，未提交、归档、安装或执行远端动作。主会话/独立 checker 可复用现存固定 JS 入口并准备新的 trellis- 临时候选，分别验收无本机覆盖和已有有效覆盖场景。候选生成后再次调用明确拒绝；重试须使用新候选。

初始化与本地合同通过不能补全 T06 的实际角色、模型、hook、权限与子代理生命周期，也不能替代 T05 正式性能门。Node 本机运行没有代替 hosted Node 18/20/22 或 Unix just 执行证明。该入口不是 OS 沙箱；边界依赖既有执行权限、受审公共输入和固定已核验 CLI。

代码停止写入，等待主会话独立检查。

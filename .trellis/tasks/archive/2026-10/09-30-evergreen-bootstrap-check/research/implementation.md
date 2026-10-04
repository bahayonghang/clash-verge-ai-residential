# T04 实施与验收证据

日期：2026-09-30。实施范围为已批准的只读诊断、fixture、说明回写和独立临时目录中的固定版本 bootstrap。T01/T03/T02 已有改动保持完整。

## 改动

- 新增 `scripts/check-harness-environment.js`：报告六工具的首选入口、版本、退出码与替代入口；读取项目版本和四覆盖哈希；分别给出总体结果、bootstrap 前提及未验证的信任状态。诊断没有工具获取或 init 功能。
- 新增 `tests/check-harness-environment.test.js`：19 个 fixture 覆盖 missing、进程/权限失败、版本未知、非零但版本匹配、warning 误取、版本不匹配、替代入口不覆盖首选故障、项目文件缺失、CLI 退出码，以及 Windows Unicode/单引号路径。
- `package.json` 仅新增语法与 fixture 列表；`justfile` 增加独立 `check-harness-environment`。CI 不依赖本机 harness 的可用性。
- `docs/agents/harnesses.md` 与 `docs/agents/residential-rule-tuning.md` 回写选择入口、版本门、隔离获取、四覆盖保护和业务 skill 独立交付流程，注明五工具适用范围。

## AC 对照

| AC | 结果 | 证据 |
| --- | --- | --- |
| AC1 | 默认 Codex npm 包装器退出 1；应用内替代入口可用。首选失败保持可见。版本/项目文件与 trust 状态分开报告 | `environment-default-after-review.json`；权限仍为 UNVERIFIED |
| AC2 | 当前 PATH Trellis 0.6.17 与项目 0.7.0-beta.3 不同，诊断退出 1、bootstrap=BLOCKED；不自动切换入口或升级 | 上述默认诊断、`review-focused.result.json` |
| AC3 | 固定隔离 CLI/core 均为 0.7.0-beta.3；同一绝对 Node/CLI 入口先通过版本门，再 init 退出 0；四覆盖 SHA256 相同 | `resolved-executor.json`、`bootstrap-result.json`、`isolated-init.log` |
| AC4 | 五平台必要资产及 Claude settings、OMP extension 存在；当前与候选目录各 8 个生成路径 ignored、4 个覆盖未 ignored；当前保护清单哈希不变 | `bootstrap-result.json`、`ignore-results.json`、`protected-before.json` |
| AC5 | 失败、未知和版本不匹配由 fixture 阻断。默认环境仍 BLOCKED；隔离通过只属于显式选定入口 | `environment-default-after-review.json`、聚焦测试与同入口 init 收据 |

本次未运行全局安装或升级、修改 PATH/trust/权限、应用安装，也未打开或写入私有 TOML/JS、真实数据库或凭据。当前保护清单是本仓库公开文件的哈希核对；没有为证明未写入而读取全局秘密或私有文件。

## 隔离执行器

路径见 `isolated-paths.json`：工具、npm cache、候选项目位于本任务新建临时根的不同子目录。使用 `--global=false --ignore-scripts`、固定包版本和两个独立空 npm 配置文件。保留 `isolated-package-lock.json`、完整 `isolated-dependency-tree.json`，并核对 8 个直接依赖的实际解析路径和版本。

首次获取因 userconfig/globalconfig 指向同一个临时文件而被 npm 拒绝，退出 1；改用两个文件后获取退出 0，安装 58 个包。原始失败保存在 `isolated-install-attempt1.log`。首轮 argv 收据中的可变列表引用曾被重试值覆盖，已按实际命令和失败日志修正，原因写入 `record_correction`。没有重复获取已成功的包。

隔离 init 使用 `--claude --codex --grok --kimi --omp --skip-existing -y`。当前仓库没有运行 init，候选 `.template-hashes.json` 没有复制回来。候选目录在 init 后仅执行本地 `git init` 以检查忽略规则；没有创建提交。临时目录保留供证据复核。

## 检查与修正

| 检查 | 结果 |
| --- | --- |
| 首批语法检查与 fixture | 0；17 通过 |
| 独立审查后的 fixture | 0；19 通过、0 skipped |
| 最终默认真实环境诊断 | 1；预期 BLOCKED，故障未被替代入口成功覆盖 |
| 固定执行器版本门 / 隔离 init / 实际依赖树 | 各 0 |
| `just ci` | 0；Node 201；Vitest 73 文件、300 测试；Rust 542 单元＋3 kill_gate 集成通过、6 ignored、doc tests 0 |
| `just docs-build` | 0；使用已有 docs 依赖 |

独立审查复现 Windows PowerShell 默认编码损坏 Unicode 入口路径。修正只影响诊断子进程输出编码，并加入原生 fixture；单引号路径与非零退出均验证通过。原始失败/修正证据分别在 `review-discovery-before.json`、`review-discovery-after.json`。显式路径同时经 `path.resolve` 归一化，避免同一路径的分隔符差异重复探测。

正式门在 13:30:34–13:31:42 UTC 完成。`formal-state-before.json` / `formal-state-after.json` 的六个产品文件哈希完全一致。未复现此前 T03 的 `SQLITE_INTERRUPT`；历史失败收据保留。完整命令、退出码及日志见 `checks.json`，原始日志另存 gzip。

## 剩余边界

默认全局 Trellis 版本与 Codex 包装器故障保持原状。隔离 bootstrap 只证明该固定执行器的初始化与静态资产，不代表五客户端 fresh-session、hook、权限或委派运行通过；后者由 T06 验证。六项 Rust ignored、性能容量门、原生安装态及 hosted Node 矩阵未在 T04 验证。未提交、归档或执行远端操作。

## 2026-10-02 缺失可选覆盖复验与回写

以上 2026-09-30 验收保留为当日合同与候选的历史证据。本轮仅验证缺失四可选覆盖的最小公开合同隔离 init，未使用完整 fresh checkout 副本。任务保持 `in_progress`，修订后的 AC3 为 **PARTIAL**，结果为 **INIT_PASS / CONTRACT_FAIL**。

67 个明确公开文件的路径、来源类别、index 跟踪状态与 SHA256 见 `bootstrap-missing-overrides-20261002/copy-manifest.json`。固定 lock 含 59 个节点（工具根与 58 个依赖），仅获取一次；CLI/core 均为 `0.7.0-beta.3`，版本门与 init 使用同一明确 Node/CLI 入口。版本检查、获取与 init 均退出 0。四覆盖在 init 前均 missing、未选用、未复制，init 后均由固定 CLI 生成默认内容。

init 前合同 checker 退出 0。init 后合同 checker 首次退出 1，包含 5 项 Kimi check AUTH/DISPATCH 标记缺失和 2 项 Codex DEPTH 适用范围说明缺失。首失败 raw stderr、默认四文件真实字节与哈希、全部逐步 native 退出均保留。Python driver 实际退出 1，PowerShell 观察 driver 退出 1、意图退出 1，exec 观察 outer 退出 1；不以 init 退出 0 覆盖合同失败。没有修补上游默认模板或复制本机覆盖。

init 生成 218 个文件，7 个必要平台资产存在，11 个具名资产/覆盖命中候选忽略规则。67 个候选复制文件与当前公开来源哈希均未变；原保护清单 9 个文件及 index、HEAD、branch 前后相同。全局配置、trust 与私有文件沿用无修改命令边界，没有新增内容读取。本轮证据 `bootstrap-missing-overrides-20261002/evidence-manifest.json` 列出 52 个文件；结果与独立 observer 分别见同目录 `result.md`、`result.json`、`observer.json`。旧验收与本轮原始收据未改写。

回写适用 Claude Code、Codex、Grok Build、Kimi Code、OMP 的隔离 bootstrap 说明：匹配入口且 init 退出 0 只证明初始化；四覆盖缺失不阻断 init，但该固定版本会生成默认覆盖。init 后必须另跑合同 checker。失败保持可见，不自动复制本机覆盖、不放宽 checker，也不声明原生角色加载。本次回写仅修改 task.json、PRD 的 AC3 状态说明、本实施说明的追加段落及 docs/agents/harnesses.md；未触及 T05、产品、四本机覆盖或默认模板。只做静态 JSON/文档针对性校验，不运行测试、构建、客户端或依赖获取。

静态 JSON、AC3 未勾选与 PARTIAL 状态、文档必要说明/具体引用及尾空白校验均通过，退出 0；原 52 项证据哈希逐项保持一致。当前仓库的 `node scripts/check-agent-contract.js` 退出 0，仅校验本次文档回写和当前已有覆盖；该结果不替代隔离候选默认覆盖的首失败退出 1。

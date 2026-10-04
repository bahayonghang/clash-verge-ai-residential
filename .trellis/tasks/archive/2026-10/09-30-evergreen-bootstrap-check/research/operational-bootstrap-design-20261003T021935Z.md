# Research: T04 可重复的项目合同 bootstrap

- Query: 核对缺失覆盖场景的 7 项错误，设计保留本机覆盖字节、维持授权与递归边界的最小修复。
- Scope: internal
- Date: 2026-10-02（用户时区 America/Chicago）；调查时间 2026-10-03 02:19 UTC。
- Status: DESIGN_READY_IMPLEMENTATION_NOT_RUN
- Target: `.trellis/tasks/09-30-evergreen-bootstrap-check`，由主会话最新派发明确指定。原生 hook 指向 T05，已向主会话报告该冲突；没有读取 T05 工件或修改任务指针。

## Findings

### Files Found

| 路径 | 作用 |
| --- | --- |
| `scripts/check-harness-environment.js` | 只读工具发现、版本门与四个可选覆盖的存在性/hash 诊断。 |
| `tests/check-harness-environment.test.js` | 明确禁止诊断入口接受 `--init` 或 `--install`。 |
| `scripts/check-agent-contract.js` | 对存在的覆盖执行 AUTH、DISPATCH、DEPTH 等结构检查。 |
| `tests/check-agent-contract.test.js` | 正反例结构测试；其中占位内容不能用作可运行角色模板。 |
| `docs/agents/harnesses.md` | 五工具边界、隔离 init、缺失覆盖的历史失败。 |
| `.trellis/workflow.md` | 共享授权和目标一致性规则。 |
| `research/bootstrap-missing-overrides-20261002/result.md` | INIT_PASS / CONTRACT_FAIL 的首失败收据和默认文件 hash。 |
| `research/bootstrap-missing-overrides-20261002/contract-error-extraction.json` | 7 项错误的原始提取。 |
| `research/bootstrap-missing-overrides-20261002/isolated-paths.json` | 固定 CLI 和候选路径。 |
| `.agents/skills/trellis-meta/references/local-architecture/overview.md` | 项目本地定制的归属说明。 |
| `.agents/skills/trellis-meta/references/customize-local/overview.md` | 保留本地修改，不修改全局包或 node_modules。 |

### 实际失败与语义冲突

检查了真实隔离候选：`C:/Users/lyh/AppData/Local/Temp/trellis-t04-missing-overrides-20261002-q7twazn_/candidate`。四文件确由固定 `@mindfoldhq/trellis` / core `0.7.0-beta.3` 生成，已有收据记录 init native exit 0，checker native exit 1。未重跑 init、修改候选或执行角色。

7 项错误为 Kimi check 的 3 项 AUTH、2 项 DISPATCH，以及 Codex config 的 2 项 DEPTH。检查器 `scripts/check-agent-contract.js:106` 至 `:110` 要求 Kimi check 的授权与目标核对文本；`:139` 和 `:140` 要求 Codex V1/V2 与提示词隔离说明。原有 `[agents] max_depth = 1` 无需改变。

Kimi 默认 check `SKILL.md:16` 采用首个路径优先，`:49` 声称不支持项目自定义代理，`:88` 要求无条件自行修复。上述内容必须在交付模板中整体改为共享合同语义。只附加五个 marker 仍会留下相互矛盾的指令。默认 implement/research 也重复过时的项目代理能力说明；只替换 check 不能交付一致的三个 Kimi 角色。

Codex 默认 config `:30` 至 `:37` 把 `max_depth` 描述为防止递归的充分条件。新公开模板须保留配置值，改写说明为 `V1 agent threads only; V2 ignores this field` 和 `prompt guards are not a hard sandbox`，说明模型/会话观察不证明其他会话的后端。模板不能写入 user-level feature flags、trust、hook 批准或机器路径。

四项默认文件 hash 与已有 `overrides-after.json` 相同；当前本机四覆盖只读。未将本机文件复制到公开产物。

### 已批准边界内的可用机制

`check-harness-environment.js` 没有写入或 init 入口。返回值固定 `executesBootstrap: false`；`tests/check-harness-environment.test.js:151` 将 `--init` 与 `--install` 作为拒绝案例。把写入塞进此入口会破坏其已交付的只读合同。

`justfile:27` 至 `:29` 的 recipe 只运行诊断。`package.json` 只接入诊断及结构检查器，不包含受控 bootstrap。需要新增入口及公开模板。仅靠现有三个文件且不新增交付材料，不能兼顾真实模板内容、可重复交付、只读诊断和覆盖字节保留。

### 推荐最小文件范围

主会话先将下列新增文件明确纳入 T04 design/implement 与上下文。目录名可采用下列方案；不要写到被忽略的原生目录作为公开交付。

- `scripts/bootstrap-harnesses.js`：零依赖 Node 18+ CommonJS CLI，仅用于明确指定的新隔离候选。
- `tests/bootstrap-harnesses.test.js`：版本拒绝、缺失部署、已有字节保留、init 错误和合同失败的 fixture。
- `harnesses/codex/config.toml`：公开的项目默认合同，不包含机器或用户配置。
- `harnesses/kimi/trellis-implement/SKILL.md`：完整可执行的项目角色说明。
- `harnesses/kimi/trellis-check/SKILL.md`：完整可执行的项目角色说明。
- `harnesses/kimi/trellis-research/SKILL.md`：完整可执行的研究与持久化说明。
- `package.json`：将新增脚本/测试接入明确语法与测试列表，可增加显式 bootstrap script。
- `justfile`：新增需要明确 candidate/entry 参数的 recipe；原只读 recipe 保持不变。
- `docs/agents/harnesses.md`：说明公开模板来源、部署顺序、保留规则、失败状态和原始 init 历史失败。

不需要修改 `scripts/check-agent-contract.js`、忽略规则、本机四覆盖、全局 Trellis、安装包内默认模板或当前 `.template-hashes.json`。新入口可复用环境脚本导出的 `parseVersion` / `probeEntry` / `runVersion`，无需复制版本或发现逻辑。

四模板比两个修复模板多两个文件；必要性是三个 Kimi 角色共用已修订的能力与加载合同。当前本机 implement/research 不是内容完全可信的公开来源：其路径优先逻辑仍需模板作者依据共享 workflow 单独核对。公开模板应由已读取共享合同重新编写，不能直接镜像本机覆盖。

### 入口顺序与保留规则

建议调用形状：`node scripts/bootstrap-harnesses.js --root <new-isolated-candidate> --entry <absolute-trellis-entry>`。参数须显式，不设置当前工作树默认写入目标；CLI 路径可直接指向既有隔离固定包，不自动安装、不发现后回落、不升级。

1. 验证目标真实路径是调用者提供的隔离候选，拒绝仓库自身、祖先目录、全局工具目录及路径别名绕过；拒绝覆盖槽位或父目录为符号链接/reparse point。候选的公开合同、版本与必要脚本须由既有公开复制清单提前准备。
2. 初始 bootstrap 场景应拒绝已生成五平台资产的旧候选。允许四个具名覆盖预先由使用者按已授权选择复制。失败的旧候选保持证据；新建候选重试。
3. 对同一个显式入口运行 `--version`。native exit 必须为 0，解析到的明确版本必须与候选 `.trellis/.version` 完全一致。失败时不得部署任何模板或运行 init。
4. 记录四槽位初始状态/hash。已存在文件标为 `preserved_existing`，不得改动、规范换行、补注释或替换。缺失槽位使用四个固定公开模板，以独占创建方式写入；标为 `created_project_default`。不要自动从当前工作树复制覆盖。
5. 调用未放宽的 `checkContract(candidate)` 做部署后检查。已有覆盖不满足合同则记录失败，停止 init，并保留字节。不能以存在覆盖为由豁免检查。
6. 同入口在候选 cwd 执行 `init --claude --codex --grok --kimi --omp --skip-existing -y`。保存 stdout/stderr、native exit、signal/timeout；失败不能用后续命令成功替代。
7. 比较四覆盖部署后与 init 后 SHA256。已存在覆盖还应与初始 SHA256 一致。回读候选公开复制文件与五平台必要资产，并调用原 checker 复验。分别报告 `INIT_PASS`、`CONTRACT_PASS` 与 `PRESERVATION_PASS`；合并入口只在所有必需结果通过时返回 0。
8. 保护清单继续覆盖当前本机四覆盖、原 template manifest 和选定公共来源；对全局 trust/config 明确只记录未发出相关写命令的边界，不读取私人内容来构造全盘未变断言。

部署前置模板有已核验的实际机制依据。已安装 `0.7.0-beta.3` 的 `dist/commands/init.js:874` 至 `:876` 将 `skipExisting` 设为 skip；`dist/utils/file-writer.js:126` 至 `:130` 遇已有文件直接返回 false，不写入该文件，也不把跳过文件登记为 init 所有。不能手工改写 manifest 把这些项目默认模板标成上游产物。

若需要入口重复调用：可以单独定义无 init 的“已完成候选校验”模式，并明确它只复验现态。当前最小实现可拒绝已生成候选，使用新候选重复验证；不能自动覆盖已生成默认或声称多次 init 的完整幂等性。

### 公开角色模板的有效内容

三个 Kimi 角色应说明项目支持 custom agents，但项目固定使用 built-in `coder` 加角色 skill；研究不能使用缺少写工具的 explore 来满足持久化。

三个角色从派发首行读取 `Active task: <path>`，仍执行 `task.py current --source`，比较非空的 dispatch、current 和注入路径。发生冲突须停止加载冲突材料，`ask the main session to confirm the dispatch target`，收到确认后显式加载确认任务；不得改写共享 task 指针消除差异。要求同样适用于 `even when the dispatch prompt supplies a path`。

implement/check 根据自身上下文 manifest 加载引用，再读 prd/design/implement。缺失或 seed-only manifest 走规范列举与任务工件 fallback；没有 prd 则报告缺失。research 只读研究资料，不读 implement/check manifest；输出固定落在已确认任务 research 目录。所有角色都先遵守 `AGENTS.md`。

implement/check 不再递归派发同角色；写入仅限用户批准的任务和文件。check 明确 `Read-only review must not repair product code or configuration`；self-fix 只在 approved implementation 和 `approved task and file scope` 内。权限、工具可用性和任务状态不构成授权。research 禁止产品、spec、平台配置和 Git 操作，发现所需产品修改只提出设计。

不将 `tests/check-agent-contract.test.js:17` 起的 `OVERRIDE_PLACEHOLDERS` 用作运行模板。该 fixture 注释说明占位内容仅满足结构要求。新 bootstrap 测试须使用真实公开模板并审查实际控制指令。

### Fixture 断言和正式验证

必要断言：

- 四项都缺失：四公开模板创建，实际模板通过原合同检查，无 unconditional self-fix 或“无项目代理”声明。
- 四项都已存在：使用含 CRLF 的有效 fixture，前后 Buffer 和 SHA256 完全一致。
- 混合场景：只创建缺失项；每项状态区分 existing 与 public default。
- 已有覆盖无效：合同失败且无改写，不运行 init；不自动补 marker。
- 版本错误/未知/入口失败/缺版本：写入前退出，进程调用记录中无 init。
- init 非零、signal、timeout：保留首次失败；后检查不能改报 bootstrap 通过。
- 模拟 init 改写覆盖：保留失败收据并返回失败，不静默修复。
- 拒绝当前树、祖先、链接目标、非预期旧候选；fixture 验证拒绝在写入和 init 前发生。
- 无覆盖内容/凭据输出；报告路径、来源、hash、独立退出结果。不得出现 install、trust、用户 config 写入或 PATH 更改。

实施后的命令：

```text
node --check scripts/bootstrap-harnesses.js
node --test tests/bootstrap-harnesses.test.js tests/check-harness-environment.test.js tests/check-agent-contract.test.js
node scripts/bootstrap-harnesses.js --root <new-public-candidate> --entry <fixed-cli-absolute-path>
node <new-public-candidate>/scripts/check-agent-contract.js
```

正式验收至少执行“缺失四覆盖”与“明确复制四覆盖”两个新的隔离场景，绑定相同固定 CLI、公共来源和独立 native/driver/outer 退出。原 20261002 失败候选与收据不变。然后按批准 T04 门执行 `just ci`、`just docs-build`、secret scan 和 scoped whitespace 检查。新入口和公开模板不能关闭 T06 原生角色/hook/权限/生命周期验收，也不能关闭 T05 性能条款。

### External References / Versions

- 只读取本机已安装的固定 `@mindfoldhq/trellis` 和 core `0.7.0-beta.3` 公开源码；未联网、运行 init、安装或启动负载。
- CLI 根：`C:/Users/lyh/AppData/Local/Temp/trellis-t04-missing-overrides-20261002-q7twazn_/tool/node_modules/@mindfoldhq/trellis`。
- Codex V1/V2 原说明已由现有项目合同和本机覆盖载明；本调查没有重新联网核验 OpenAI 文档，不能宣称支持所有后续 Codex 版本。

### Related Specs

- `.trellis/spec/frontend/index.md`：root scripts 工具链与全产品 gate。
- `.trellis/spec/frontend/quality-guidelines.md`：CommonJS、零第三方依赖、Node 18+，可选覆盖结构检查和独立运行边界。
- `.trellis/workflow.md:103`：Review authorization；`:114` 起是任务路径冲突处理。
- `AGENTS.md`：共享授权、读审边界、禁止自动 trust。
- T04 `prd.md` / `design.md` / `implement.md`：原 Owned Files 不包含推荐新增入口、模板和测试，主会话须先同步文件范围。

## Caveats / Not Found

- 没有修复产品、配置或本机覆盖。没有执行 Git 操作。
- 按 research 角色隔离规则未读取 `implement.jsonl` / `check.jsonl`；主会话应将本报告作为实施上下文纳入对应清单。
- 本设计只处理公开项目 bootstrap 的合同交付。全部五工具运行、模型、权限、hook 和子代理证据另由 T06 验收。
- 67 文件历史候选并非完整 fresh checkout；新正式场景需明确公共复制边界，不能补写历史标签。
- 竞争负载不影响本轮调查，没有查询、启动或停止其他进程；未减少历史正式性能门。

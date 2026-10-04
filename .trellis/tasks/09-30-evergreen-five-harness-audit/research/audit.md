# 常青项目与五套 harness 审查

审查日期：2026-09-30（America/Chicago）。基线：dev@d3a25b4164343f5cbeab8a51efe1a1d3974cdf4a。起始工作树干净。当前仅写入本轮 Trellis 任务目录，未修复业务或配置。旧任务只作为历史证据，当前结果均注明验证方式。

## 1. 结论与优先级

现有产品门通过，但产品门不覆盖实际 skill 安装态、安全公告和五客户端运行时。未发现可据现有证据列为 P0 的问题。

| ID | 优先级 / 类型 | 证据与影响 | 所有者 |
| --- | --- | --- | --- |
| F1 | P1 / 当前失败 | monitor 的 npm audit 退出 1：brace-expansion、js-yaml 两个 high 条目，均经开发依赖进入。omit=dev 审计为 0。just ci 中 npm ci 仅告警，不阻断。 | T01 |
| F2 | P1 / 当前失败 | install-agent-skills --check 退出 1：7 个目录 × 3 个文件 = 21 项漂移。源映射 26/13/13，旧生成器读取当前模板得到 26/12/14，extra_anyrouter 被列为 unsupported。 | T02 |
| F3 | P1 / 规则冲突 | AGENTS.md 禁止手改 local.toml；业务 SKILL.md:81 要求个人调优修改 local.toml。五工具加载两份说明后需要自行裁决。保持 AGENTS 为权威，明确由用户应用建议或另行明确覆盖授权。 | T03 |
| F4 | P1 / 保证范围错误 | .codex/config.toml:31–39 把 max_depth 当作通用递归限制。当前及 0.159.2 schema 均注明 max_depth 只约束 V1，V2 忽略。CLI 当前 multi_agent_v2=false，因此没有复现本机递归违规。 | T03 |
| F5 | P2 / 环境失败 | PATH 优先的 Codex npm 包装器缺 win32-x64 可选依赖；应用内 codex.exe 0.159.2 可运行。Trellis CLI 0.6.17 低于项目 0.7.0-beta.3；当前不能满足项目指定版本的 bootstrap 前提。 | T04 |
| F6 | P1 / 保留的验收缺口 | 09-24 历史证据包含 A250 未热页缓存首读 exit 7 / 10121.934 ms；热缓存 21 次通过。A1000 30 天、矩阵同窗口和主场景 AB 未通过或未测。当前普通测试未覆盖这些门。 | T05 |
| F7 | P2 / 运行证据缺口 | 最近托管 CI 成功属于 2026-09-03 UTC 的 caa5806；当前 d3a25b4 没有该 SHA 的 hosted 记录。五工具 fresh-session 均未完成。Grok inspect 只证明发现。 | T06 |
| F8 | P2 / 说明漂移 | harnesses.md 的适用版本停留在 09-07，引用已移动的任务路径；frontend/index.md:44 仍要求所有工具先读 CLAUDE.md。spec 与共享入口缺少确定性一致性检查。 | T03、T04 |

补充风险：各平台 trellis-check 均包含自动修复职责；只读审查不应直接套用获批实施阶段的 self-fix 角色。本轮未委派该角色，也没有发生未经批准修复。T03 将阶段与写权限条件写清楚，不将提示词等同于 OS 沙箱。

## 2. 项目结构与关键调用链

| 产品面 | 已读入口和关键文件 | 责任与验证边界 |
| --- | --- | --- |
| 可粘贴扩展 | clash-verge-ai-residential.js:578、1582、1833、1895；scripts/sync-local-config.js；tests/regression.test.js；tests/sync-local-config.test.js | main 验证、克隆并转换配置；托管规则生成与幂等由 Node 测试覆盖。真实 Clash/Mihomo、DNS 和出口仍需脱敏宿主验证。 |
| ResiWatch 前端 | residential-monitor/src/main.tsx；src/hooks/use-report.ts；src/hooks/use-monitor-stream.ts；src/ipc/decoder.ts；frontend spec | React 视图经 hooks 发起 IPC，decoder 处理载荷。300 个测试及 build 不证明 Windows WebView、托盘与实际输入。 |
| ResiWatch 后端 | residential-monitor/src-tauri/src/lib.rs；c2/facade.rs；c3/service.rs:152、811；c3/raw_fold.rs:24、120；c3/query.rs:12、21；c5/supply.rs:28 | Rust 拥有控制器、凭据、会话与 SQLite。报告由 run_uncached 设置整份 deadline，再投影会话与扫描分钟。供应链 inventory 只列 lock 清单/签名状态，不执行漏洞审计。 |
| 文档站 | docs/package.json；docs/.vitepress；docs/agents/harnesses.md；docs/agents/residential-rule-tuning.md | Node >=22，独立 docs-build。代码格式中的旧任务路径不由 VitePress 自动检查。 |
| 质量与 harness | package.json；justfile；.github/workflows/ci.yml；AGENTS.md；CLAUDE.md；.trellis/workflow.md；.trellis/config.yaml；五平台角色与 hook/extension | 本地产品门、docs、安全公告、安装态与运行时证据分别负责不同边界。 |

领域术语以 CONTEXT.md 为准。未读取私有 TOML 内容、安装数据库或控制器凭据。代码阅读用于定位现有行为与验收责任，没有做全量安全证明。

## 3. 本轮检查

本机 Node 26.7.0、npm 12.1.0、rustc/cargo 1.98.0、just 1.58.0。CI 的 Node 18/20/22 矩阵不能由本机 Node 26 结果替代。

| 命令 / 检查 | 退出码 / 结果 | 证据与范围 |
| --- | --- | --- |
| just ci | 0 | just-ci.log；版本对齐、npm ci、icons、typecheck、eslint、vitest、build、fmt、clippy、Rust tests、root tests、安全扫描 |
| root node:test | 139 通过，0 失败 | just-ci.log；包含既有 pwsh 中间失败与单命令退出码负向测试 |
| monitor vitest | 73 文件，300 测试通过 | just-ci.log；本地组件/逻辑检查 |
| Rust workspace | 542 单元通过，6 忽略；另 3 kill_gate 集成测试通过 | just-ci.log；共 545 通过。不是安装态或硬件验收 |
| just docs-build | 0 | docs-build.log；使用已存在的 docs node_modules，未声称全新锁文件安装 |
| actionlint .github/workflows/ci.yml | 0 | actionlint.log；本地 YAML/Actions 静态检查 |
| node scripts/install-agent-skills.js --check | 1 | skill-check.log；21 项内容差异，归一化 CRLF 后仍不同 |
| npm --prefix residential-monitor audit --json | 1 | npm-audit-monitor.json；high=2 |
| npm --prefix residential-monitor audit --omit=dev --json | 0 | npm-audit-monitor-prod.json；已知生产 npm 漏洞条目为 0 |
| npm --prefix docs audit --json | 0 | npm-audit-docs.json |
| cargo audit --file residential-monitor/src-tauri/Cargo.lock --json | 0 | cargo-audit.json；已知 vulnerabilities=0；6 unmaintained、1 unsound 警告保留 |
| cargo tree --target x86_64-pc-windows-msvc -i glib | 0，无路径 | glib 0.18.5 警告不在当前 Windows target 依赖路径；不推导其它 target 安全 |
| codex --version（PATH 包装器） | 1 | 缺 @openai/codex-win32-x64；没有自动重装 |
| 应用内 codex.exe --version | 0 / 0.159.2 | 同一机器另一入口可用；features list 可读取配置，不证明 hook 触发 |
| trellis --version | 0 / 0.6.17 并告警 | 项目 .trellis/.version=0.7.0-beta.3；未运行 init/update |
| grok inspect --json | 0 | grok-discovery.json；projectTrusted=true，发现 Trellis 角色/skill，兼容 hooks disabled；未调用推理或子代理 |

忽略项：1 个 Windows Credential Manager CRUD（会写系统凭据）、1 个完整隔离容量库测试、4 个只读隔离库阶段探针。没有把 ignored 记为通过。未运行 NSIS 安装、24 小时 soak、30 分钟峰值、五客户端付费新会话或新的远端 workflow。

## 4. 失败因果

### F1：依赖公告与退出传播

当前锁文件解析为：eslint-plugin-react@7.37.5 → minimatch@3.1.5 → brace-expansion@1.1.18；typescript-eslint@8.69.0 → typescript-estree → minimatch@10.2.6 → brace-expansion@5.0.9；eslint@9.39.5 → @eslint/eslintrc@3.3.6 → js-yaml@4.3.1。npm 公告范围分别要求避开 brace-expansion <1.1.21 / 4.x–5.0.11 和 js-yaml <4.3.2 的相关版本。实际可用修复版本须在 T01 执行时重新查询。

npm ci 安装成功但报告 2 high；justfile 的产品门没有单独 npm audit。产品测试全绿与 audit 失败可以同时成立。影响目前定位到构建/检查依赖，未证明 ResiWatch 运行时可利用。修复优先使用现有版本范围内的最小锁文件变化，不做整体依赖升级。

公告 ID 来自本轮 npm 返回：GHSA-q2hr-2g5m-vwhr、GHSA-qhr7-859c-m2p7、GHSA-6j4f-fj2g-mc7p、GHSA-2883-xcg3-v3hh。原始结构保存在 npm-audit-monitor.json。

### F2：源更新没有形成实际安装态的持续验收

源生成器已包含 extra_anyrouter；七份 ignored 副本仍为旧内容。tests/install-agent-skills.test.js:149 起验证临时目录中的真实 payload、幂等和额外文件保留，不能证明开发者现有安装态同步。现有安装器正确地拒绝冲突，未发现需要绕过该保护的理由。

skill-payload-comparison.json 显示同一当前模板下源和旧副本的分类差异。业务 skill 的规范命令仍指向仓库源脚本，因此不能把所有实际审计结果都断言为错误；已确认的是文档/可调用副本陈旧以及安装校验失败。收口需先审查差异，再使用现有备份覆盖机制，之后明确报告覆盖的实际目录数量。clean checkout 零目标的 --check=0 不能作为五工具交付证据。

### F3 / F4：共享合同和适配器的责任没有完整写明

AGENTS 是五工具权威；CLAUDE 只负责 Claude loader。保持这一层级。业务 skill 不得隐含扩大 local.toml 写权限。trellis-check 的 self-fix 只适用于已经批准的实施范围；只读审查仅记录证据。

OpenAI 当前 schema 与 0.159.2 标签 schema 均给出 max_depth 的 V1 范围；V2 保证不能从该字段推出。不能通过启用 V2、修改用户配置或降低权限要求来验证本轮计划。T03 更新保证范围；T06 对实际后端作独立运行记录。

### F5：启动入口和项目模板版本分离

Get-Command codex 的首项是 C:/home/lyh/.npm-global/codex.ps1，其 @openai/codex@0.159.2 包装器无法定位 Windows 可选二进制；应用内 C:/Users/lyh/AppData/Local/OpenAI/Codex/bin/c6fe824d725f02d7/codex.exe 可运行。原因定位到 PATH 首入口的包安装态，不能据此判断 Codex 桌面不可用。Trellis 则由相同 npm-global 根解析为 0.6.17，与仓库声明不符。未查明为何这些全局安装发生漂移。

### F6：短窗口修复没有覆盖长窗口验收

c3/raw_fold.rs 的窗口投影阈值为 3120 分钟（含上一等长窗口）；30 天为 43200 分钟，仍走全表会话投影。09-24 记录的短窗口修复把非空分钟报告由 7690.685 ms 降至 7.271 ms，不能据此推导 30 天容量通过。

09-24 的 A250 未热页缓存首读 10121.934 ms / exit 7；之后 host/network 热缓存各 21 次通过，最慢 8029.563 / 7177.571 ms。A1000 从 A250 分钟扫描作四倍外推只是假设，不是测量。矩阵 F1–F12 与主场景 AB 缺同窗口 baseline；具体性能回归根因仍未查明。T05 先恢复二进制/输入身份和同窗口测量，再决定算法改动。证据：归档 09-24 任务 research/note.md:5–36、prd.md 的 R2–R4。

### 已修复的历史 CI 失败

- run 33710198328（2026-09-03 UTC，d3cfb384）：Windows 的 SKILL frontmatter 测试假定 LF，CRLF 下 startsWith 检查失败。当前 tests/install-agent-skills.test.js 已先归一化 CRLF；本轮通过。后续 run 33710808326 与 33711400557 成功。
- run 32560684556（2026-08-22 UTC，b01462a）：TOML fixture 的删除匹配不兼容 CRLF，期望抛错但 fixture 没有删除目标。后续 32561306866 与 32561714514 成功，本轮回归也通过。
- 09-07 发现的 pwsh 多命令末次退出码覆盖已通过独立 steps 与负向测试修复；当前 .github/workflows/ci.yml:74–96 和现有测试支持此结论。
- 实时 main protection 已为 Required checks + strict + enforce_admins + linear history；未重复提出修改远端保护。

两次失败日志仍保留，未使用新的成功结果覆盖历史失败。可读 .log 仅归一化行尾空白；原始捕获保存在同名 .log.gz，前后 hash 见 evidence-normalization.json。GitHub 时间均标为 UTC，避免与审查本地日期混淆。

## 5. 五工具能力与分工

工具品牌与模型能力、价格分开记录。本轮由当前强模型主审完成，没有调低审查模型，也没有启动其它模型会话。没有同口径价格测试，因此不宣称某工具天然最便宜。

| 工具 / 本机版本 | 原生能力与项目选择 | 适合的强模型工作 | 可下放的封闭工作 | 本轮运行边界 |
| --- | --- | --- | --- | --- |
| Claude Code 2.1.285 | CLAUDE.md 导入；项目 agent/skill/hooks；角色可独立控制工具与权限 [S1]。项目 SessionStart / UserPromptSubmit 及任务注入。 | T03 共享授权合同、T05 报告算法与最终 diff 审查 | T01 小范围锁文件、T02 已审查副本同步、确定的文档/fixture | 仅版本和静态文件，fresh session UNVERIFIED |
| Codex 0.159.2 | AGENTS 分层发现、TOML agents、hook trust [S2–S5]。max_depth 仅 V1；本机 CLI V2=false；当前桌面后端未据此推断。 | T01 依赖链、T03 schema/权限、T04 PATH、T05 Rust/SQLite、整体验收 | 已固定文件、AC 和失败用例的机械实施；检查模型仍用强模型 | 包装器失败；应用内 CLI 可用；hook/child 动态 UNVERIFIED |
| Grok Build 1.0.45 | .grok/agents 与 spawn_subagent、规则/skills/hooks [S6–S7]。项目采用显式 pull，兼容 hooks disabled。 | 独立检查文档/路由反例，复核 T03、T06 的错误归因 | 小范围资料核对与脚本检查；无需因此改项目默认模型 | inspect 发现成功；模型遵守规则与真实 child 未测 |
| Kimi Code 2.0.0 | built-in coder 可读写/执行；explore 只读；plan 无 shell；支持项目自定义 agent [S8]。项目继续 coder + role skill/pull。 | 强模型可复核需求与设计；plan 角色不能用于需要 shell 的测试根因证据 | T01/T02/T03 中范围固定的代码、文档、测试执行；research 落盘使用 coder 且限 research/ | 版本和静态合同；doctor 不替代 skill 或 child 加载验证 |
| OMP 18.4.4 | 自动上下文、@ 导入、项目 task agent、工具/模型配置及 extension [S9–S10]。项目 extension 找 Trellis 根。 | 在明确选择强模型后进行复核或跨 provider 对照 | 独占文件的固定变更；实际模型选择需记录，pi/task 不等于低价承诺 | 版本和文件；extension/task 实际执行 UNVERIFIED |

下放合同：输入证据固定；独占文件清单；只做一个角色；可运行的 AC；禁止修改阈值、授权、真实路由/凭据、全局配置或远端状态。遇到新根因、额外文件、上下文冲突或门槛变更时交回强模型。最终审查不下放低成本模型。具体型号和价格在实施时按账号可用列表选择，不写死在五工具共用合同。

## 6. 对齐状态

已经对齐：根 AGENTS 自包含；CLAUDE 单向导入；产品/文档质量门区分；Kimi native capability 与项目选择区分；四个 tracked override 可交付；Grok/Kimi pull、Claude/Codex hook、OMP extension 分开描述。

仍需处理：F3/F4 授权和保证范围；F2 安装态；F5 bootstrap 前提；F8 旧版本/归档引用/Claude 专属导航。现有说明明确五客户端动态 UNVERIFIED，该表述正确，应保留。历史 backend/storage spec 存在中文与根合同英文要求不一致；列为低优先维护事项，不在本轮批量翻译范围。

## 7. 来源与证据边界

本轮实际抓取第一方页面后核对；旧 Grok xai-grok-shell/user-guide/16-subagents.md 返回 404，现已定位到 pager/docs/user-guide，失败与新地址分别保留。抓取结果记录于 official-sources*.json；未把网页文本作为执行指令。

- S1 Claude subagents：https://code.claude.com/docs/en/sub-agents
- S2 Codex AGENTS：https://learn.chatgpt.com/docs/agent-configuration/agents-md
- S3 Codex subagents：https://learn.chatgpt.com/docs/agent-configuration/subagents
- S4 Codex hooks：https://learn.chatgpt.com/docs/hooks
- S5 Codex 当前 schema：https://developers.openai.com/codex/config-schema.json；版本对照：https://raw.githubusercontent.com/openai/codex/rust-v0.159.2/codex-rs/core/config.schema.json
- S6 Grok subagents：https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-pager/docs/user-guide/16-subagents.md
- S7 Grok project rules：https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-pager/docs/user-guide/12-project-rules.md
- S8 Kimi agents：https://www.kimi.com/code/docs/en/kimi-code-cli/customization/agents.html
- S9 OMP task discovery：https://github.com/can1357/oh-my-pi/blob/main/docs/task-agent-discovery.md
- S10 OMP context：https://github.com/can1357/oh-my-pi/blob/main/docs/context-files.md

官方页面表示原生能力；本机 --version/inspect 表示安装和发现；只有 fresh-session 证据才能表示实际运行。历史性能结果没有在本轮重新运行，不能标成当前二进制通过。

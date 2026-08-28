---
title: residential-monitor C0 to C5
platform: codex
status: draft
---

# Context

仓库是 Clash Verge Rev 全局扩展脚本，不是已有桌面应用。父任务 08-18-residential-monitor-mvp 已批准完整 Windows 11 v1 规划。六个子任务都是 planning，目录名为 mvp 只为保持指针，产品范围是完整 v1。当前分支是 dev，工作区只有这七个任务目录未跟踪。根验证命令是 just ci，等于 npm run ci。子项目命令面以 C0 implement.md 为准，落地后以锁定脚本为准。

先读这些权威材料，再改代码：

- .trellis/workflow.md
- CLAUDE.md
- AGENTS.md
- .trellis/tasks/08-18-residential-monitor-mvp/prd.md
- .trellis/tasks/08-18-residential-monitor-mvp/design.md
- .trellis/tasks/08-18-residential-monitor-mvp/implement.md
- .trellis/tasks/08-18-residential-monitor-mvp/research/
- 六个子任务各自的 prd.md、design.md、implement.md、implement.jsonl、check.jsonl
- package.json
- justfile
- .github/workflows/ci.yml

执行前加载 trellis-before-dev。每个子任务结束前加载 trellis-check。父任务不承担大爆炸实现，只保留跨子任务验收。

子任务顺序与目录：

1. C0 08-18-monitor-foundation-spike
2. C1 08-18-monitor-collector-storage
3. C2 08-18-monitor-desktop-realtime
4. C3 08-18-monitor-reporting-data
5. C4 08-18-monitor-alerting-diagnostics
6. C5 08-18-monitor-release-hardening

# Contract

目标结果：在 residential-monitor 交付完整 Windows 11 v1 ResiWatch 桌面应用，并完成 C0 到 C5 六个子任务各自 prd.md 可观察验收标准与 implement.md Gate。应用采集 Clash Verge Rev 与 mihomo 的全部连接事实，按用户重点目标分类，提供实时监控、历史报告、导出、告警、保留、备份恢复与 NSIS current-user 安装升级。产品只提供观测下界，不冒充代理商账单。

验证：每个子任务只使用该任务 prd.md 可观察验收标准与 implement.md Gate 作为完成枚举源。命令以 C0 implement.md 计划中的稳定命令面为准，名称若调整必须同步规范、CI、帮助文本和证据。落地后每个子任务结束至少运行：

- python ./.trellis/scripts/task.py start 对应子任务，成功后才写产品代码
- npm --prefix residential-monitor ci
- npm --prefix residential-monitor run typecheck
- npm --prefix residential-monitor run lint
- npm --prefix residential-monitor test
- npm --prefix residential-monitor run build
- cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check
- cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings
- cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace
- just monitor-check
- just ci
- npm run check:secrets

C0 另需可复现的 adopt、reject 或 fallback 证据，覆盖性能 harness、SQLite binding、CredentialStore port、TCP 与 named pipe 矩阵、NSIS 能力 spike 和 CI 基线。完整 30 天库与 10k 30 分钟峰值必须真实运行，不得线性外推。C1 另需 replay、守恒、migration、kill 与 Channel 测试证据。C2 到 C5 另需各自 implement.md 列出的自动测试与安装态命令。把退出码、关键输出和证据路径写入对话与对应任务目录。无法执行的真机项不得标完成。

约束：不发明域规则、兼容承诺或性能数字。不修改 Clash 或 mihomo 配置，不代理或抓取流量内容。不把控制器 meter 与可归因观测总量混称为同一个全局口径。缺口不得写成零。secret 不得进入 URL、日志、SQLite、Channel、导出、诊断或仓库。公开模板 HOME_PROXY_TEMPLATE 保持占位。C0 实验 schema 不得成为 C1 正式 migration。后一子任务不得改前一子任务已发布 migration。C1 只用 fake resolver，Windows Credential Manager 产品 adapter 留给 C2。v1 不做 DPAPI fallback、Windows Service、macOS 或 Linux 发布、应用内自动更新、PDF 或 Excel、云同步或外发遥测。根脚本保持 Node 18 以上、CommonJS、零依赖和复制粘贴契约。CI 可增加 Windows Rust 与 Node 子项目检查，但不得改变根 Required checks 聚合名称。UI 文案、代码注释与子项目文档用中文。

边界：允许写入 residential-monitor、.trellis/spec/residential-monitor、08-18-residential-monitor-mvp 与六个子任务目录、justfile、.github/workflows/ci.yml、scripts/check-template-safety.js 中为覆盖新文件类型所需的扫描规则，以及子任务 implement.md 明确要求的 README、docs 或 CHANGELOG。禁止写入 clash-verge-ai-residential.js 路由逻辑、clash-verge-ai-residential.local.toml、clash-verge-ai-residential.local.js、真实控制器 secret、住宅代理信息、未脱敏连接日志和用户数据库。不要手工改写 task.json.status 来绕过 task.py start。不要 push 到 main。不要发布 GitHub Release，除非用户另发指令。

迭代策略：一次只做一个子任务。先读权威材料并运行 trellis-before-dev，再 start，再按该子任务 implement.md 检查点推进。每个检查点先写通过条件再做实验。失败先读日志、测试输出和研究记录，再改策略。同一失败连续两次后必须换证据来源，例如最小复现、对照 fixture 或另一命令。C0 完成后把决策证据加入 C1 上下文，之后每个子任务同样引用上一任务验收。不要并行开启下一子任务。文中轮次、时长或 token 限制只是软停止条款，不等于平台运行时预算。

完成条件：

1. C0 的 prd.md 可观察验收标准 AC1 到 AC12 与 implement.md Gate 有证据文件和命令输出。
2. C1 的 prd.md 可观察验收标准 AC1 到 AC15 与 implement.md 自动化验证有证据。
3. C2 的 prd.md C2-AC1 到 C2-AC11 有证据。
4. C3 的 prd.md C3-AC1 到 C3-AC11 有证据。
5. C4 的 prd.md C4-AC1 到 C4-AC10 有证据。
6. C5 的 prd.md C5-AC1 到 C5-AC14 有证据，或已在暂停条件停止并列出未完成项。
7. 根 just ci 与 npm run check:secrets 退出码为 0，git status 只显示允许写入的路径。
8. 每个已完成子任务都经过 trellis-check，并留下可复现命令。

暂停条件：C0 必选能力 reject 且没有可执行 fallback；task.py start 因缺少会话身份或 TRELLIS_CONTEXT_ID 失败；缺少 Windows 11 真机、Clash Verge 控制器或用户授权的 TCP secret；即将向本机写入 NSIS 安装、登录自启动、Credential Manager 条目或系统通知；需要连续 24 小时 soak；需要代码签名证书或 GitHub Release 资产发布；需要付费、生产数据、破坏性删除或法律判断；连续两次换证据来源后仍无法推进。暂停时报告已完成验收项、证据路径、阻塞原因和下一步人工决定。

invocation:
```text
/goal 先保存并严格遵循 .planning/goal-residential-monitor-v1.md，按 C0 到 C5 顺序实施 08-18-residential-monitor-mvp 的六个子任务，直到该合同的验证与完成条件满足。
验证：先读取该合同与各子任务 prd.md、design.md、implement.md 及父任务 research，再按 implement.md 检查点执行；每个子任务结束时运行 just monitor-check、just ci、npm run check:secrets，把退出码和关键输出写入对话，并把证据路径写入对应任务目录。
约束：不发明域规则；不把 C0 实验 schema 当作 C1 正式 migration；C0 必选决策没有 adopt 或可执行 fallback 时不得启动下一子任务；不改公开模板凭证；不向用户正在使用的控制器发送真实 DELETE；不宣称账单精度。
边界：只写入 residential-monitor、.trellis/spec/residential-monitor、父任务与六个子任务目录、justfile、.github/workflows/ci.yml，以及子任务 implement.md 要求的文档；禁止写入 clash-verge-ai-residential.js 路由、*.local.toml、*.local.js、真实 secret 和用户数据库。
迭代策略：一次只做一个子任务；先运行 python ./.trellis/scripts/task.py start，再按该子任务 implement.md 推进；每次有意义改动后重跑对应检查；同一失败连续两次后必须换证据来源；文中轮次或时间限制只是软停止条款，不等于平台运行时预算。
完成条件：C0 到 C5 各自 prd.md 可观察验收标准与 implement.md Gate 均有命令退出码或证据文件可核验，且根 just ci 退出码为 0；若已在暂停条件停止，则对话中列出未完成验收项与证据缺口。
暂停条件：C0 必选能力 reject 且无 fallback、task.py start 因缺少会话身份失败、缺少 Windows 11 真机或 Clash Verge 控制器、即将写入本机 NSIS 安装或登录自启动、需要 24 小时 soak、代码签名、GitHub Release 发布、付费、生产数据或破坏性操作时暂停。
```

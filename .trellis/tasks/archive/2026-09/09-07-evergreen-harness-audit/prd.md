# 常青项目审查与五套 Harness 对齐

## Goal

审查当前常青项目的工程与协作工作流，建立 Claude Code、Codex、Grok Build、Kimi Code、OMP（Oh My Pi）可共同使用、可以逐项批准的改造计划。

## Background

本轮基线为 `dev` / `5578576787dba73ba1b96985a2fc408fc246bfed`，审查开始时工作树干净。仓库包含可粘贴 Clash 扩展、ResiWatch（React/Tauri/SQLite）、VitePress 文档和 Trellis/skill 工具链。

## Requirements

- R1 阅读结构、关键实现、质量门与现有项目规则，用当前代码和运行证据区分已修复问题、实际缺陷与未验证能力。
- R2 运行现有自动化测试并记录命令、环境、通过/失败/忽略及缺失证据；追踪本地与可访问托管工作流失败根因。
- R3 对照五套 harness 的规则发现、技能、子代理、权限和检查能力；不得把本地文件存在当成实际客户端加载成功。
- R4 强模型负责规划、根因判断和最终审查；只把目标和验收清晰、修改范围小的执行工作推荐给较便宜模型，不虚构价格或跨产品能力排名。
- R5 建立有优先级、依赖和文件所有权的 Trellis 父子任务，每项改动列明必须通过的检查。
- R6 批准实施后，把验收通过的规则回写到项目说明或项目 skill 源库，并标明适用工具。共享知识库/用户全局配置不是本轮写入目标。
- R7 本轮只写任务、研究和规划材料。用户批准最终计划之前不改产品代码、工具规则、安装态或真实路由，不执行 `task.py start`，不提交、不推送、不发起远端工作流。

## Acceptance Criteria

- [x] AC1 / R1–R2：有结构/关键入口说明、测试结果表和失败根因证据，记录当前 SHA 与历史 run SHA 的区别。
- [x] AC2 / R3–R4：五套 harness 均有第一方资料或本地证据、适合的规划/审查工作、可下放执行任务及无法证明的边界。
- [x] AC3 / R5：父任务关联全部子任务，任务均保持 planning；各子任务有文件清单、依赖、观察性验收与有效上下文 JSONL。
- [x] AC4 / R6：每个批准项有明确回写落点与适用工具；关闭任务前核对实际改动、检查记录和规则的一致性。对照见 [implement.md](implement.md) 与 [research/ac4-writeback.md](research/ac4-writeback.md)。静态/本地完成。hosted CI 为 UNVERIFIED。五客户端 live smoke 为 UNVERIFIED。不得宣称五工具运行时对齐。
- [x] AC5 / R7：本轮 diff 限于新建任务目录，最终摘要明确等待批准，不把规划验收当实施完成。

任务地图、优先级、文件/测试矩阵见 [design.md](design.md)。AC4 关闭对照见 [implement.md](implement.md)。

## Out of scope

- 大规模重构、拆分可粘贴单文件、增加运行依赖、模型基准竞赛或付费客户端实验。
- 修改 `*.local.toml` / `*.local.js`、生产控制器、Credential Manager、应用安装态、自动启动或路由。
- 30 天库、24 小时 soak、NSIS 真机和五客户端真实新会话的完成声明；没有直接证据时保持 UNVERIFIED。
- 重复实施已归档 09-03 工程整改和 09-07 路由优化中的已验证修复。

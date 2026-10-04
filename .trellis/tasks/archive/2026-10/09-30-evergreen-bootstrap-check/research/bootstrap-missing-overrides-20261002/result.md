# T04 缺失可选覆盖的隔离 init 复验

2026-10-02，14:14:43–14:15:05 UTC。状态：INIT_PASS_CONTRACT_FAIL。本轮执行已结束，默认覆盖保持失败时的内容。

候选是 67 个明确公开文件构成的最小合同隔离目录。候选不代表完整 fresh checkout，也不提供五客户端原生加载、hook、权限或委派证据。完整复制路径、来源类别、当前 index 跟踪状态和 SHA256 见 `copy-manifest.json`。其中两个新增 checker 与父任务 audit 为明确选用的公开未跟踪文件。

## 隔离范围与执行器

独立根为 `C:\Users\lyh\AppData\Local\Temp\trellis-t04-missing-overrides-20261002-q7twazn_`，包含分离的 `tool`、`npm-cache`、`candidate` 与 `evidence`。本轮没有向候选复制 `.git`、原生平台目录、四个本机覆盖、当前 `.template-hashes.json`、私有文件、数据库、build 或依赖目录。

Node 为 `D:\GreenSoftware\node\node.EXE`，npm CLI 为 `C:\home\lyh\.npm-global\node_modules\npm\bin\npm-cli.js`。采用保存的 lock：`fd71957d4e2f20706bf8499f8cbe91e188acabb86fd94ff78febd06e94c4cfa7`。59 个节点包含工具根与 58 个依赖；实际 58 个依赖版本全部与 lock 匹配，安装后 lock 字节未变。获取只运行一次 `npm ci`，使用两个不同路径的空 npmrc，以及 `--global=false --ignore-scripts --no-audit --no-fund`。完整 argv 与原始日志见每步收据。

隔离 CLI/core 均为 `0.7.0-beta.3`。所选 CLI 入口 SHA256 为 `dba1db2ed1d34f7d1e151d69303f14ce54aeb9f9e5dab60dc125aca692ab519a`。版本检查与 init 使用同一绝对 Node 和 CLI 入口。版本门通过后，在候选目录执行 `init --claude --codex --grok --kimi --omp --skip-existing -y`。

## 独立退出码

| 步骤 | Native exit | 秒 |
| --- | ---: | ---: |
| init 前合同 checker | 0 | 0.0709 |
| npm ci 首次获取 | 0 | 11.0181 |
| 所选 CLI 版本检查 | 0 | 0.3540 |
| init 前环境诊断 | 0 | 4.7846 |
| 五平台隔离 init | 0 | 0.7432 |
| init 后合同 checker | 1 | 0.0784 |
| init 后环境诊断 | 0 | 4.7409 |
| 候选忽略规则检查 | 0 | 0.0400 |

Python driver 实际退出 1；PowerShell 观察 driver 退出 1、意图退出 1；exec 外层实际观察退出 1。外层耗时 22.6333 秒。`observer.json` 绑定 exec session `29005`、completion chunk `108e7c`，并分别列出 native、driver 和 outer 结果。原始外层收据未被派生 observer 改写。

## 四个可选覆盖

init 前四项均 missing、未选用、未复制。init 后四项均由固定 CLI 生成默认内容。`generated-overrides/` 保存真实字节；`overrides-before.json`、`overrides-after.json` 保存逐项状态及哈希。此场景没有选用本机覆盖，因此不声明本机覆盖字节保留通过。

| 默认生成文件 | SHA256 |
| --- | --- |
| `.codex/config.toml` | `9f2d20e28f0bc9c886312eca3ad3bba41533ef4615aaaafe25e98152302267bb` |
| `.kimi-code/skills/trellis-implement/SKILL.md` | `fb1fa4c36fea58437d20f8a43f0028d1976333bc5bbb4487de1e9c32c48e6e8d` |
| `.kimi-code/skills/trellis-check/SKILL.md` | `882c7a2e5fe966de7511c30bab4d5409e7c9dcf2872ea48d1a8f14e6bc6f2bdb` |
| `.kimi-code/skills/trellis-research/SKILL.md` | `b5c0e7a7282d0b133e3700d2815f893e38adc89c817c61e49f7025c66b58a995` |

## 首次合同失败

init 后 checker 报告 7 项错误，原始 stderr 及其哈希保留于 `contract-after-init.*`，派生逐行提取见 `contract-error-extraction.json`。

- Kimi check 缺少遵循 AGENTS.md、只读审查不得修产品、批准任务与文件范围三项授权标记。
- Kimi check 缺少即使 dispatch 指定路径仍需检查目标一致性，以及向主会话确认目标两项标记。
- Codex config 已有 `[agents] max_depth = 1`，但缺少 V1/V2 适用范围及 prompt guard 不构成 hard sandbox 两项说明。

Kimi 默认 check 内容还明确要求直接 self-fix，并按 dispatch 路径优先解析目标；这些默认内容与候选强化合同缺少对应限制。没有执行该角色，也没有据此修改权限或产品。默认覆盖未被修补，本机覆盖未复制到候选。环境诊断仅报告入口可用、版本和四覆盖状态，不能覆盖合同 checker 的失败。

## 资产与保护边界

init 新生成 218 个文件。全部新路径/hash 见 `generated-assets.json`，候选完整前后清单见 `candidate-*-manifest.json`。7 个必要平台资产存在，11 个具名平台资产/覆盖均命中候选 `.gitignore`；本检查使用只读 `git check-ignore --no-index`，没有在当前仓库或候选创建 Git 提交。

67 个复制文件在 init 前后全部未变；67 个当前公开来源文件在执行前后哈希也一致。原保护清单的 9 个文件，以及当前 index、HEAD 和 branch 前后相同。对全局 config、项目 trust 和私有文件仅记录命令的无修改边界；没有为证明未写入而新增读取这些内容。保护证据不声明全盘逐字节比较。

本轮没有修改产品源码、当前四覆盖、Git index、HEAD/branch 或当前 template manifest。研究产物仅新增在 T04 research 与本轮唯一隔离根。没有全局安装、登录、trust、PATH、commit、archive、push 或客户端会话操作。

本轮未从最小候选运行产品完整 `just ci` 或文档构建，也未用候选静态资产关闭 T06 或产品 native 验收。AC/meta 与后续修复由主会话结合独立审查协调；本轮保留 INIT_PASS 与 CONTRACT_FAIL 两个结果。

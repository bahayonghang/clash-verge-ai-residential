# OMP 18.4.4 原生 task 路线准备

日期：2026-09-30。状态：LOCAL SOURCE REVIEW ONLY。没有运行 OMP 推理、task 或全局配置查询。

主会话要求准备独立的正常原生 child 路线。该路线在执行前选定，与无 task、无 extension 的 parent-only 路线分开。正常路线不因为先前 parent 失败而扩大参数重试。

## 本机来源

已安装 package.json 的 name 为 @oh-my-pi/pi-coding-agent，version 为 18.4.4。包位置由已核验 omp.ps1 包装器中的相对入口确定：包装器先用同目录 bun.exe，缺失时调用既有 PATH 的 bun.exe，再把原 argv 转给 node_modules/@oh-my-pi/pi-coding-agent/dist/cli.js。包装器以 LASTEXITCODE 退出。没有安装、替换运行时或修改 PATH。

以下路径均相对该已安装包：

| 文件 | 行号与本次核对内容 |
| --- | --- |
| src/tools/builtin-names.ts | 1–25：task 是原生工具名，另列 read/bash/edit/write 等 |
| src/tools/index.ts | 583：task 构造 TaskTool；628–636：没有显式工具列表时使用正常工具发现；798–810：task 仍受当前深度策略限制，默认从允许的内建工具集合构造 |
| src/task/index.ts | 541–543：名称 task，approval=exec；661–670：实际 schema 随当前 plan/isolation/batch/effort/eval 设置构造；816–818：从当前 cwd 与 effective extension roots 发现角色 |
| src/task/types.ts | 70–100：单次和 batch schema；119–164：动态构造，单次含 agent/task/solutionSpace，batch 含 context/tasks；不能以一份静态类型替代当前实际 schema |
| src/task/discovery.ts | 4–16、76–109：项目 .omp/agents 参与发现且优先于用户、extension、bundled 角色；实际发现仍待运行事件 |
| 项目 .omp/agents/trellis-research.md | 6–7：角色已有 read/write/bash/find/search/web_search 与 pi/task 模型选择器；本轮不修改 |

## 预先选定的命令

通过现有 PowerShell 包装器，使用此前 help 已核验的参数：

    omp.ps1 --mode json --no-session --no-title --no-prewalk --no-lsp --no-pty -p <统一 child prompt 单个 argv>

捕获器使用 pwsh -NoLogo -NoProfile -NonInteractive -File 调用已记录的绝对包装器路径。没有 shell 拼接 prompt。正常 child 路线不指定 tools、approval-mode、plan、model、extension 或新配置；沿用现有配置。没有新增 yolo、plan-yolo、trust、sandbox bypass 或安装参数。

这组命令没有调用 child 的静态保证。当前策略可能过滤 task，角色发现可能失败，执行审批可能拒绝。父会话必须按实际暴露的 schema 派发一次既定 trellis-research；单次 task 或 batch 形式都只允许一个 child。字段名称、必填字段和 Active task 首行的位置以本次 schema 为准。不得切换任务深度、角色、模型或审批配置补齐。

纯输出 child 禁止文件写入。既定角色的研究持久化合同不在此合成协议的验收范围。已有原生实现的 headless yolo、角色工具列表及 user approval policies 的证据沿用 smoke-prerequisites.md:124–176；这些资料不证明本次强制只读，也不自动扩大本次授权。

默认 extension 发现可能在正常启动发生；只记录本次真实事件。未读全局配置、未新增信任或插件。出现新 trust 请求或权限拒绝时保留并停止该路线，不自动批准。

## 来源身份

| 文件 | SHA256 |
| --- | --- |
| package.json | a68e81ba5969937bfdc5e19896d42ff1ce474c2ab4809d0a791a56bb60870031 |
| src/task/types.ts | adb0a963417f149456650a53056fe5982532a2b3b9e3e394f51f4ed2b8e7cc9a |
| src/task/index.ts | 02b2b6e619188a9109d342bb19a383a4509f204f0ff34c8558b9239cede83cdd |
| src/task/discovery.ts | 57d11ef632fd04e041a4fb8fc0e50ac7bbc378c5948a1870a7ec067ca23106c7 |
| src/tools/index.ts | edc1cbc3a93655c7f664abfc166635984d1a653d773100429ed3f2ba7de0c195 |
| src/tools/builtin-names.ts | d47464a6a913c1222dd50a8ffc54470579dbb69f06323ca0a799a36836adc5f1 |

该文件可以用作 child readiness 的 installed_source_schema 依据。实际工具暴露、role 调用、模型、权限与完成事件仍必须由运行收据提供。

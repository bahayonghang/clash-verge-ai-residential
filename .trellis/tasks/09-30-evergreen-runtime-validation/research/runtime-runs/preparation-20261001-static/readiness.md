# T06 静态准备复核

日期：2026-10-01（America/Chicago）。状态：PREPARED_NOT_RUN。

本轮仅读取公开合同、T06 研究记录及捕获器源码，核对五个已有入口的存在性，并列 Codex app bin 文件元数据。只哈希小型捕获器文件；没有哈希外部大文件、生成候选快照、运行 help、测试、构建或客户端。T05 正式负载结束信号仍待主会话下达。

## 准备修订

旧准备收据对应捕获器 d44c0d1e，后续去敏三例对应 6ab03f26；本轮初读为 9198bb1e。旧收据仅证明各自版本与各自合成范围。两份旧收据均不证明真实客户端、hook、extension、child 或实际模型。

旧 Codex app 入口缺失。主会话明确授权在 T06 research 内记录现存入口并修捕获器。现存入口由文件元数据定位；版本、help、哈希、V1/V2 与实际模型尚未验证。新绑定文件保留历史缺失，没有改父任务 `tool-versions.json`。

捕获器增加逐工具入口状态、OMP 依赖状态、冻结后逐工具身份比较。任何一个入口失败会保存该工具 BLOCKED，后续独立工具仍可记录。源码保护变化与原有捕获停止条件继续停止整批。本轮的定向回归验证器只用模拟 capture/snapshot，尚未执行。

## 原生命令与范围

固定 argv 仍为 `runtime-capture-plan.md` 的五行，不增加 model、yolo、trust 或新权限参数。Claude restricted parent 与带 Agent 的 child 分开。Codex 使用 read-only、never、ephemeral；新版本尚待核对。Grok 使用既定 plan 候选和 8 回合上限。Kimi 使用 --plan 与 coder + role skill。OMP parent 关闭 extension 并只提供 read/grep/glob；child 使用预先批准的正常 task 路线。

parent 和 child 均使用固定公开输入，分别启动新会话。child 必须走项目既定路线，仅派发一次，纯输出、不落研究文件。语言拒绝只支持 language_refusal；没有独立权限事件时 enforcement 保持 UNVERIFIED。CLI version/help、源码 schema 和 readiness 文件仅作为准备依据。

冻结保护包括 tracked/index 状态、公开新增文件、任务文档、批准的 ignored harness 资产及七份业务 skill。T06 runtime-runs 输出排除，其它任务研究记录仍受保护。HEAD 只表示基线提交；工作树候选以 source_snapshot_id 标识。必须先暂停其它代理对保护范围的写入，才能串行执行。

## 执行前待办

1. 收到 T05 正式负载结束信号；确认无其它代理继续修改保护文件。
2. 对现存 Codex 入口做有界 --version/help，保存旧入口失效，记录当前 hash/version 与参数差异；主会话审核。
3. 对最终捕获器执行语法、50 个 private_path 案例、去敏和逐工具阻断定向回归；保存本轮 synthetic 收据。失败按原样保留并修本研究执行器。
4. 主会话审核最终捕获器、绑定、prompt 与 child readiness，再 freeze 当前 dirty candidate。不要把 HEAD 当作未提交候选身份。
5. 主会话下达绑定 snapshot 的 parent GO；五工具串行，保存首个错误。child readiness 与 child GO 单独处理。
6. 逐事件审查模型、真实读取、实际 planning/权限、native child、hook/extension/pull、语言拒绝与保护范围差异。没有证据的字段保留 UNKNOWN/UNVERIFIED。

Hosted 只读查看既有 runs。未提交候选没有 exact-SHA hosted 证据；不 push、不开 PR、不触发远端 workflow。WebView、托盘、真实控制器、凭据与长期 soak 的未验证状态不变。

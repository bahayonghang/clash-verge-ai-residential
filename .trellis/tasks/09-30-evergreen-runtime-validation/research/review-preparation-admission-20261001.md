# T06 准备准入独立静态复审（2026-10-01）

结论：PREPARATION_CHECKS_PASS_RUNTIME_NOT_RUN。新入口元数据、当前捕获器身份、91 项定向合成检查、3 次本机 Python 合成捕获和 10 行 plan 收据一致；本轮未发现需要修复的准备范围问题。版本/help 和 synthetic 成功不关闭 T06 AC1/AC3，也不构成任何客户端推理 PASS。

## 当前入口与身份

复读 preparation-admission-20261001T131945Z-9fc25d6c/review-input.md，并沿索引检查 identity、verification、native-synthetic 与 admission 收据。当前 de8a38d2100ae498/codex.exe 实测 version 为 codex-cli 0.159.2，version/help/exec-help 分别 exit=0，inference_started=false。入口大小 324702000 B、hash34549ded6e2aee87c911c62d025e52e26c488683d0f489cd68f756baef1a6df6 与 runtime binding 相等；旧 c6fe 入口缺失。没有在容量负载期间重复哈希大 exe 或运行客户端。

真实 help 包含 exec、-c key=value、sandbox read-only、ephemeral 和 json；全局 help 包含 approval never。当前计划使用 -c approval_policy="never"，无 model/yolo 追加。相同版本文本不证明旧新二进制相同；新入口证据独立保存，未继承旧 V1/V2。live model/backend 保持 UNKNOWN，verified_for_runtime=true 只为元数据准入。

命令 receipt 保留原 stdout/stderr 字节 hash，保存文本明确标为 redacted decoded output。三条文本在换行规范化后的 UTF-8 hash 与 receipt 相符；保存文件不声明为原字节副本。redactions=[]，stderr 为空。

当前 runtime-capture.py 窄 hash 独立核对为392b2f4c211c03f0bb9479dcbb5e6804660a8b3a82f31c618ef814d28036cf27，验证器为1e33d36a0f017dfe71c412c4ddb7b08376975440307b201b6f77b089b8b25a2c；均匹配所有本轮收据。binding 文件窄 hash 与 admission 的f207ec1da389173b38726338b222f93ed917efdc7dfa13a54c4dce0ccc93bed9 相等。

## 准备覆盖与停止语义

验证器真实运行 receipt exit=0，stdout 中91项均pass。按源码与记录计数：50 private_path、3 普通文本不误删、2 合成 auth-header 去敏、10 单 prompt 与10无模型覆盖断言、4逐工具阻断场景共16断言。该覆盖范围为定向 synthetic，不扩展为全部真实秘密或客户端行为证明。

四个阻断场景包括 missing Codex、changed Codex identity、missing OMP dependency、unverified Codex；每个验证五份独立 receipt 与其余工具继续范围。源码 execute 先核对冻结候选与 GO，再按工具保存阻断。候选变化、敏感输出、超时、输出上限、捕获错误和退出等待超时仍停止整批；单工具入口/依赖失败不丢弃其余工具。

admission 记录两源码 compile通过、parent/child各5行NOT_RUN、Popen/subprocess.run禁止启动guard通过、无真实snapshot、candidate manifest不存在。main 的 plan 路径只创建bindings/argv并打印NOT_RUN，与无启动guard收据一致。没有用plan成功证明真实角色可用。

3份真实本机Python合成capture逐份核对：normal exit0/stop=null；auth exit7/CLIENT_REPORTED_BLOCKER；timeout exit0/TIMEOUT/termination_requested=true。timeout的exit0没有覆盖TIMEOUT。本轮3例验证capture正常/错误/超时收口；去敏覆盖来自91项验证器。事件输出保留normalized_redacted_events表示和接收端跨流顺序边界，不声明绝对无孤儿进程或权限硬隔离。

## 待真实执行的条件

等待 raw_fold 范围回退与最终产品门完成，停止保护文件的并行写入，再freeze当前dirty candidate。HEAD不能替代未提交候选身份。execute需匹配该snapshot的GO，child还需reviewed_by_main、同snapshot readiness及可追踪schema依据。入口/依赖重新核对由正式freeze执行；本次没有freeze或GO。

实际模型、权限、planning状态、角色发现、child生命周期、hook/extension/pull、语言拒绝与前后候选差异待五客户端真实事件。T05容量期间不启动推理或重复准备测试。

验证：仅静态JSON/源码/小文件身份核对PASS；本轮Lint、TypeCheck、Tests/clients/builds均未运行。已有synthetic和metadata运行按对应收据引用，不将静态复审列为新动态PASS。

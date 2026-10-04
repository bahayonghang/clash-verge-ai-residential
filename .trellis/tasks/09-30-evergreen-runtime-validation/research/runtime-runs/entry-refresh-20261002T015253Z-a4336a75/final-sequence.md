# T06 最终执行准备

日期：2026-10-01（America/Chicago）；本批元数据收据 UTC 为 2026-10-02。状态：PREPARED_STOP_WRITING。没有 freeze，没有客户端推理。本轮仅刷新已知入口的 version/help 和 Codex app bin 身份，不读取全局私有配置，不扩大来源、模型、账户、权限或信任。

## 当前入口

| 工具 | 本轮实测版本 | 元数据退出 | 模型与原生 schema |
| --- | --- | --- | --- |
| Claude Code | 2.1.286 | version/help 均 0 | UNKNOWN，待真实会话 |
| Codex app CLI | 0.159.2 | version/help/exec-help 均 0 | live V1/V2、实际模型 UNKNOWN |
| Grok Build | 1.0.46，build 2765805b9442 | version/help 均 0 | UNKNOWN，待真实会话 |
| Kimi Code | 2.0.0 | version/help 均 0 | coder 是否获准与实际模型待真实会话 |
| OMP | 18.4.10 | version/help 均 0 | task 动态 schema、角色发现与模型 UNKNOWN |

当前 Codex app 入口为 a51e250fa15c740a/codex.exe，SHA256 为 fcd5eafefb4ff4a607f244e099e0974f66e17966b6ffda6948de2ef3a7a79530。旧 c6fe 与此前准入 de8a 路径均已不存在。当前绑定保留两个历史来源，新增 current_version_observations；父 tool-versions.json 未改。四个未变路径的捕获字段 recorded_version_output 仍是历史记录；当前版本以本批 entries.json 及对应原生 version 收据为准，不将该字段解释为本次实际模型。

本轮 binding SHA256 为 62fe054d7be453b0501833aafd6bf33fafe2001bb6a20da31937c75b4e492c20。捕获器仍为 392b2f4c211c03f0bb9479dcbb5e6804660a8b3a82f31c618ef814d28036cf27，源码未变，既有 91 项 synthetic 和独立准入复审仍仅证明该准备版本。

## Parent flags

本轮真实 help 支持既定参数，没有增删 flags。Claude 使用 restricted、strict-mcp-config、plan、permission-prompts none、Read/Glob/Grep、no-session-persistence、stream-json、verbose 和 include-hook-events。Codex 使用 exec、sandbox read-only、approval_policy=never、ephemeral 和 json。Grok 使用 permission-mode plan、disable-web-search、streaming-messages-json、max-turns 8。Kimi 使用 plan 与 stream-json。OMP 使用 json、no-session、no-title、no-prewalk、no-lsp、no-pty、no-extensions、read/grep/glob 与 approval-mode write。

没有 model override。argv 表达的是启动请求；planning、权限、模型、hook/extension 生效状态须由真实事件证明。help/version 不填客户端 PASS。

## 执行顺序

1. 主会话已将 current task 切到 T06。等待所有角色 STOP_WRITING、检查完成和主会话明确 parent GO。当前不因任务指针变更提前执行。
2. 按既定执行器运行 plan，再 freeze 整个当前 dirty candidate，记录 source_snapshot_id、HEAD、branch、index、公开文件和批准 ignored assets。freeze 前保存入口真实身份；缺失或变化按工具保留 BLOCKED。HEAD 不替代工作树候选。唯一 manifest 在动态运行前创建。
3. 使用同一 source_snapshot_id 与 parent GO 执行 --phase parent --harness all。顺序为 Claude、Codex、Grok、Kimi、OMP，每工具单独新进程，默认 180 秒，阶段进程预算 15 分钟。动态时仅 runtime-runs 下保存收据，不修改其它受保护文件。
4. parent 结束先反馈逐工具退出、停止原因、原始事件位置和保护 diff；不得自动进入 child。实际模型、读取、负向语言拒绝和未知项按各自证据记录。
5. 主会话另行审核 child readiness 和下达 child GO。采用 reviewed_local_prerequisite 时明确旧版本 schema 仅为历史准备；本次实际工具 schema、角色发现、许可、调用、Active task 首行和 child 生命周期均由当前新会话证明。
6. child 同样按五工具串行，每工具只派一个既定研究 child，纯输出，不写研究文件。Claude Agent、Codex 项目 research 的当前原生 backend、Grok spawn_subagent、Kimi coder + role skill、OMP task。不能替换 explore/临时角色、切模型或扩大权限。角色持久化不由纯输出协议验证。

OMP 18.4.4 的 installed-source/schema 记录不得证明 18.4.10；此前 Claude/Grok 版本的权限源码和发现记录也不转移。本次缺 schema、角色或实际许可时原样保存 BLOCKED。Codex 版本仍为 0.159.2，但二进制 hash 改变，旧 V1 enabled/V2 disabled 不转移为 live backend 结论。

## 停止与保护条件

单工具缺入口/依赖、身份变化、native 不支持、认证、额度、trust 或权限错误保存首份收据，不通过购买额度、登录、安装、自动信任、bypass、换模型或更宽参数继续该路线。普通客户端非零退出可继续下一独立工具；不吞掉其它工具的收据。

保护源码、公开新增文件、index、HEAD 或 branch 变化即停批。敏感输出、捕获故障、输出上限、超时及退出等待超时也停批。退出 0 不覆盖 TIMEOUT 或主动终止。保存去敏事件、原始捕获字节内存 hash、真实退出、PID、时间与停止原因；Job 只为尽力清理，不提供硬权限或绝对无残留保证。

准备产物、语言拒绝、parent-only 显式 pull 和无 diff 均不证明 OS 强制、原生 child、hook/extension 或产品安装态通过。T05 matrix/capacity、hosted exact-SHA、WebView、托盘、控制器、凭据与 soak 的边界保留。任何运行结果都不自动关闭父任务。

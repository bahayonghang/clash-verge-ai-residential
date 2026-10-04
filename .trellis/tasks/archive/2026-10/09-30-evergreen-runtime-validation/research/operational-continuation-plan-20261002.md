# T06 基本运行恢复方案

日期：2026-10-02，America/Chicago。用户明确要求「竞争负载不用管，继续实施其他内容，能正常运行就行」。本轮恢复基本运行路径，不要求等待其它仓库负载结束。原性能阈值与完整 runtime 验收条款保持独立状态。

## 范围与执行

主会话负责范围与运行准入；实施代理修复任务内捕获与启动方案，检查代理独立核对。允许新增本任务 research/runtime-capture-operational.py、operational-parent-prompt.txt、operational-verification.py、operational-* 方案与收据及 operational-runs/ 输出。原捕获器、prompt、快照、首失败与历史客户端收据不修改。产品、四本机覆盖、全局配置、账户、模型、信任及权限不修改。

先从已安装 CLI 的 help 或公开源码核对 Kimi 2.0.0 参数。现有 `--plan -p` 明确互斥。修复必须保留只读意图和原生限制；不能通过自动移除权限限制或切模型取得成功。已安装同版本 exe 内嵌公开 JS 提供 ACP plan 路线。本轮另批准新增 research/operational-kimi-acp.py 与必要合成 fixture，执行 initialize、session/new、session/set_mode(plan)，收到 plan 确认后才发送 session/prompt。所有权限请求均取消或拒绝，不切模式、不派 child。协议字段以本 exe 精确源码位置为依据。driver 事件交给既有脱敏及停止机制，不保存原字节。driver、入口与 hash 纳入保护快照；独立审查通过前不启动真实 ACP 推理。若本机接口不能保持 plan，记录具体阻断。

新基本运行 prompt 只读取具名公开文件并返回简短结果，不派发 child、不写文件、不触碰私有路径。明确 Windows 文本读取使用 UTF-8。每工具单独新会话、独立限时、记录实际入口版本/hash和去敏事件；最小读取成功只证明基本调用，不补全旧五文件规划协议、模型、权限、hook 或 child 验收。

保留已有 secret 识别、敏感输出停止、限量、超时、前后保护快照与独立退出语义。OMP credentialId 首失败保持；不得为获得成功放宽捕获器。准备与受影响合成回归通过、独立审查后，主会话给新候选运行准入。运行期间保护文件停止修改。每条路线至多一次基本调用；实际认证/API/权限/额度失败保留，不安装或自动重试、换账户或模型。

本轮合同允许基本调用在竞争负载下运行；记录这种条件，不把该运行称为正式性能验收。提交、归档与远端交付仍不在本轮范围。

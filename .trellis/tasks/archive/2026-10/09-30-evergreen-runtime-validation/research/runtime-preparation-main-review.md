# T06 捕获准备主会话审查

日期：2026-09-30。状态：PREPARED；尚未 freeze 或启动客户端。

主会话已核对 runtime-capture.py、两份统一 prompt、runtime-capture-plan.md、omp-native-entry-review.md 和 preparation-f3e7d09a 的合成验证收据。准备满足已批准的有界运行范围；实际执行等待 T05 最终候选稳定和正式性能负载结束。此结论不关闭 T06 任一运行验收项。

- 默认 plan 不启动推理。freeze 使用排他创建，保存公开文件、必要 ignored harness 资产、Git 元数据、执行器和 prompt 身份。execute 需要对应 snapshot 的主会话 GO。
- 每工具独立新进程，顺序执行，默认 180 秒。保存逐工具退出、失败原因和标准化去敏事件。原始字节只在内存计算哈希；去敏流不标为原始流。
- parent-only 与 child 为预先选择的不同方案。OMP child 采用已核验的正常原生 task 入口；不因 parent 失败扩大参数。源码 schema 依据只用于准备，实际暴露、委派、角色、模型和权限须由运行事件证明。
- 五工具纯输出 child 不验证研究角色持久化。遇到角色合同冲突、认证、额度、trust 或权限拒绝时保留具体结果，不改模型或放宽配置。语言拒绝与 OS 强制分别记录。
- Job 负责本次进程生命周期的尽力清理。Popen 后绑定存在竞态，不能据此声明硬沙箱或绝对无残留。
- 除明确 runtime 输出外的保护文件发生变化时停止。无 diff 仅支持受保护范围的前后内容相同，不证明无瞬时写入、仓库外副作用或全文件系统零写入。

准备验证包含 17 项检查及 3 个合成 Python 进程：正常退出 0、认证错误退出 7、超时标记独立于退出码。全部仅为 synthetic 证据。真实五客户端、原生 child、hook/extension、实际模型与候选差异检查仍未运行。

运行前先按已批准顺序启动 T06，保存最终候选和 child readiness；运行期间暂停其它代理对保护文件的写入。父/子运行阶段各自确认结束后再回写报告。Hosted exact-SHA、WebView、真实控制器、凭据和长期安装态验收继续独立。

后续静态复查发现 private_path 未覆盖私有配置的备份后缀。runtime-capture.py 已补备份/压缩副本分类，runtime-private-path-cases.json 已准备字符串正反例；本次修订尚未执行验证。旧 17 项收据不覆盖新版本。T05 独占测量期间不运行新增测试或客户端；结束后须先通过准备复核，再 freeze 和 GO。

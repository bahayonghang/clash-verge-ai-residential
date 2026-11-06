# Ubuntu / FlClash 验收

日期：2026-11-06。实际 Ubuntu 24.04.4 LTS x86_64，FlClash 包 0.9.2+2026102401；GUI 与核心进程运行中。本次按用户明确授权使用运行 GUI 的桌面账户，未把另一个 SSH 用户的配置当作有效证据。

| 层次 / 项目 | 状态 | 证据 / 缺口 |
| --- | --- | --- |
| 环境、安装、运行身份 | PASS | OS、包与进程只读检查 |
| 共享 JS 合同 | PASS | 同机 Node regression 104/104；不是 QuickJS 原生运行证明 |
| 生成配置结构 | PASS（限抽样字段） | 白名单投影见下表 |
| 实时核心版本 / 加载身份 | UNVERIFIED | 本次支持的现有本地 HTTP 控制器不可用；未启用接口或连接生产 IPC |
| 真实业务及 DNS 五类路径 | UNVERIFIED | 无目标请求与实时链路、DNS 日志的关联 |
| 显式本地代理 HTTPS 连通 | PASS（限传输） | 4 个公开样本均完成证书验证，AI 根路径返回 404、国内外对照 200；不证明选路 |
| 进程归因 | FAIL（对“需要查找进程”要求） | find-process-mode=off；不等于域名规则失效 |
| TCP DNS 接管 | UNVERIFIED | 只有显式 any:53，缺少 tcp://any:53；需实际请求验证 |
| 顶层 IPv6 与当前推荐配置一致性 | FAIL | top-level ipv6=true，而脚本警告/现有设置建议要求 false；dns.ipv6=false 不覆盖此项 |
| 实际 IPv6 泄漏 / TUN / WebRTC | UNVERIFIED | 开关差异不证明发生泄漏；没有对应路径观测 |
| 持久化 | UNVERIFIED | 未刷新、重启或切换 Profile |

## 白名单投影

rule 模式；1 个单成员 AI-家宽 组，61 条 AI 规则；TUN 开启；dns.ipv6=false，fake-ip，respect-rules=true，direct-nameserver-follow-policy=true；无 fallback 或 fake-ip-range6。ChatGPT、Anthropic API、Grok 登录和正则 DNS 均绑定住宅；普通解析绑定其他出站，bootstrap DIRECT；私网为 system。没有输出任何真实 endpoint 或认证数据。

本机 /etc/resolv.conf 指向 systemd-resolved stub，但没有观测桌面应用实际解析器/备用路径；不能据此 PASS 私网 DNS。

## 实际 HTTPS 请求

在用户授权的桌面账户内，读取现有代理端口，仅经本地 HTTP CONNECT 对 api.anthropic.com、auth.grok.com、www.wikipedia.org、www.baidu.com 执行 HEAD /，不发送服务 API key、Cookie 或登录数据。前两项 HTTP 404、后两项 200，全部 TLS 证书验证成功。没有直连回退，没有切换 Profile；这只证明样本的 TLS/HTTP 可用性，不证明实际住宅出口、DNS 路径、TUN 或完整登录/推理。

## 环境元数据缺口

系统包与 GUI 进程已核实；包来源渠道、桌面会话类型（X11/Wayland）、Profile 原始来源、实际接入模式、选中脚本及字节身份均为 UNVERIFIED。SSH 桌面账户读取不等同于 GUI 操作或 QuickJS 原生执行。源码版本与来源 SHA 见宿主合同。

## 结论边界

这份记录不是整体兼容 PASS。没有进行生产配置修改、订阅刷新、节点切换、客户端重启、系统 DNS/TUN 变更或 browser 策略变更。没有把原生核心/文件证据当作真实业务、DNS 出站或 WebRTC 证明。报告不记录网络地址、账户或凭据。

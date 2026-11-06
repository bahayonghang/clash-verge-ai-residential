# 运行证据分层与平台矩阵

## 证据模型

每份报告写：OS/发行版、客户端版本、核心版本、脚本版本/提交、接入模式、配置来源、文档版本、检查时间、观察来源、每项状态/证据/限制、恢复方法、下次复查条件。
状态仅 PASS、FAIL、UNVERIFIED、NOT_APPLICABLE；无法接触环境用 UNVERIFIED，不以“不适用”隐藏缺设备。

四层证据各自成立：
- 脚本静态输出：规则与 DNS 合同；
- 原生核心接受：语法、资源、依赖能力；
- GUI 最终有效配置：覆写及持久化；
- 实际请求：命中链、DNS 出站、TUN/IPv6/WebRTC 和功能。

## 平台具体化

Windows：按实际网卡区分物理/TUN，Get-DnsClientServerAddress 等仅提供配置线索；公网系统 DNS、DHCP 缺失或浏览器独立 DoH 不直接判合格。
Ubuntu：先识别实际 resolver 管理方式；适用时检查 resolvectl status、/etc/resolv.conf 的链接及 NetworkManager 配置。127.0.0.53 是 stub 线索，不是实际上游证明。容器/WSL 的内核实验不是 Ubuntu 桌面 TUN 验收。
FlClash macOS system_dns coordinator 的源码结论不套到这两平台；macOS/Android 后续独立设计。

## 样本

复用实际启用开关与已有正负样本。exact/suffix/regex 要分别覆盖，Cursor 索引默认关闭应作为负样本；要验证开启态可用隔离虚构配置，不修改私人 TOML。
DNS 五类与真实业务分别测；公共 DNS 检测站只作为当前测试路径的线索。WebRTC 用用户认可端点，记录 ICE 类型和实际路径；零候选/未工作端点不得判 PASS。
最终配置若恢复 IPv6 fake-IP，要先核对地址池和 ULA 规则；发现风险记录，不自动复制参考的拒绝规则。

## 恢复与隐私

不把现用代理关掉做对照，不同时起两个 TUN。任何后续获准的变更先保留本地前像并提供离线恢复方式。公开研究仅留脱敏摘要，不复制订阅、认证、真实网络地址或完整浏览器配置。

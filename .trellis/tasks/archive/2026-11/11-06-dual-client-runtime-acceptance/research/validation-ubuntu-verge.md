# Ubuntu / Clash Verge Rev 验收

日期：2026-11-06。用户指定的实际 Ubuntu 24.04.4 LTS x86_64，kernel 6.17.0-22；已安装 Verge 2.5.8，随附 Mihomo 1.19.32 linux amd64。不是 WSL/容器验收。本次候选公开脚本版本 5.11.0，根脚本未改。

| 层次 / 项目 | 状态 | 证据 / 缺口 |
| --- | --- | --- |
| 环境与安装身份 | PASS | SSH 只读 os-release、包信息与核心 -v |
| 共享 JS 合同 | PASS | 实际 Ubuntu Node v24.14.1 执行最终 regression，104/104，通过退出码 0 |
| 原生核心隔离验证 | PASS | 4 个生成配置 -t 与 36 次 TXT / 两级回环 SOCKS 断言通过 |
| GUI 最终生效配置 | UNVERIFIED | Verge GUI 未运行；未读取历史配置充当当前运行证据 |
| 实际业务链和 DNS 五类路径 | UNVERIFIED | 未取得 Verge 当前加载/请求/解析器关联证据 |
| TUN、IPv6、WebRTC | UNVERIFIED | 远端当前运行 FlClash；未切换应用或操作远端浏览器 |
| 刷新/重启持久化 | UNVERIFIED | 未修改或重启 |

当前 SSH 用户的 /etc/resolv.conf 指向 systemd-resolved stub；这只证明 stub 配置线索，不能推出实际上游或无 DNS 外发。Ubuntu FlClash 由另一桌面账户运行，已在用户授权后用该账户完成其配置投影；不混用账户证据。

## 环境元数据缺口

系统包版本已核实；包来源渠道、桌面会话类型（X11/Wayland）、当前 Profile 来源、脚本挂载/启用及实际绑定字节均为 UNVERIFIED。本次为 SSH 检查，未取得 Verge GUI 会话。源码版本与来源 SHA 见宿主合同，不代替绑定身份。

## 结论边界

这份记录不是整体兼容 PASS。没有进行生产配置修改、订阅刷新、节点切换、客户端重启、系统 DNS/TUN 变更或 browser 策略变更。没有把原生核心/文件证据当作真实业务、DNS 出站或 WebRTC 证明。报告不记录网络地址、账户或凭据。

# Windows / Clash Verge Rev 验收

日期：2026-11-06。Windows 11 build 26300；已安装 Verge 2.5.10，随附 Mihomo 1.19.33 windows amd64。本次候选公开脚本版本 5.11.0，根脚本内容未改。

| 层次 / 项目 | 状态 | 证据 / 缺口 |
| --- | --- | --- |
| 安装身份 | PASS | 安装信息与 exe 版本一致；独立核心 -v 输出已读取 |
| 共享 JS 合同 | PASS | Windows regression 104 项通过；包括单/双参数、无 CommonJS 的入口 |
| 原生核心隔离验证 | PASS | 当前脚本生成 4 个开关组合，4 次 -t 与 36 次实际 TXT 查询和 SOCKS 选路断言通过 |
| GUI 最终生效配置 | UNVERIFIED | Verge GUI 未运行；未为验收启动或替换当前运行的 FlClash |
| 实际 AI/负样本业务链 | UNVERIFIED | 未向生产 Profile 发业务请求并关联实际出站 |
| DNS 五类生产路径 | UNVERIFIED | 回环探针只覆盖隔离 DNS，不是公网 DoH/TLS 或系统 resolver |
| TUN、IPv6、WebRTC | UNVERIFIED | 未取得 Verge 接管与目标浏览器路径证据 |
| 刷新/重启持久化 | UNVERIFIED | 未执行会影响现用代理的切换或重启 |

源代码合同见宿主任务 research/host-contracts.md；原生探针隔离条件见其 validation.md。先前 VM 通过不是 Boa 实机证明。

## 环境元数据缺口

安装版本已核实；安装器渠道、当前 Profile 来源、脚本挂载/启用状态与实际绑定字节均为 UNVERIFIED。Windows 桌面可访问，但本次未启动 Verge GUI。源码版本与来源 SHA 见宿主合同，不能代替实际绑定身份。

## 结论边界

这份记录不是整体兼容 PASS。没有进行生产配置修改、订阅刷新、节点切换、客户端重启、系统 DNS/TUN 变更或 browser 策略变更。没有把原生核心/文件证据当作真实业务、DNS 出站或 WebRTC 证明。报告不记录网络地址、账户或凭据。

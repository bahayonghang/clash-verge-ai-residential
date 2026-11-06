# 双客户端分流、DNS 与 WebRTC 运行验收

## Goal

让 Windows / Ubuntu 上的 Clash Verge Rev 和 FlClash 各自有可复现的验收与复查记录，验证“最终生效和真实路径”，不把单元测试当作联网证明。

## Background

依赖：11-06-dual-client-host-compat 的宿主合同与候选脚本。父任务研究 §3–7 为证据基线。
当前 docs/dns-and-leak-model.md 已解释 fake-IP、SOCKS 域名转发、正则 DNS 与 fail-closed 限制；扩展这些内容，不新建网络管理系统。
用户明确首期两客户端 × Windows/Ubuntu，macOS 和 Android 后续。

## Requirements

| ID | 可观察要求 |
| --- | --- |
| V1 | 四组合独立记录 OS/客户端/核心/脚本版本、接入模式、配置来源与来源文档 SHA |
| V2 | 分开核验脚本输出、内核接受、宿主最终配置和真实请求；同时使用 AI 正样本与非 AI 负样本 |
| V3 | DNS 分住宅 AI、非 AI 海外、国内、私网和节点引导路径；记录浏览器独立 DNS、备用路径及系统 resolver 的实际来源 |
| V4 | TUN、IPv4/IPv6、WebRTC 与需要的语音功能独立验收；每项为 PASS/FAIL/UNVERIFIED/NOT_APPLICABLE 并附证据 |
| V5 | 持久化检查与用户可离线执行的恢复方法齐备；脱敏、明确直接观察/用户回传/未测，保留缺口 |
| V6 | 记录规则及客户端更新后的复查触发条件，不自动订阅、定时更改或发布规则 |

## Acceptance Criteria

- [ ] VA1 → V1：四份组合记录完成，缺设备/浏览器/权限明确列出；所有适用项未通过前，不宣称四组合兼容验收完成。
- [ ] VA2 → V2：至少覆盖启用的 exact/suffix/regex AI 样本、认证资源边界、市场/下载/遥测负样本、LAN 与原 Profile 兜底；包括加载失败、GUI 覆写或订阅刷新导致失效的识别方法。
- [ ] VA3 → V3：五类查询分别有证据或 UNVERIFIED；system 不是内网证明，HTTP IP/ECS/解析器地区不是 DNS 路径证明；fake-IP 回答与 SOCKS 远端解析单列。
- [ ] VA4 → V4：零 ICE 候选不得自动 PASS；srflx/relay 结合预期策略和实际路径，不要求所有 IP 与 HTTP 字面一致。未经授权不关闭代理做对照，不安装扩展或改变浏览器策略。
- [ ] VA5 → V5：刷新与重启只在获准时执行；未执行记 UNVERIFIED。报告不含真实 endpoint、凭据、Cookie、完整订阅、个人路径或原始业务日志；恢复无需依赖 Agent 在线。
- [ ] VA6 → V6：复查机制记录版本变化和规则来源变化；不承诺防封、固定 IP 或节省比例，不把零观测等同死规则。
- [x] VA7：文档构建及适用产品 gate 通过；实机结果与这些 gate 分列，不能互相替代。见 research/validation-summary.md。

## File Scope

docs/dns-and-leak-model.md、docs/troubleshooting.md、docs/multi-profile.md 及 docs/en 对应文件；新建 docs/client-validation.md、docs/en/client-validation.md；docs/.vitepress/config.mjs 仅在确认实际导航配置需要时添加入口。运行结果只写本任务 research/ 下脱敏 Markdown。
优先扩展现有文档。scripts/check-residential-proxy.js 可作为有明确授权时的辅助探针，但本任务不改其代码，不把它的显式代理测试升级为 TUN/DNS/WebRTC 证明。

## Out of Scope / Approval

不改产品算法、控制器、凭据、系统 DNS/TUN、浏览器策略或本地 TOML；发现兼容缺陷回到宿主子任务/父任务，不在验收中隐式修复。未单独授权的安装、重启、代理切换和公网实验不执行。
用户已于 2026-11-06 批准实施；宿主合同已确定，最终实机验收必须针对其实际候选版本。

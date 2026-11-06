# 双客户端验收汇总

日期：2026-11-06。实施已批准；宿主合同、测试、文档完成，四组合完整运行验收尚未完成。公开脚本版本 5.11.0，工程基线 b9c43a756e0f3fbf9d5276affec555901fdea46c；根脚本没有修改，AI-only 路由、域名、默认开关保持原样。

## 已交付

- 核对固定版本 Verge Boa 与 FlClash QuickJS 调用、后置字段覆写及 DNS 所有权，见宿主任务 research/host-contracts.md。
- 新增 3 个无 CommonJS 全脚本入口测试，覆盖单/双参数及相关隔离合同；不以 V8 测试冒充真实宿主执行。
- 中英文接入、DNS、排错与 client-validation 验收流程完成。明确 residentialDns:false 站点有意分离业务与 DNS；恢复步骤可离线查看。
- 已建立 Windows/Ubuntu × Verge/FlClash 四份报告；未知元数据、未测路径及失败项明确保留。

## 检查结果

| 检查 | 结果及边界 |
| --- | --- |
| just ci | PASS：root 267、monitor 前端 300、Rust 550 + 3 通过、6 ignored；含版本、fmt、clippy、构建、敏感信息检查 |
| Windows / 实际 Ubuntu regression | 各 104/104 PASS |
| 原生 Mihomo 隔离验证 | Windows 1.19.33、Ubuntu 1.19.32 各 4 次配置检查和 36 次实际 DNS / 两级 SOCKS 断言 PASS；不证明 GUI 或生产路径 |
| 最终 just docs-build | PASS，审查修订后重跑 |
| 最终 npm run check:secrets / git diff --check | PASS |
| 三个 Trellis 任务上下文验证 | PASS |
| 独立审查 | 宿主及运行文档均完成；修正 DNS 例外说明，未改产品 |

## 实机观察

| 组合 | 已观察 | 尚未完成 |
| --- | --- | --- |
| Windows / Verge 2.5.10 | 安装身份、共享 JS 合同、随附核心隔离探针 | GUI 未运行，最终加载、真实链路与持久化未验证 |
| Ubuntu / Verge 2.5.8 | 实际 Ubuntu 24.04.4、共享 JS 合同、随附核心隔离探针 | 未切换当前 FlClash，GUI、实际路径与持久化未验证 |
| Windows / FlClash 0.9.2 | GUI 住宅组、生成配置白名单投影、4 个 HTTPS 样本 | 实时绑定与核心版本、实际路由/DNS/TUN、持久化未验证 |
| Ubuntu / FlClash 0.9.2 | 授权桌面账户的生成配置投影、4 个 HTTPS 样本 | 同上；远端浏览器未验证 |

每端 HTTPS 样本：api.anthropic.com、auth.grok.com 根路径 HEAD 返回 404；www.wikipedia.org、www.baidu.com 返回 200；全部证书验证成功。使用现有本地 HTTP CONNECT，无直连回退，无服务 API key/Cookie。只证明 TLS/HTTP 连通，不证明推理/登录、住宅出口或 DNS。

Windows Ego 的官方 Trickle ICE 样本完成，host=1、srflx=1、relay=0、prflx=0，并存在 701 错误。未申请媒体权限，未记录候选地址。未关联 srflx 与预期出口，故不能判无泄漏；任务空间已关闭。

## 具体差异与下一步

1. 两端 FlClash 生成配置 find-process-mode=off，与需要进程归因的设置要求不符；不能据此判定域名规则失效。
2. 两端缺少显式 tcp://any:53，只有 any:53；TCP DNS 实际接管仍须请求/路径证据，不能直接判泄漏。
3. Ubuntu 顶层 ipv6=true、dns.ipv6=false；顶层值与当前建议不符，是否存在实际 IPv6 泄漏尚未验证。
4. 没有可直接复用的受支持本地 HTTP 控制器；未开启它或接入生产 IPC。FlClashCore 不接受通用 -v/-h/-t 参数。
5. 下一阶段需明确允许的客户端切换/重启窗口，再逐端核对脚本绑定和最终加载、关联真实正负业务与五类 DNS 路径、测试 TCP DNS/IPv6/WebRTC，最后执行刷新/重启复查并恢复原客户端。若需更改设置或启用观测接口，应先提出具体变更和恢复步骤。

本次没有修改生产配置、订阅、节点、系统 DNS/TUN、浏览器策略或本地 TOML/JS，没有记录主机地址、账户、密码或原始配置。凭据仅用于用户授权的登录，未写入仓库。远端隔离测试临时目录已清理。

宿主和运行子任务保留 in_progress，父任务保持协调用途；不归档、不提交、不声称四组合完整兼容通过。Ego Browser 获取网页的用户偏好已写入个人 memory note。

## 用户决定

2026-11-06，用户明确选择“暂不切换，保留验收缺口”。本轮不再推进客户端切换、刷新/重启复查；上述缺口保留，后续仅在用户重新授权相应操作后继续。已有检查结果与交付物不受影响。

# Windows / FlClash 验收

日期：2026-11-06。Windows 11 build 26300，FlClash 0.9.2+2026102401，GUI/核心进程运行中。脚本研究版本见宿主任务；没有证明当前绑定脚本字节与本次候选完全相同。

| 层次 / 项目 | 状态 | 证据 / 缺口 |
| --- | --- | --- |
| 安装和运行身份 | PASS | exe 版本、进程确认 |
| 共享 JS 合同 | PASS | Windows regression 104/104；不是 QuickJS 原生执行证据 |
| GUI 住宅组 | PASS | 只读打开已有窗口，Proxies 页显示 AI-家宽，唯一显示成员 家宽-SOCKS5 |
| 生成配置结构 | PASS（限抽样字段） | 见下表；不等于全配置审计或实时加载 |
| 运行核心版本 / 实时有效配置 | UNVERIFIED | 自定义 FlClashCore 无独立 -v/-h/-t；生成配置未提供本次支持的现有本地 HTTP 控制器，未开启它 |
| 实际业务链、DNS 五类 | UNVERIFIED | 没有业务请求与实时连接/解析出站关联 |
| 显式本地代理 HTTPS 连通 | PASS（限传输） | 4 个公开样本均完成证书验证并返回 HTTP，见下文；不替代路由/TUN/DNS |
| TCP DNS 接管 | UNVERIFIED | 缺少显式 tcp://any:53；不能仅此断定泄漏，需实际 TCP 查询和路径证据 |
| 进程归因 | FAIL（对“需要查找进程”要求） | 生成文件 find-process-mode=off；不等于 AI 域名路由失败 |
| TUN/IPv6/WebRTC 完整路径 | UNVERIFIED | 开关及 DNS 配置不是实际接管证据；Windows Ego 配置档取得 ICE 样本，但没有实际出站关联 |
| 持久化 | UNVERIFIED | 没有刷新或重启 |

## 白名单配置投影

只在本地解析最终生成文件，输出白名单布尔值/计数，不输出 nodes、endpoint、密码、secret 或完整 YAML。

| 字段 | 观察 |
| --- | --- |
| mode / AI 组 | rule；1 个 AI 组，唯一成员家宽 SOCKS5 |
| AI 规则 | 61 条指向 AI-家宽 |
| 顶层 IPv6 / DNS IPv6 | false / false |
| TUN / UDP DNS / TCP DNS 显式项 | true / any:53 存在 / tcp://any:53 不存在 |
| DNS 模式 / respect-rules / direct-follow-policy | fake-ip / true / true |
| fallback / fake-ip-range6 | 均无 |
| 普通解析 / bootstrap | 有其他显式出站绑定 / DIRECT 绑定 |
| ChatGPT、Anthropic API、Grok 登录、正则策略 | 均有住宅绑定 |
| 正则 provider | 存在 |
| LAN / home.arpa | system，实际上游未验证 |

窗口随后不可用；重新列举未找到可操作窗口，停止 UI 操作，没有反复激活或更改连接开关。

## 非破坏真实请求与浏览器样本

通过已存在的本地 HTTP 代理建立 HTTPS CONNECT，对 api.anthropic.com、auth.grok.com、www.wikipedia.org、www.baidu.com 执行 HEAD /；前两项返回 404，后两项 200，全部 TLS 证书验证成功。请求不携带服务 API key、Cookie 或登录数据；不会绕过代理回退直连。404 仅证明该根路径得到 HTTP 响应，不证明登录或推理功能；显式代理请求不证明 TUN 接管或 DNS/业务选路。

Windows Ego Browser 当前配置档使用官方 Trickle ICE 页面与其默认 stun:stun.l.google.com:19302。首次指针点击被页面拦截，经重新观察后用按钮键盘动作采集成功：完成状态 Done，1 host、1 srflx、0 relay、0 prflx，同时页面含 701 错误。没有申请麦克风/摄像头权限，未改变浏览器策略、代理或 TUN，没有记录候选地址；任务空间随后关闭。

该轮只证明此配置档取得了候选，不能判无泄漏：701 与成功候选并存，需要分路径定位；尚未把 srflx 关联到预期代理出站，未验证其他浏览器配置档、通话功能或完整 IPv6。不能将它解释为零候选通过或所有 STUN 路径通过。

## 环境元数据缺口

exe 版本和现有 GUI 已观察；安装器渠道、Profile 原始来源、实际接入模式、选中脚本绑定及字节身份仍为 UNVERIFIED。生成配置有项目特征不能证明候选脚本已挂载。源码版本与来源 SHA 见宿主合同。

## 结论边界

这份记录不是整体兼容 PASS。没有进行生产配置修改、订阅刷新、节点切换、客户端重启、系统 DNS/TUN 变更或 browser 策略变更。没有把原生核心/文件证据当作真实业务、DNS 出站或 WebRTC 证明。报告不记录网络地址、账户或凭据。

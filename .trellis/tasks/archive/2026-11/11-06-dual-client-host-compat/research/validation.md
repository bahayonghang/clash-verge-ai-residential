# 宿主兼容验证记录

日期：2026-11-06。未修改根脚本、渲染器、路由名单、默认开关或默认 fixture。

## 实施与质量门

- 新增 3 项完整脚本无 CommonJS 入口测试；Windows 与实际 Ubuntu Node v24.14.1 分别 104/104 通过。
- canonical `just ci` 通过：root 267/267、前端 300/300、Rust 550 + 3 通过、6 ignored；版本、fmt、clippy、构建与敏感信息扫描成功。
- 初次沙箱运行在 npm.ps1 授权检查处受阻，未进入产品测试；使用正常主机环境运行同一 `just ci` 后通过，未更改执行策略或降低 gate。
- `just docs-build` 通过，VitePress 2.0.0-alpha.19。
- 独立审查记录见 `check.md`；此文件不预判尚未完成的审查结果。

## 已安装环境

| 平台 | GUI | 独立核心 / 证据 |
| --- | --- | --- |
| Windows 11 build 26300 | Verge 2.5.10，未运行 GUI | 随附 Mihomo 1.19.33 windows amd64 |
| Windows 11 build 26300 | FlClash 0.9.2+2026102401，运行中 | 自定义 IPC 核心，不使用 -v/-h/-t；独立核心版本未核实 |
| Ubuntu 24.04.4，kernel 6.17.0-22，x86_64 | Verge 2.5.8，未运行 GUI | 随附 Mihomo 1.19.32 linux amd64 |
| 同一 Ubuntu | FlClash 0.9.2+2026102401，运行中 | 通过用户授权的桌面账户完成生成配置白名单投影；核心版本未核实 |

Ubuntu SSH 使用用户指定的现有主机；最初 SSH 账户与 GUI 账户不同，未把错误账户的配置作为证据。随后用户授权桌面账户，只读投影成功。报告不保存主机地址、用户名或认证材料。

## 隔离原生核心探针

复用归档任务 `10-04-regex-residential-dns/research/check-core.cjs`，会话临时副本仅增加 Windows 隐藏子进程及临时目录清理边界检查。

Windows Mihomo 1.19.33 和 Ubuntu Mihomo 1.19.32 均通过：
- 从当前公开脚本输出生成 Vertex/Cursor 四种开关组合；每份原生 -t 返回 0。
- 每份执行 9 次实际 TXT 查询，共每平台 36 次；正向主机按开关进入住宅 resolver，6 个负样本保持机场 resolver。
- 住宅解析观察到 Airport → Home → residential resolver 两级 SOCKS CONNECT，普通解析走 airport resolver。

所有服务只监听回环随机端口；虚构 SOCKS 拒绝公网及测试范围外目标；TUN 关闭，不改系统代理/DNS，不读取真实配置。仅替换 resolver 地址并移除 geosite 依赖，保留脚本生成的 provider/policy/链路。每次核心及套接字在 finally 中关闭。

初次 Windows 沙箱探针无法到达自己的回环控制器，记录为环境受阻；正常主机隔离运行随后成功。这个结果不是 GUI 加载、公众 DoH TLS、供应商 UDP、完整登录或 WebRTC 验收。

## 运行交接

Windows/Ubuntu 的 FlClash 生成文件中均观察到单成员住宅组、61 条目标为 AI-家宽的规则、住宅关键 DNS 策略、本地正则 provider、独立 bootstrap 和无 fallback。只是配置文件证据，不证明实时加载。
两者 `find-process-mode=off` 且只有 UDP any:53，没有显式 tcp://any:53；Ubuntu 顶层 ipv6=true，Windows 为 false，两者 dns.ipv6=false。运行任务负责记录配置偏差与未验证路径，不在本任务中自动修改设置。

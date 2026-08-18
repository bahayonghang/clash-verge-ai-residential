# Clash Verge Rev 控制器接入初始调研（2026-08-18 本机实测）

> 状态：本文件保留最初的本机环境与 API 样本；公开契约、named pipe 鉴权 / ACL、动态管道和 `chains` 稳定性结论以同目录 `controller-compatibility-audit.md` 的固定版本核验为准。

## 本机环境事实

- 进程：`clash-verge.exe`（UI）、`verge-mihomo.exe`（内核，Services 会话）、`clash-verge-service.exe` 均在运行。
- 配置目录：`%APPDATA%\io.github.clash-verge-rev.clash-verge-rev\`
  - `verge.yaml`：应用设置，`enable_external_controller: false`。
  - `clash-verge.yaml`：运行时合并配置（权威来源）。实测顶层键：
    - `external-controller: ''`（TCP 控制器关闭）
    - `secret: <有值，已打码>`
    - `external-controller-pipe: \\.\pipe\verge-mihomo`
  - `config.yaml`：基础模板（`external-controller: 127.0.0.1:9097`），会被运行时覆盖。自动发现只能以 `clash-verge.yaml` 为准。
- 实测 `127.0.0.1:9097` 无监听（netstat 无记录，curl 连接失败）。
- 结论：本机当前唯一接入通道为 Windows 命名管道 `\\.\pipe\verge-mihomo`。用户在 Verge 设置中开启"外部控制器"后 TCP 才可用。监控应用需同时支持两种传输。
- 后续兼容性审计已通过该管道完成无 Bearer 的 `/version`、`/connections` 与 `/traffic` 只读请求；本机 v2.5.2 可接入，但固定管道名不是未来 Verge 版本的稳定契约。

## mihomo RESTful API（外部控制器）

- 鉴权：HTTP 头 `Authorization: Bearer <secret>`；WebSocket 亦可用查询参数 `?token=<secret>`。
- `GET /version` → `{"version":"...","meta":true}`，用作连通性测试。
- `GET /proxies` → `{"proxies":{"<名称>":{"name":"…","type":"Selector|URLTest|Fallback|LoadBalance|Relay|Socks5|…","now":"…","all":[…]}}}`。
  含 `all` 字段的条目为代理组；节点（如 `家宽-SOCKS5`，type `Socks5`）无 `all`。用于设置页拉取可选统计目标（组与节点都可选）。
- `GET /connections` → 一次性快照，结构与 WS 帧相同，用于首屏。
- WebSocket `/connections`（约每 1s 推送一帧）：

```json
{
  "downloadTotal": 0,
  "uploadTotal": 0,
  "connections": [
    {
      "id": "uuid",
      "metadata": {
        "network": "tcp", "host": "api2.cursor.sh",
        "destinationIP": "…", "destinationPort": "443",
        "sourceIP": "198.18.0.1", "sourcePort": "…",
        "process": "…", "processPath": "…"
      },
      "upload": 0, "download": 0,
      "start": "ISO8601 时间",
      "chains": ["家宽-SOCKS5", "AI-家宽"],
      "rule": "DomainSuffix", "rulePayload": "api2.cursor.sh"
    }
  ]
}
```

- 帧内 `upload`/`download` 为该连接自建立起的累计字节；帧只含当前活跃连接，连接关闭后从帧中消失，最后一帧至关闭之间的字节不可获取（≤1s 粒度误差，账本为下界近似）。
- 本机样本中 `chains[0]` 为具体出站、末位为外层代理组；mihomo 公开 API 未承诺该顺序，分类不得依赖首尾，只保留原始顺序用于诊断。
- WebSocket `/traffic` → `{"up":0,"down":0}`，全局瞬时速率（B/s）。
- `DELETE /connections/{id}` → 关闭指定连接。

## 命名管道接入要点（Windows）

- `tokio::net::windows::named_pipe::ClientOptions::open(r"\\.\pipe\verge-mihomo")` 返回实现 `AsyncRead + AsyncWrite` 的流。
- WebSocket：`tokio_tungstenite::client_async("ws://127.0.0.1/connections", pipe_stream)` 可在任意异步流上握手；pipe 不附带 token，TCP 才按 secret 鉴权。
- REST（`/version`、`/proxies`、`DELETE`）：统一流上的 HTTP 实现需兼容 `Content-Length`、`chunked`、connection-close framing 与多段读取；`/proxies` 可达数百 KB。最初“手写最小解析器”的想法已被完整 v1 设计否决，实施应采用成熟 HTTP/1.1 实现。
- mihomo 官方文档与源码已确认：named pipe 不校验 `secret`，安全边界是 Windows ACL。管道请求不得携带真实 TCP secret；管道 access denied 与 TCP 401 必须使用不同状态。

## 仓库内相关事实

- 扩展脚本常量：`AI_GROUP = "AI-家宽"`、`HOME_PROXY_NAME = "家宽-SOCKS5"`（clash-verge-ai-residential.js:44-45）。
- 统计目标判定：用户多选的名称集合与 `chains` 求交集，非空即计入；首次运行默认勾选名称含「家宽」的组/节点。名称集合可手动补充，兼容用户改名或多个相关分组。

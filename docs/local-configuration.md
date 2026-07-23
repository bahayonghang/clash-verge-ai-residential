# 本地 TOML 配置与同步

公开的 `clash-verge-ai-residential.js` 必须始终保留 `xxx` 占位符。本地凭据使用 `clash-verge-ai-residential.local.toml` 保存，再由 `just render-local` 单向渲染为 `clash-verge-ai-residential.local.js`。这两个本地文件均被 `.gitignore` 排除，只有示例文件 `clash-verge-ai-residential.local.toml.example` 会进入版本控制。

## 前置条件

- Node.js 18 或更高版本。
- [just](https://github.com/casey/just) 命令行工具。

在项目根目录执行以下命令。Justfile 会在 Windows 使用内置 Windows PowerShell，在 macOS/Linux 使用 `sh`，因此 Windows 不需要安装 Git Bash 或额外的 Unix shell。

## 首次配置

仓库当前工作区已初始化 `clash-verge-ai-residential.local.toml`。新克隆的仓库请从示例创建它：

```powershell
Copy-Item clash-verge-ai-residential.local.toml.example clash-verge-ai-residential.local.toml
```

macOS/Linux 可使用：

```bash
cp clash-verge-ai-residential.local.toml.example clash-verge-ai-residential.local.toml
```

编辑本地 TOML：

```toml
[home_proxy]
name = "家宽-SOCKS5"
type = "socks5"
server = "residential.example.com"
port = 1080
username = "your-username"
password = "your-password"
udp = true
dialer-proxy = "🚀节点选择"
```

字段含义：

| 字段 | 说明 |
| --- | --- |
| `name` | 必须与公开模板的 `HOME_PROXY_NAME` 一致，目前为 `家宽-SOCKS5`。 |
| `type` | 当前只允许 `socks5`。 |
| `server` / `port` | 家宽 SOCKS5 主机与 `1-65535` 的端口。 |
| `username` / `password` | 认证信息；无认证服务必须将两个值都设为 `""`。 |
| `udp` | SOCKS5 服务支持 UDP 时设为 `true`。 |
| `dialer-proxy` | 本地机场 Profile 中实际存在的上游代理组或节点名。 |

TOML 的字符串使用双引号；用户名或密码中有双引号、反斜杠时需使用 TOML 转义。`#` 在引号内是值的一部分，在引号外开始注释。

## 生成本地脚本

每次修改 TOML 后执行：

```bash
just render-local
```

`render-local` 表示单向渲染，而非双向同步：它读取公开的 `clash-verge-ai-residential.js` 与本地 TOML，生成 `clash-verge-ai-residential.local.js`，不会修改公开模板或反向写入 TOML。将**生成的本地脚本**粘贴到 Clash Verge Rev 的全局扩展脚本中，然后刷新 Profile。

`just sync` 仍作为兼容别名保留，但新文档和自动生成文件均使用 `just render-local`。

同步会在写入前拒绝以下配置：缺少字段、未知字段、无效 TOML 字符串、非 SOCKS5 类型、端口超出范围、空上游名称，或 `name` 与模板保留名称不一致。错误会直接显示字段或行号，修正 TOML 后重新运行即可。

## 不保存凭据的模式

也可以保留 `server`、`username` 和 `password` 为 `"xxx"`，并在每个 Clash Profile 中预置同名的 `家宽-SOCKS5` 节点。运行时脚本会复用 Profile 中该节点的 endpoint 和凭据。无认证 SOCKS5 则在 TOML 中把 `username`、`password` 都改为 `""`。

无论使用哪种模式，都不要提交本地 TOML、生成的 `.local.js`、生成 Profile 或未脱敏的连接日志。

## 校验

运行全部公开模板检查与回归测试：

```bash
just ci
```

`just ci` 等价于 `npm run ci`。它不会读取或上传本地 TOML；本地生成脚本也会被模板安全扫描排除，以免凭据干扰公开仓库检查。完成后仍应在 Clash Verge Rev 中确认 `家宽-SOCKS5.dialer-proxy` 能解析到实际机场组，并从 Connections 验证 AI 请求命中 `AI-家宽`。

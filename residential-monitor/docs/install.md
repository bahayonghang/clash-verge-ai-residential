# 安装

v1 使用 NSIS current-user 安装包。不要求管理员权限。安装包不偷偷启用登录自启动。

## 构建

```text
just monitor-build
```

产物位于 `residential-monitor/src-tauri/target/release/bundle/nsis/`。该命令只出包，不安装。

`just monitor-build` 与 `just tinstall` 会先把 `residential-monitor/package.json` 的 `version` 写入 `tauri.conf.json`、`Cargo.toml`、`Cargo.lock` 和 `package-lock.json`。NSIS 安装包名、关于页和 Windows 文件版本都读这些文件。只改 `package.json` 不会改变已有安装包。

`just tinstall` 会在本机以 NSIS `/S /D=%LOCALAPPDATA%\ResiWatch` 静默执行 current-user 安装。安装包文件名必须包含当前 `package.json` 版本，目录里残留的旧版本安装包不会被选用。若 `residential-monitor` 正在运行，安装前先结束该进程。若注册表里上次安装目录在 `%TEMP%` 下或旧产品名目录，安装后会把 `data\` 迁到新目录。安装结束后配方退出，不启动应用。未再确认前不要运行。

## 稳定标识

| 项 | 值 |
|---|---|
| identifier | `io.github.bahayonghang.residential-monitor` |
| AUMID | 与 identifier 相同 |
| 产品名 | ResiWatch |
| 二进制 | `residential-monitor` |
| 安装目录 | `%LOCALAPPDATA%\ResiWatch` |
| 凭据 target | `io.github.bahayonghang.residential-monitor/controller` |
| 自启动参数 | `--background` |

## 安装后检查

- Start Menu 出现产品名。
- Start Menu 与任务栏图标是彩色屋顶+柱状图，边缘清楚，不是放大后的灰房子。
- 关闭窗口只隐藏到托盘。
- 第二实例只聚焦现有窗口。
- WebView2 由安装包 bootstrapper 处理。
- 数据位于用户 LocalAppData 下的 identifier 目录。

正式通知、登录自启动和 Credential Manager 真机写入属于安装态验收，开发态通过不能替代。

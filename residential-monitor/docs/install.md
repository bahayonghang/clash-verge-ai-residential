# 安装

v1 使用 NSIS current-user 安装包。不要求管理员权限。安装包不偷偷启用登录自启动。

## 构建

```text
just monitor-build
```

产物位于 `residential-monitor/src-tauri/target/release/bundle/nsis/`。该命令只出包，不安装。

`just tinstall` 会在本机以 NSIS `/S` 静默执行 current-user 安装。安装结束后配方退出，不启动应用。未再确认前不要运行。

## 稳定标识

| 项 | 值 |
|---|---|
| identifier | `io.github.bahayonghang.residential-monitor` |
| AUMID | 与 identifier 相同 |
| 产品名 | 家宽流量监控 |
| 二进制 | `residential-monitor` |
| 凭据 target | `io.github.bahayonghang.residential-monitor/controller` |
| 自启动参数 | `--background` |

## 安装后检查

- Start Menu 出现产品名。
- 关闭窗口只隐藏到托盘。
- 第二实例只聚焦现有窗口。
- WebView2 由安装包 bootstrapper 处理。
- 数据位于用户 LocalAppData 下的 identifier 目录。

正式通知、登录自启动和 Credential Manager 真机写入属于安装态验收，开发态通过不能替代。

# Windows 安装态与自启动验证证据

验证日期：2026-08-29

## 命令直接采集

- `just tinstall` 成功完成 release 构建、NSIS 打包和 current-user 静默安装。
- 安装包：`ResiWatch_0.3.0_x64-setup.exe`，SHA256 `9DF18EB5D07EE29EA7D38A2C37E191B7F943AF2D39EC9F2965A30817C7A5E95D`。
- 安装目录：`C:\Users\lyh\AppData\Local\ResiWatch`。
- 主程序：`residential-monitor.exe`，20,185,088 bytes，ProductName `ResiWatch`，ProductVersion/FileVersion `0.3.0`，SHA256 `D86C8B106DA44F9C07239ECEA049D72C3BB85FCFAF49B4AE131B9E5BD9A14A63`。
- 同目录存在 `monitor-bench.exe`、`uninstall.exe` 和 `data\`；HKCU 卸载信息、开始菜单和桌面快捷方式均指向该安装目录。
- 安装配方按文档结束了安装前运行的 `residential-monitor.exe` PID 27172；安装后没有自动启动应用。
- 安装前后，HKCU Run key 均不存在 `ResiWatch`、`residential-monitor` 或 `io.github.bahayonghang.residential-monitor` 值，证明安装器没有隐式启用登录自启动。
- 原有三个 SQLite 文件安装后仍存在，安装后二次读取的 SHA256 稳定。安装前文件被运行中的应用独占且仍在增长，因此没有声称安装前后字节级完全一致。
- 在用户报告重新启用启动项后，最终命令回读确认 HKCU Run key 的 `ResiWatch` 值为 `C:\Users\lyh\AppData\Local\ResiWatch\residential-monitor.exe --background`，不存在其它候选名称；当时只有一个用户手动前台启动的 ResiWatch 进程，命令行不含 `--background`。

## 用户人工报告

- 用户于 2026-08-29 确认：安装版设置页启用后回读为开启；一次真实 Windows 登录没有弹出主窗口，托盘可用且只有一个 collector。
- 用户随后在设置页关闭并确认回读为关闭；下一次真实 Windows 登录没有自动启动应用。完成该关闭验证后，用户再次启用了启动项。

## 证据来源与边界

- 安装包、文件、注册表和最终进程状态由本会话命令直接采集。
- 两次登录周期、设置页 UI 回读与最后重新启用动作由用户人工执行并报告；本会话没有保存登录过程截图或录屏，不将其描述为命令采集或自动化证据。
- 当前任务范围内的 Windows 登录自启动验收已完成；跨已发布旧版本升级保留、卸载器自动清理仍不在本证据覆盖范围内。

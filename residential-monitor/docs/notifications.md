# 系统通知

告警触发与「告警页 → 测试通知」使用 Windows 系统通知（toast）。默认发送。

## 关闭方式

设置环境变量 `RESIDENTIAL_MONITOR_ALLOW_TOAST`，值为 `0` 或 `false` 时不发送系统通知：

```powershell
# 用户级关闭，重启应用后生效
[Environment]::SetEnvironmentVariable("RESIDENTIAL_MONITOR_ALLOW_TOAST", "0", "User")
```

关闭后告警中心内的事件记录不受影响，只是不再弹系统通知。

## 限制

- 当前无应用内静默入口。关闭系统通知只能用上面的环境变量。
- v1 只在 Windows 11 提供系统通知。其他平台 `测试通知` 会显示不可用原因。
- 系统专注助手（Focus Assist）或通知关闭时，用户可能看不到 toast，应用不另行提示。
- 开发态（`tauri dev`）下 toast 归属 PowerShell 的名称与图标；安装版构建才归属本应用。

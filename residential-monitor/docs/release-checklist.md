# Release checklist

发布前必须有同一候选提交的证据索引。本任务本身不创建或发布 GitHub Release。

1. C0–C4 独立验收可追溯。
2. `just monitor-check` 与 `just ci` 退出码 0。
3. `just monitor-c5-auto` 通过。
4. C0 基线 checksum 核验通过后，才做 NSIS 升级。
5. 安装态端到端、通知、Focus Assist、自启动需再确认后执行。自启动必须核对 `%LOCALAPPDATA%\ResiWatch\residential-monitor.exe --background`、真实登录后隐藏窗口/托盘/唯一 collector，以及关闭后下一次登录不再启动；未取得证据时标记 **UNVERIFIED**，不得以单测替代。自启动的安装/Run key 已于 2026-08-29 由命令采集，两个登录周期由用户人工报告；通知与 Focus Assist 仍未完成。
6. 三档 30 天库、13 个月高基数、10k×30 分钟、并发和 24 小时 soak 均实际运行，或对应 AC 记未通过。
7. 低空间 fail closed 有命令输出。
8. canonical installer、SHA-256、SBOM / 依赖清单绑定同一提交。
9. 签名或针对具体哈希的未签名例外。
10. 文档、Release notes、已知限制与候选一致。
11. 已发布 tag 下不得替换同名资产。

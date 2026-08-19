# 家宽流量监控

Windows 11 本地桌面应用。持续采集 Clash Verge Rev / mihomo 的全部连接事实，提供实时监控、历史报告、导出、告警、保留和备份恢复。

产品只提供观测下界。不把控制器 meter 与可归因观测总量混称为同一全局口径。缺口、未知和 C3 能力不支持不得写成零。

v1 只支持 Windows 11 NSIS current-user。无应用内自动更新。无 Windows Service。不发布 macOS / Linux。无遥测。

## 命令

```text
just tdev              开发态桌面壳
just monitor-check     子项目质量门
just monitor-build     生成 NSIS，不安装
just tinstall          构建并静默安装。运行中的应用会先结束。不启动应用。会改本机安装态，需再确认
just monitor-c5-auto   C5 自动硬化门。不含 30 天库、24 小时 soak、本机安装
```

## 文档

- [安装](docs/install.md)
- [首次配置](docs/first-run.md)
- [控制器兼容](docs/controller.md)
- [隐私](docs/privacy.md)
- [数据目录](docs/data-directory.md)
- [备份恢复](docs/backup-restore.md)
- [报告口径](docs/reporting.md)
- [覆盖与尾差](docs/coverage.md)
- [告警](docs/alerts.md)
- [故障排查](docs/troubleshooting.md)
- [升级与卸载](docs/upgrade-uninstall.md)
- [已知限制](docs/known-limits.md)
- [Release checklist](docs/release-checklist.md)

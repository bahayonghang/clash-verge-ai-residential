# 故障排查

| 现象 | 下一步 |
|---|---|
| TCP 鉴权失败 | 检查 loopback 与 secret，不要把 secret 贴进日志 |
| 管道访问拒绝 / 忙 / 不兼容 | 启用 TCP External Controller |
| 缺口显示未知 | 正常。不要改成 0 |
| 存储故障 / 未来 schema | 进入 Recovery Shell，使用经验证备份 |
| 备份或 VACUUM 提示空间不足 | 清理磁盘后重试。当前库应仍可打开 |
| 通知不可用 | 查看应用内告警中心 |
| 能力不支持 | 缩小时间范围或改用支持的维度 |
| SmartScreen | 未签名候选的预期行为。见 Release notes |

诊断导出后扫描不得出现 `bearer `、`password=`、`secret=`。

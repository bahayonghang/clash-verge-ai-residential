# 故障排查

| 现象 | 下一步 |
|---|---|
| 实时连接显示尚未配置 | 到设置页填写 loopback 地址并测试连接 |
| 实时连接显示当前没有活跃连接 | 控制器已连接。表空表示此刻没有活跃连接，不是故障 |
| 实时连接一直空且健康为已取消 | 断开后需托盘立即重连或再次测试连接，采集才会恢复取帧 |
| TCP 鉴权失败 | 检查 loopback 与 secret，不要把 secret 贴进日志 |
| 启动进入 Recovery 或采集反复失败 | 设置页或 Recovery 壳打开日志目录。路径为 `%LOCALAPPDATA%\io.github.bahayonghang.residential-monitor\logs`。不要把 secret 贴进日志 |
| 管道访问拒绝 / 忙 / 不兼容 | 启用 TCP External Controller |
| 缺口显示未知 | 正常。不要改成 0 |
| 存储故障 / 未来 schema | 进入 Recovery Shell，使用经验证备份 |
| 备份或 VACUUM 提示空间不足 | 清理磁盘后重试。当前库应仍可打开 |
| 通知不可用 | 查看应用内告警中心 |
| 能力不支持 | 缩小时间范围或改用支持的维度 |
| SmartScreen | 未签名候选的预期行为。见 Release notes |

诊断导出后扫描不得出现 `bearer `、`password=`、`secret=`。

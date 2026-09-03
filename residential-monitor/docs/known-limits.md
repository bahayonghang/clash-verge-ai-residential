# 已知限制

- 观测下界，不是代理商账单。
- 家宽有两种口径，不能当成同一集合。核算口径（`residential_tags` / `is_residential_target`）只匹配已配置 target 的精确节点名，写入 `primary_category_id`。实时筛选口径（`is_residential_filter`）在精确匹配之外还接受节点名含「家宽」。家宽页实时段用筛选口径，聚合段用核算口径。
- named pipe 尽力兼容，TCP 是稳定回退。
- 10,000 活跃短峰不是 30 天持续容量。
- C3 自动 DELETE 关闭。freelist 不是已释放磁盘。
- 未签名安装包会触发 SmartScreen。
- C0 升级基线安装包缺失。C5-AC5 未通过。
- 登录自启动的 Rust adapter、应用 commands、设置页与无副作用自动测试已经实现。2026-08-29 的命令采集已核对 current-user NSIS 默认不启用以及最终启动项路径/参数；用户人工报告已覆盖启用后的真实登录和关闭后的下一次登录。跨已发布旧版本保留和卸载器自动清理仍为 **UNVERIFIED**；自动测试不会写 HKCU 启动项。
- 完整 30 天 `A=50/250/1000`、13 个月高基数、24 小时 soak、安装态通知 / Focus Assist、代码签名和 GitHub Release 在本候选未完成。

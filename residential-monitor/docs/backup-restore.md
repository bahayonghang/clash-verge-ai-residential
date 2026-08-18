# 备份与恢复

- 使用 SQLite Online Backup，不复制热库文件。
- 目标先写 `.partial`，校验后再改名。失败不得留下被当成成功的资产。
- 恢复进入维护路径：验证候选 checksum、integrity、schema，再受控替换并前向迁移。
- 失败保留当前可用库。
- 低空间下 backup、restore、VACUUM fail closed。
- 不自动 VACUUM。用户主动 VACUUM 前检查约两倍数据库空间。
- secret 不随数据库备份。跨机恢复需要重新输入凭据。

# 隐私

- 无遥测，无远程内容，无 CDN。
- TCP secret 只存在于 Credential Manager 或当前进程内存。
- SQLite、日志、Channel、预览、导出、诊断和 Release 资产说明不得包含明文凭据。
- 诊断只含白名单字段。完整域名、IP、进程路径和凭据内容不得进入诊断包。
- 导出可对域名和进程路径脱敏。导出前扫描 `bearer ` / `password=` / `secret=`。
- CSP 只允许本地资源。capability 不授予文件系统、SQL 或 opener。

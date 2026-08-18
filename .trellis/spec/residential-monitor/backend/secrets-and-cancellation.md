# 密钥与取消

- secret 只存在于 Credential Manager 或当前进程内存。
- 日志、SQLite、Channel、错误、诊断和导出不得包含 secret。
- C1 使用 FakeCredentialStore；Windows adapter 属于 C2。
- 长操作必须可取消。SQLite 使用 interrupt / progress。

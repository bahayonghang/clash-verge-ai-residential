# 控制器兼容

- TCP External Controller 是受支持路径。secret 只通过 Authorization header 发送，不进入 URL。
- TCP 只接受 loopback。
- named pipe 是尽力兼容路径。不发送 secret。ACL 拒绝不是密钥错误。
- 固定管道名和 `chains` 顺序不是稳定契约。
- 私有 pipe 不兼容时引导启用 TCP。

## 状态

连接中、已连接、TCP 鉴权失败、管道访问拒绝、管道忙超时、端点不存在、协议不兼容、PID 不匹配、核心重启、睡眠缺口、存储故障。

Clash Verge 真机 named pipe 矩阵未在 C5 重跑。以 C0 `controller-profiles.json` 与自动化 fixture 为准。

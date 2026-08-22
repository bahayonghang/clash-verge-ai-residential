# 本机进程字段核对（2026-08-22）

核对对象：Clash Verge Rev `verge-mihomo` + 家宽流量监控开发态库。不含 secret、完整路径或域名。

## Clash 配置

- `Merge.yaml` / `clash-verge.yaml`：`profile.find-process-mode: always`（写在 `profile` 下）。
- 顶层 `find-process-mode`：文件中不存在。
- 运行中 `/configs`：顶层 `find-process-mode: strict`；`profile` 下无该键。
- TUN：启用；栈 `gVisor`。系统代理关闭。
- 扩展脚本 `ENABLE_AI_PROCESS_FALLBACK = false`，不改顶层查找进程。

## 当前连接快照

- 126 条：Tun/tcp 78、Inner/tcp 40、Tun/udp 8。
- `metadata.process` 与 `processPath` 均为空。

## 本机库（`%TEMP%\io.github.bahayonghang.residential-monitor\monitor.sqlite3`）

- 会话 79229；`process_id` 非空 1；字典仅 `mihomo`。
- 近 24 小时约 30843 个会话，进程已知字节 0。

## 结论

界面 always 未进入内核。`strict` 且无进程路由规则时，内核不会填写进程字段。历史空字段不能回填。

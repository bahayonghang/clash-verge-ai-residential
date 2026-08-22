# 查找进程与进程路由分开

家宽脚本默认要让监控读到进程 identity，但不把整进程流量送进家宽。扩展脚本把 Mihomo 顶层 `find-process-mode` 设为 `always`，且仅在 `routing.ai_process_fallback` 为 true 时才注入 `PROCESS-NAME` / `PROCESS-PATH`。Clash Verge 写在 `profile.find-process-mode` 下的值内核不用，脚本必须写顶层键。

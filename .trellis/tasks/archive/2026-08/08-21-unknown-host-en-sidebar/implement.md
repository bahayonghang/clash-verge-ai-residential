# 实施：父任务收口

1. `task.py start 08-21-en-sidebar-layout`，做完后 `trellis-check`，不 archive 直到父任务收口（或按子任务独立 archive）。
2. `task.py start 08-21-unknown-host-attribution`，做完后 `trellis-check`。
3. 父任务核对 AC-P1–P4：`just monitor-check`，中英侧栏与主机页。
4. 需要时把 identity 回退与 `__unknown__` 过滤写入 `.trellis/spec`（`trellis-update-spec`）。
5. 父任务 archive。

子任务互不改对方文件。顺序可对调；推荐先侧栏（范围小），再归因。

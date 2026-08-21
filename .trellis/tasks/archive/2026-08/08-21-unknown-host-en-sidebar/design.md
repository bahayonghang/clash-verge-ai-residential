# 设计：父任务边界

实现细节在子任务 `design.md`。父任务只约束跨子任务接口。

- 主机 identity 优先级与未知过滤是 `08-21-unknown-host-attribution` 的契约。侧栏不读报告 DTO。
- 侧栏排版不改 `ui_sidebar_width` 默认值；归因不改壳布局。
- 收口时在 220px 英文设置页与中文主机页各走一遍：品牌锁、未知行下钻、IP 行可辨认。

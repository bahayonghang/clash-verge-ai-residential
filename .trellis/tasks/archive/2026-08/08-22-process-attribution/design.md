# 设计：进程归因（父任务）

## 边界

父任务不改产品代码。两个子任务按包切开：

| 子任务 | 写入 |
|---|---|
| `08-22-process-lookup-observation` | `clash-verge-ai-residential.js`、回归测试、`docs/configuration.md`、`docs/local-configuration.md`、`CHANGELOG.md` |
| `08-22-process-page-capability` | `residential-monitor` 采集无关的查询/前端、`residential-monitor/docs/reporting.md` |

监控仍只读控制器快照。脚本不修改用户重点目标或家宽 SOCKS5。

## 数据流

```
Clash Verge Merge / 脚本
  顶层 find-process-mode: always
    → Mihomo 连接元数据 process / processPath
      → 监控 resolve_process_identity → process_id
        → 进程页排名 / 跨维下钻 / 字段归因
```

写在 `profile.find-process-mode` 的值不进入该链路。脚本必须写顶层键。

## 口径

- 进程页默认：时间窗内全部会话。
- 核算口径开关：`primary_category_id` 非空（任一重点目标）。筛选口径（节点名含「家宽」）不用于该开关。
- 字段归因与时间 coverage 仍分列。

## 子任务顺序

二者验收独立。建议先合脚本子任务，用户重载 Profile 后看实时连接进程列；监控子任务不依赖内核已经 always。

## 回滚

脚本回滚后，已设为 always 的运行中内核保持到下次配置生成。监控回滚后，已允许的进程未知过滤从查询面消失；已存储的 `process_id` 不变。

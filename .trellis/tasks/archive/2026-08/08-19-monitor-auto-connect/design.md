# 技术设计：启动即自动连接与监控

## 范围

本子任务只改启动 / 打开窗口的生命周期触发和其测试，不改变核算、采样频率、coverage、writer、凭据格式或前端统计。它依赖父任务共享研究，并与 `08-19-monitor-tray-status` 的托盘接线顺序协调。

## 策略

```text
AppFacade::boot
  ├─ NormalReady + persisted address -> collector_running=true, Connecting
  ├─ NormalReady + no address        -> collector_running=true, plan_tick idle / 未配置
  └─ RecoveryOnly                    -> no collector I/O

open_main_window / owner activation
  -> open_window()
  -> if NormalReady + persisted address + (Cancelled | paused): reconnect/resume
  -> show + focus

one collector_loop_tick
  -> plan_tick -> fetch_snapshot -> apply_tick_result -> publish Channel / tray
```

“持久地址”由 `AppFacade.settings.address` 经过现有 Rust `plan_tick` 校验定义；前端显示回退值不参与策略。冷启动不重复调用 reconnect，因为 boot 已把 owner collector 置为 running；只在打开隐藏窗口、手动断开或暂停后恢复状态。

## 单实例激活

沿用 `InstanceClaim` 和现有稳定 identifier。若当前 Windows owner 没有收到 second-instance 通知，增加同 identifier 命名的 Windows event：second instance `OpenEventW + SetEvent` 后退出；owner 在 Tauri setup 中启动一次等待任务，收到事件后调用同一个 `open_main_window`。事件任务不持有 `AppFacade` 长锁，不创建 collector，不负责文件 / 凭据 I/O。非 Windows 的现有 claim 路径保持；若并行托盘任务交付了等价 activation seam，则复用它并删除重复方案。

## 错误与安全

- 无地址：维持未配置，不访问 `127.0.0.1:9097`。
- 非 loopback / 解析失败：由现有 `plan_tick` / SettingsWorkflow fail closed。
- 鉴权、端点、协议、存储错误：复用已有 `SessionStatus`、`AppErrorDto` 和脱敏 action。
- `disconnect_controller` 后窗口保持打开不自动恢复；只有 reconnect / resume / reopen / cold start policy 触发。
- secret 仍只由现有 workflow 在短暂 Rust 内存 /凭据库中解析，不进入 activation event、日志或 Channel。

## 测试设计

- `c2::desktop`：open / focus / second instance 不拥有 collector；打开窗口动作本身不创建第二循环。
- `c2::facade`：有地址 reconnect 离开 Cancelled，无地址不自动补默认地址；manual disconnect + reopen 恢复。
- `c2::collector`：已有有效配置 tick 取帧；无地址、Cancelled、RecoveryOnly skip；重复 reopen 不改变唯一采集器数量。
- Windows activation：可测试的纯命名 / 状态 seam；真实通知区 / installed single-instance 仍列为手工证据，不用 fixture 冒充。

## 回滚

移除 open/reopen policy 和 activation listener 即回到现有显式断开 / 托盘重连；不回退数据库、凭据或 C1 collector 代码。

# 启动即自动连接与监控

## Goal

应用启动时依据确认后的配置策略自动进入连接与持续采集，无需用户先访问设置页或点击“测试连接”；首启、后台启动、失败、暂停和手动断开均保持可诊断且不创建第二条采集循环。

## Confirmed Facts

- `collector_loop_tick` 是唯一持续采集循环，约 1 Hz 调用 `/connections`，并在正常分支、collector running、非 shutdown、非 `Cancelled`、地址合法且为 loopback 时取帧。
- `test_controller` 会保存设置并只取一帧；它不能替代持续 collector。
- `AppFacade::boot` 正常分支初始为 `Connecting`，持久 `ControllerSettings.address` 默认为空；前端回退显示的 `127.0.0.1:9097` 不等于已保存配置。
- 订阅必须走既有 Tauri Channel `subscribe_monitor` / `resync_monitor`，WebView 不拥有采集生命周期。

## Requirements

- 已保存且通过现有 Rust 校验的 loopback 地址在前台与 `--background` owner 启动后自动进入采集；无需前端模拟点击测试按钮。
- 启动协调只唤醒既有 collector / reconnect 状态，不创建新 Tokio loop、WebView interval 或第二条 writer。
- 窗口保持打开时执行 `disconnect_controller` 后保持 `Cancelled`；冷启动、从托盘 / 第二实例重新打开主窗口，或显式 `reconnect_now` / `resume_collector` 才恢复采集。
- 连接结果继续通过 Rust health / live projection 与 Monitor Channel 发布，前端不根据按钮成功自行伪造“已连接”。
- 地址无效、非 loopback、凭据不可用、鉴权失败、端点缺失、协议不兼容、暂停和 RecoveryOnly 均 fail closed，并保留下一步动作。
- 启动自动连接不得改变 secret 持久化、日志脱敏、SQLite、诊断或导出边界。
- 首次无持久地址时保持未配置，不尝试前端回退显示的 `127.0.0.1:9097`。

## Acceptance Criteria

- [ ] 有效持久地址 fixture 启动后第一次 collector tick 会请求 `/connections`，状态与 live projection 更新，无需调用 `test_controller`。
- [ ] 前台和 `--background` 使用同一 owner collector；第二实例、WebView 重建与多次订阅不会增加采集循环数量。
- [ ] 手动断开后窗口保持打开期间停止取帧；显式重连、冷启动或重新打开主窗口恢复下一拍。
- [ ] 无地址、非法 / 非 loopback 地址、鉴权失败和端点缺失测试保持专门状态，不写零、不伪造 Connected。
- [ ] Monitor Channel bootstrap / 后续状态与托盘摘要来自同一 Rust health，前端订阅失败时仍保留可诊断空态。
- [ ] 相关 Rust tests、fmt、clippy 与前端订阅 / 状态 tests 通过，secret 扫描无新增命中。

## Out of Scope

- 新建 Windows Service、第二套自动重试器、多控制器或远程控制器支持。
- 改变登录自启动 opt-in、采样频率、核算、coverage 或 writer 语义。

## Key Decisions

- 仅已保存配置自动连接；首次安装不主动访问 `127.0.0.1:9097`。
- 冷启动、`--background` owner 启动与用户重新打开主窗口都属于自动恢复触发；普通界面重绘不属于。
- 第二实例只向既有 owner 发激活信号，不拥有 collector / writer，也不建立第二条连接循环。

## Planning Status

- `implemented`；实现、独立检查和 Rust / frontend 自动门已通过；真实安装态 Windows 与控制器证据仍为 `UNVERIFIED`。

# 空组回退防护验证

## 授权与范围

2026-10-04，用户批准“按照建议顺序实施”。本任务先实施，DNS 子任务随后执行。不包含提交、推送、真实 local 配置或生产 controller 修改。

## 实际内核 F7

- 内核：Mihomo Meta v1.19.32 windows amd64，Go 1.26.8，构建时间 2026-09-30。
- 命令：`node .trellis/tasks/10-04-upstream-empty-fallback-guard/research/check-core.cjs <installed-mihomo.exe>`。
- 使用独立临时目录、随机回环 controller 端口；DNS/TUN 关闭，没有代理入站端口；所有虚构代理地址为 127.0.0.1。
- `-t` 返回 0。
- `Proxy` 为 select、include-all-proxies=true、filter 无成员、exclude-filter 排除住宅节点，empty-fallback 指向 `家宽-SOCKS5`。
- 独立 controller 返回 `Proxy.now=家宽-SOCKS5`、`Proxy.all=[家宽-SOCKS5]`。住宅节点同时配置 `dialer-proxy=Proxy`。
- 结果：内核接受该配置并在空组选择中使用住宅 fallback；exclude-filter 不会消除该回指关系。未向住宅代理或业务主机发送连接，不声称已复现运行时堆栈溢出。
- 独立内核已结束，其临时目录已移除。未读取用户配置，未操作运行中的 Verge/controller。

## 自动化与审查

- `node --test tests/regression.test.js`：81/81 通过（实施子代理执行）。
- `npm run check`：通过（实施子代理执行）。
- 主仓库 `just ci`：退出码 0；包含 monitor 安装/类型检查/lint/300 项前端测试/build、Rust fmt/clippy/workspace tests、root CI 与 secret scan。
- 主仓库 `just docs-build`：退出码 0。
- 非阻塞提示：npm allow-scripts 配置来源提示、Vite 插件耗时提示；没有因此修改无关配置。
- 已更新 state-management 的可达组 fallback 契约。
- 独立审查：六个文件未发现需修复问题，AC1–AC4 与测试映射完整。审查读取当前文件及已展示 diff；隔离环境限制了再次运行主仓库 git diff，未把该限制隐去。
- 实施与验证完成。未提交、未归档；任务状态保留 in_progress 供后续提交收尾。

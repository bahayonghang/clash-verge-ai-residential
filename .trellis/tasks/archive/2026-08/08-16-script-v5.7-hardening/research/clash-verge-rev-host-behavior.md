# Clash Verge Rev 宿主行为调研（2026-08-16）

来源：clash-verge-rev dev 分支源码（`src-tauri/src/enhance/mod.rs`、`enhance/script.rs`、
`config/config.rs`、`core/validate.rs`、`config/prfitem.rs`）+ 官方文档
https://www.clashverge.dev/guide/script.html。引擎为 boa_engine 0.21.1。

## 对本脚本的关键影响（按重要度）

### 1.【重要】权威字段强制还原：脚本的 tun / ipv6 修改会被 Verge 丢弃

enhance 链顺序：… → 4) app 生成 tun/dns → 5) AuthoritativeFields::capture 快照
→ 6) 全局 Merge → 全局 Script → 7) profile Merge → profile Script
→ 8) authoritative.enforce 还原 → 9) cleanup_proxy_groups。

`CONTROL_PLANE_KEYS` 含 `tun`、`ipv6`（还有 external-controller、ports、mode、
allow-lan 等）。脚本在步骤 6/7 写入的 `config.tun` 修改（hardenTun 的
dns-hijack 补齐、strict-route）与 `config.ipv6 = false` 都会在步骤 8 被
**强制还原为 App 设置页的值**（dns.ipv6 在开启 DNS 覆盖时同样被还原）。

含义：
- `hardenTun` 与 `config.ipv6 = false` 在当前 Clash Verge Rev 上无效
  （旧版本行为可能不同，代码保留仍有意义）。
- `dns` 主体（nameserver / nameserver-policy / fake-ip 等）不在权威键内，
  脚本重建的 DNS 配置能存活。
- 正确做法是引导用户在 Verge 设置页关掉 IPv6 开关、在 TUN 设置里配 dns-hijack。
- 修复方向：脚本 info 日志提示 + docs 更新（configuration.md /
  dns-and-leak-model.md / troubleshooting.md），代码保留但加注释说明宿主覆盖。

### 2.【重要】默认配置下全局 Script 会执行两遍

profile 未指定自己的 merge/script 时，`current_script()` 默认返回全局 "Script"
项 → 同一脚本以"全局项"和"profile 项"各跑一次，第二次拿到的是第一次改过的 config。
脚本的幂等设计（回归测试"脚本执行两次保持幂等"）正好覆盖此行为 —— 必须继续守住。

### 3. 脚本抛异常 → 静默回退原始配置（fail-closed 的实际表现）

main 抛错时包装层捕获 `__error_flag__` → 整个脚本的修改被丢弃，配置按
"脚本未运行"继续生成；日志记录在 Script 卡片的 chainLogs（前端可见）+
Verge 日志（Type::Config）。不会导致配置整体失败。
含义：fail-closed 抛错安全，但用户感知依赖看卡片日志；warn/info 输出是
唯一通道，值得在抛错前补一条醒目 console.error 级别提示（console.error 支持）。

### 4. 执行环境限制

- boa_engine 0.21.1（Test262 94.12%）；脚本用到的 spread/Set/Map/includes
  均支持；`Object.hasOwn`/`replaceAll` 也支持（仅 Node 侧 sync 脚本使用，无碍）。
- 5 秒超时、1000 万循环迭代上限、console 输出 1000 条 / 1MB 上限、
  config JSON 10MB 上限、profileName 1024 字符上限。
- 无网络/文件 IO。
- profileName 是 profile 显示名（profiles.yaml 的 name），中文原样传入。

### 5. 生成后校验

enhance 无白名单校验；最终配置经 `mihomo -t` 子进程测试。校验失败：
首次启动回退默认最小配置，运行中不应用新配置。
含义：脚本产出的任何非法键/值会在这一步被拦下 —— 这也是 geosite.dat
下载失败（nameserver-policy 的 geosite: 键）的放大器（见 mihomo 调研第 4 条）。

### 6. cleanup_proxy_groups

自动移除引用不存在节点/组的 proxy-groups 条目 —— 脚本 upsert 的 AI-家宽
引用同文件内家宽节点，安全。

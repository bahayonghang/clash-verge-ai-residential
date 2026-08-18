# Research: Windows 11 本地监控桌面应用长期架构

- **Query**: 基于 Tauri 2、官方插件、SQLite 官方资料，研究完整 Windows 11 本地网络监控 app 的长期基础：静态 vanilla 前端 vs TypeScript/Vite；events vs Channel；app_config_dir/app_local_data_dir；CSP/capabilities；托盘、自启动、通知；GitHub Releases 手动升级；凭据存储（Windows Credential Manager/DPAPI）；SQLite WAL、synchronous、schema migration、备份恢复、分层保留；安装包与 Windows 通知限制。给出一手来源、建议与拒绝方案，服务于重写 design.md。
- **Scope**: mixed（仓库内设计/约束 + Tauri、SQLite、Microsoft、GitHub 一手资料）
- **Date**: 2026-08-18

> 后续状态：本研究完成后，主 `prd.md`、`design.md` 与 `implement.md` 已按结论重写。下文提到的“旧 design / PRD 冲突”描述的是调研输入快照，不代表当前主规划；实施使用精炼的 `desktop-monitor-implementation-context.md` 和最新主文档。主设计在后续一致性评审中把 SQLite durability 从本研究的 `NORMAL` 基线提高为 `FULL`，并删除 DPAPI fallback：v1 只用 Credential Manager 持久化 secret，不可用时仅允许进程内临时 secret。

## Findings

### Files Found

| File Path | Description |
|---|---|
| `.trellis/tasks/08-18-residential-monitor-mvp/prd.md` | 完整 v1 需求基线；连续采集、分层保留、告警、导出、隐私和手动升级要求集中在第 15–34、71–78 行 |
| `.trellis/tasks/08-18-residential-monitor-mvp/design.md` | 调研开始时仍为旧 MVP 设计；现已重写为 TypeScript/Vite、SQLite、Credential Manager 与后端权威查询 |
| `.trellis/tasks/08-18-residential-monitor-mvp/research/clash-controller-api.md` | Clash 控制器、命名管道、连接帧与误差边界的本机实测 |
| `.trellis/spec/frontend/index.md` | 现有 frontend spec 明确只描述可粘贴 Clash 扩展，不是桌面 UI（第 3–6 行）；其“不要构建步骤”不能直接套用到独立子应用 |
| `.trellis/spec/guides/cross-layer-thinking-guide.md` | 要求后端、数据库、前端边界有单一契约和集中解码（第 19–50、74–101 行） |
| `package.json` | 根项目当前零第三方依赖，但只承担 Node 18 脚本与测试（第 7–15 行） |

### 结论摘要

研究建议把旧 design.md 的技术底座改为（当前主设计已采纳）：

1. **Vanilla TypeScript + Vite，不引入 UI 框架**；构建产物仍是完全本地的静态 HTML/CSS/JS。
2. **Rust 后端拥有采集、SQLite、凭据、托盘、自启动、通知和升级迁移**；WebView 只拥有受限查询/操作命令和一个实时 Channel。
3. **SQLite 取代 JSON 账本**，存于 `app_local_data_dir`；使用 WAL、单写入器、分层事实/汇总表、版本化前向迁移和在线备份。
4. **控制器 secret 存 Windows Credential Manager**；设置文件只保存 credential reference。DPAPI CurrentUser 是后备实现，不是并列双写方案。
5. **NSIS current-user 安装包 + GitHub Releases 手动升级**；v1 不接 `tauri-plugin-updater`，不产 updater artifacts，不支持 portable 作为正式分发。
6. **显式 CSP、显式 capability、`withGlobalTauri: false`、不加载远程内容**；前端不得直连数据库、文件系统或 secret。

调研开始时 `prd.md` 仍硬性写着“静态 HTML/CSS/JS，不引入 npm 依赖与打包器”；当前 PRD 已同步改为 Vanilla TypeScript + Vite，不再与主设计冲突。

## 1. 前端：无构建静态 vanilla vs Vanilla TypeScript/Vite

### 一手事实

- Tauri 把前端当作静态 Web host，支持 SPA/MPA/SSG，不原生支持服务端渲染。[Tauri Frontend Configuration](https://v2.tauri.app/start/frontend/)
- Tauri 对普通 JavaScript **和 TypeScript** 项目都推荐 Vite；官方 Vite 集成用 `devUrl`、`beforeDevCommand`、`beforeBuildCommand` 和 `frontendDist: "../dist"`。[Tauri Frontend Configuration](https://v2.tauri.app/start/frontend/)；[Tauri Vite](https://v2.tauri.app/start/frontend/vite/)
- 无 bundler 也受支持：`frontendDist` 可以直接指向含 `index.html` 的源码目录，Tauri CLI 会提供开发服务器。[Tauri Develop](https://v2.tauri.app/develop/)
- `withGlobalTauri` 默认是 `false`；开启后会把 API 注入 `window.__TAURI__`。[Tauri Configuration](https://v2.tauri.app/reference/config/)

### 建议

采用 **Vanilla TypeScript + Vite**，不采用 React/Vue/Svelte：

- 这个 app 已有多页面状态、实时流、查询筛选、告警规则、导出、迁移错误和大量 Rust ↔ UI DTO。TypeScript 能让 Channel 消息判别联合、命令参数/结果和视图状态在编译期对齐。
- Vite 只属于开发/构建链；最终仍输出本地静态文件，不引入服务端、CDN 或远程运行时。
- 最小前端依赖面：`typescript`、`vite`、`@tauri-apps/api`，再按确需加入官方插件的 JS package；锁文件必须提交，CI 使用 lockfile 的 frozen install。
- 使用 ES modules 和显式 import，保持 `withGlobalTauri: false`。把 IPC DTO、运行时 guard/decoder、formatters、store、views 分开；不要把数据库 schema 泄漏到组件。
- TypeScript 不能校验运行时 IPC 数据。Channel/command 边界仍需检查 `schemaVersion`、`kind` 和必需字段；后端是权威校验者。

### 拒绝方案

- **拒绝“为了零 npm 而继续 `main.js` + `window.__TAURI__`”**：它对小型 MVP 可行，但完整 v1 会把 DTO 漂移、全局 API、页面状态和测试负担留到运行时；旧 design.md 第 28 行的理由已不匹配当前 PRD 的规模。
- **拒绝引入 UI 框架作为默认答案**：当前界面复杂度需要类型和模块边界，但尚无证据需要虚拟 DOM、路由框架或大型组件生态。Vanilla TS 保留较小依赖和包体。
- **拒绝把根项目 frontend spec 的“不要构建步骤”机械套到子应用**：该 spec 自己声明它只覆盖可粘贴 Clash 脚本，并非 UI。

## 2. Rust ↔ 前端：Commands、Events 与 Channel

### 一手事实

- Tauri event system 是动态、fire-and-forget、异步、仅 JSON payload、无返回值；不为低延迟或高吞吐设计，底层会直接执行 JavaScript。异步 listener 在快速连续事件下可能乱序。[Calling Rust from Frontend](https://v2.tauri.app/develop/calling-rust/)；[Calling Frontend from Rust](https://v2.tauri.app/develop/calling-frontend/)
- Channel 为快速、有序数据流设计，Tauri 自己也用它传下载进度、子进程输出和 WebSocket 消息。[Calling Frontend from Rust](https://v2.tauri.app/develop/calling-frontend/)
- Commands 适合有参数、返回值和错误的 request/response。[Tauri IPC](https://v2.tauri.app/concept/inter-process-communication/)

### 建议

采用三分法：

1. **Commands**：设置读写、测试控制器、分页历史查询、报告导出、备份/恢复、关闭连接、通知测试等 request/response 操作。
2. **单个 Channel**：前端创建 `Channel<MonitorStreamMessage>`，调用 `subscribe_monitor` 注册；消息使用判别联合，例如 `snapshot`、`connectionDelta`、`healthChanged`、`alertChanged`，每条含单调 `seq`、`schemaVersion` 和后端时间。
3. **Events**：只保留真正的低频广播型生命周期信号；单窗口 v1 甚至可以完全不依赖自定义 event。托盘、通知和 collector 不应通过 WebView event 驱动。

实时 UI 不应每秒推送全量 30 天历史或完整数据库结果：

- Rust 侧先聚合并限频；实时摘要可 1 Hz，连接变化发 delta 或受控快照。
- 后端维护 latest-only/coalescing，不能假设 Channel 提供业务级 backpressure。
- 大历史、域名/进程排行走分页命令，Channel 只通知“数据版本变更”。
- WebView 重载或窗口重建后重新执行订阅命令；后端 collector 生命周期不得依赖订阅存在。

### 拒绝方案

- **拒绝把 `emit("monitor://snapshot")` 当主数据总线**：event 官方明确不适合有序高吞吐流，旧 design.md 第 40–41、60–65 行应改。
- **拒绝前端轮询 SQLite 或 Rust 全量 snapshot**：会放大序列化和 WebView 渲染成本。
- **拒绝直接 `eval` JavaScript**：Tauri 文档把它列为最低层方式；这里没有必要扩大注入面。

## 3. 进程与模块边界

建议的数据流：

```text
Clash adapter (TCP / named pipe)
  -> collector actor (reconnect, auth, frame normalization)
  -> accounting actor (connection baselines, non-negative deltas, coverage)
  -> single SQLite writer (batched transaction)
  -> query/report service
  -> Tauri commands + one live Channel
  -> Vanilla TS views

OS adapters:
  CredentialStore | Autostart | Notification | Tray | FileDialog
```

关键约束：

- collector、writer、retention、alerts 是 Rust 后台服务，在 Tauri setup 后启动，不属于窗口对象。
- UI 窗口隐藏或销毁不影响采集；没有订阅者时停止制作 UI payload，但继续落库。
- 所有外部数据先在 Rust 边界归一化；前端只消费版本化 DTO，不接触 Clash 原始 JSON。
- 只保留一个 SQLite 写入 actor。查询使用受控只读连接；备份、迁移、恢复通过 storage service 串行协调。
- 在单实例插件 callback、托盘和通知点击中，只做“显示并聚焦主窗口/导航到目标”，不启动第二个 collector。

## 4. 数据目录：`app_config_dir` vs `app_local_data_dir`

### 一手事实

- `app_config_dir()` 解析为 `config_dir/{bundle_identifier}`；`app_local_data_dir()` 解析为 `local_data_dir/{bundle_identifier}`。[Tauri PathResolver](https://docs.rs/tauri/latest/tauri/path/struct.PathResolver.html)
- Windows 的 local data 对应 `FOLDERID_LocalAppData`（默认 `%LOCALAPPDATA%`）；config 对应 Roaming config 路径。Microsoft 把 `FOLDERID_RoamingAppData` 定义为 `%APPDATA%`，把 `FOLDERID_LocalAppData` 定义为 `%LOCALAPPDATA%`。[Tauri path API](https://v2.tauri.app/reference/javascript/api/namespacepath/)；[Microsoft KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- bundle identifier 被用于应用路径和 WebView 数据目录，因此必须唯一且稳定。[Tauri Config](https://docs.rs/tauri/latest/tauri/struct.Config.html)

### 建议目录所有权

```text
app_config_dir/
  preferences.json       # 小型、非敏感、可漫游的 UI 偏好；原子替换

app_local_data_dir/
  monitor.sqlite3        # 权威事实与汇总
  backups/               # SQLite Backup API 生成的版本化快照
  exports/               # 仅临时中转；用户正式导出到其选择路径
  machine.json           # 控制器地址/pipe、发现缓存等机器专属非密配置

app_log_dir/
  residential-monitor.log*

Windows Credential Manager
  io.github.bahayonghang.residential-monitor/clash-controller
```

- 数据库、备份和 controller machine settings 必须在 LocalAppData，不能随 roaming profile 漫游，也不能放安装目录。
- 只有真正可漫游且非敏感的偏好才放 app_config_dir。若不打算支持漫游，所有 operational settings 可统一存 SQLite/app_local_data_dir；不要为了 API 名字是“config”就把机器专属控制器配置放进 Roaming。
- identifier 一经发布不得修改；改 identifier 会改变 app dirs、WebView 数据、凭据 target 和安装身份。
- 日志使用官方 `tauri-plugin-log` 的 Rust `LogDir` target，设置轮转/容量上限并做 secret redaction；Windows 官方目录是 `%LOCALAPPDATA%/{identifier}/logs`。[Tauri Logging](https://v2.tauri.app/plugin/logging/)

### 拒绝方案

- **拒绝数据库放 `app_config_dir`**：高频、较大、机器专属数据不适合 Roaming。
- **拒绝数据库/设置放 executable 或 install directory**：current-user/per-machine 安装位置会变化，且升级覆盖应用文件。
- **拒绝在 WebView localStorage 里保存权威设置或数据**：它属于 WebView 状态，不提供业务迁移、备份和完整性契约。

## 5. CSP、capabilities 与 IPC 权限

### 一手事实

- Tauri 的 CSP **只有配置后才启用**；构建时会为本地脚本/样式处理 nonce/hash。应限制为仅可信来源。[Tauri CSP](https://v2.tauri.app/security/csp/)
- Capability 决定哪个 window/webview 获得哪些 core/plugin 权限；未匹配 capability 的 WebView 没有 IPC 权限。[Tauri Capabilities](https://v2.tauri.app/security/capabilities/)
- `src-tauri/capabilities/` 中的 capability 默认都会启用；若在 `tauri.conf.json` 显式列出，则只启用列出的 capability。[Tauri Capabilities](https://v2.tauri.app/security/capabilities/)
- 自定义 commands 默认对所有 window/webview 可用；要纳入细粒度权限需在 `build.rs` 使用 `AppManifest::commands`，再定义 app permissions。[Tauri Capabilities](https://v2.tauri.app/security/capabilities/)
- 官方 opener/fs/dialog 等插件的高风险命令默认受 capability/scope 控制。[Tauri Opener](https://v2.tauri.app/plugin/opener/)；[Tauri Dialog](https://v2.tauri.app/plugin/dialog/)

### 建议

- `withGlobalTauri: false`；生产只加载 bundle 内资源，不加载 CDN 字体、图表库、远程图片或远程页面。
- 显式配置生产 CSP，基线可从以下形状开始，并以实际构建验证结果为准：

```text
default-src 'self';
script-src 'self';
style-src 'self';
img-src 'self' data:;
font-src 'self';
connect-src ipc: http://ipc.localhost;
object-src 'none';
base-uri 'none';
frame-ancestors 'none'
```

- 不使用 inline script/handler；style 也尽量打包到 CSS，避免用 `'unsafe-inline'` 放宽。
- capability 只匹配 `main`，平台限定 `windows`，显式列出 `core:default` 与 app commands。不要用 `"windows": ["*"]`，不要配置 remote URLs。
- autostart、notification、log、SQLite、credential、tray 尽量只在 Rust 侧调用，因此无需把对应插件命令授予 WebView。
- “打开 Releases”最好是一个 Rust command，内部使用固定 HTTPS URL；若前端直接使用 opener，只 scope 到仓库 Releases URL，不授予任意 URL/path。
- 备份/恢复/导出可由 Rust 侧调用官方 dialog plugin；dialog 文档也提醒安全优先时应使用 dedicated command，而不是给 WebView 广泛文件 scope。[Tauri Dialog](https://v2.tauri.app/plugin/dialog/)
- command 参数全部视为不可信：日期范围、分页、排序、导出路径、连接 ID、设置长度都在 Rust 校验；SQL 永不从前端透传。
- secret、完整进程路径等敏感值不返回无需要的页面，永不出现在日志、错误串、Channel payload 或崩溃报告。

### 拒绝方案

- **拒绝“保持默认 CSP”**：Tauri 官方说明 CSP 未配置就不启用；旧 design.md 第 81 行的表述是错误安全假设。
- **拒绝 `withGlobalTauri: true`**：完整 app 已有构建链，应使用显式 imports 和最小 capability。
- **拒绝 `fs:default`、`opener:allow-default-urls` 或插件 `default` 集合未经审计直接加入**：默认 permission set 不是当前 app 的最小权限证明。
- **拒绝把 SQL plugin 暴露给前端**：见 SQLite 章节。

## 6. 托盘、关闭语义、自启动、通知与单实例

### 托盘与关闭

- Tauri 2 提供 `tray-icon` feature 和 `TrayIconBuilder`，支持菜单与 click/double-click 等事件。[Tauri System Tray](https://v2.tauri.app/learn/system-tray/)
- `WindowEvent::CloseRequested` 可调用 `prevent_close()`；`RunEvent::ExitRequested` 可调用 `prevent_exit()`。[Tauri WindowEvent](https://docs.rs/tauri/latest/tauri/enum.WindowEvent.html)；[CloseRequestApi](https://docs.rs/tauri/latest/tauri/struct.CloseRequestApi.html)

建议：

- Rust 创建托盘和菜单：打开主窗口、暂停/继续采集、立即重连、状态摘要、明确退出。
- 点窗口 X：`prevent_close()` 后 `hide()`，不停止 collector；首次关闭时显示一次“仍在托盘采集”的非侵扰说明。
- “明确退出”走单一 shutdown coordinator：停止接收新帧 → flush writer → 结束/标记 coverage interval → checkpoint/关闭 DB → 删除托盘 → programmatic exit。不要让 `prevent_exit` 把明确退出也拦掉。
- collector 状态和 tray 文案来自同一 Rust state；WebView 不在时托盘仍可用。

### 自启动与单实例

- 官方 `tauri-plugin-autostart` 支持 Windows，提供 enable/disable/isEnabled 和对应权限。[Tauri Autostart](https://v2.tauri.app/plugin/autostart/)
- 官方 `tauri-plugin-single-instance` 支持 Windows且纯 Rust；第二实例 callback 可聚焦已有窗口。[Tauri Single Instance](https://v2.tauri.app/plugin/single-instance/)

建议：

- autostart 默认关闭，由用户明确 opt-in；注册启动参数 `--background`，登录启动时不弹主窗口。
- 设置页显示实际 `isEnabled()`，不要只相信数据库中的期望值；启用/禁用失败要持久化为健康告警。
- single-instance 插件尽早注册。第二次启动只显示/聚焦现有窗口，不创建第二 writer/collector。
- 登录自启动后先完成迁移和存储检查，再开始 coverage；启动失败也要在下次可运行时形成可见 gap/health 记录。

### Windows 通知

- 官方 notification plugin 在 Windows **只对已安装 app 正常工作**；开发模式会显示 PowerShell 名称和图标。[Tauri Notifications](https://v2.tauri.app/plugin/notification/)
- Microsoft 要求传统 unpackaged desktop app 有安装在 Start/All Programs 的 shortcut 和有效 AUMID；推荐由 installer 创建。没有有效 shortcut 就不能正常发 toast。[Microsoft Desktop Toast + AUMID](https://learn.microsoft.com/en-us/windows/win32/shell/enable-desktop-toast-with-appusermodelid)
- Windows app notifications 不支持 elevated/admin app；调用可能静默不显示。[Microsoft App Notifications](https://learn.microsoft.com/en-us/windows/apps/develop/notifications/app-notifications/)

建议：

- 通知由 Rust alert service 发：先在 SQLite 原子记录 alert 状态/历史，再 best-effort 发送系统通知。系统通知失败不能丢失告警。
- 首次启用告警时检查/request permission，并提供“测试通知”；UI 明确说明 Focus Assist、用户系统设置和未安装 dev build 可能阻止显示。
- 不以“send 返回成功”当作用户看见；应用内告警中心是权威呈现。
- 正式通知验收必须使用真实安装包、普通用户权限、稳定 identifier/AUMID；`cargo tauri dev` 不能作为品牌名/图标验收。
- 通知点击若需要导航，应先确认当前 Tauri plugin 对 Windows activation 的支持并做安装态真机测试；v1 可先把 toast 做纯信息提示，点击仅打开应用，不依赖复杂 action/input。

### 拒绝方案

- **拒绝“关闭最后窗口即退出”**：直接违反连续采集。
- **拒绝在前端初始化托盘/collector**：WebView reload 会造成重复或中断。
- **拒绝默认开启自启动**：属于用户可见的系统行为，必须明确选择。
- **拒绝以管理员权限运行 app/installer 后常驻**：app 本身不需要提升，提升进程还会破坏通知。
- **拒绝把系统通知作为唯一告警记录**：Windows 可静默、聚合或禁用通知。

## 7. 凭据存储：Credential Manager、DPAPI、Stronghold

### 一手事实

- `CredWriteW`/`CredReadW` 操作当前 token/logon session 对应用户的 credential set。[Microsoft CredWriteW](https://learn.microsoft.com/en-us/windows/win32/api/wincred/nf-wincred-credwritew)；[CredReadW](https://learn.microsoft.com/en-us/windows/win32/api/wincred/nf-wincred-credreadw)
- `CRED_TYPE_GENERIC` 是由应用定义/认证的通用凭据，Windows 安全存储但不赋予其他认证语义。[Microsoft CREDENTIALW](https://learn.microsoft.com/en-us/windows/win32/api/wincred/ns-wincred-credentialw)
- `CRED_PERSIST_LOCAL_MACHINE` 对同一用户在本机后续 logon session 可见，不会漫游到其他机器。[Microsoft CREDENTIALW](https://learn.microsoft.com/en-us/windows/win32/api/wincred/ns-wincred-credentialw)
- DPAPI `CryptProtectData` 默认通常要求相同用户凭据且同一计算机；`CRYPTPROTECT_LOCAL_MACHINE` 会允许本机任意用户解密。[Microsoft CryptProtectData](https://learn.microsoft.com/en-us/windows/win32/api/dpapi/nf-dpapi-cryptprotectdata)
- 管理员重置用户密码等场景可能让 DPAPI CurrentUser 数据不可恢复。[Microsoft DPAPI example](https://learn.microsoft.com/en-us/windows/win32/seccrypto/example-c-program-using-cryptprotectdata)
- Tauri 官方 Stronghold plugin 使用 IOTA Stronghold，初始化需要把 vault password 派生为 32-byte hash。[Tauri Stronghold](https://v2.tauri.app/plugin/stronghold/)

### 建议

定义 Rust seam：

```text
CredentialStore
  put(target, secret)
  get(target)
  delete(target)
  availability()
```

Windows v1 实现：

- `CRED_TYPE_GENERIC`
- target：稳定 identifier + 用途，例如 `io.github.bahayonghang.residential-monitor/clash-controller`
- persistence：`CRED_PERSIST_LOCAL_MACHINE`
- `UserName` 可为空或放非敏感 profile label；secret 放 credential blob
- 配置/SQLite 只存 `{ credentialTarget, hasSecret }`，不存 secret 值
- 保存设置采用补偿事务：先写/更新 Credential Manager，成功后提交 non-secret setting；失败则保持旧引用
- 删除配置/卸载数据清理与 credential delete 分开，避免普通升级误删 secret
- secret 只在 Rust collector 建立请求时短暂进入内存，不返回前端；错误信息只报告“未配置/读取失败/鉴权失败”

DPAPI 仅作为 Credential Manager 因策略不可用时的**后备 adapter**：

- 使用 CurrentUser，不得使用 `CRYPTPROTECT_LOCAL_MACHINE`
- 使用 app-specific optional entropy，密文放 `app_local_data_dir`
- 密文备份默认排除或明确标记“仅原用户原机器可恢复”
- 恢复后若无法解密，要求用户重新输入 secret，不能回退为明文

安全边界必须写清：Credential Manager 和 DPAPI 主要防止磁盘明文泄露/误提交，不防同一用户上下文中的恶意进程或已被攻陷的主机。

### 拒绝方案

- **拒绝 secret 写 `settings.json`/SQLite 明文字段**：旧 design.md 第 80 行必须改。
- **拒绝自制 AES key + 密文同目录**：没有解决 key storage。
- **拒绝 DPAPI LocalMachine scope**：官方明确本机任意用户可解密。
- **拒绝 v1 同时维护 Credential Manager + DPAPI 双份 secret**：会产生轮换、删除和恢复不一致；DPAPI 只能是替代 backend。
- **暂不选 Stronghold 为 Windows v1 默认**：它需要另一个 vault password 的来源/解锁 UX；若把 password 硬编码或仍交给 DPAPI，复杂度高于直接使用 OS credential vault。它可作为未来跨平台 adapter，不应直接把 Stronghold JS API/secret 暴露给 WebView。

## 8. SQLite 所有权与驱动选择

### 官方 SQL plugin 的定位

`tauri-plugin-sql` 是“让 frontend 通过 sqlx 与数据库通信”的官方插件，并支持 Rust 注册 migrations 和 preload。[Tauri SQL](https://v2.tauri.app/plugin/sql/)

### 建议

- **不使用 `tauri-plugin-sql` 的 frontend Database API**。数据库只由 Rust storage/query services 访问，前端拿领域 DTO。
- Rust 可选成熟 SQLite binding（具体 crate 在实现计划中锁定）；关键是：
  - 编译/捆绑的 SQLite 版本可知且可测试；
  - 暴露 Online Backup API 或可靠等价能力；
  - 支持 busy timeout、WAL checkpoint、事务和参数化查询；
  - 启动时记录 `sqlite_version()`，但不记录用户数据。
- 截至本研究日期，WAL 多连接应使用 **SQLite 3.51.3+**，或带官方 backport 的 3.50.7/3.44.6。SQLite 官方披露 3.7.0–3.51.2 存在极罕见 WAL-reset corruption race，3.51.3 修复；触发条件是同一 WAL 有两个以上连接同时 write/checkpoint。[SQLite WAL-reset bug](https://www.sqlite.org/wal.html)
- Windows 正式构建不要无意依赖未知系统 SQLite 版本；在依赖锁和 CI 中验证捆绑版本。

### 拒绝方案

- **拒绝 WebView 任意 SQL**：capability 一旦授予，XSS/前端错误能绕过领域校验、隐私投影、保留水位和迁移约束。
- **拒绝继续 JSON 文件账本**：无法可靠支持全连接明细、并发查询、迁移、分层保留、告警历史、一致导出和在线备份。

## 9. SQLite WAL、连接和 durability

### 一手事实

- WAL 允许 reader 与 writer 并发，但仍只有一个 writer；checkpoint 把 WAL 内容转回主库。[SQLite WAL](https://sqlite.org/wal.html)
- WAL 必须与数据库在同一主机；不支持 network filesystem。[SQLite WAL](https://sqlite.org/wal.html)
- WAL 文件属于数据库持久状态。把主库与 WAL 分离会丢已提交事务或导致损坏。[SQLite WAL](https://www2.sqlite.org/wal.html)
- WAL + `synchronous=NORMAL` 保持数据库一致性/抗损坏，但断电或 hard reset 后最近事务可能回滚；应用 crash 不会因该设置丢 durability。`FULL` 在每次 commit 后额外 sync WAL，提供断电 durability。[SQLite PRAGMA synchronous](https://sqlite.org/pragma.html)
- `foreign_keys` 应由每个连接显式开启，不能依赖默认值；`busy_timeout` 为锁冲突安装等待 handler。[SQLite PRAGMA](https://sqlite.org/pragma.html)

### 建议基线

启动连接配置：

```sql
PRAGMA journal_mode = WAL;      -- 检查返回值确为 wal
PRAGMA synchronous = NORMAL;   -- 每个连接显式设置
PRAGMA foreign_keys = ON;      -- 每个连接显式设置
PRAGMA busy_timeout = 5000;    -- 推荐值，需压力测试
```

运行策略：

- 一个专用 writer actor 顺序接收采集批次、alert、coverage 和 retention work；按 1 秒或有限行数批量 transaction，避免每条 delta 一次 commit。
- 查询连接只读、短事务、分页，禁止持有长期 read transaction 阻塞 checkpoint。
- 默认 WAL auto-checkpoint 可先保留，再根据真实写入量监测 WAL 大小；也可由 writer 在低负载执行 PASSIVE checkpoint。退出和备份前由 storage coordinator 做受控 checkpoint。
- `synchronous=NORMAL` 与监控数据“近似下界”匹配，但 design 必须声明：断电/hard reset 可丢最近已显示的少量采集事务。若用户把告警审计视为不可丢，可把 alert/config/migration transaction 切到 `FULL`，或全库采用 `FULL` 后用基准决定。
- 不得使用 `synchronous=OFF`。
- DB 只能位于本机 LocalAppData，不允许用户把 live DB 改到 OneDrive/SMB/network share。
- 长连接在 open 时执行 `PRAGMA optimize=0x10002`，之后每天和 schema/index 变化后执行 `PRAGMA optimize`；这是 SQLite 3.46+ 的官方建议。[SQLite PRAGMA optimize](https://sqlite.org/pragma.html)
- 监控并上报：DB file/WAL 大小、最后成功 commit/checkpoint/backup、retention watermark、`SQLITE_BUSY`/I/O error。

## 10. Schema 基础与时间模型

建议的逻辑表边界（字段在 design.md 再定稿）：

```text
schema_migration            version, checksum, applied_at_utc, app_version
machine_setting             non-secret operational settings
target_set / target_item    versioned target names and priority
coverage_interval           start/end/reason (running, disconnected, unauthorized,
                            sleeping_or_clock_gap, paused, storage_failure, app_exit)
connection_session          Clash id + start/end + full normalized metadata
connection_chain            session + chain name + observed position
connection_minute           session + utc_minute + upload_delta + download_delta
traffic_hourly              utc_hour + stable dimensions + bytes + coverage
traffic_daily               utc_day + stable dimensions + bytes + coverage
alert_rule / alert_instance / alert_event
retention_state             per-tier watermark and last successful run
```

设计原则：

- **保存 delta，不保存每秒完整 frame**。每秒 frame 只用于更新连接 baseline；持久化 session metadata 和按分钟 delta，既可把跨小时长连接正确切分，也避免 30 天逐秒快照爆炸。
- 所有控制器连接都入同一事实表；“目标分类/其他/全局”是同源投影，不能有两套账本。
- 在 30 天 raw 明细期内，按当前 `target_set` 和用户优先级重算分类；保存当时 target set/version 仅用于解释历史视图，不替代 raw reclassification。
- 小时/日汇总必须记录足够的稳定维度或明确的 classification version。raw 过期后不能承诺任意新规则的完整重算；UI/导出要标识“基于当时分类版本”。
- 时间源统一存 **UTC integer epoch**（建议毫秒，汇总 bucket 用 UTC 秒/分钟边界）；本地时区只在查询/报告边界应用。不要用旧 design.md 的 `YYYY-MM-DDTHH` 本地文本主键，它在 DST 回拨时不唯一，跨时区也不可解释。
- 连接 metadata 允许 nullable/unknown；来源 payload 版本与 normalized schema version 分开。
- 新表优先使用 SQLite `STRICT`，获得刚性类型约束，且 `integrity_check` 会检查 STRICT column 类型。[SQLite STRICT tables](https://www.sqlite.org/stricttables.html)
- 索引由真实查询驱动，至少覆盖时间范围、session/time、host/process/rule/chains 查询；保留清理按时间索引。不要为所有隐私字段盲目建索引。
- 对链列表优先子表或稳定 canonical representation；不能依赖 `chains` 数组顺序决定主分类（PRD 第 20、65 行已明确）。

## 11. 分层保留与清理

默认策略对应 PRD：

- connection/session/minute/host/process/rule/chain 明细：30 天
- hourly：13 个月
- daily：长期
- alert history、coverage、schema migration：单独政策，不能被普通 traffic retention 意外删除

安全算法：

1. 以 UTC cutoff 和 `retention_state` watermark 找待处理区间。
2. 同一 `BEGIN IMMEDIATE` transaction 内，先幂等 UPSERT 到下一层汇总，再核对行数/字节和 coverage，再推进 watermark。
3. 只有汇总和 watermark 成功后，才删除低层已覆盖行。
4. 每批限定时间/行数，避免长事务卡采集；下一轮可从 watermark 继续。
5. hourly 过期前先保证 daily 已生成；raw 过期前先保证 hourly 与必要 stable dimensions 已生成。
6. manual clean 与 scheduled clean 调用同一 service，不允许 UI 直接 DELETE。
7. 估算占用用实际 page/freelist 和按层行数，不把“删除行”误报成文件立即缩小；VACUUM 是独立、可取消/需空间的维护动作。

拒绝：

- **拒绝仅按 `created_at < cutoff` 直接 DELETE raw**：崩溃可留下汇总缺口。
- **拒绝把 hourly/day 在查询时永久临时聚合**：raw 删除后数据消失，且长期查询成本不受控。
- **拒绝把未采集 interval 填 0**：coverage 表必须与每个 report bucket 合并。

## 12. Schema migration 与升级失败处理

### 一手事实

- `PRAGMA user_version` 是 SQLite 不解释的 32-bit application-owned integer。[SQLite PRAGMA](https://sqlite.org/pragma.html)
- `BEGIN IMMEDIATE` 立即取得 write transaction；WAL 下它与 EXCLUSIVE 等价于阻止其他 writer，但 reader 仍可继续。[SQLite Transaction](https://www.sqlite.org/lang_transaction.html)
- SQLite transaction 保证全成或全不成。[SQLite Atomic Commit](https://www.sqlite.org/atomiccommit.html)

### 建议

- 同时使用：
  - `PRAGMA user_version`：快速得到当前 schema 整数；
  - `schema_migration`：记录每个 migration 的 version、不可变 checksum、应用时间、app version。
- migration 只在 Rust startup storage service 执行，且在 collector、Channel 和查询连接启动前完成。
- 每个版本是显式、顺序、前向 migration；DDL/DML + migration row + `user_version` 在同一 `BEGIN IMMEDIATE` transaction。
- migration 文件/常量一经发布不可编辑；checksum 不同即拒绝继续并提示恢复。
- app version 与 schema version 解耦。若 DB schema 高于当前 binary 支持上限，旧版必须 fail closed，不得猜测兼容或自动降级 schema。
- migration 前自动做 Online Backup 快照；migration 后运行 `foreign_key_check`，关键升级运行 `quick_check`/`integrity_check` 和 smoke query。
- 大型 backfill 拆成：
  1. 兼容 schema 变更；
  2. 有 checkpoint 的后台 backfill；
  3. 后续版本再收紧约束。
  不要在首次打开窗口前做不可观测的超长 transaction。
- upgrade migration 失败时：不启动 collector，不写新数据，保留原 DB/备份，展示可导出的诊断和恢复入口。
- downgrade 不是 migration 路径。若手动安装旧版，旧版看到更高 schema 必须拒绝打开；Release notes 明确最低可回退版本。

### 拒绝方案

- **拒绝只靠 `CREATE TABLE IF NOT EXISTS`**：无法表达列/索引/数据 backfill 和不可逆变更。
- **拒绝 down migrations 作为用户回滚保证**：数据降级常有信息丢失；可靠回滚是恢复升级前备份并安装兼容 binary。
- **拒绝 migration 与 collector 并发**：会产生 schema race 和部分新数据。

## 13. 备份、恢复与灾难边界

### 一手事实

- SQLite Online Backup API 可增量把 live source 复制到另一数据库，短时间持有 read lock，允许其他用户继续工作。[SQLite Backup API](https://sqlite.org/backup.html)
- `VACUUM INTO` 也能生成一致、压缩的 live snapshot，但不能增量，CPU 较多；输出文件必须不存在或为空，意外中断可能留下损坏输出。[SQLite VACUUM](https://sqlite.org/lang_vacuum.html)
- 直接在 active transaction 时复制 DB 可能混合新旧页而损坏；WAL/hot journal 必须与主库一起处理。[How To Corrupt SQLite](https://sqlite.org/howtocorrupt.html)

### 建议

- 自动备份使用 **Online Backup API**，不要 `fs::copy(monitor.sqlite3)`。
- 触发点：
  - 每次 schema migration 前（必须）
  - 用户手动备份
  - 可选每日低负载备份
- 文件名包含 UTC timestamp、schema version、app version；先写 `.partial`，完成后关闭目标、运行 `PRAGMA integrity_check`，成功才原子 rename 为 `.sqlite3`。
- 备份目录本身有保留上限（例如最近 3 个 migration backup + 最近 7 个日备份，最终数字由 PRD/design 决定）。
- backup manifest 含 checksum、schema/app version、创建时间、源 DB logical stats；不包含 controller secret。
- 恢复必须进入 maintenance mode：
  1. 停 collector/writer/query；
  2. 先备份当前库；
  3. 以只读方式打开候选并验证 checksum、`integrity_check`、schema compatibility；
  4. 关闭所有连接；
  5. 用受控 restore/swap 生成新的主库，确保没有把旧 `-wal/-shm` 与新库混用；
  6. 执行所需 forward migrations；
  7. reopen 并 smoke check 后才恢复采集。
- 用户选择任意路径时由 Rust dialog + dedicated command 完成；不要把数据库目录开放给 WebView fs。
- 设置/凭据备份与 database backup 分离。Credential Manager secret 默认不导出；跨机恢复要求重新输入 secret。

### 拒绝方案

- **拒绝复制单个 `.sqlite3` 文件当 hot backup**。
- **拒绝恢复时覆盖正在打开的 DB**。
- **拒绝把 `-wal` 当缓存随意删除**：SQLite 官方明确它是持久状态的一部分。
- **拒绝只验证文件存在/能 open**：必须做 integrity、schema 和关键业务查询验证。

## 14. Windows 安装包、GitHub Releases 与手动升级

### 一手事实

- Tauri Windows 正式 bundle 是 WiX `.msi` 或 NSIS `-setup.exe`。[Tauri Windows Installer](https://v2.tauri.app/distribute/windows-installer/)
- NSIS 默认 `currentUser`，无需管理员权限，安装到 `%LOCALAPPDATA%`，installer metadata 在 HKCU；`perMachine` 进入 Program Files/HKLM 并要求管理员。[Tauri Windows Installer](https://v2.tauri.app/distribute/windows-installer/)
- 默认 WebView2 `downloadBootstrapper` 需要网络且不增加包体；`embedBootstrapper` 仍需网络、约 +1.8 MB；`offlineInstaller` 不需网络、约 +127 MB；`skip` 不推荐。[Tauri Windows Installer](https://v2.tauri.app/distribute/windows-installer/)
- Windows code signing 可避免浏览器下载后 SmartScreen 的“不可信”警告；不是执行 app 的硬性要求，但未签名用户必须接受警告。[Tauri Windows Code Signing](https://v2.tauri.app/distribute/sign/windows/)
- GitHub Releases 支持 release notes 和 binary assets；官方 Tauri GitHub pipeline 使用 `tauri-apps/tauri-action` 构建并创建 release。[GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github)；[Tauri GitHub Pipeline](https://v2.tauri.app/distribute/pipelines/github/)
- GitHub immutable release 会锁定 tag/assets 并生成 release attestation；可用 `gh release verify` / `verify-asset` 验证。[GitHub Immutable Releases](https://docs.github.com/en/code-security/concepts/supply-chain-security/immutable-releases)
- Tauri updater plugin 要求 update signature，不能关闭；还需 updater artifacts、key 生命周期和 endpoint。[Tauri Updater](https://v2.tauri.app/plugin/updater/)

### v1 建议

- 正式主渠道：**NSIS currentUser `-setup.exe`**。
  - app 普通用户运行；
  - 不触发 UAC；
  - 与 per-user Credential Manager、autostart 和 LocalAppData 一致；
  - installer 建立 Start Menu identity，满足通知安装态要求。
- `.msi` 仅在明确有企业部署需求时作为第二资产；不要同时发布多个没有说明的安装器让普通用户猜。
- WebView2：
  - 在线 GitHub 分发默认保留 `downloadBootstrapper` 即可；
  - 若需求明确为离线安装，发布单独标明的 `offlineInstaller` 资产并接受约 127 MB 增量；
  - 不把 `embedBootstrapper` 描述成离线方案，它仍需网络。
- 尽早固定且永不随版本变化：identifier、productName、binary name、publisher、安装模式、Credential target。
- 公共发布前配置 Windows code signing 和 timestamp；签 app binary 与 installer。没有证书时 Release notes 必须诚实说明 SmartScreen。
- GitHub release 流程：tag/draft → Windows CI build/test → sign → 上传 canonical installer + SHA-256/checksum/SBOM（如有）→ 安装/升级真机 smoke → 发布 immutable release。
- About 页展示 app version、schema version、SQLite version、数据目录、最后备份，并通过固定 HTTPS 链接打开仓库 Releases。
- 手动升级操作：应用内只提示“前往 Releases”，用户下载新 installer；installer 前要求退出现有 tray app。首次新版本启动先备份、迁移、校验，再启动 collector。
- v1 不注册 `tauri-plugin-updater`，`createUpdaterArtifacts: false`，不保存 updater private key，不给 updater/process capabilities。
- installer 降级与 DB 降级是两层问题：即使 installer 允许覆盖，旧 binary 也必须用 schema compatibility gate 拒绝新 DB。

### 拒绝方案

- **拒绝 portable exe/zip 作为正式主渠道**：Tauri notification 官方明确 Windows 只对 installed apps 正常工作；portable 还绕过 Start shortcut/AUMID、稳定升级和卸载身份。
- **拒绝 NSIS `perMachine`/`both` 作为默认**：会要求管理员；app 本身不需要 elevation，通知也不支持 elevated process。
- **拒绝 v1“顺手加自动更新”**：会引入 signing key、endpoint、下载/退出/安装和失败恢复的新系统，直接违反当前 PRD 的手动升级范围。
- **拒绝在安装目录保存 SQLite**：升级/卸载不能成为数据生命周期操作。
- **拒绝无上一版本升级测试就发布**：migration、autostart registration、Credential target、tray exit、通知 identity 都必须从已发布版本到候选版本真机验证。

## 15. 建议写入新版 design.md 的决策表

| 主题 | 采用 | 拒绝/延后 |
|---|---|---|
| 前端 | Vanilla TypeScript + Vite + 本地静态产物 | 无构建全局 JS；UI framework 暂不引入 |
| IPC | Commands + 单有序 Channel；低频 event 可选 | 高频全量 event snapshot |
| 后端所有权 | Rust actor/service 拥有 collector/storage/OS integrations | WebView 驱动后台生命周期 |
| 数据目录 | DB/backups/machine settings 在 app_local_data_dir；偏好才进 app_config_dir | DB 放 Roaming、install dir、localStorage |
| 安全 | 显式 CSP/capability，global Tauri off，固定本地资源 | 默认 CSP 假设、remote content、宽插件权限 |
| secret | Windows Credential Manager；DPAPI CurrentUser fallback | 明文、DPAPI LocalMachine、自制 key |
| SQLite | bundled patched SQLite、WAL、single writer、NORMAL、FK、timeout | JSON ledger、system unknown SQLite、OFF sync |
| schema | STRICT + UTC epoch + raw/minute/hour/day + coverage | 本地时间文本 hour key、每秒 frame 永久化 |
| migration | user_version + migration history/checksum + forward only | IF NOT EXISTS、自动 down migration |
| backup | Online Backup API + integrity validation + maintenance restore | live file copy、删除 WAL |
| 常驻 | Rust tray + close-to-hide + autostart opt-in + single instance | close-to-exit、默认自启动 |
| 告警 | DB 权威 + best-effort installed-app notifications | toast 作为唯一历史 |
| 发布 | signed NSIS currentUser + GitHub immutable Release + manual upgrade | portable 主渠道、v1 updater plugin |

## 16. 建议的设计验收/风险验证

新版 design.md 应把以下内容写成实施前/发布前 gate：

1. `cargo tauri dev` 与生产 CSP 都能加载 Vite 产物，生产无 CSP violation、remote request 或 inline exception。
2. Channel 在 10k 活跃连接模拟下保持顺序、内存有界；隐藏/重建窗口不影响 collector，重订阅可恢复最新状态。
3. 同时进行 writer、历史查询、checkpoint、Online Backup 的压力测试；确认 SQLite 实际版本 >= 3.51.3。
4. kill process、应用 crash、Windows restart/强制断电模拟后，验证 NORMAL 模式的数据损失边界和 DB integrity。
5. migration fixture 覆盖“上一正式 schema → 当前 schema”、重复启动、迁移中断、DB newer-than-app、checksum mismatch。
6. retention fixture 证明 raw → hourly → daily 汇总守恒，cleanup 中断可重试，coverage gap 不变成 0。
7. backup/restore 覆盖 live backup、坏 checksum、integrity failure、旧 schema restore+forward migration、残留 WAL/SHM。
8. Credential Manager 覆盖首次保存、轮换、删除、读取失败、卸载后/升级后行为；日志/导出/Channel 不含 secret。
9. 真机安装态覆盖 NSIS currentUser、Start Menu、tray、autostart `--background`、single instance、普通权限通知、Focus Assist/通知禁用。
10. 从上一 Release 手动安装新版本：进程退出、installer 覆盖、identifier/AUMID 不变、设置/DB/credential 保留、migration 成功。
11. 未签名测试包与正式签名包分别记录 SmartScreen 行为；正式资产校验 SHA-256/immutable release。

### Related Specs

- `.trellis/spec/guides/cross-layer-thinking-guide.md` — 后端/数据库/前端契约应有单一 owner，前端不重复解释 raw payload。
- `.trellis/spec/frontend/index.md` — 现有 spec 仅适用于根目录 Clash 扩展；桌面子应用需要后续补充自己的 frontend/backend/storage specs。
- `.trellis/spec/frontend/type-safety.md` — 其 plain JS guard 规则仍提示运行时边界要验证，但不禁止独立桌面子应用采用 TypeScript。

## External References

### Tauri 2

- [Frontend Configuration](https://v2.tauri.app/start/frontend/) — 静态 host、Vite 推荐和无 SSR 边界
- [Vite](https://v2.tauri.app/start/frontend/vite/) — `devUrl`/`frontendDist`/build 集成
- [Calling Rust from Frontend](https://v2.tauri.app/develop/calling-rust/) — commands、Channel、events
- [Calling Frontend from Rust](https://v2.tauri.app/develop/calling-frontend/) — event 吞吐/顺序限制，Channel 语义
- [PathResolver](https://docs.rs/tauri/latest/tauri/path/struct.PathResolver.html) — app-specific paths
- [CSP](https://v2.tauri.app/security/csp/) — CSP 仅配置后生效
- [Capabilities](https://v2.tauri.app/security/capabilities/) — window/plugin/custom command 权限
- [System Tray](https://v2.tauri.app/learn/system-tray/) — tray-icon
- [Autostart](https://v2.tauri.app/plugin/autostart/) — 官方自启动插件
- [Single Instance](https://v2.tauri.app/plugin/single-instance/) — 官方单实例插件
- [Notifications](https://v2.tauri.app/plugin/notification/) — Windows installed-only 限制
- [Logging](https://v2.tauri.app/plugin/logging/) — app log dir
- [SQL](https://v2.tauri.app/plugin/sql/) — 官方插件定位与 migration
- [Stronghold](https://v2.tauri.app/plugin/stronghold/) — vault/password 模型
- [Windows Installer](https://v2.tauri.app/distribute/windows-installer/) — NSIS/MSI、install mode、WebView2
- [Windows Code Signing](https://v2.tauri.app/distribute/sign/windows/) — SmartScreen/签名
- [GitHub Pipeline](https://v2.tauri.app/distribute/pipelines/github/) — tauri-action
- [Updater](https://v2.tauri.app/plugin/updater/) — updater signature/key/artifacts

### SQLite

- [Write-Ahead Logging](https://sqlite.org/wal.html) — WAL、checkpoint、网络文件系统和 WAL-reset fix
- [PRAGMA](https://sqlite.org/pragma.html) — synchronous、foreign_keys、busy_timeout、user_version、optimize
- [Transaction](https://www.sqlite.org/lang_transaction.html) — BEGIN IMMEDIATE
- [Atomic Commit](https://www.sqlite.org/atomiccommit.html) — transaction atomicity
- [Online Backup API](https://sqlite.org/backup.html) — live incremental backup
- [VACUUM](https://sqlite.org/lang_vacuum.html) — VACUUM INTO 边界
- [How To Corrupt SQLite](https://sqlite.org/howtocorrupt.html) — live copy/hot journal 风险
- [STRICT Tables](https://www.sqlite.org/stricttables.html) — rigid typing 与 integrity check

### Microsoft / GitHub

- [Credential Write](https://learn.microsoft.com/en-us/windows/win32/api/wincred/nf-wincred-credwritew)
- [Credential Read](https://learn.microsoft.com/en-us/windows/win32/api/wincred/nf-wincred-credreadw)
- [CREDENTIALW](https://learn.microsoft.com/en-us/windows/win32/api/wincred/ns-wincred-credentialw)
- [CryptProtectData](https://learn.microsoft.com/en-us/windows/win32/api/dpapi/nf-dpapi-cryptprotectdata)
- [Windows Known Folder IDs](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [Desktop Toast + AUMID](https://learn.microsoft.com/en-us/windows/win32/shell/enable-desktop-toast-with-appusermodelid)
- [Windows app notification limits](https://learn.microsoft.com/en-us/windows/apps/develop/notifications/app-notifications/)
- [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github)
- [GitHub Immutable Releases](https://docs.github.com/en/code-security/concepts/supply-chain-security/immutable-releases)

## Caveats / Not Found

- 本研究没有替代安装态真机验证。Tauri notification plugin 对点击激活、Action Center 留存、AUMID/快捷方式的具体行为必须以最终 NSIS 模板和 Windows 11 真机为准。
- Rust SQLite crate、连接池和 Windows Credential Manager binding 尚未选型；选型必须证明实际 bundled SQLite 版本、Backup API 能力、license、维护状态和错误映射，不能只看 API 表面。
- 分钟 bucket、批量 commit 间隔、busy timeout、WAL checkpoint 和备份保留数字是建议基线，需要用模拟连接量与真实 Verge 数据压测后定稿。
- `synchronous=NORMAL` 有明确的断电最近事务回滚风险；若产品把告警审计或用量账本定义为断电也不得丢，必须改用 FULL 或拆分耐久性策略，并接受写入成本。
- Credential Manager/DPAPI 不防同用户恶意进程；该限制应进入 threat model 和隐私说明。
- GitHub immutable releases 是否对当前仓库计划/设置可用需要发布负责人确认；不可用时至少发布签名 installer 与独立 SHA-256，并禁止替换已发布同名资产。

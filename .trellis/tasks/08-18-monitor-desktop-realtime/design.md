# 技术设计：桌面外壳与实时监控（C2）

## 设计目标

1. 将 Windows 生命周期、凭据和 WebView 视为 C1 采集内核之外的适配层，任何窗口操作都不改变 collector / writer 生命周期。
2. 通过原子 bootstrap 与有序增量，让 10k 短时活跃连接在 UI 中保持一致、可操作和内存有界。
3. 前端只消费稳定 DTO；C2 不接触 SQLite schema、mihomo 原始 payload 或核算规则。
4. 正常数据库不可用时仍提供不依赖业务 schema 的 Recovery Shell。

## 依赖与所有权

### C1 前置交付

C2 开始实施前必须冻结并验证以下 C1 能力：

- collector / accounting / writer 在 Tauri 后台独立运行；
- 版本化 live projection、进程级单调 `seq`、原子 `bootstrap { snapshot, baseSeq }` 和 `resync`；
- 生命周期命令可表达暂停、继续、重连、休眠 / 恢复和 shutdown coverage；
- 单条连接控制及稳定错误分类；
- 存储健康与最小 `RecoveryFacade`；
- 通过 facade 暴露的设置、目标与实时查询 seam。

C2 只调用这些接口。若 C1 的名称或 DTO 在实现前变化，先在 C1 完成契约评审和回归，再更新本设计；C2 不以直接 SQL 或读取内部状态绕过接口。

### C2 所有权

```text
Windows / Tauri events
        │
        ▼
DesktopRuntimeCoordinator
  single instance · tray · close-to-hide · autostart · sleep/resume · shutdown
        │ lifecycle commands
        ▼
C1 AppFacade / LiveProjection / StorageHealth
        │
        ├──► MonitorChannelBridge ──► Typed decoder / reducer ──► 实时 UI
        ├──► ConnectionListQuery ───► keyset page ──────────────► 虚拟化列表
        ├──► ApplicationShell ──────► route registry / operation progress
        └──► RecoveryFacade ────────► Recovery Shell

Credential Manager ── CredentialStore adapter ── SettingsWorkflow
File dialog plugin ── FileDialogPort ───────────► user-selected handles
```

- `DesktopRuntimeCoordinator` 拥有系统事件到 C1 生命周期命令的映射。
- `SettingsWorkflow` 拥有凭据、机器设置与偏好更新的补偿顺序。
- `MonitorChannelBridge` 拥有订阅、bootstrap、序号校验、resync 和 WebView 重建。
- `ApplicationShell` 拥有稳定导航注册、后续 view extension seam 和 operation progress 展示；C3 / C4 只注册内容，不重写桌面生命周期。
- `FileDialogPort` 是 Rust 专用受限 seam，C2 只冻结选择契约，不读取或写入报告 / 备份内容。
- TypeScript reducer 是前端唯一 DTO cache 更新入口；视图不各自解释 Channel payload。
- Recovery Shell 只依赖 `RecoveryFacade`，与正常 AppFacade 初始化分支隔离。

## 桌面生命周期

### 启动顺序

1. 注册 single-instance；第二实例只向现有实例发送聚焦请求后退出。
2. 解析 `--background`，初始化最小日志、系统 seams 和 C1 后台服务。
3. 根据数据库启动结果进入正常 AppFacade 或 Recovery Shell 分支。
4. 创建托盘；非后台启动才显示主窗口。
5. 正常分支允许 WebView 建立订阅；WebView 是否存在不参与 collector / writer 所有权。

关闭按钮执行 `prevent_close + hide`。托盘“退出”或显式退出命令进入单一 shutdown coordinator：停止接帧、flush C1 writer、结束 coverage、受控 checkpoint / 关闭数据库、删除托盘、退出进程。并发退出请求按同一个状态机去重。

系统休眠、恢复、暂停、继续和立即重连均转为 C1 已定义的生命周期输入。C2 不自行生成 coverage 或修改核算状态。

### 自启动

自启动默认关闭，只在首次引导或设置页经用户确认后变更。保存后读取 OS 实际状态回显，不把插件调用成功等同于最终状态。`--background` 启动不聚焦窗口；用户从托盘或第二实例唤起时才显示。

## 凭据与设置补偿

稳定设置只保存 credential target 和 `hasSecret`，不保存 secret。建议更新序列：

1. 在 Rust 入口校验 loopback 地址、字段长度和目标设置。
2. 需要新 secret 时先写临时 credential target，并立即验证读取。
3. 使用新引用执行 controller probe。
4. 原子保存机器设置 / 目标策略引用。
5. 成功后删除不再引用的旧 credential；删除失败作为可恢复清理项报告，不使新设置失效。
6. 任一步失败时删除临时 credential，并保留旧配置与旧 credential。

Credential Manager 不可用时，`SettingsWorkflow` 只接受标记为 session-only 的 secret handle。handle 仅存在 Rust 受控内存中，不能序列化；退出、配置替换或 probe 取消后清除。v1 不创建 DPAPI、JSON 或数据库 fallback。

UI 只接收 `hasSecret`、credential 状态和脱敏错误。日志和错误映射采用白名单字段。

## 原子 Channel 与前端 reducer

### 订阅协议

C1 在同一 state lock 下登记订阅者、捕获 projection 与当前 `seq`，并把第一条消息构造成：

```text
bootstrap {
  schemaVersion,
  snapshot,
  baseSeq,
  backendTime
}
```

后续消息必须满足 `seq > baseSeq`。前端 reducer 维护 `subscriptionId`、`schemaVersion` 和 `lastSeq`：

- `seq == lastSeq + 1`：应用增量；
- `seq <= lastSeq`：忽略重复或陈旧消息并记录脱敏诊断；
- `seq > lastSeq + 1`：冻结当前实时应用，取消旧订阅并请求 `resync`；
- schema 不支持：停止消费并显示升级 / 重载动作，不做宽松猜测。

窗口重建会生成新的 `subscriptionId`，清空旧 DTO cache，以新 bootstrap 为唯一基线。旧 Channel 的迟到消息因 subscription identity 不匹配而丢弃。

### 消息与状态所有权

Channel 只承载实时、小型、有界消息：摘要变化、keyed connection delta、健康变化和 data version。大列表页、详情和未来报告均走 Commands。

一个 exhaustive reducer 处理所有消息变体。视图只读取 reducer 产出的 typed state，不自行更新累计字节、分类或 coverage。托盘摘要直接读取 Rust live projection，避免依赖 WebView cache。

## 10k 实时数据路径

### Keyed delta 与 coalescing

- C1 完成连接生命周期和字节 delta 核算后，C2 以稳定连接身份键合并 UI 变更。
- 每个约 1 秒发布窗口，每个键最多产生一个最终 `upsert` 或 `remove`；remove 优先于同窗口早期 upsert。
- 摘要使用最新权威 projection；不从 UI 中当前可见行反推总量。
- coalescing 队列设置键数、消息大小和时间边界。超过边界时不无限积压，而是使订阅失效并要求新 bootstrap。
- 原始 snapshot、C1 accounting 输入和 durable commit 不经过 UI coalescer，因此 UI 降载不能丢账。

### 虚拟化与 keyset

连接列表通过稳定 `sort_key + connection_identity` 游标请求页面，默认页和最大页大小沿用 C1 / 父设计冻结值；不使用深 OFFSET。排序或筛选变化会废弃旧游标并重新取第一页。

前端只保留当前窗口及有限 overscan 的行模型。keyed delta 更新缓存中的命中项；未缓存项只更新失效标记，不向数组头部反复插入。详情按 identity 单独查询并在连接消失后转为“最后可见”状态。

10k、1 Hz、至少 30 分钟是 UI 短时峰值 gate。测试只证明实时路径，不证明 10k 平均活跃连接的 30 天数据库容量。

### WebView 重建

WebView 不存在时，Rust 保留 C1 live projection，但不为无订阅者序列化高频 UI payload。collector、accounting、writer、coverage 和 health 继续运行。新 WebView 通过 bootstrap 直接获得当前基线，不回放窗口缺席期间的全部 UI 消息。

## 实时页面与关闭连接

概览 DTO 显式区分 controller meter、attributed observed、分类 / 其他、unattributed gap、over-attributed、coverage 和健康，不提供含糊的单一“全局流量”字段。

关闭连接命令包含当前连接 identity 和用户动作产生的 request ID。`204` 只转换为 `CloseRequestAccepted`；UI 将行标记为“已发送关闭请求”。后续 C1 snapshot 的 remove 才转换为“已关闭”。超时后显示“未确认”，允许刷新，不推断连接仍存在或已关闭。后台任务不得自动发出关闭命令。

## Recovery Shell

启动协调器先获取存储启动结果，再选择互斥分支：

```text
NormalReady  ──► AppFacade + 主应用
RecoveryOnly ──► RecoveryFacade + Recovery Shell
```

Recovery Shell 使用独立、最小、版本化 DTO，只提供：

- 应用 / schema / SQLite 版本和脱敏失败分类；
- migration backup 列表；
- 候选备份校验；
- 打开本地数据目录；
- C3 restore 能力的占位状态。

该分支不得创建普通 read pool、调用 ReportService、读取业务表或假定当前 schema 可迁移。C3 后续只通过扩展 `RecoveryFacade` 接入实际 restore command，不改变 Recovery Shell 的数据库无关启动条件。

## 应用壳、文件与操作进度 seam

- `ApplicationShell` 注册固定 route ID：overview、live、reports、alerts、settings-data。未实现 route 返回版本化 `UnavailableUntilChild`，而不是空白页面或伪数据。
- C3 / C4 通过显式 view registration 替换 reports / alerts 占位；route ID、主导航、窗口生命周期和实时 reducer 不随子任务改写。
- `FileDialogPort` 只允许预声明用途（report-export、backup-create、backup-restore）和对应打开 / 保存模式，返回用户明确选择的句柄 / 路径；WebView 不获得通用 fs scope。
- `OperationProgress` 是独立的低频版本化 DTO：`operationId`、kind、phase、current、total、unit、canCancel、status、redactedError。C2 提供呈现、取消动作和重订阅；C3 的 report / export / backup / restore 实现生产这些消息。
- 文件选择取消、operation 失败或窗口重建不会改变 C1 collector / writer；C2 不把 operation progress 混入高频连接 delta。

## 安全与兼容

- Tauri capability 只授予主窗口所需 app commands 与明确 Windows 插件能力；前端不获得 SQL、任意文件、shell 或 Credential Manager 权限。
- `withGlobalTauri: false`，生产 CSP 只加载打包资源，不使用远程 URL、CDN 或 inline handler。
- TCP secret 只用于 loopback TCP Authorization header；named pipe 不发送 secret。
- C1 的 transport / protocol 错误码原样映射到专门状态，不把 pipe ACL 拒绝显示成密钥错误。
- Command 参数在 Rust 边界校验；前端 decoder 对未知消息变体 fail closed。

## 验证策略

- 生命周期状态机：single-instance、close-to-hide、后台启动、并发退出、休眠 / 恢复和 WebView 重建。
- CredentialStore fake 与 Windows 真机：CRUD、轮换、失败补偿、进程临时 secret 清除和 secret 扫描。
- Channel 确定性并发测试：订阅 / bootstrap 竞态、gap、重复、乱序、旧 subscription 迟到消息和 resync。
- 10k 回放：每秒变更、高 churn、多筛选 / 排序、虚拟滚动、隐藏 / 重建窗口；记录交互延迟、Channel 大小、CPU、RSS 和队列水位。
- Recovery fixture：数据库版本过高、checksum 不符、迁移 / integrity 失败，证明不触碰正常 schema。
- 安装态 smoke：托盘、自启动、单实例、Credential Manager 与明确退出。

## 独立回滚

- C2 发布前可关闭桌面 UI 和 OS capability，保留 C1 无头采集与数据库。
- C2 不拥有 schema migration，因此回滚不得重建、降级或删除数据库。
- 凭据设置变更失败时通过补偿恢复旧引用；回滚只删除确认未被引用的临时 credential。
- Recovery Shell 若不稳定，可保持只读诊断入口并禁用候选验证；不得回退为启动普通业务 schema。
- 已交付的 C1 interface 不因 C2 回滚而改变。

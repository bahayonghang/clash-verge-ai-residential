# 双客户端宿主合同核对

日期：2026-11-06。范围：Windows / Ubuntu；保持现有 AI-only 路由、域名、默认开关、DNS 算法与用户规则优先级。本文是固定源码和仓库测试证据，环境/原生核心实测由 `validation.md` 单独记录。

## 来源与读取方式

按用户要求使用 Ego Browser 读取官方仓库的固定源码。独立任务空间 5 已用 `finish({keep:[]})` 关闭。没有操作客户端、读取个人配置/TOML、连接生产 IPC 或更改网络设置。

- Clash Verge Rev v2.5.10：`c83f7ad529c7c9cb3d2d54a0955ef8c1b22a411b`。
- Clash Verge Rev v2.5.8：`4337bd85758d54d7b9044b2da3c9ac1176eae1f3`。`git ls-remote` 同时返回注解标签对象 `3fee9e8747c88580b67d28eff892ffc3418c07d4` 与 peeled commit；本文使用后者，两版相关代码均已读取。
- FlClash v0.9.2：`7fd417fb9a9a277c6a9339a8ac87f907df7c25f7`。安装包的 `+2026102401` 是父会话盘点值；固定源码不能单独证明发行二进制与源码完全一致。

## Clash Verge Rev

- [2.5.8 脚本入口](https://github.com/clash-verge-rev/clash-verge-rev/blob/4337bd85758d54d7b9044b2da3c9ac1176eae1f3/src-tauri/src/enhance/script.rs#L149) 与 [2.5.10 入口](https://github.com/clash-verge-rev/clash-verge-rev/blob/c83f7ad529c7c9cb3d2d54a0955ef8c1b22a411b/src-tauri/src/enhance/script.rs#L149)：Boa 执行脚本并调用 `main(config, profileName)`，输入/输出经过 JSON；不是 Node CommonJS 模块运行器。
- [2.5.8 管线](https://github.com/clash-verge-rev/clash-verge-rev/blob/4337bd85758d54d7b9044b2da3c9ac1176eae1f3/src-tauri/src/enhance/mod.rs#L916) 与 [2.5.10 管线](https://github.com/clash-verge-rev/clash-verge-rev/blob/c83f7ad529c7c9cb3d2d54a0955ef8c1b22a411b/src-tauri/src/enhance/mod.rs#L916)：规则编辑器 → 内置脚本/TUN/DNS 设置 → 捕获应用字段 → 全局扩展 → Profile 扩展 → 恢复应用字段。
- 两版 `AuthoritativeFields::enforce`（同文件 338–359）对 DNS 使用 `dns.extend(self.dns)`，所以已接管的 `nameserver-policy` 子字段替换整张表，不是递归合并每条策略。需区分 DNS 覆写启用且字段非空与未启用的情况。
- 官网浮动文字与固定版本的恢复行为有差异，见父任务 `research/agent-route-analysis.md`。以目标版本源码及最终配置核对，不能推导所有未来版本均相同。
- 脚本抛错不等于流量阻断；Verge 脚本包装层错误路径会保留输入配置。不得把结构校验称为运行时 fail-closed。

## FlClash 完整脚本调用链与后置覆盖

1. [setup.dart:493–515](https://github.com/chen08209/FlClash/blob/7fd417fb9a9a277c6a9339a8ac87f907df7c25f7/lib/providers/actions/setup.dart#L493) 只为非 Custom Profile 且选择 Script 模式的 Profile 读取所选脚本；有脚本时调用 `handleEvaluate`。
2. [javascript.dart:13–35](https://github.com/chen08209/FlClash/blob/7fd417fb9a9a277c6a9339a8ac87f907df7c25f7/lib/common/javascript.dart#L13) 将 null `proxy-providers` 归一为 Map，JSON 编码后交给 Rust，返回值须为 Map。
3. [api/script.rs](https://github.com/chen08209/FlClash/blob/7fd417fb9a9a277c6a9339a8ac87f907df7c25f7/plugins/rust_api/rust/src/api/script.rs#L3) 委托 [script/mod.rs:41](https://github.com/chen08209/FlClash/blob/7fd417fb9a9a277c6a9339a8ac87f907df7c25f7/plugins/rust_api/rust/src/script/mod.rs#L41)：rquickjs 创建上下文并注册 console，加载脚本，取得 `main`，只传一个解析后的 config。没有 Profile 名称，也不要求 module/require。运行器支持 Promise，本文脚本为同步返回。
4. 返回 setup.dart 后，522 行起调用 `makeRealProfileTask`。因此脚本成功不是最终配置。
5. [task.dart](https://github.com/chen08209/FlClash/blob/7fd417fb9a9a277c6a9339a8ac87f907df7c25f7/lib/common/task.dart#L188)：188 行应用 generalOverrideJson；201 行重写 find-process-mode；213 行起合成 TUN；278–310 行生成 DNS，按选择字段合并 override，可能追加 system://；328–372 行处理规则，附加规则可前置，显式规则集可替换原规则。
6. [clash_config.dart:1483](https://github.com/chen08209/FlClash/blob/7fd417fb9a9a277c6a9339a8ac87f907df7c25f7/lib/models/clash_config.dart#L1483) 的 mergeDnsOverride 只有 fallback-filter 做嵌套合并，其余被选 DNS 字段直接替换。nameserver-policy 不享有逐条合并。
7. [Profile 预览](https://github.com/chen08209/FlClash/blob/7fd417fb9a9a277c6a9339a8ac87f907df7c25f7/lib/views/profiles/preview.dart#L14) 使用 getProfileWithId，再进入同一配置生成管线；可用于最终配置比较，但不是当前核心加载或网络流量证据。
8. [system_dns.dart:233](https://github.com/chen08209/FlClash/blob/7fd417fb9a9a277c6a9339a8ac87f907df7c25f7/lib/common/system_dns.dart#L233) 的系统 DNS 协调器限定 macOS 且非 safe-mode，不能将其当成 Windows/Ubuntu 行为。system:// 本身也不证明 LAN DNS 出口。

接入 UI 来源：[扩展模式](https://github.com/chen08209/FlClash/blob/7fd417fb9a9a277c6a9339a8ac87f907df7c25f7/lib/views/profiles/extend/extend.dart#L152)、[脚本选择](https://github.com/chen08209/FlClash/blob/7fd417fb9a9a277c6a9339a8ac87f907df7c25f7/lib/views/profiles/extend/script.dart#L32)。每个普通 Profile 要选 Script 并绑定脚本。未制造一个假的 profileName 以适配名称覆写。

## FlClashCore 检查边界

- [桌面 main.go](https://github.com/chen08209/FlClash/blob/7fd417fb9a9a277c6a9339a8ac87f907df7c25f7/core/cmd/core/main.go#L11)：无参数退出；第一个参数直接传 StartServer，没有 -v/-h/-t 参数解析。不能照搬独立 Mihomo CLI 检查。
- [server.go:171](https://github.com/chen08209/FlClash/blob/7fd417fb9a9a277c6a9339a8ac87f907df7c25f7/core/server.go#L171) 将该参数作为 IPC 地址拨号，并跟随宿主连接生命周期。不要接管生产 IPC。
- [protocol.go:17–69](https://github.com/chen08209/FlClash/blob/7fd417fb9a9a277c6a9339a8ac87f907df7c25f7/core/protocol.go#L17) 没有版本查询 RPC；InitParams.version 是 SDK 值，不能当核心版本。
- [config.go:66–101](https://github.com/chen08209/FlClash/blob/7fd417fb9a9a277c6a9339a8ac87f907df7c25f7/core/config.go#L66) 的 validateConfig 只读指定文件并做 RawConfig unmarshal，不等价于独立 Mihomo 完整 `-t`。
- [path.dart:77–123](https://github.com/chen08209/FlClash/blob/7fd417fb9a9a277c6a9339a8ac87f907df7c25f7/lib/common/path.dart#L77)：应用支持目录中的 config.yaml、database.sqlite 与 scripts/{id}.js；支持目录由 path_provider 决定。原始配置和数据库可能含秘密，不读取或上传。最终字段投影、现有只读控制器查询或 GUI 检查由父会话在授权范围内处理。

## 缺口与最小实现

既有 regression 已覆盖规则/providers 的 null、错误结构、输入不变、重复增强、DNS 托管清理、负样本和规则顺序。没有重建宿主模拟框架，也没有重复制作路由 fixture。

新增 3 个测试：

- 无 CommonJS、require、process 或 console 的 VM 中，用单参数 main 执行完整公开脚本，JSON 输出与已有合同一致；输入不变、重复执行幂等、错误结构抛错且不改输入。
- 同一边界用带 Profile 名的双参数入口执行。
- 名称专用上游仅在名称传入时选中，缺省名称继续选择通用上游，不猜测 Profile。

`node --test tests/regression.test.js`：104 PASS，0 FAIL（新增前 101 项）。VM 是 Node V8，只证明无模块依赖的调用合同，不冒充 Boa/QuickJS 实机执行。

未复现需要修改根脚本的宿主缺陷，因此 `clash-verge-ai-residential.js`、渲染器、schema、路由默认值和 fixture 均不变。中英文 configuration/local-configuration/multi-profile/troubleshooting 补齐版本、绑定、后置覆盖、告警边界与验收层次。四组合真实宿主执行和真实业务链路仍需运行验收子任务；孤立核心检查不能替代它们。

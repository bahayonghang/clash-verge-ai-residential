# 本机 Cursor 仓库索引与家宽流量证据

## Scope

分析日期为 2026-08-17（Asia/Shanghai）。证据来自本机 Cursor 日志、Windows 网络使用统计和
当前 Clash Verge Rev 生成配置。本文只记录脱敏后的时间、数量、域名和路由结论，不复制仓库内容、
凭据、codebase ID 或完整工作区路径。

## Cursor 索引日志

日志根目录：`C:\Users\lyh\AppData\Roaming\Cursor\logs`。

- `00:45:02`，`anysphere.cursor-retrieval` 明确记录：
  `Creating Indexing Repo client: https://repo42.cursor.sh` 和
  `Creating repo client with backend url: https://repo42.cursor.sh`。
- 同日 Cursor Structured Logs 明确为 `https://repo42.cursor.sh` 创建 `httpVersion: 2`、
  ALPN `h2` 的 transport；本机 User `settings.json` 不包含
  `cursor.general.disableHttp2`，未触发已知的 `api2.cursor.sh` HTTP/1.1 回退开关。
- `01:41:38`，握手返回远端索引为空，随后记录从头上传 `1032` 个文件。
- `15:10:44` 与 `15:10:47`，两个日志流记录同一个根哈希的 `2523` 文件从头上传；这可能包含
  同一操作的重复日志，不能直接按两次网络传输相加。
- `15:15:43` 与 `15:15:44`，记录 `2345` 文件重新上传；同一时段集中出现同步失败。
- `15:16:19`，另一工作区记录 `1430` 文件从头上传。
- 当天所有 `syncing batch ... bytes` 日志字段求和为 `68.35 MiB`，共有 `651` 条 batch 记录；
  `Completed job unsuccessfully, will retry` 有 `190` 条。这个和包含重复日志及重试，是日志声明量的
  上界近似，不是抓包字节数，也不能解释全部系统网卡流量。

日志中可提取到的 Cursor 主机包括：

- `repo42.cursor.sh`
- `api2.cursor.sh`
- `api3.cursor.sh`
- `api4.cursor.sh`
- `agentn.api5.cursor.sh`
- `agentn.global.api5.cursor.sh`
- `marketplace.cursorapi.com`

其中只有 `repo42.cursor.sh` 被索引扩展日志直接标记为 Indexing Repo backend；其他主机出现在
Cursor 日志中不等于它们承载仓库上传。

## Windows 小时流量

Windows `ConnectionProfile.GetNetworkUsageAsync` 在同一连接配置下的 2026-08-17 汇总为：

- 上行 `25.506 GiB`
- 下行 `15.574 GiB`
- 2026-08-16 上行约 `3.96 GiB`

上行最高时段：

| 时段 | 上行 |
| --- | ---: |
| 01:00-02:00 | 9.84 GiB |
| 15:00-16:00 | 4.688 GiB |
| 02:00-03:00 | 1.85 GiB |
| 13:00-14:00 | 1.669 GiB |
| 14:00-15:00 | 1.559 GiB |

两个最大峰值分别与 `01:41` 和 `15:10` 的 Cursor 全量索引上传重合，因此 Cursor 是高概率贡献者。
但 Windows 统计包含同一连接配置下的所有进程、LAN、DIRECT 和代理传输，没有进程级归因能力；
时间相关性不能证明全部 `25.506 GiB` 都来自 Cursor。

## 其他本机 AI 工具排查

- Grok Build 主要活跃于 `19:00-24:00`，请求体日志累计约 `1026.49 MiB`，时间上无法解释
  `01:00-02:00` 与 `15:00-16:00` 两个最大峰值。
- Claude 的检查以更新、插件和 Skills 同步为主，实际下载为零；Remote-SSH 目标是局域网地址且
  二进制已是最新，未发现 GB 级家宽传输证据。

## 当前 Clash 路由事实

当前运行配置和仓库脚本均包含：

```text
DOMAIN-REGEX,^repo[0-9]+\.cursor\.sh$,AI-家宽
```

因此，本机已确认的 `repo42.cursor.sh` 会命中 `AI-家宽`。把该正则从默认启用的 Cursor 核心规则中
拆出，可以确定地让这个索引专属主机回落到原 Profile 路由；是否还有共享主机承载部分上传，必须
结合 Cursor 官方网络配置和进一步 Connections/抓包证据说明，不能从现有日志推断。本机
Cursor 3.16.17 的发行包还表明，显式禁用 HTTP/2 或服务端强制回退时，RepositoryService 可改用
共享的 `api2.cursor.sh`；该模式无法在保留多数 Cursor API 的同时只靠域名规则排除索引。

## Evidence Limits

- Cursor 扩展日志不是线级抓包，batch 字节字段不是网卡计费字节。
- Clash Verge 服务日志没有保存 2026-08-17 早间连接级历史，当前 Profile 只能证明规则存在，
  不能回放昨天每条连接的累计字节。
- 在缺少按进程、域名和时间累计的 Connections 快照前，Cursor 对总流量的精确占比保持
  `UNVERIFIED`。

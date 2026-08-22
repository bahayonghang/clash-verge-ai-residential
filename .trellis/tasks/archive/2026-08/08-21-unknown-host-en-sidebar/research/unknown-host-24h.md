# 本机近 24 小时「未知」主机组成

查询时间：2026-08-21。库路径：`%TEMP%\io.github.bahayonghang.residential-monitor\monitor.sqlite3`（开发态 `data_dir` 为 `temp_dir()/identifier`，不是 LocalAppData）。只读打开。窗口：UTC 近 86400 秒。

截图主机页「近 24 小时」第 1 行：未知，上行 853.8 MiB，下行 3.5 GiB，连接 20692，份额 30.9%。本查询未知行：上行 895321222 B（853.8 MiB），下行 3807849948 B（3.55 GiB），会话 20692。与截图同一行。

## 总量

| 口径 | 会话 | 上行 | 下行 |
|---|---:|---:|---:|
| 全部 | 37128 | 1.87 GiB | 11.49 GiB |
| `s.host` 有值 | 16436 | 1.04 GiB | 7.95 GiB |
| `s.host` 为 NULL（排名 `__unknown__`） | 20692 | 853.8 MiB | 3.55 GiB |

未知下行占总量 30.8%。`connection_session.host` 只有 NULL 或非空，没有空字符串。全部未知会话都有 `connection_session_attr` 行，且 `host_id` 全为 NULL。

## 未知行交叉表（规则 × 链路）

| 规则 | 链路 | 会话 | 上行 | 下行 |
|---|---|---:|---:|---:|
| IPCIDR | DIRECT | 5357 | 10.6 MiB | 3.02 GiB |
| （无规则） | 🇺🇸 US 07>Proxy | 14565 | 813.7 MiB | 511.9 MiB |
| Match | 🇺🇸 US 07>Proxy>Others | 242 | 28.3 MiB | 22.8 MiB |
| （无规则） | DIRECT | 383 | 0.95 MiB | 4.24 MiB |
| IPCIDR | 🇺🇸 US 07>Proxy>Telegram | 21 | 176 KiB | 4.04 MiB |
| （无规则） | 家宽-SOCKS5>AI-家宽 | 124 | 97 KiB | 281 KiB |

字节主导项是 **IPCIDR + DIRECT**（约 85% 未知下行）。连接数主导项是 **无规则 + 机场 Proxy**（14565 / 20692）。

网络：tcp 20226 会话 / 3.32 GiB 下行；udp 466 会话 / 234 MiB 下行。未知会话的 `process_id` 与 `primary_category_id` 全部为 NULL。已知主机的 `process_id` 也几乎全空（16435 / 16436），进程缺失是采集面全局缺口，不是未知行特有。

## 机制

1. 控制器只读 `metadata.host`，不读 `sniffHost`。`destinationIP` 进入实时 DTO，不写入 `connection_session`。见 `controller.rs` `normalize_connection`，`storage.rs` `ensure_session_on`。
2. `ensure_session_on` 首次插入后不更新 `host`。分钟事实路径以 `host=None` 建会话。
3. `RANK_RAW`：`coalesce(s.host,'')=''` → `'__unknown__'`。前端 `isUnknownIdentity` 禁止下钻。
4. neko collector 对照：`domain = metadata.host || metadata.sniffHost || ""`，另有 `ip_stats`。见 `ref/neko-master/apps/collector/src/modules/collector/gateway.collector.ts:438-439`。

## 历史数据边界

已写入的未知会话没有目的 IP。不能把旧 `__unknown__` 行拆成 IP 排名。规则 / 链路 / 网络组成可从现有 attr 查出。

# 父任务边界

## 子任务分工

```text
设置 ui_locale (zh|en)
        │
        ▼
08-19-ui-locale-zh-en
  Rust 目录：托盘、通知、错误句、路由标题、窗口标题
  TS 目录：五页与 Recovery 铬文案
  DTO：bootstrap.settings / 独立字段 uiLocale
        │
        ▼
08-19-live-table-filter
  解析端口 / 入站 / start
  投影填 duration 与 rate
  ConnectionQuery 增加 residentialOnly + clauses
  renderLive 十二列 + 筛选行
```

父任务不改产品代码。跨子任务合同：

- `uiLocale` 取值只有 `zh` | `en`。缺省 `zh`。
- 实时表不得在前端再滤全表；筛选参数必须进 `query_live_connections`。
- `schemaVersion` 保持 `1`。只追加可选字段。
- `identity::PRODUCT_NAME` 与 `DELETE_CONFIRM_PHRASE` 不改。

## 兼容

已安装用户没有 `ui_locale` 键时按中文。旧客户端忽略新查询字段时，服务端对缺省 `residentialOnly` 必须显式：新产品默认 true，旧查询 JSON 无该键时按 false，以免破坏只想看全表的自动化。新产品前端始终传该键。

新产品默认「只看家宽」由前端默认查询带 `residentialOnly: true` 实现，不把服务端缺省改成 true。

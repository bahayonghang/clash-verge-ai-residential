# 升级与卸载

## 升级

首次 v1 必须使用 C0 冻结的早期 NSIS 安装包和 schema fixture。不得用当前代码临时重做旧版本。

当前仓库未找到带 checksum 的 C0 基线资产。C5-AC5 因此未通过。

升级前明确退出托盘应用。升级后应保留数据库、备份、设置、凭据引用、自启动状态和历史告警。自启动状态由 Windows 启动项而非 SQLite 持有；升级后应从设置页回读核对。升级中断必须 fail closed。当前尚无带真实自启动项的已发布基线，跨版本保留行为为 **UNVERIFIED**。

v1 之后改用上一正式 Release。不注册 updater plugin。About 页只提供固定 GitHub Releases 地址。

## 普通卸载

NSIS 普通卸载删除二进制和快捷方式，保留安装目录下的 `data\`、备份、设置和 Credential Manager 项。这不是卸载失败。

若曾启用登录自启动，建议卸载前先在「启动与后台运行」卡片关闭并确认系统回读为关闭。当前未取得卸载器自动清理登录启动项的安装态证据，不应假定普通卸载会移除该项。

## 应用内删除

设置页「删除全部本地数据」先列出声明对象，要求输入确认短语 `删除全部本地数据`，再分项删除。部分失败显示「部分失败」，不显示「已全部删除」。

当前实现清除数据目录中的声明文件和当前进程凭据引用。未再确认前不写本机 Credential Manager。

## 卸载后手动清理

1. `%LOCALAPPDATA%\ResiWatch\data`（主库与 spool）
2. `%LOCALAPPDATA%\io.github.bahayonghang.residential-monitor`（日志）
3. Windows 凭据管理器中的 `io.github.bahayonghang.residential-monitor/controller`
4. 登录自启动中带 `--background` 的产品项（若用户曾经启用）

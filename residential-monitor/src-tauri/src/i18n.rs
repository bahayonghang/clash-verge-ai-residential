//! 界面语言。安装产品名常量不在这里改。

use serde::{Deserialize, Serialize};

pub const SETTING_KEY: &str = "ui_locale";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum UiLocale {
    #[default]
    Zh,
    En,
}

impl UiLocale {
    pub fn parse(raw: Option<&str>) -> Self {
        match raw.map(str::trim) {
            Some("en") => Self::En,
            _ => Self::Zh,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Zh => "zh",
            Self::En => "en",
        }
    }

    pub fn html_lang(self) -> &'static str {
        match self {
            Self::Zh => "zh-CN",
            Self::En => "en",
        }
    }
}

pub fn t(locale: UiLocale, key: &str) -> &'static str {
    match entry(key) {
        Some((zh, en)) => match locale {
            UiLocale::Zh => zh,
            UiLocale::En => en,
        },
        None => "",
    }
}

pub const HEALTH_KEYS: &[&str] = &[
    "connecting",
    "connected",
    "disconnected",
    "tcp_unauthorized",
    "pipe_access_denied",
    "pipe_busy_timeout",
    "endpoint_missing",
    "protocol_incompatible",
    "pid_mismatch",
    "core_restarted",
    "cancelled",
    "non_loopback",
    "storage_failure",
    "storage_backpressure",
    "sleeping_or_clock_gap",
    "paused",
    "coverage_gap",
    "capability_expired",
    "notification_unavailable",
    "migration_failed",
    "restore_failed",
    "no_data",
];

pub fn health_title(locale: UiLocale, session: &str) -> &'static str {
    t(locale, &format!("health.{session}"))
}

pub fn health_action(locale: UiLocale, session: &str) -> &'static str {
    t(locale, &format!("health.{session}.action"))
}

fn entry(key: &str) -> Option<(&'static str, &'static str)> {
    Some(match key {
        "product.display_name" => ("家宽流量监控", "Residential Traffic Monitor"),
        "product.slogan" => (
            "观测下界，不是账单。secret 不会出现在此页面。",
            "Observed lower bound, not a bill. The secret does not appear on this page.",
        ),
        "route.overview" => ("概览", "Overview"),
        "route.live" => ("实时连接", "Live connections"),
        "route.reports" => ("分析报告", "Reports"),
        "route.alerts" => ("告警", "Alerts"),
        "route.settings-data" => ("设置 / 数据管理", "Settings / data"),
        "tray.open" => ("打开窗口", "Open window"),
        "tray.pause" => ("暂停采集", "Pause collection"),
        "tray.resume" => ("继续采集", "Resume collection"),
        "tray.reconnect" => ("立即重连", "Reconnect now"),
        "tray.quit" => ("退出", "Quit"),
        "notify.test_title" => ("测试通知", "Test notification"),
        "notify.test_body" => (
            "这是测试通知，不会写入告警历史。",
            "This is a test notification. It does not write alert history.",
        ),
        "notify.alert_title" => ("家宽流量监控告警", "Residential Traffic Monitor alert"),
        "notify.alert_body" => ("告警事件", "Alert event"),
        "export.html_title" => ("家宽流量报告", "Residential traffic report"),
        "export.html_totals" => (
            "总量 上行 {} 下行 {}。覆盖 {}。",
            "Totals upload {} download {}. Coverage {}.",
        ),
        "export.html_rankings" => (
            "排名（与图表同一结果）",
            "Rankings (same result as the chart)",
        ),
        "export.col_name" => ("名称", "Name"),
        "export.col_upload" => ("上行", "Upload"),
        "export.col_download" => ("下行", "Download"),
        "health.connecting" => ("正在连接控制器", "Connecting to the controller"),
        "health.connecting.action" => ("等待连接完成", "Wait for the connection to finish"),
        "health.connected" => ("已连接", "Connected"),
        "health.connected.action" => ("无需操作", "No action required"),
        "health.disconnected" => ("控制器已断开", "Controller disconnected"),
        "health.disconnected.action" => (
            "检查 Verge / mihomo 后立即重连",
            "Check Verge / mihomo, then reconnect",
        ),
        "health.tcp_unauthorized" => ("TCP 鉴权失败", "TCP authentication failed"),
        "health.tcp_unauthorized.action" => ("检查 secret 后重试", "Check the secret, then retry"),
        "health.pipe_access_denied" => ("管道访问被拒绝", "Named pipe access denied"),
        "health.pipe_access_denied.action" => (
            "改用 TCP External Controller",
            "Use the TCP external controller",
        ),
        "health.pipe_busy_timeout" => ("管道忙超时", "Named pipe busy timeout"),
        "health.pipe_busy_timeout.action" => ("稍后重试或改用 TCP", "Retry later or use TCP"),
        "health.endpoint_missing" => ("控制器端点不存在", "Controller endpoint is missing"),
        "health.endpoint_missing.action" => {
            ("检查地址或重新发现", "Check the address or discover again")
        }
        "health.protocol_incompatible" => ("协议不兼容", "Protocol is not compatible"),
        "health.protocol_incompatible.action" => (
            "启用 TCP External Controller",
            "Enable the TCP external controller",
        ),
        "health.pid_mismatch" => ("管道进程身份不匹配", "Named pipe process identity mismatch"),
        "health.pid_mismatch.action" => ("重新发现后改用 TCP", "Discover again, then use TCP"),
        "health.core_restarted" => ("核心已重启", "Core restarted"),
        "health.core_restarted.action" => ("等待重新建立会话", "Wait for the session to return"),
        "health.cancelled" => ("操作已取消", "Operation cancelled"),
        "health.cancelled.action" => ("可立即重连", "You can reconnect now"),
        "health.non_loopback" => ("拒绝非回环地址", "Non-loopback address rejected"),
        "health.non_loopback.action" => ("改为 127.0.0.1", "Change the address to 127.0.0.1"),
        "health.storage_failure" => ("存储故障", "Storage failure"),
        "health.storage_failure.action" => {
            ("打开恢复界面检查磁盘", "Open recovery and check the disk")
        }
        "health.storage_backpressure" => ("存储背压", "Storage backpressure"),
        "health.storage_backpressure.action" => (
            "等待写入恢复；缺口不是零",
            "Wait for writes to resume; a gap is not zero",
        ),
        "health.sleeping_or_clock_gap" => ("睡眠或时钟缺口", "Sleep or clock gap"),
        "health.sleeping_or_clock_gap.action" => (
            "恢复后核对覆盖区间",
            "After resume, check the coverage window",
        ),
        "health.paused" => ("采集已暂停", "Collection paused"),
        "health.paused.action" => ("在托盘选择继续采集", "Resume collection from the tray"),
        "health.coverage_gap" => ("存在采集缺口", "Collection gap present"),
        "health.coverage_gap.action" => (
            "查看覆盖原因，勿把缺口当零",
            "Read the coverage reason; do not treat a gap as zero",
        ),
        "health.capability_expired" => ("数据能力已过期", "Data capability expired"),
        "health.capability_expired.action" => (
            "缩小范围或改用支持的维度",
            "Narrow the range or use a supported dimension",
        ),
        "health.notification_unavailable" => ("系统通知不可用", "System notifications unavailable"),
        "health.notification_unavailable.action" => {
            ("应用内告警仍完整", "In-app alerts remain complete")
        }
        "health.migration_failed" => ("迁移失败", "Migration failed"),
        "health.migration_failed.action" => (
            "使用 Recovery Shell 恢复备份",
            "Restore a backup in Recovery Shell",
        ),
        "health.restore_failed" => ("恢复失败", "Restore failed"),
        "health.restore_failed.action" => {
            ("当前可用库未覆盖", "The current usable database remains")
        }
        "health.no_data" => ("暂无采样", "No sample yet"),
        "health.no_data.action" => ("确认采集已启动", "Confirm that collection is running"),
        "session.connecting" => ("正在连接控制器。", "Connecting to the controller."),
        "session.connected" => ("已连接。", "Connected."),
        "session.auth_failed" => ("TCP 鉴权失败。", "TCP authentication failed."),
        "session.pipe_access_denied" => ("管道访问被拒绝。", "Named pipe access denied."),
        "session.pipe_busy_timeout" => ("管道忙超时。", "Named pipe busy timeout."),
        "session.endpoint_missing" => ("控制器端点不存在。", "Controller endpoint is missing."),
        "session.protocol_incompatible" => (
            "协议不兼容，请改用 TCP。",
            "Protocol is not compatible. Use TCP.",
        ),
        "session.pid_mismatch" => (
            "管道进程身份不匹配。",
            "Named pipe process identity mismatch.",
        ),
        "session.core_restarted" => ("核心已重启。", "Core restarted."),
        "session.cancelled" => ("操作已取消。", "Operation cancelled."),
        "session.non_loopback" => ("拒绝非回环地址。", "Non-loopback address rejected."),
        "action.retry_connect" => ("重试连接", "Retry the connection"),
        "action.check_secret" => (
            "检查本机 secret 后重试",
            "Check the local secret, then retry",
        ),
        "action.enable_tcp" => (
            "启用 TCP External Controller",
            "Enable the TCP external controller",
        ),
        "action.check_address" => (
            "检查控制器地址或重新发现",
            "Check the controller address or discover again",
        ),
        "action.fix_db" => ("先修复数据库", "Repair the database first"),
        "action.restore_db" => ("先恢复数据库", "Restore the database first"),
        "action.retry" => ("重试", "Retry"),
        "action.check_disk" => ("检查磁盘后重试", "Check the disk, then retry"),
        "action.open_data_dir" => (
            "打开数据目录检查文件",
            "Open the data directory and check files",
        ),
        "action.open_log_dir" => ("打开日志目录", "Open the log directory"),
        "error.open_log_dir" => ("无法打开日志目录。", "Cannot open the log directory."),
        "action.change_loopback" => ("改用 127.0.0.1:端口", "Use 127.0.0.1 and a port"),
        "action.retry_later" => ("稍后重试", "Retry later"),
        "action.change_path" => ("更换路径后重试", "Change the path, then retry"),
        "error.recovery_status" => ("无法读取恢复诊断。", "Cannot read recovery diagnostics."),
        "error.recovery_only" => (
            "当前处于恢复模式，不能保存设置。",
            "Recovery mode cannot save settings.",
        ),
        "error.recovery_only_probe" => (
            "恢复模式不能测试控制器。",
            "Recovery mode cannot probe the controller.",
        ),
        "error.recovery_only_targets" => (
            "恢复模式不能保存目标。",
            "Recovery mode cannot save targets.",
        ),
        "error.recovery_only_report" => (
            "恢复模式不能运行普通报告。",
            "Recovery mode cannot run a normal report.",
        ),
        "error.recovery_only_close" => (
            "恢复模式不能关闭连接。",
            "Recovery mode cannot close a connection.",
        ),
        "error.not_configured" => ("尚未配置控制器。", "The controller is not configured."),
        "action.complete_wizard" => ("完成设置向导", "Finish the setup wizard"),
        "error.encode" => ("设置编码失败。", "Settings encoding failed."),
        "error.storage" => ("设置写入失败。", "Settings write failed."),
        "error.wizard" => ("向导状态写入失败。", "Wizard state write failed."),
        "error.targets" => ("目标写入失败。", "Target write failed."),
        "error.locale" => ("语言设置写入失败。", "Locale setting write failed."),
        "error.theme" => ("外观设置写入失败。", "Theme setting write failed."),
        "error.layout" => ("表格列布局写入失败。", "Table column layout write failed."),
        "error.invalid_address" => ("控制器地址无效。", "Controller address is not valid."),
        "error.diagnostics" => (
            "诊断生成失败。采集未中断。",
            "Diagnostics failed. Collection did not stop.",
        ),
        "error.diagnostics_export" => (
            "诊断导出失败。采集与告警未回滚。",
            "Diagnostics export failed. Collection and alerts did not roll back.",
        ),
        "error.outbox" => ("通知扫描失败。", "Notification scan failed."),
        "error.alert_rules" => ("无法读取告警规则。", "Cannot read alert rules."),
        "error.alert_write" => ("规则写入失败。", "The rule was not written."),
        "error.alert_center" => ("无法读取告警中心。", "Cannot read the alert center."),
        "error.invalid_backup" => ("候选备份无效。", "The backup candidate is not valid."),
        "error.restore_reopen" => (
            "恢复后无法打开数据库，仍停留在 Recovery Shell。",
            "The database could not be opened after restore. Recovery Shell remains.",
        ),
        "action.open_data_dir_short" => ("打开数据目录", "Open the data directory"),
        "action.other_backup" => ("选择其他备份", "Choose another backup"),
        "action.check_backup" => ("检查备份后重试", "Check the backup, then retry"),
        "settings.invalid_address" => ("控制器地址无效。", "Controller address is not valid."),
        "settings.non_loopback" => (
            "TCP 只接受本机回环地址。",
            "TCP accepts loopback addresses only.",
        ),
        "settings.field_too_long" => ("字段超过长度上限。", "The field exceeds the length limit."),
        "settings.too_many_targets" => ("重点目标数量超过上限。", "Too many focus targets."),
        "settings.empty_target" => ("目标名称不能为空。", "A target name cannot be empty."),
        "settings.credential_unavailable" => (
            "凭据存储不可用，只能使用当前进程临时 secret。",
            "Credential storage is unavailable. Only a process-local secret can be used.",
        ),
        "settings.credential_failed" => ("凭据操作失败。", "Credential operation failed."),
        "settings.probe_failed" => ("控制器探测失败。", "Controller probe failed."),
        "settings.unavailable" => ("设置无法保存。", "Settings cannot be saved."),
        "settings.check_controller" => (
            "检查控制器或凭据后重试",
            "Check the controller or credential, then retry",
        ),
        "report.invalid_query" => ("查询参数无效。", "The query is not valid."),
        "report.capability_unsupported" => (
            "当前数据层不支持该查询。",
            "The current data tier does not support this query.",
        ),
        "report.cancelled" => ("查询已取消。", "The query was cancelled."),
        "report.deadline_exceeded" => ("查询超过时限。", "The query exceeded the deadline."),
        "report.token_expired" => (
            "报告快照已过期，请重新运行。",
            "The report snapshot expired. Run the report again.",
        ),
        "report.quota_exceeded" => ("报告快照配额已满。", "The report snapshot quota is full."),
        "report.storage_busy" => ("存储正忙。", "Storage is busy."),
        "report.insufficient_space" => (
            "磁盘空间不足，已停止。",
            "Disk space is insufficient. Stopped.",
        ),
        "report.failed" => ("存储失败。", "Storage failed."),
        "report.action.invalid_query" => (
            "检查时间范围、维度和分页",
            "Check the time range, dimension, and page",
        ),
        "report.action.capability_unsupported" => (
            "缩小范围或改用支持的维度",
            "Narrow the range or use a supported dimension",
        ),
        "report.action.cancelled" => ("可重新运行报告", "You can run the report again"),
        "report.action.deadline_exceeded" => ("缩小范围后重试", "Narrow the range, then retry"),
        "report.action.token_expired" => ("重新运行报告", "Run the report again"),
        "report.action.quota_exceeded" => ("释放旧报告后再试", "Release an old report, then retry"),
        "report.action.storage_busy" => (
            "等待写入完成后再试",
            "Wait for writes to finish, then retry",
        ),
        "report.action.insufficient_space" => ("清理磁盘后重试", "Free disk space, then retry"),
        "report.action.failed" => (
            "打开数据管理检查磁盘",
            "Open data management and check the disk",
        ),
        _ => return None,
    })
}

#[cfg(test)]
mod i18n_tests {
    use super::*;

    #[test]
    fn parse_falls_back_to_zh() {
        assert_eq!(UiLocale::parse(None), UiLocale::Zh);
        assert_eq!(UiLocale::parse(Some("")), UiLocale::Zh);
        assert_eq!(UiLocale::parse(Some("fr")), UiLocale::Zh);
        assert_eq!(UiLocale::parse(Some("en")), UiLocale::En);
        assert_eq!(UiLocale::parse(Some(" en ")), UiLocale::En);
    }

    #[test]
    fn catalog_differs_by_locale() {
        assert_eq!(t(UiLocale::Zh, "product.display_name"), "家宽流量监控");
        assert_eq!(
            t(UiLocale::En, "product.display_name"),
            "Residential Traffic Monitor"
        );
        assert_ne!(
            t(UiLocale::Zh, "session.connected"),
            t(UiLocale::En, "session.connected")
        );
    }

    #[test]
    fn health_keys_resolve_in_both_locales() {
        for key in HEALTH_KEYS {
            let title_zh = health_title(UiLocale::Zh, key);
            let title_en = health_title(UiLocale::En, key);
            assert_ne!(title_zh, *key, "{key} missing zh title");
            assert_ne!(title_en, *key, "{key} missing en title");
            assert_ne!(title_zh, title_en, "{key} title not translated");
        }
    }
}

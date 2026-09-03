//! ResiWatch 历史库 CLI。bin 只做 clap 解析与退出码映射。

mod audit;
mod envelope;
mod maint;
mod patterns;
mod render;
mod resolve;

use crate::c0_contract::SCHEMA_VERSION as KNOWN_SCHEMA;
use crate::c3::query::{
    plan_capability, timezone_offset_secs, DimensionKind, Granularity, RankingRow, ReportError,
    ReportFilters, ReportQuery, RAW_RETAIN_DAYS_DEFAULT, REPORT_DEADLINE_MS, TOP_N_DEFAULT,
    TOP_N_MAX,
};
use crate::c3::service::run_uncached;
use crate::c3::share::query_residential_share;
use crate::c3::sql::RESIDENTIAL_ACCOUNTING_FILTER;
use crate::storage::open_interruptible_reader;
use clap::{Parser, Subcommand, ValueEnum};
use envelope::{
    attribution_from_quality, coverage_from_status, coverage_from_view, CapabilityEcho, Envelope,
    TruncationEcho, WindowEcho, SCHEMA_VERSION,
};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

pub use resolve::resolve_existing_db;

#[derive(Debug, Error)]
pub enum DbCliError {
    #[error("{0}")]
    InvalidArgs(String),
    #[error("库不存在: {}", .0.display())]
    DatabaseMissing(PathBuf),
    #[error("{0}")]
    Busy(String),
    #[error("{0}")]
    FailClosed(String),
    #[error("{0}")]
    Capability(String),
    #[error("{0}")]
    Cancelled(String),
}

impl DbCliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::InvalidArgs(_) => 2,
            Self::DatabaseMissing(_) => 3,
            Self::Busy(_) => 4,
            Self::FailClosed(_) => 5,
            Self::Capability(_) => 6,
            Self::Cancelled(_) => 7,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Json,
    Table,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum RankBy {
    Host,
    Process,
    Chain,
    Rule,
    Network,
    Category,
}

impl From<RankBy> for DimensionKind {
    fn from(value: RankBy) -> Self {
        match value {
            RankBy::Host => DimensionKind::Host,
            RankBy::Process => DimensionKind::Process,
            RankBy::Chain => DimensionKind::Chain,
            RankBy::Rule => DimensionKind::Rule,
            RankBy::Network => DimensionKind::Network,
            RankBy::Category => DimensionKind::Category,
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "monitor-db",
    about = "ResiWatch 历史库只读查询与受控维护",
    disable_help_subcommand = true
)]
struct Cli {
    /// 显式库文件路径
    #[arg(long, global = true)]
    db: Option<PathBuf>,
    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Json)]
    format: OutputFormat,
    #[arg(long, global = true)]
    redact: bool,
    #[arg(long, global = true)]
    since: Option<i64>,
    #[arg(long, global = true)]
    until: Option<i64>,
    #[arg(long, global = true)]
    last: Option<String>,
    #[arg(long, global = true, default_value = "Asia/Shanghai")]
    tz: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// UI 口径 Top N 排名，不返回份额
    Rank {
        #[arg(long, value_enum, default_value_t = RankBy::Host)]
        by: RankBy,
        #[arg(long, default_value_t = TOP_N_DEFAULT)]
        top: u32,
        #[arg(long, default_value_t = true)]
        residential: bool,
    },
    /// 家宽字节占可归因观测的份额
    Share,
    /// 死规则、越界与开关聚合
    Audit {
        #[arg(long)]
        rules: PathBuf,
        #[arg(long)]
        map: Option<PathBuf>,
    },
    /// 维护
    Maint {
        #[command(subcommand)]
        command: MaintCmd,
    },
}

#[derive(Subcommand, Debug)]
enum MaintCmd {
    Status,
    Retention {
        #[arg(long)]
        confirm: bool,
    },
    Backup {
        #[arg(long)]
        path: PathBuf,
        #[arg(long)]
        confirm: bool,
    },
    Restore {
        #[arg(long)]
        path: PathBuf,
        #[arg(long)]
        confirm: bool,
        #[arg(long)]
        offline_confirmed: bool,
    },
    Vacuum {
        #[arg(long)]
        confirm: bool,
        #[arg(long)]
        offline_confirmed: bool,
        #[arg(long)]
        allow_long: bool,
    },
    Purge {
        #[arg(long)]
        confirm: bool,
        #[arg(long)]
        offline_confirmed: bool,
        #[arg(long)]
        phrase: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct QueryOptions {
    pub start_utc: i64,
    pub end_utc: i64,
    pub timezone: String,
    pub now_utc: i64,
    pub redact: bool,
}

pub fn invoke() -> i32 {
    match Cli::try_parse() {
        Ok(cli) => match execute(cli) {
            Ok(code) => code,
            Err(error) => {
                eprintln!("{error}");
                error.exit_code()
            }
        },
        Err(error) => {
            let _ = error.print();
            if error.use_stderr() {
                2
            } else {
                0
            }
        }
    }
}

fn execute(cli: Cli) -> Result<i32, DbCliError> {
    let now_utc = chrono::Utc::now().timestamp();
    let cancel = Arc::new(AtomicBool::new(false));
    match &cli.command {
        Commands::Rank { .. } | Commands::Share | Commands::Audit { .. } => {
            let db = resolve_existing_db(cli.db.as_deref())?;
            let options = query_options(&cli, now_utc)?;
            let (envelope, code) = match &cli.command {
                Commands::Rank {
                    by,
                    top,
                    residential,
                } => run_rank(&db, &options, *by, *top, *residential, &cancel)?,
                Commands::Share => run_share(&db, &options, &cancel)?,
                Commands::Audit { rules, map } => {
                    audit::run_audit(&db, &options, rules, map.as_deref(), &cancel)?
                }
                Commands::Maint { .. } => unreachable!(),
            };
            render::print_envelope(&envelope, cli.format, cli.redact);
            Ok(code)
        }
        Commands::Maint { command } => {
            let db = resolve_existing_db(cli.db.as_deref())?;
            let flags = maint_flags(command);
            let (envelope, code) = match command {
                MaintCmd::Status => maint::run_status(&db, now_utc)?,
                MaintCmd::Retention { .. } => maint::run_retention(&db, &flags, now_utc, &cancel)?,
                MaintCmd::Backup { .. } => maint::run_backup(&db, &flags, now_utc, &cancel)?,
                MaintCmd::Restore { .. } => maint::run_restore(&db, &flags, now_utc, &cancel)?,
                MaintCmd::Vacuum { .. } => maint::run_vacuum(&db, &flags, now_utc)?,
                MaintCmd::Purge { .. } => maint::run_purge(&db, &flags, now_utc)?,
            };
            render::print_envelope(&envelope, cli.format, cli.redact);
            Ok(code)
        }
    }
}

fn maint_flags(command: &MaintCmd) -> maint::MaintFlags {
    match command {
        MaintCmd::Status => maint::MaintFlags {
            confirm: false,
            offline_confirmed: false,
            allow_long: false,
            phrase: None,
            path: None,
        },
        MaintCmd::Retention { confirm } => maint::MaintFlags {
            confirm: *confirm,
            offline_confirmed: false,
            allow_long: false,
            phrase: None,
            path: None,
        },
        MaintCmd::Backup { path, confirm } => maint::MaintFlags {
            confirm: *confirm,
            offline_confirmed: false,
            allow_long: false,
            phrase: None,
            path: Some(path.clone()),
        },
        MaintCmd::Restore {
            path,
            confirm,
            offline_confirmed,
        } => maint::MaintFlags {
            confirm: *confirm,
            offline_confirmed: *offline_confirmed,
            allow_long: false,
            phrase: None,
            path: Some(path.clone()),
        },
        MaintCmd::Vacuum {
            confirm,
            offline_confirmed,
            allow_long,
        } => maint::MaintFlags {
            confirm: *confirm,
            offline_confirmed: *offline_confirmed,
            allow_long: *allow_long,
            phrase: None,
            path: None,
        },
        MaintCmd::Purge {
            confirm,
            offline_confirmed,
            phrase,
        } => maint::MaintFlags {
            confirm: *confirm,
            offline_confirmed: *offline_confirmed,
            allow_long: false,
            phrase: phrase.clone(),
            path: None,
        },
    }
}

fn query_options(cli: &Cli, now_utc: i64) -> Result<QueryOptions, DbCliError> {
    timezone_offset_secs(&cli.tz, now_utc).map_err(map_report)?;
    let (start_utc, end_utc) = parse_window(cli.since, cli.until, cli.last.as_deref(), now_utc)?;
    Ok(QueryOptions {
        start_utc,
        end_utc,
        timezone: cli.tz.clone(),
        now_utc,
        redact: cli.redact,
    })
}

fn parse_window(
    since: Option<i64>,
    until: Option<i64>,
    last: Option<&str>,
    now_utc: i64,
) -> Result<(i64, i64), DbCliError> {
    if last.is_some() && (since.is_some() || until.is_some()) {
        return Err(DbCliError::InvalidArgs(
            "--last 不能与 --since/--until 同时使用".into(),
        ));
    }
    if let Some(spec) = last {
        let secs = parse_last(spec)?;
        return Ok((now_utc.saturating_sub(secs), now_utc));
    }
    match (since, until) {
        (Some(start), Some(end)) if end > start => Ok((start, end)),
        (Some(_), Some(_)) => Err(DbCliError::InvalidArgs("until 必须大于 since".into())),
        (None, None) => Ok((now_utc.saturating_sub(86_400), now_utc)),
        _ => Err(DbCliError::InvalidArgs(
            "--since 与 --until 必须成对出现".into(),
        )),
    }
}

fn parse_last(spec: &str) -> Result<i64, DbCliError> {
    let (digits, factor) = if let Some(body) = spec.strip_suffix('d') {
        (body, 86_400)
    } else if let Some(body) = spec.strip_suffix('h') {
        (body, 3_600)
    } else if let Some(body) = spec.strip_suffix('m') {
        (body, 60)
    } else {
        return Err(DbCliError::InvalidArgs("--last 只接受 Nd / Nh / Nm".into()));
    };
    let count: i64 = digits
        .parse()
        .map_err(|_| DbCliError::InvalidArgs("--last 数字无效".into()))?;
    if count <= 0 {
        return Err(DbCliError::InvalidArgs("--last 必须为正".into()));
    }
    Ok(count.saturating_mul(factor))
}

fn run_rank(
    db: &Path,
    options: &QueryOptions,
    by: RankBy,
    top: u32,
    residential: bool,
    cancel: &Arc<AtomicBool>,
) -> Result<(Envelope, i32), DbCliError> {
    if !(1..=TOP_N_MAX).contains(&top) {
        return Err(DbCliError::InvalidArgs(format!(
            "top 必须在 1..={TOP_N_MAX}"
        )));
    }
    let mut query = base_query(options);
    query.grouping = DimensionKind::from(by);
    query.top_n = top;
    if residential {
        query.filters.category = Some(RESIDENTIAL_ACCOUNTING_FILTER.into());
    }
    match run_uncached(
        db,
        query,
        options.now_utc,
        RAW_RETAIN_DAYS_DEFAULT,
        cancel,
        Some(Duration::from_millis(REPORT_DEADLINE_MS)),
    ) {
        Ok(result) => {
            let rankings: Vec<serde_json::Value> = result
                .rankings
                .iter()
                .map(|row| rank_row_json(row, options.redact))
                .collect();
            let envelope = Envelope {
                schema_version: SCHEMA_VERSION,
                command: "rank".into(),
                generated_utc: result.generated_utc,
                window: WindowEcho {
                    start_utc: options.start_utc,
                    end_utc: options.end_utc,
                    timezone: options.timezone.clone(),
                    granularity: "hour".into(),
                },
                data_version: result.data_version,
                capability: CapabilityEcho {
                    layer: format!("{:?}", result.data_tier).to_ascii_lowercase(),
                    supported: true,
                    reason: None,
                },
                coverage: coverage_from_view(&result.coverage),
                attribution_quality: attribution_from_quality(&result.attribution_quality),
                truncation: TruncationEcho {
                    status: "complete".into(),
                    row_cap: top as i64,
                    rows: rankings.len() as i64,
                },
                named_sql: result.named_sql,
                result: serde_json::json!({ "rankings": rankings }),
                notes: vec!["rank 是 Top N 诊断视图，不返回份额，不承诺全窗口守恒。".into()],
            };
            Ok((envelope, 0))
        }
        Err(error @ ReportError::CapabilityUnsupported(_)) => Ok((
            Envelope::unsupported(
                "rank",
                WindowEcho {
                    start_utc: options.start_utc,
                    end_utc: options.end_utc,
                    timezone: options.timezone.clone(),
                    granularity: "hour".into(),
                },
                options.now_utc,
                &error.to_string(),
                Vec::new(),
            ),
            6,
        )),
        Err(error) => Err(map_report(error)),
    }
}

fn run_share(
    db: &Path,
    options: &QueryOptions,
    _cancel: &Arc<AtomicBool>,
) -> Result<(Envelope, i32), DbCliError> {
    let probe = raw_probe(options);
    match plan_capability(&probe, options.now_utc, RAW_RETAIN_DAYS_DEFAULT) {
        Err(error @ ReportError::CapabilityUnsupported(_)) => {
            return Ok((
                Envelope::unsupported(
                    "share",
                    WindowEcho {
                        start_utc: options.start_utc,
                        end_utc: options.end_utc,
                        timezone: options.timezone.clone(),
                        granularity: "hour".into(),
                    },
                    options.now_utc,
                    &error.to_string(),
                    Vec::new(),
                ),
                6,
            ));
        }
        Err(error) => return Err(map_report(error)),
        Ok(_) => {}
    }
    let share = query_residential_share(
        db,
        options.start_utc,
        options.end_utc,
        &options.timezone,
        options.now_utc,
    )
    .map_err(map_report)?;
    let zero_flow = share.coverage_status != "uncovered"
        && share.residential_upload == Some(0)
        && share.residential_download == Some(0);
    let uncovered = share.coverage_status == "uncovered";
    let envelope = Envelope {
        schema_version: SCHEMA_VERSION,
        command: "share".into(),
        generated_utc: share.generated_utc,
        window: WindowEcho {
            start_utc: options.start_utc,
            end_utc: options.end_utc,
            timezone: options.timezone.clone(),
            granularity: "hour".into(),
        },
        data_version: data_version_from_path(db).unwrap_or(0),
        capability: CapabilityEcho {
            layer: "raw".into(),
            supported: true,
            reason: None,
        },
        coverage: coverage_from_status(&share.coverage_status, None, None),
        attribution_quality: envelope::AttributionEcho {
            status: if uncovered { "unavailable" } else { "complete" }.into(),
            known_bytes: if uncovered {
                None
            } else {
                Some(
                    share.residential_upload.unwrap_or(0) as i64
                        + share.residential_download.unwrap_or(0) as i64,
                )
            },
            missing_bytes: if uncovered { None } else { Some(0) },
        },
        truncation: TruncationEcho {
            status: "complete".into(),
            row_cap: 0,
            rows: 0,
        },
        named_sql: share
            .named_sql
            .iter()
            .map(|item| (*item).to_string())
            .collect(),
        result: serde_json::json!({
            "residentialUpload": share.residential_upload,
            "residentialDownload": share.residential_download,
            "attributedUpload": share.attributed_upload,
            "attributedDownload": share.attributed_download,
            "zeroFlow": zero_flow,
        }),
        notes: Vec::new(),
    };
    Ok((envelope, 0))
}

fn base_query(options: &QueryOptions) -> ReportQuery {
    ReportQuery {
        range_start_utc: options.start_utc,
        range_end_utc: options.end_utc,
        display_timezone: options.timezone.clone(),
        granularity: Granularity::Hour,
        grouping: DimensionKind::Host,
        filters: ReportFilters {
            category: Some(RESIDENTIAL_ACCOUNTING_FILTER.into()),
            ..ReportFilters::default()
        },
        ..ReportQuery::default()
    }
}

fn raw_probe(options: &QueryOptions) -> ReportQuery {
    ReportQuery {
        include_sessions: true,
        ..base_query(options)
    }
}

fn rank_row_json(row: &RankingRow, redact: bool) -> serde_json::Value {
    let unknown = row.identity == "__unknown__";
    serde_json::json!({
        "identity": redact_host(&row.identity, redact),
        "unknown": unknown,
        "upload": row.upload,
        "download": row.download,
        "connectionCount": row.connection_count,
        "zeroFlow": row.upload == 0 && row.download == 0,
    })
}

pub(crate) fn redact_host(value: &str, redact: bool) -> String {
    if redact && value != "__unknown__" && !value.is_empty() {
        render::redact_identity(value)
    } else {
        value.to_string()
    }
}

pub(crate) fn open_reader(path: &Path) -> Result<Connection, DbCliError> {
    let connection = open_interruptible_reader(path).map_err(map_storage)?;
    reject_future_schema(&connection)?;
    Ok(connection)
}

pub(crate) fn reject_future_schema(connection: &Connection) -> Result<i32, DbCliError> {
    let version: i32 = connection
        .query_row("pragma user_version", [], |row| row.get(0))
        .map_err(|_| DbCliError::FailClosed("读取 user_version 失败".into()))?;
    if version > KNOWN_SCHEMA {
        return Err(DbCliError::FailClosed(format!(
            "schema user_version {version} 高于本二进制已知版本 {KNOWN_SCHEMA}"
        )));
    }
    Ok(version)
}

pub(crate) fn data_version(connection: &Connection) -> Result<u64, DbCliError> {
    connection
        .query_row(
            "select watermark from data_version where id = 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|value| value.max(0) as u64)
        .or(Ok(0))
}

fn data_version_from_path(path: &Path) -> Result<u64, DbCliError> {
    let connection = open_reader(path)?;
    data_version(&connection)
}

pub(crate) fn map_report(error: ReportError) -> DbCliError {
    match error {
        ReportError::InvalidQuery(message) => DbCliError::InvalidArgs(message.into()),
        ReportError::CapabilityUnsupported(message) => DbCliError::Capability(message.into()),
        ReportError::Cancelled(message) | ReportError::DeadlineExceeded(message) => {
            DbCliError::Cancelled(message.into())
        }
        ReportError::StorageBusy(message) => DbCliError::Busy(message.into()),
        ReportError::InsufficientSpace(message) => DbCliError::FailClosed(message.into()),
        other => DbCliError::FailClosed(other.to_string()),
    }
}

pub(crate) fn map_storage(error: crate::storage::StorageError) -> DbCliError {
    let text = error.to_string();
    if text.contains("future schema") {
        DbCliError::FailClosed("schema user_version 高于本二进制已知版本".into())
    } else {
        DbCliError::FailClosed(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::StorageCoordinator;
    use tempfile::tempdir;

    fn seeded() -> (tempfile::TempDir, PathBuf) {
        let dir = tempdir().expect("dir");
        let path = dir.path().join("monitor.sqlite3");
        let coordinator = StorageCoordinator::open(&path).expect("open");
        coordinator.seed_report_fixture().expect("seed");
        drop(coordinator);
        (dir, path)
    }

    fn options(now: i64) -> QueryOptions {
        QueryOptions {
            start_utc: 0,
            end_utc: 3_600,
            timezone: "UTC".into(),
            now_utc: now,
            redact: false,
        }
    }

    #[test]
    fn help_lists_all_subcommands() {
        let help = Cli::try_parse_from(["monitor-db", "--help"])
            .unwrap_err()
            .to_string();
        for name in ["rank", "share", "audit", "maint"] {
            assert!(help.contains(name), "{name} missing in {help}");
        }
        let maint = Cli::try_parse_from(["monitor-db", "maint", "--help"])
            .unwrap_err()
            .to_string();
        for name in [
            "status",
            "retention",
            "backup",
            "restore",
            "vacuum",
            "purge",
        ] {
            assert!(maint.contains(name), "{name} missing in {maint}");
        }
    }

    #[test]
    fn missing_db_returns_code_3() {
        let dir = tempdir().expect("dir");
        let path = dir.path().join("missing.sqlite3");
        let err = resolve_existing_db(Some(&path)).expect_err("missing");
        assert_eq!(err.exit_code(), 3);
        assert!(!path.exists());
    }

    #[test]
    fn rank_has_no_share_field() {
        let (_dir, path) = seeded();
        let cancel = Arc::new(AtomicBool::new(false));
        let (envelope, code) =
            run_rank(&path, &options(3_600), RankBy::Host, 20, true, &cancel).expect("rank");
        assert_eq!(code, 0);
        let json = serde_json::to_value(&envelope).expect("json");
        assert!(!json["result"]["rankings"].as_array().unwrap().is_empty());
        assert!(json["result"].get("residentialUpload").is_none());
        assert!(json["result"].get("share").is_none());
        let identities: Vec<&str> = json["result"]["rankings"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|row| row["identity"].as_str())
            .collect();
        assert!(identities.contains(&"a.example"));
        assert!(!identities.iter().any(|item| item.contains("unrelated")));
    }

    #[test]
    fn share_returns_bytes() {
        let (_dir, path) = seeded();
        let cancel = Arc::new(AtomicBool::new(false));
        let (envelope, code) = run_share(&path, &options(3_600), &cancel).expect("share");
        assert_eq!(code, 0);
        let json = serde_json::to_value(&envelope).expect("json");
        assert!(json["result"]["residentialDownload"].as_u64().unwrap() > 0);
    }

    #[test]
    fn capability_unsupported_returns_envelope_6() {
        let (_dir, path) = seeded();
        let cancel = Arc::new(AtomicBool::new(false));
        let mut opts = options(100 * 86_400);
        opts.start_utc = 0;
        opts.end_utc = 3_600;
        let (_envelope, code) = run_share(&path, &opts, &cancel).expect("share");
        assert_eq!(code, 6);
    }

    #[test]
    fn table_matches_json_download() {
        let (_dir, path) = seeded();
        let cancel = Arc::new(AtomicBool::new(false));
        let (envelope, _) = run_share(&path, &options(3_600), &cancel).expect("share");
        let json = serde_json::to_value(&envelope).expect("json");
        let download = json["result"]["residentialDownload"].to_string();
        let mut buf = Vec::new();
        {
            use std::io::Write;
            let rendered = serde_json::to_string(&json["result"]).expect("result");
            writeln!(&mut buf, "{rendered}").ok();
        }
        let text = String::from_utf8(buf).expect("utf8");
        assert!(text.contains(&download));
    }

    #[test]
    fn redact_hides_host() {
        let identity = redact_host("claude.ai", true);
        assert!(!identity.contains("claude"));
        assert!(identity.ends_with("#9"));
    }

    #[test]
    fn cancel_flag_is_recognized() {
        let (_dir, path) = seeded();
        let cancel = Arc::new(AtomicBool::new(true));
        let result = run_uncached(
            &path,
            base_query(&options(3_600)),
            3_600,
            RAW_RETAIN_DAYS_DEFAULT,
            &cancel,
            Some(Duration::from_millis(1)),
        );
        if let Err(error) = result {
            let mapped = map_report(error);
            assert!(matches!(
                mapped,
                DbCliError::Cancelled(_) | DbCliError::FailClosed(_)
            ));
        }
    }

    #[test]
    fn future_schema_fail_closed() {
        let dir = tempdir().expect("dir");
        let path = dir.path().join("future.sqlite3");
        StorageCoordinator::open(&path).expect("open");
        {
            let connection = rusqlite::Connection::open(&path).expect("open");
            connection
                .execute_batch("pragma user_version = 99")
                .expect("ver");
        }
        let err = open_reader(&path).expect_err("future");
        assert_eq!(err.exit_code(), 5);
    }

    #[test]
    fn vacuum_requires_offline_confirmed() {
        let (_dir, path) = seeded();
        let err = maint::run_vacuum(
            &path,
            &maint::MaintFlags {
                confirm: true,
                offline_confirmed: false,
                allow_long: false,
                phrase: None,
                path: None,
            },
            3_600,
        )
        .expect_err("offline");
        assert_eq!(err.exit_code(), 5);
    }

    #[test]
    fn purge_requires_offline_confirmed() {
        let (_dir, path) = seeded();
        let err = maint::run_purge(
            &path,
            &maint::MaintFlags {
                confirm: true,
                offline_confirmed: false,
                allow_long: false,
                phrase: None,
                path: None,
            },
            3_600,
        )
        .expect_err("offline");
        assert_eq!(err.exit_code(), 5);
        assert!(path.exists());
    }

    #[test]
    fn purge_confirm_without_phrase_is_invalid_args() {
        let (_dir, path) = seeded();
        let err = maint::run_purge(
            &path,
            &maint::MaintFlags {
                confirm: true,
                offline_confirmed: true,
                allow_long: false,
                phrase: None,
                path: None,
            },
            3_600,
        )
        .expect_err("phrase");
        assert!(matches!(err, DbCliError::InvalidArgs(_)));
        assert!(err.to_string().contains("purge 需要 --phrase"));
        assert_eq!(err.exit_code(), 2);
        assert!(path.exists());
    }

    #[test]
    fn retention_without_confirm_only_previews() {
        let (_dir, path) = seeded();
        let cancel = Arc::new(AtomicBool::new(false));
        let (envelope, code) = maint::run_retention(
            &path,
            &maint::MaintFlags {
                confirm: false,
                offline_confirmed: false,
                allow_long: false,
                phrase: None,
                path: None,
            },
            3_600,
            &cancel,
        )
        .expect("preview");
        assert_eq!(code, 0);
        let json = serde_json::to_value(&envelope).expect("json");
        assert_eq!(json["result"]["preview"], true);
        assert_eq!(json["result"]["autoDeleteEnabled"], false);
        assert!(envelope
            .notes
            .iter()
            .any(|note| note.contains("auto_delete_enabled=false")));
    }
}

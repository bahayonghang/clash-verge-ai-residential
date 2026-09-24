//! 家宽占可归因观测的份额。未知由 coverage 决定，不由 totals 是否为 0 决定。

use crate::c3::query::{
    coverage_union_secs, timezone_offset_secs, CoverageSlice, ReportError, MAX_RANGE_SECS,
    REPORT_DTO_VERSION,
};
use crate::c3::sql::{render_residential_membership_sql, COVERAGE_RAW, SHARE_RESIDENTIAL_RAW};
use crate::storage::open_interruptible_reader;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResidentialShare {
    pub schema_version: u32,
    pub residential_upload: Option<u64>,
    pub residential_download: Option<u64>,
    pub attributed_upload: Option<u64>,
    pub attributed_download: Option<u64>,
    pub coverage_status: String,
    pub named_sql: Vec<&'static str>,
    pub generated_utc: i64,
    pub target_count: u32,
    pub policy_version: Option<u32>,
}

pub fn query_residential_share(
    db_path: &Path,
    range_start_utc: i64,
    range_end_utc: i64,
    display_timezone: &str,
    now_utc: i64,
) -> Result<ResidentialShare, ReportError> {
    query_residential_share_cancellable(
        db_path,
        range_start_utc,
        range_end_utc,
        display_timezone,
        now_utc,
        &Arc::new(AtomicBool::new(false)),
    )
}

pub fn query_residential_share_cancellable(
    db_path: &Path,
    range_start_utc: i64,
    range_end_utc: i64,
    display_timezone: &str,
    now_utc: i64,
    cancel: &Arc<AtomicBool>,
) -> Result<ResidentialShare, ReportError> {
    let started = Instant::now();
    validate_share_range(range_start_utc, range_end_utc, display_timezone)?;
    crate::c3::service::poll_interrupt(cancel, "residential share")?;
    let reader = open_interruptible_reader(db_path).map_err(map_storage)?;
    let deadline = Duration::from_millis(crate::c3::query::PAGE_DEADLINE_MS);
    crate::c3::service::attach_cancel(&reader, cancel, started, deadline)?;
    reader.execute_batch("begin deferred").map_err(map_sqlite)?;
    let built = query_residential_share_on(&reader, range_start_utc, range_end_utc, now_utc);
    let closed = reader.execute_batch("commit");
    if !reader.is_autocommit() {
        let _ = reader.execute_batch("rollback");
    }
    if cancel.load(Ordering::SeqCst) {
        return Err(ReportError::Cancelled("residential share"));
    }
    if started.elapsed() > deadline {
        return Err(ReportError::DeadlineExceeded("residential share"));
    }
    let report = built?;
    closed.map_err(map_sqlite)?;
    Ok(report)
}

pub fn query_residential_share_on(
    connection: &Connection,
    range_start_utc: i64,
    range_end_utc: i64,
    now_utc: i64,
) -> Result<ResidentialShare, ReportError> {
    build_share(connection, range_start_utc, range_end_utc, now_utc)
}

fn validate_share_range(
    range_start_utc: i64,
    range_end_utc: i64,
    display_timezone: &str,
) -> Result<(), ReportError> {
    if range_end_utc <= range_start_utc {
        return Err(ReportError::InvalidQuery("range"));
    }
    if range_end_utc - range_start_utc > MAX_RANGE_SECS {
        return Err(ReportError::InvalidQuery("range too large"));
    }
    timezone_offset_secs(display_timezone, range_start_utc)?;
    Ok(())
}

fn build_share(
    connection: &Connection,
    range_start_utc: i64,
    range_end_utc: i64,
    now_utc: i64,
) -> Result<ResidentialShare, ReportError> {
    if !crate::c3::service::raw_range_is_retained(connection, range_start_utc, range_end_utc)? {
        return Err(ReportError::CapabilityUnsupported(
            "该区间明细已清理，无法精确计算家宽份额",
        ));
    }
    let (policy_version, target_count) = load_target_meta(connection);
    let slices = load_coverage_slices(connection, range_start_utc, range_end_utc)?;
    let covered_sec = covered_sec_from_slices(range_start_utc, range_end_utc, &slices);
    let coverage_status = coverage_status(&slices, covered_sec, range_end_utc - range_start_utc);
    let mut named_sql = vec!["coverage_raw"];
    if covered_sec == 0 {
        return Ok(ResidentialShare {
            schema_version: REPORT_DTO_VERSION,
            residential_upload: None,
            residential_download: None,
            attributed_upload: None,
            attributed_download: None,
            coverage_status,
            named_sql,
            generated_utc: now_utc,
            target_count,
            policy_version,
        });
    }
    let (residential_upload, residential_download, attributed_upload, attributed_download) =
        load_share_bytes(connection, range_start_utc, range_end_utc)?;
    named_sql.push("share_residential_raw");
    Ok(ResidentialShare {
        schema_version: REPORT_DTO_VERSION,
        residential_upload: Some(residential_upload),
        residential_download: Some(residential_download),
        attributed_upload: Some(attributed_upload),
        attributed_download: Some(attributed_download),
        coverage_status,
        named_sql,
        generated_utc: now_utc,
        target_count,
        policy_version,
    })
}

fn load_coverage_slices(
    connection: &Connection,
    range_start_utc: i64,
    range_end_utc: i64,
) -> Result<Vec<CoverageSlice>, ReportError> {
    let mut statement = connection.prepare(COVERAGE_RAW).map_err(map_sqlite)?;
    let rows = statement
        .query_map(params![range_start_utc, range_end_utc], |row| {
            Ok(CoverageSlice {
                kind: row.get(0)?,
                reason: row.get(1)?,
                started_utc: row.get(2)?,
                ended_utc: row.get(3)?,
            })
        })
        .map_err(map_sqlite)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(map_sqlite)
}

/// 与报告和保留校验共用区间并集，不能把没有观测的时间补成覆盖。
fn covered_sec_from_slices(start: i64, end: i64, slices: &[CoverageSlice]) -> i64 {
    coverage_union_secs(start, end, slices).0
}

fn coverage_status(slices: &[CoverageSlice], covered_sec: i64, span: i64) -> String {
    if covered_sec == 0 {
        return "uncovered".into();
    }
    if covered_sec < span || slices.iter().any(|item| item.kind == "gap") {
        "partial".into()
    } else {
        "covered".into()
    }
}

fn load_share_bytes(
    connection: &Connection,
    range_start_utc: i64,
    range_end_utc: i64,
) -> Result<(u64, u64, u64, u64), ReportError> {
    let start_min = range_start_utc.div_euclid(60);
    let end_min = range_end_utc.div_euclid(60);
    let sql = render_residential_membership_sql(SHARE_RESIDENTIAL_RAW);
    connection
        .query_row(&sql, params![start_min, end_min], |row| {
            Ok((
                as_u64(row.get(0)?),
                as_u64(row.get(1)?),
                as_u64(row.get(2)?),
                as_u64(row.get(3)?),
            ))
        })
        .map_err(map_sqlite)
}

fn as_u64(value: i64) -> u64 {
    u64::try_from(value).unwrap_or(0)
}

fn load_target_meta(connection: &Connection) -> (Option<u32>, u32) {
    let policy_version = connection
        .query_row(
            "select policy_version from target_set where set_id = 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .ok()
        .flatten()
        .and_then(|value| u32::try_from(value).ok());
    let target_count = connection
        .query_row(
            "select count(*) from target_item where set_id = 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0);
    let target_count = u32::try_from(target_count).unwrap_or(0);
    (policy_version, target_count)
}

fn map_sqlite(error: rusqlite::Error) -> ReportError {
    if crate::sqlite_probe::map_sqlite_error(&error) == "cancelled" {
        return ReportError::Cancelled("sqlite interrupt");
    }
    if crate::sqlite_probe::map_sqlite_error(&error) == "busy" {
        return ReportError::StorageBusy("sqlite busy");
    }
    ReportError::Failed("sqlite query")
}

fn map_storage(_error: crate::storage::StorageError) -> ReportError {
    ReportError::Failed("open reader")
}

#[cfg(test)]
mod residential_share_tests {
    use super::*;
    use crate::c3::sql::lookup;
    use crate::storage::StorageCoordinator;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, StorageCoordinator) {
        let dir = tempdir().expect("dir");
        let path = dir.path().join("share.sqlite3");
        let coordinator = StorageCoordinator::open(&path).expect("open");
        coordinator.seed_report_fixture().expect("seed");
        (dir, coordinator)
    }

    fn query(coordinator: &StorageCoordinator, start: i64, end: i64) -> ResidentialShare {
        query_residential_share(coordinator.path(), start, end, "local", 4_000).expect("share")
    }

    #[test]
    fn display_share_cancellation_is_scoped_and_creates_no_spool() {
        let (dir, coordinator) = setup();
        let cancel = Arc::new(AtomicBool::new(true));
        let error = query_residential_share_cancellable(
            coordinator.path(),
            0,
            3_600,
            "UTC",
            4_000,
            &cancel,
        )
        .expect_err("cancelled");
        assert_eq!(error.code(), "cancelled");
        let independent = query_residential_share(coordinator.path(), 0, 3_600, "UTC", 4_000)
            .expect("independent consumer");
        assert_eq!(independent.schema_version, REPORT_DTO_VERSION);
        assert!(!dir.path().join("report-spool").exists());
        assert!(coordinator.connection().is_autocommit());
    }

    #[test]
    fn inverted_range_returns_invalid_query() {
        let dir = tempdir().expect("dir");
        let path = dir.path().join("share.sqlite3");
        let error =
            query_residential_share(&path, 2_000, 1_000, "UTC", 4_000).expect_err("inverted");
        assert_eq!(error.code(), "invalid_query");
    }

    #[test]
    fn range_over_max_returns_invalid_query() {
        let dir = tempdir().expect("dir");
        let path = dir.path().join("share.sqlite3");
        let error =
            query_residential_share(&path, 0, MAX_RANGE_SECS + 1, "UTC", 4_000).expect_err("range");
        assert_eq!(error.code(), "invalid_query");
    }

    #[test]
    fn uncovered_range_returns_four_nones() {
        let (_dir, coordinator) = setup();
        let share = query(&coordinator, 10_000, 11_000);
        assert_eq!(share.residential_upload, None);
        assert_eq!(share.residential_download, None);
        assert_eq!(share.attributed_upload, None);
        assert_eq!(share.attributed_download, None);
        assert_eq!(share.coverage_status, "uncovered");
        assert_eq!(share.named_sql, vec!["coverage_raw"]);
        assert_eq!(
            lookup("share_residential_raw").map(|_| "share_residential_raw"),
            Some("share_residential_raw")
        );
    }

    #[test]
    fn deleted_or_partially_deleted_facts_never_become_a_covered_zero_share() {
        let (_dir, coordinator) = setup();
        coordinator
            .connection()
            .execute_batch(
                "insert into retention_state values('day_exact_v1',0,'deleted','fixture',4000);
            delete from connection_minute where session_pk=1;",
            )
            .expect("部分清理");
        let error = query_residential_share(coordinator.path(), 1000, 2500, "UTC", 4000)
            .expect_err("raw 已不完整");
        assert_eq!(error.code(), "capability_unsupported");
        coordinator
            .connection()
            .execute("delete from connection_minute", [])
            .expect("全部清理");
        assert_eq!(
            query_residential_share(coordinator.path(), 1000, 2500, "UTC", 4000)
                .expect_err("不能返回零")
                .code(),
            "capability_unsupported"
        );
    }

    #[test]
    fn legacy_epoch_and_closed_intervals_do_not_prove_positive_coverage() {
        let (_dir, coordinator) = setup();
        coordinator.connection().execute_batch("delete from coverage_interval;
            insert into coverage_interval(kind,reason,started_utc,ended_utc) values('epoch','legacy',0,4000),('closed','legacy',0,4000);").expect("历史区间");
        let share = query(&coordinator, 1000, 2500);
        assert_eq!(share.attributed_download, None);
        assert_eq!(share.coverage_status, "uncovered");
    }

    #[test]
    fn full_gap_range_returns_four_nones() {
        let (_dir, coordinator) = setup();
        let share = query(&coordinator, 2500, 2800);
        assert_eq!(share.residential_upload, None);
        assert_eq!(share.residential_download, None);
        assert_eq!(share.attributed_upload, None);
        assert_eq!(share.attributed_download, None);
        assert_eq!(share.coverage_status, "uncovered");
        assert_eq!(share.named_sql, vec!["coverage_raw"]);
    }

    #[test]
    fn covered_zero_residential_returns_some_zeros() {
        let (_dir, coordinator) = setup();
        coordinator
            .connection()
            .execute_batch(
                "
                insert or ignore into connection_session(session_pk, epoch_id, connection_id, started_utc, host)
                values (9, 1, 'none', 5000, 'z.example');
                insert or ignore into connection_session_attr(
                    session_pk, host_id, process_id, rule_id, network_id, chain_key,
                    policy_version, primary_category_id, started_utc, ended_utc
                ) values (9, null, null, null, null, 'DIRECT', 1, null, 5000, null);
                insert or ignore into connection_minute(utc_minute, session_pk, upload, download)
                values (90, 9, 7, 11);
                insert or ignore into coverage_interval(interval_id, kind, reason, started_utc, ended_utc)
                values (9, 'covered', 'running', 5400, 5600);
                ",
            )
            .expect("seed zero residential");
        let share = query(&coordinator, 5400, 5600);
        assert_eq!(share.residential_upload, Some(0));
        assert_eq!(share.residential_download, Some(0));
        assert_eq!(share.attributed_upload, Some(7));
        assert_eq!(share.attributed_download, Some(11));
        assert_eq!(share.coverage_status, "covered");
        assert_eq!(
            share.named_sql,
            vec!["coverage_raw", "share_residential_raw"]
        );
    }

    #[test]
    fn named_sql_echo_matches_executed_when_covered() {
        let (_dir, coordinator) = setup();
        let share = query(&coordinator, 1000, 2500);
        assert_eq!(
            share.named_sql,
            vec!["coverage_raw", "share_residential_raw"]
        );
        for name in &share.named_sql {
            assert!(lookup(name).is_some(), "{name}");
        }
        assert!(share.residential_upload.is_some());
        assert!(share.attributed_download.is_some());
        assert_ne!(share.residential_download, Some(0));
    }

    #[test]
    fn legacy_null_categories_recover_from_chain_without_double_counting() {
        let (_dir, coordinator) = setup();
        coordinator
            .connection()
            .execute_batch(
                "insert into target_item(set_id, position, name) values
                    (1, 0, '家宽'), (1, 1, '备用');
                 update connection_session_attr set primary_category_id = null;
                 insert into connection_chain(session_pk, position, node) values
                    (1, 0, 'AI-家宽'),
                    (1, 1, '家宽-SOCKS5'),
                    (1, 2, '备用'),
                    (2, 0, 'PROXY>家宽节点');",
            )
            .expect("legacy chain fixture");
        let share = query(&coordinator, 1000, 2500);
        assert_eq!(share.residential_upload, Some(30));
        assert_eq!(share.residential_download, Some(90));
        assert_eq!(share.residential_upload, share.attributed_upload);
        assert_eq!(share.residential_download, share.attributed_download);
    }
}

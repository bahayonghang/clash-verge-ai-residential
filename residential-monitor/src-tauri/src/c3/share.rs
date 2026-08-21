//! 家宽占可归因观测的份额。未知由 coverage 决定，不由 totals 是否为 0 决定。

use crate::c3::query::{
    timezone_offset_secs, CoverageSlice, ReportError, MAX_RANGE_SECS, REPORT_DTO_VERSION,
};
use crate::c3::sql::{COVERAGE_RAW, SHARE_RESIDENTIAL_RAW};
use crate::storage::open_interruptible_reader;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::path::Path;

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
    validate_share_range(range_start_utc, range_end_utc, display_timezone)?;
    let reader = open_interruptible_reader(db_path).map_err(map_storage)?;
    build_share(&reader, range_start_utc, range_end_utc, now_utc)
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
    let (policy_version, target_count) = load_target_meta(connection);
    let slices = load_coverage_slices(connection, range_start_utc, range_end_utc)?;
    let covered_sec = covered_sec_from_slices(range_start_utc, range_end_utc, &slices);
    let coverage_status = coverage_status(&slices, covered_sec);
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

/// 无切片视为无采集覆盖，covered_sec = 0。有切片时与报告 raw 覆盖相同：span − gap。
fn covered_sec_from_slices(start: i64, end: i64, slices: &[CoverageSlice]) -> i64 {
    if slices.is_empty() {
        return 0;
    }
    let span = (end - start).max(0);
    let gap: i64 = slices
        .iter()
        .filter(|item| item.kind == "gap")
        .map(|item| {
            let ended = item.ended_utc.unwrap_or(end);
            (ended.min(end) - item.started_utc.max(start)).max(0)
        })
        .sum();
    (span - gap).max(0)
}

fn coverage_status(slices: &[CoverageSlice], covered_sec: i64) -> String {
    if covered_sec == 0 {
        return "uncovered".into();
    }
    if slices.iter().any(|item| item.kind == "gap") {
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
    connection
        .query_row(SHARE_RESIDENTIAL_RAW, params![start_min, end_min], |row| {
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
}

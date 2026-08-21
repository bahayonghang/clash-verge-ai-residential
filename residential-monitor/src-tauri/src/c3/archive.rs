//! 冻结小时 / 日 ReportResult 档案。过期删除只针对本表。

use crate::c3::query::{
    closed_local_day_bounds, closed_local_hour_bounds, default_auto_report_query, local_day_bounds,
    local_hour_bounds, query_fingerprint, DimensionKind, Granularity, ReportError, ReportQuery,
    ReportResult, DIMENSION_RETAIN_DAYS, REPORT_DTO_VERSION,
};
use crate::c3::snapshot::ReportSnapshotStore;
use crate::c3::ReportService;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub const ARCHIVE_DTO_VERSION: u32 = 1;
pub const ARCHIVE_HOUR_RETAIN_DAYS: i64 = 30;
pub const ARCHIVE_LIST_DEFAULT: u32 = 50;
pub const ARCHIVE_LIST_MAX: u32 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchiveKind {
    Hour,
    Day,
}

impl ArchiveKind {
    fn as_sql(self) -> &'static str {
        match self {
            Self::Hour => "hour",
            Self::Day => "day",
        }
    }

    fn parse(raw: &str) -> Result<Self, ReportError> {
        match raw {
            "hour" => Ok(Self::Hour),
            "day" => Ok(Self::Day),
            _ => Err(ReportError::InvalidQuery("archive kind")),
        }
    }

    fn granularity(self) -> Granularity {
        match self {
            Self::Hour => Granularity::Hour,
            Self::Day => Granularity::Day,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportArchiveSummary {
    pub archive_id: String,
    pub kind: ArchiveKind,
    pub range_start_utc: i64,
    pub range_end_utc: i64,
    pub display_timezone: String,
    pub grouping: String,
    pub status: String,
    pub generated_utc: i64,
    pub data_version: Option<u64>,
    pub coverage_status: Option<String>,
    pub totals_upload: Option<i64>,
    pub totals_download: Option<i64>,
    pub connection_count: Option<i64>,
    pub error_code: Option<String>,
    pub note_zh: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportArchivePage {
    pub schema_version: u32,
    pub items: Vec<ReportArchiveSummary>,
    pub next: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ArchiveJob {
    pub kind: ArchiveKind,
    pub range_start_utc: i64,
    pub range_end_utc: i64,
    pub query: ReportQuery,
    pub fingerprint: String,
}

pub struct ReportArchiveService;

impl ReportArchiveService {
    pub fn purge_expired(connection: &Connection, now_utc: i64) -> Result<u64, ReportError> {
        let hour_cut = now_utc.saturating_sub(ARCHIVE_HOUR_RETAIN_DAYS * 86_400);
        let day_cut = now_utc.saturating_sub(DIMENSION_RETAIN_DAYS * 86_400);
        let n = connection
            .execute(
                "delete from report_archive
                  where (kind = 'hour' and range_end_utc < ?1)
                     or (kind = 'day' and range_end_utc < ?2)",
                params![hour_cut, day_cut],
            )
            .map_err(map_sqlite)?;
        Ok(n as u64)
    }

    pub fn next_job(
        connection: &Connection,
        now_utc: i64,
    ) -> Result<Option<ArchiveJob>, ReportError> {
        let ok = load_ok_keys(connection)?;
        let hours = walk_closed_periods(
            now_utc,
            now_utc.saturating_sub(ARCHIVE_HOUR_RETAIN_DAYS * 86_400),
            local_hour_bounds,
            closed_local_hour_bounds,
        )?;
        let days = walk_closed_periods(
            now_utc,
            now_utc.saturating_sub(DIMENSION_RETAIN_DAYS * 86_400),
            local_day_bounds,
            closed_local_day_bounds,
        )?;
        if let Some(range) = hours.first() {
            if let Some(job) = job_if_missing(ArchiveKind::Hour, *range, &ok) {
                return Ok(Some(job));
            }
        }
        if let Some(range) = days.first() {
            if let Some(job) = job_if_missing(ArchiveKind::Day, *range, &ok) {
                return Ok(Some(job));
            }
        }
        for range in hours.iter().skip(1) {
            if let Some(job) = job_if_missing(ArchiveKind::Hour, *range, &ok) {
                return Ok(Some(job));
            }
        }
        for range in days.iter().skip(1) {
            if let Some(job) = job_if_missing(ArchiveKind::Day, *range, &ok) {
                return Ok(Some(job));
            }
        }
        Ok(None)
    }

    pub fn persist_outcome(
        connection: &Connection,
        job: &ArchiveJob,
        outcome: Result<ReportResult, ReportError>,
        now_utc: i64,
    ) -> Result<(), ReportError> {
        match outcome {
            Ok(result) => persist_ok(connection, job, result, now_utc),
            Err(error) => persist_failed(connection, job, &error, now_utc),
        }
    }

    pub fn list(
        connection: &Connection,
        kind: Option<&str>,
        after: Option<&str>,
        limit: Option<u32>,
    ) -> Result<ReportArchivePage, ReportError> {
        let kind_filter = match kind {
            None => None,
            Some(raw) => Some(ArchiveKind::parse(raw)?),
        };
        let limit = limit
            .unwrap_or(ARCHIVE_LIST_DEFAULT)
            .clamp(1, ARCHIVE_LIST_MAX) as i64;
        let (after_start, after_id) = parse_cursor(after);
        let kind_sql = kind_filter.map(ArchiveKind::as_sql).unwrap_or("");
        let mut statement = connection
            .prepare(
                "select archive_id, kind, range_start_utc, range_end_utc, display_timezone,
                        grouping, status, generated_utc, data_version, coverage_status,
                        totals_upload, totals_download, connection_count, error_code, note_zh
                   from report_archive
                  where (?1 = 0 or kind = ?2)
                    and (?3 = 0 or range_start_utc < ?4
                         or (range_start_utc = ?4 and archive_id < ?5))
                  order by range_start_utc desc, archive_id desc
                  limit ?6",
            )
            .map_err(map_sqlite)?;
        let rows = statement
            .query_map(
                params![
                    i64::from(kind_filter.is_some()),
                    kind_sql,
                    i64::from(after_start.is_some()),
                    after_start.unwrap_or(0),
                    after_id.as_deref().unwrap_or(""),
                    limit + 1
                ],
                map_summary_row,
            )
            .map_err(map_sqlite)?;
        let mut items = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| ReportError::Failed("archive list"))?;
        let next = if items.len() > limit as usize {
            items.truncate(limit as usize);
            items
                .last()
                .map(|item| format!("{}|{}", item.range_start_utc, item.archive_id))
        } else {
            None
        };
        Ok(ReportArchivePage {
            schema_version: ARCHIVE_DTO_VERSION,
            items,
            next,
        })
    }

    pub fn load_frozen(
        connection: &Connection,
        archive_id: &str,
    ) -> Result<ReportResult, ReportError> {
        let (status, json): (String, Option<String>) = connection
            .query_row(
                "select status, result_json from report_archive where archive_id = ?1",
                [archive_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(map_sqlite)?
            .ok_or(ReportError::InvalidQuery("archive id"))?;
        if status != "ok" {
            return Err(ReportError::Failed("archive failed"));
        }
        let json = json.ok_or(ReportError::Failed("archive json missing"))?;
        let mut result: ReportResult =
            serde_json::from_str(&json).map_err(|_| ReportError::Failed("archive json"))?;
        if result.schema_version != REPORT_DTO_VERSION {
            return Err(ReportError::Failed("archive schema"));
        }
        result.reconcile_legacy_attribution_quality();
        Ok(result)
    }

    pub fn get(
        connection: &Connection,
        store: &mut ReportSnapshotStore,
        archive_id: &str,
        now_utc: i64,
    ) -> Result<ReportResult, ReportError> {
        let result = Self::load_frozen(connection, archive_id)?;
        let query = result.query_echo.clone();
        store.insert(&query, result, now_utc, false)
    }
}

fn persist_ok(
    connection: &Connection,
    job: &ArchiveJob,
    mut result: ReportResult,
    now_utc: i64,
) -> Result<(), ReportError> {
    if existing_ok(connection, job)? {
        return Ok(());
    }
    result.report_snapshot_token.clear();
    let json = serde_json::to_string(&result).map_err(|_| ReportError::Failed("encode archive"))?;
    let grouping = grouping_sql(job.query.grouping);
    if let Some(archive_id) = existing_id(connection, job)? {
        connection
            .execute(
                "update report_archive set
                    range_end_utc = ?1,
                    display_timezone = ?2,
                    grouping = ?3,
                    status = 'ok',
                    generated_utc = ?4,
                    data_version = ?5,
                    coverage_status = ?6,
                    totals_upload = ?7,
                    totals_download = ?8,
                    connection_count = ?9,
                    result_json = ?10,
                    error_code = null,
                    note_zh = null
                  where archive_id = ?11",
                params![
                    job.range_end_utc,
                    job.query.display_timezone,
                    grouping,
                    result.generated_utc,
                    result.data_version as i64,
                    result.coverage.status,
                    result.totals.upload,
                    result.totals.download,
                    result.totals.connection_count,
                    json,
                    archive_id
                ],
            )
            .map_err(map_sqlite)?;
        return Ok(());
    }
    let archive_id = new_archive_id(now_utc, job);
    connection
        .execute(
            "insert into report_archive(
                archive_id, kind, range_start_utc, range_end_utc, display_timezone,
                grouping, query_fingerprint, status, generated_utc, data_version,
                coverage_status, totals_upload, totals_download, connection_count,
                result_json, error_code, note_zh
             ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'ok', ?8, ?9, ?10, ?11, ?12, ?13, ?14, null, null)",
            params![
                archive_id,
                job.kind.as_sql(),
                job.range_start_utc,
                job.range_end_utc,
                job.query.display_timezone,
                grouping,
                job.fingerprint,
                result.generated_utc,
                result.data_version as i64,
                result.coverage.status,
                result.totals.upload,
                result.totals.download,
                result.totals.connection_count,
                json
            ],
        )
        .map_err(map_sqlite)?;
    Ok(())
}

fn persist_failed(
    connection: &Connection,
    job: &ArchiveJob,
    error: &ReportError,
    now_utc: i64,
) -> Result<(), ReportError> {
    if existing_ok(connection, job)? {
        return Ok(());
    }
    let grouping = grouping_sql(job.query.grouping);
    let code = error.code();
    let note = error.message_zh();
    if let Some(archive_id) = existing_id(connection, job)? {
        connection
            .execute(
                "update report_archive set
                    status = 'failed',
                    generated_utc = ?1,
                    data_version = null,
                    coverage_status = null,
                    totals_upload = null,
                    totals_download = null,
                    connection_count = null,
                    result_json = null,
                    error_code = ?2,
                    note_zh = ?3
                  where archive_id = ?4",
                params![now_utc, code, note, archive_id],
            )
            .map_err(map_sqlite)?;
        return Ok(());
    }
    let archive_id = new_archive_id(now_utc, job);
    connection
        .execute(
            "insert into report_archive(
                archive_id, kind, range_start_utc, range_end_utc, display_timezone,
                grouping, query_fingerprint, status, generated_utc, data_version,
                coverage_status, totals_upload, totals_download, connection_count,
                result_json, error_code, note_zh
             ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'failed', ?8, null, null, null, null, null, null, ?9, ?10)",
            params![
                archive_id,
                job.kind.as_sql(),
                job.range_start_utc,
                job.range_end_utc,
                job.query.display_timezone,
                grouping,
                job.fingerprint,
                now_utc,
                code,
                note
            ],
        )
        .map_err(map_sqlite)?;
    Ok(())
}

fn job_if_missing(
    kind: ArchiveKind,
    range: (i64, i64),
    ok: &HashSet<(String, i64, String)>,
) -> Option<ArchiveJob> {
    if range.1 <= range.0 {
        return None;
    }
    let query = default_auto_report_query(kind.granularity(), range.0, range.1);
    let fingerprint = query_fingerprint(&query);
    if ok.contains(&(kind.as_sql().to_string(), range.0, fingerprint.clone())) {
        return None;
    }
    Some(ArchiveJob {
        kind,
        range_start_utc: range.0,
        range_end_utc: range.1,
        query,
        fingerprint,
    })
}

type TimeBoundsFn = fn(&str, i64) -> Result<(i64, i64), ReportError>;

fn walk_closed_periods(
    now_utc: i64,
    retain_end: i64,
    bounds: TimeBoundsFn,
    closed: TimeBoundsFn,
) -> Result<Vec<(i64, i64)>, ReportError> {
    let (mut start, mut end) = match closed("local", now_utc) {
        Ok(range) => range,
        Err(_) => return Ok(Vec::new()),
    };
    let mut out = Vec::new();
    while end >= retain_end {
        if end > start {
            out.push((start, end));
        }
        if start <= 0 {
            break;
        }
        match bounds("local", start.saturating_sub(1)) {
            Ok((prev_s, prev_e)) => {
                if prev_s >= start {
                    break;
                }
                start = prev_s;
                end = prev_e;
            }
            Err(_) => break,
        }
        if out.len() > 2_000 {
            break;
        }
    }
    Ok(out)
}

fn load_ok_keys(connection: &Connection) -> Result<HashSet<(String, i64, String)>, ReportError> {
    let mut statement = connection
        .prepare(
            "select kind, range_start_utc, query_fingerprint from report_archive
              where status = 'ok'",
        )
        .map_err(map_sqlite)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(map_sqlite)?;
    let mut set = HashSet::new();
    for row in rows {
        set.insert(row.map_err(|_| ReportError::Failed("archive keys"))?);
    }
    Ok(set)
}

fn existing_ok(connection: &Connection, job: &ArchiveJob) -> Result<bool, ReportError> {
    let found: Option<i64> = connection
        .query_row(
            "select 1 from report_archive
              where kind = ?1 and range_start_utc = ?2 and query_fingerprint = ?3
                and status = 'ok'",
            params![job.kind.as_sql(), job.range_start_utc, job.fingerprint],
            |row| row.get(0),
        )
        .optional()
        .map_err(map_sqlite)?;
    Ok(found.is_some())
}

fn existing_id(connection: &Connection, job: &ArchiveJob) -> Result<Option<String>, ReportError> {
    connection
        .query_row(
            "select archive_id from report_archive
              where kind = ?1 and range_start_utc = ?2 and query_fingerprint = ?3",
            params![job.kind.as_sql(), job.range_start_utc, job.fingerprint],
            |row| row.get(0),
        )
        .optional()
        .map_err(map_sqlite)
}

fn map_summary_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ReportArchiveSummary> {
    Ok(ReportArchiveSummary {
        archive_id: row.get(0)?,
        kind: ArchiveKind::parse(&row.get::<_, String>(1)?).unwrap_or(ArchiveKind::Hour),
        range_start_utc: row.get(2)?,
        range_end_utc: row.get(3)?,
        display_timezone: row.get(4)?,
        grouping: row.get(5)?,
        status: row.get(6)?,
        generated_utc: row.get(7)?,
        data_version: row.get::<_, Option<i64>>(8)?.map(|item| item as u64),
        coverage_status: row.get(9)?,
        totals_upload: row.get(10)?,
        totals_download: row.get(11)?,
        connection_count: row.get(12)?,
        error_code: row.get(13)?,
        note_zh: row.get(14)?,
    })
}

fn parse_cursor(after: Option<&str>) -> (Option<i64>, Option<String>) {
    after
        .and_then(|item| item.split_once('|'))
        .and_then(|(utc, id)| {
            utc.parse()
                .ok()
                .map(|value| (Some(value), Some(id.to_string())))
        })
        .unwrap_or((None, None))
}

fn grouping_sql(kind: DimensionKind) -> &'static str {
    match kind {
        DimensionKind::Category => "category",
        DimensionKind::Host => "host",
        DimensionKind::Process => "process",
        DimensionKind::Rule => "rule",
        DimensionKind::Chain => "chain",
        DimensionKind::Network => "network",
    }
}

fn new_archive_id(now_utc: i64, job: &ArchiveJob) -> String {
    let raw = format!(
        "c3-archive-{}-{}-{}-{}",
        now_utc,
        job.kind.as_sql(),
        job.range_start_utc,
        job.fingerprint
    );
    hex::encode(Sha256::digest(raw.as_bytes()))[..32].to_string()
}

fn map_sqlite(error: rusqlite::Error) -> ReportError {
    if crate::sqlite_probe::map_sqlite_error(&error) == "busy" {
        return ReportError::StorageBusy("archive sqlite");
    }
    ReportError::Failed("archive sqlite")
}

/// 测试与单连接路径：查询走独立 reader，写入走传入的 writer 连接。
pub fn run_one_archive_job(
    connection: &Connection,
    db_path: &Path,
    spool_dir: &Path,
    now_utc: i64,
    raw_retain_days: i64,
) -> Result<bool, ReportError> {
    ReportArchiveService::purge_expired(connection, now_utc)?;
    let Some(job) = ReportArchiveService::next_job(connection, now_utc)? else {
        return Ok(false);
    };
    let mut store = ReportSnapshotStore::open(spool_dir);
    let cancel = Arc::new(AtomicBool::new(false));
    let outcome = ReportService::run(
        db_path,
        &mut store,
        job.query.clone(),
        now_utc,
        raw_retain_days,
        &cancel,
        None,
    );
    let token = outcome
        .as_ref()
        .ok()
        .map(|item| item.report_snapshot_token.clone());
    ReportArchiveService::persist_outcome(connection, &job, outcome, now_utc)?;
    if let Some(token) = token {
        store.release(&token);
    }
    Ok(true)
}

#[cfg(test)]
mod archive_service_tests {
    use super::*;
    use crate::c3::query::{
        empty_result, AttributionStatus, CapabilityPlan, DataTier, DrilldownCapability,
        TargetPolicy,
    };
    use crate::storage::StorageCoordinator;
    use tempfile::tempdir;

    fn dummy_ok(query: ReportQuery, now: i64) -> ReportResult {
        let plan = CapabilityPlan {
            tier: DataTier::Raw,
            named_sql: vec![],
            drilldown: DrilldownCapability {
                sessions: true,
                current_policy: true,
                cross_dimension: true,
                exact_top_n: true,
                note_zh: String::new(),
            },
            deadline_ms: 10_000,
        };
        let mut result = empty_result(query, &plan, 7);
        result.generated_utc = now;
        result.totals.upload = 11;
        result.totals.download = 22;
        result.totals.connection_count = 3;
        result.coverage.status = "covered".into();
        result.policy_metadata.target_policy = TargetPolicy::Historical;
        result
    }

    fn job_at(kind: ArchiveKind, start: i64, end: i64) -> ArchiveJob {
        let query = default_auto_report_query(kind.granularity(), start, end);
        let fingerprint = query_fingerprint(&query);
        ArchiveJob {
            kind,
            range_start_utc: start,
            range_end_utc: end,
            query,
            fingerprint,
        }
    }

    #[test]
    fn insert_ok_is_idempotent() {
        let dir = tempdir().expect("dir");
        let coordinator = StorageCoordinator::open(&dir.path().join("a.sqlite3")).expect("open");
        let now = 20 * 86_400;
        let job = job_at(ArchiveKind::Hour, now - 7_200, now - 3_600);
        let result = dummy_ok(job.query.clone(), now);
        ReportArchiveService::persist_outcome(
            coordinator.connection(),
            &job,
            Ok(result.clone()),
            now,
        )
        .expect("first");
        let mut changed = result.clone();
        changed.totals.download = 99;
        ReportArchiveService::persist_outcome(coordinator.connection(), &job, Ok(changed), now)
            .expect("second");
        let page =
            ReportArchiveService::list(coordinator.connection(), None, None, None).expect("list");
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].status, "ok");
        assert_eq!(page.items[0].totals_download, Some(22));
        let mut store = ReportSnapshotStore::open(dir.path());
        let loaded = ReportArchiveService::get(
            coordinator.connection(),
            &mut store,
            &page.items[0].archive_id,
            now,
        )
        .expect("get");
        assert_eq!(loaded.totals.download, 22);
        assert!(!loaded.report_snapshot_token.is_empty());
    }

    #[test]
    fn failed_row_can_be_replaced_by_ok() {
        let dir = tempdir().expect("dir");
        let coordinator = StorageCoordinator::open(&dir.path().join("a.sqlite3")).expect("open");
        let now = 20 * 86_400;
        let job = job_at(ArchiveKind::Day, now - 86_400, now);
        ReportArchiveService::persist_outcome(
            coordinator.connection(),
            &job,
            Err(ReportError::DeadlineExceeded("tick")),
            now,
        )
        .expect("fail");
        let page = ReportArchiveService::list(coordinator.connection(), Some("day"), None, None)
            .expect("list");
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].status, "failed");
        let archive_id = page.items[0].archive_id.clone();
        ReportArchiveService::persist_outcome(
            coordinator.connection(),
            &job,
            Ok(dummy_ok(job.query.clone(), now)),
            now,
        )
        .expect("ok");
        let page = ReportArchiveService::list(coordinator.connection(), Some("day"), None, None)
            .expect("list2");
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].archive_id, archive_id);
        assert_eq!(page.items[0].status, "ok");
        assert_eq!(page.items[0].totals_upload, Some(11));
    }

    #[test]
    fn purge_expired_hour_and_day() {
        let dir = tempdir().expect("dir");
        let coordinator = StorageCoordinator::open(&dir.path().join("a.sqlite3")).expect("open");
        let now = 400 * 86_400;
        let old_hour = job_at(ArchiveKind::Hour, now - 32 * 86_400, now - 31 * 86_400);
        let old_day = job_at(ArchiveKind::Day, now - 400 * 86_400, now - 399 * 86_400);
        let live = job_at(ArchiveKind::Hour, now - 7_200, now - 3_600);
        ReportArchiveService::persist_outcome(
            coordinator.connection(),
            &old_hour,
            Ok(dummy_ok(old_hour.query.clone(), now)),
            now,
        )
        .expect("old hour");
        ReportArchiveService::persist_outcome(
            coordinator.connection(),
            &old_day,
            Ok(dummy_ok(old_day.query.clone(), now)),
            now,
        )
        .expect("old day");
        ReportArchiveService::persist_outcome(
            coordinator.connection(),
            &live,
            Ok(dummy_ok(live.query.clone(), now)),
            now,
        )
        .expect("live");
        let removed =
            ReportArchiveService::purge_expired(coordinator.connection(), now).expect("purge");
        assert_eq!(removed, 2);
        let page =
            ReportArchiveService::list(coordinator.connection(), None, None, None).expect("list");
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].range_start_utc, live.range_start_utc);
    }

    #[test]
    fn persist_survives_reopen() {
        let dir = tempdir().expect("dir");
        let path = dir.path().join("a.sqlite3");
        let now = 20 * 86_400;
        let job = job_at(ArchiveKind::Hour, now - 7_200, now - 3_600);
        let frozen = {
            let coordinator = StorageCoordinator::open(&path).expect("open");
            let result = dummy_ok(job.query.clone(), now);
            ReportArchiveService::persist_outcome(coordinator.connection(), &job, Ok(result), now)
                .expect("insert");
            let page = ReportArchiveService::list(coordinator.connection(), None, None, None)
                .expect("list");
            let json: String = coordinator
                .connection()
                .query_row(
                    "select result_json from report_archive where archive_id = ?1",
                    [&page.items[0].archive_id],
                    |row| row.get(0),
                )
                .expect("json");
            json
        };
        let coordinator = StorageCoordinator::open(&path).expect("reopen");
        let page =
            ReportArchiveService::list(coordinator.connection(), None, None, None).expect("list2");
        assert_eq!(page.items.len(), 1);
        let json: String = coordinator
            .connection()
            .query_row(
                "select result_json from report_archive where archive_id = ?1",
                [&page.items[0].archive_id],
                |row| row.get(0),
            )
            .expect("json2");
        assert_eq!(json, frozen);
        let decoded: ReportResult = serde_json::from_str(&json).expect("decode");
        assert_eq!(decoded.totals.download, 22);
        assert_eq!(decoded.coverage.status, "covered");
        assert!(decoded.report_snapshot_token.is_empty());
    }

    #[test]
    fn legacy_frozen_result_without_attribution_quality_stays_loadable() {
        let dir = tempdir().expect("dir");
        let coordinator = StorageCoordinator::open(&dir.path().join("a.sqlite3")).expect("open");
        let now = 20 * 86_400;
        let job = job_at(ArchiveKind::Hour, now - 7_200, now - 3_600);
        ReportArchiveService::persist_outcome(
            coordinator.connection(),
            &job,
            Ok(dummy_ok(job.query.clone(), now)),
            now,
        )
        .expect("insert");
        let page =
            ReportArchiveService::list(coordinator.connection(), None, None, None).expect("list");
        let archive_id = &page.items[0].archive_id;
        let json: String = coordinator
            .connection()
            .query_row(
                "select result_json from report_archive where archive_id = ?1",
                [archive_id],
                |row| row.get(0),
            )
            .expect("json");
        let mut legacy: serde_json::Value = serde_json::from_str(&json).expect("decode");
        legacy
            .as_object_mut()
            .expect("object")
            .remove("attributionQuality");
        coordinator
            .connection()
            .execute(
                "update report_archive set result_json = ?1 where archive_id = ?2",
                params![serde_json::to_string(&legacy).expect("encode"), archive_id],
            )
            .expect("downgrade fixture");

        let loaded = ReportArchiveService::load_frozen(coordinator.connection(), archive_id)
            .expect("legacy archive");
        assert_eq!(
            loaded.attribution_quality.status,
            AttributionStatus::Unavailable
        );
        assert_eq!(loaded.attribution_quality.known_upload, 0);
        assert_eq!(loaded.attribution_quality.known_download, 0);
        assert_eq!(loaded.attribution_quality.missing_upload, 11);
        assert_eq!(loaded.attribution_quality.missing_download, 22);
        assert_eq!(loaded.attribution_quality.missing_connections, 3);
    }

    #[test]
    fn next_job_prefers_newest_hour_then_newest_day() {
        let dir = tempdir().expect("dir");
        let coordinator = StorageCoordinator::open(&dir.path().join("a.sqlite3")).expect("open");
        let now = chrono::Utc::now().timestamp();
        let first = ReportArchiveService::next_job(coordinator.connection(), now)
            .expect("job")
            .expect("hour");
        assert_eq!(first.kind, ArchiveKind::Hour);
        let (expect_s, expect_e) = closed_local_hour_bounds("local", now).expect("closed hour");
        assert_eq!(first.range_start_utc, expect_s);
        assert_eq!(first.range_end_utc, expect_e);
        ReportArchiveService::persist_outcome(
            coordinator.connection(),
            &first,
            Ok(dummy_ok(first.query.clone(), now)),
            now,
        )
        .expect("hour ok");
        let second = ReportArchiveService::next_job(coordinator.connection(), now)
            .expect("job2")
            .expect("day");
        assert_eq!(second.kind, ArchiveKind::Day);
        let (day_s, day_e) = closed_local_day_bounds("local", now).expect("closed day");
        assert_eq!(second.range_start_utc, day_s);
        assert_eq!(second.range_end_utc, day_e);
        ReportArchiveService::persist_outcome(
            coordinator.connection(),
            &second,
            Ok(dummy_ok(second.query.clone(), now)),
            now,
        )
        .expect("day ok");
        let third = ReportArchiveService::next_job(coordinator.connection(), now)
            .expect("job3")
            .expect("older");
        assert_eq!(third.kind, ArchiveKind::Hour);
        assert!(third.range_start_utc < first.range_start_utc);
    }

    #[test]
    fn failed_job_is_selected_again() {
        let dir = tempdir().expect("dir");
        let coordinator = StorageCoordinator::open(&dir.path().join("a.sqlite3")).expect("open");
        let now = chrono::Utc::now().timestamp();
        let first = ReportArchiveService::next_job(coordinator.connection(), now)
            .expect("job")
            .expect("hour");
        ReportArchiveService::persist_outcome(
            coordinator.connection(),
            &first,
            Err(ReportError::StorageBusy("tick")),
            now,
        )
        .expect("fail");
        let again = ReportArchiveService::next_job(coordinator.connection(), now)
            .expect("job2")
            .expect("retry");
        assert_eq!(again.kind, first.kind);
        assert_eq!(again.range_start_utc, first.range_start_utc);
        assert_eq!(again.fingerprint, first.fingerprint);
    }

    #[test]
    fn run_one_uses_independent_reader() {
        let dir = tempdir().expect("dir");
        let path = dir.path().join("a.sqlite3");
        let coordinator = StorageCoordinator::open(&path).expect("open");
        coordinator.seed_report_fixture().expect("seed");
        let now = chrono::Utc::now().timestamp();
        let wrote = run_one_archive_job(
            coordinator.connection(),
            coordinator.path(),
            dir.path(),
            now,
            30,
        )
        .expect("run");
        assert!(wrote);
        let page = ReportArchiveService::list(coordinator.connection(), Some("hour"), None, None)
            .expect("list");
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].status, "ok");
        run_one_archive_job(
            coordinator.connection(),
            coordinator.path(),
            dir.path(),
            now,
            30,
        )
        .expect("second tick");
        let hours = ReportArchiveService::list(coordinator.connection(), Some("hour"), None, None)
            .expect("hours");
        let days = ReportArchiveService::list(coordinator.connection(), Some("day"), None, None)
            .expect("days");
        assert_eq!(
            hours
                .items
                .iter()
                .filter(|item| item.status == "ok")
                .count(),
            1
        );
        assert_eq!(days.items.len(), 1);
    }

    #[test]
    fn list_omits_result_json_and_rejects_bad_kind() {
        let dir = tempdir().expect("dir");
        let coordinator = StorageCoordinator::open(&dir.path().join("a.sqlite3")).expect("open");
        let err = ReportArchiveService::list(coordinator.connection(), Some("month"), None, None)
            .expect_err("kind");
        assert_eq!(err.code(), "invalid_query");
        let page = ReportArchiveService::list(coordinator.connection(), None, None, Some(10))
            .expect("empty");
        assert_eq!(page.schema_version, 1);
        assert!(page.items.is_empty());
        assert_eq!(page.next, None);
        let encoded = serde_json::to_string(&page).expect("json");
        assert!(!encoded.contains("resultJson"));
        assert!(!encoded.contains("result_json"));
    }

    #[test]
    fn list_next_cursor_is_null_on_last_page() {
        let dir = tempdir().expect("dir");
        let coordinator = StorageCoordinator::open(&dir.path().join("a.sqlite3")).expect("open");
        let now = 20 * 86_400;
        for offset in [3_600_i64, 7_200, 10_800] {
            let job = job_at(ArchiveKind::Hour, now - offset - 3_600, now - offset);
            ReportArchiveService::persist_outcome(
                coordinator.connection(),
                &job,
                Ok(dummy_ok(job.query.clone(), now)),
                now,
            )
            .expect("insert");
        }
        let first =
            ReportArchiveService::list(coordinator.connection(), None, None, Some(2)).expect("p1");
        assert_eq!(first.items.len(), 2);
        assert!(first.next.is_some());
        let second = ReportArchiveService::list(
            coordinator.connection(),
            None,
            first.next.as_deref(),
            Some(2),
        )
        .expect("p2");
        assert_eq!(second.items.len(), 1);
        assert_eq!(second.next, None);
    }
}

//! 冻结小时 / 日 ReportResult 档案。过期删除只针对本表。

use crate::c3::query::{
    closed_local_day_bounds, closed_local_hour_bounds, default_auto_report_query, local_day_bounds,
    local_hour_bounds, query_fingerprint, timezone_offset_secs, DimensionKind, Granularity,
    ReportError, ReportQuery, ReportResult, DIMENSION_RETAIN_DAYS, REPORT_DTO_VERSION,
};
use crate::c3::service::{attach_cancel, poll_interrupt, run_uncached};
use crate::c3::snapshot::ReportSnapshotStore;
use crate::c3::sql::RESIDENTIAL_ACCOUNTING_FILTER;
use crate::storage::open_interruptible_reader;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub const ARCHIVE_DTO_VERSION: u32 = 1;
pub const ARCHIVE_HOUR_RETAIN_DAYS: i64 = 30;
pub const ARCHIVE_MANUAL_RETAIN_DAYS: i64 = 7;
pub const ARCHIVE_LIST_DEFAULT: u32 = 50;
pub const ARCHIVE_LIST_MAX: u32 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchiveKind {
    Hour,
    Day,
    Manual,
}

impl ArchiveKind {
    fn as_sql(self) -> &'static str {
        match self {
            Self::Hour => "hour",
            Self::Day => "day",
            Self::Manual => "manual",
        }
    }

    fn parse(raw: &str) -> Result<Self, ReportError> {
        match raw {
            "hour" => Ok(Self::Hour),
            "day" => Ok(Self::Day),
            "manual" => Ok(Self::Manual),
            _ => Err(ReportError::InvalidQuery("archive kind")),
        }
    }

    fn granularity(self) -> Granularity {
        match self {
            Self::Hour => Granularity::Hour,
            Self::Day => Granularity::Day,
            Self::Manual => Granularity::Hour,
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

const ARCHIVE_RETRY_SECS: i64 = 60;
const ARCHIVE_MAX_RETRY_SECS: i64 = 3_600;

/// 自动档案候选的最小持久描述。完整查询只在作业即将执行时重建，避免
/// 长期积压为每个候选保留 ReportQuery 的多个空字段和重复字符串。
#[derive(Debug, Clone)]
struct ArchiveDescriptor {
    kind: ArchiveKind,
    range_start_utc: i64,
    range_end_utc: i64,
    display_timezone: String,
    fingerprint: String,
    retry_after: i64,
    failures: u32,
}

impl ArchiveDescriptor {
    fn from_job(job: &ArchiveJob, retry_after: i64, failures: u32) -> Self {
        Self {
            kind: job.kind,
            range_start_utc: job.range_start_utc,
            range_end_utc: job.range_end_utc,
            display_timezone: job.query.display_timezone.clone(),
            fingerprint: job.fingerprint.clone(),
            retry_after,
            failures,
        }
    }

    fn to_job(&self) -> ArchiveJob {
        let mut query = default_auto_report_query(
            self.kind.granularity(),
            self.range_start_utc,
            self.range_end_utc,
        );
        query.display_timezone = self.display_timezone.clone();
        ArchiveJob {
            kind: self.kind,
            range_start_utc: self.range_start_utc,
            range_end_utc: self.range_end_utc,
            query,
            fingerprint: self.fingerprint.clone(),
        }
    }
}

#[derive(Debug)]
struct PendingArchive {
    descriptor: ArchiveDescriptor,
}

/// 唯一后台 owner 的有界队列。仅启动、时区变化、时钟回拨和周期边界重建。
#[derive(Debug, Default)]
pub struct ArchiveScheduler {
    pending: VecDeque<PendingArchive>,
    in_flight: Option<PendingArchive>,
    refresh_after: i64,
    zone_check_after: i64,
    zone: Option<(String, i32)>,
    last_tick: Option<i64>,
    purge_after: i64,
    job_after: i64,
}

impl ArchiveScheduler {
    pub fn next_job(
        &mut self,
        db_path: &Path,
        now_utc: i64,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Option<ArchiveJob>, ReportError> {
        self.next_job_in_zone(db_path, now_utc, "local", cancel)
    }

    fn next_job_in_zone(
        &mut self,
        db_path: &Path,
        now_utc: i64,
        timezone: &str,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Option<ArchiveJob>, ReportError> {
        if self.in_flight.is_some() {
            return Ok(None);
        }
        if self.last_tick.is_some_and(|last| now_utc < last) {
            self.refresh_after = now_utc;
            self.zone_check_after = now_utc;
            self.purge_after = now_utc;
        }
        self.last_tick = Some(now_utc);
        if self.zone.is_none() || now_utc >= self.zone_check_after {
            // 系统时区最多每分钟探测一次，稳态秒级 tick 不调用本地历法转换。
            self.zone_check_after = now_utc.saturating_add(60);
            let zone = (
                timezone.to_string(),
                timezone_offset_secs(timezone, now_utc)?,
            );
            if self.zone.as_ref() != Some(&zone) {
                self.refresh_after = now_utc;
                self.zone = Some(zone);
            }
        }
        if now_utc >= self.refresh_after {
            // 发现失败也必须退避，不能下一秒重扫。
            self.refresh_after = now_utc.saturating_add(ARCHIVE_RETRY_SECS);
            self.refresh(db_path, now_utc, timezone, cancel)?;
        }
        if now_utc < self.job_after {
            return Ok(None);
        }
        let Some(index) = self
            .pending
            .iter()
            .position(|item| item.descriptor.retry_after <= now_utc)
        else {
            return Ok(None);
        };
        let pending = self.pending.remove(index).expect("已定位档案作业");
        self.update_job_after();
        let job = pending.descriptor.to_job();
        self.in_flight = Some(pending);
        Ok(Some(job))
    }

    fn refresh(
        &mut self,
        db_path: &Path,
        now_utc: i64,
        timezone: &str,
        cancel: &Arc<AtomicBool>,
    ) -> Result<(), ReportError> {
        poll_interrupt(cancel, "archive discovery")?;
        let due = local_hour_bounds(timezone, now_utc)?
            .1
            .min(local_day_bounds(timezone, now_utc)?.1);
        let jobs = candidate_jobs(now_utc, timezone)?;
        let reader = open_interruptible_reader(db_path)
            .map_err(|_| ReportError::StorageBusy("archive reader"))?;
        attach_cancel(&reader, cancel, Instant::now(), Duration::from_secs(10))?;
        // 按唯一索引查询有界候选，不加载 manual 或无关历史键。
        let mut statement = reader
            .prepare(
                "select status, generated_utc from report_archive
              where kind = ?1 and range_start_utc = ?2 and query_fingerprint = ?3",
            )
            .map_err(map_sqlite)?;
        let mut next = VecDeque::new();
        for job in jobs {
            if cancel.load(std::sync::atomic::Ordering::SeqCst) {
                return Err(ReportError::Cancelled("archive discovery"));
            }
            let state: Option<(String, i64)> = statement
                .query_row(
                    params![job.kind.as_sql(), job.range_start_utc, job.fingerprint],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()
                .map_err(map_sqlite)?;
            if state.as_ref().is_some_and(|(status, _)| status == "ok") {
                continue;
            }
            let retry_after = state.as_ref().map_or(now_utc, |(_, failed_at)| {
                failed_at
                    .saturating_add(ARCHIVE_RETRY_SECS)
                    .min(now_utc.saturating_add(ARCHIVE_MAX_RETRY_SECS))
            });
            next.push_back(PendingArchive {
                descriptor: ArchiveDescriptor::from_job(
                    &job,
                    retry_after,
                    u32::from(state.is_some()),
                ),
            });
        }
        let previous: HashMap<_, _> = self
            .pending
            .drain(..)
            .map(|item| {
                (
                    // 将旧描述符的指纹所有权移入索引，刷新期间不再复制长字符串。
                    item.descriptor.fingerprint,
                    (item.descriptor.retry_after, item.descriptor.failures),
                )
            })
            .collect();
        for item in &mut next {
            if let Some((retry_after, failures)) = previous.get(&item.descriptor.fingerprint) {
                // 尚未尝试的作业没有退避；时钟回拨后仍立即可运行。
                item.descriptor.retry_after = if *failures == 0 {
                    now_utc
                } else {
                    (*retry_after).min(now_utc.saturating_add(ARCHIVE_MAX_RETRY_SECS))
                };
                item.descriptor.failures = *failures;
            }
        }
        self.pending = next;
        self.update_job_after();
        self.refresh_after = due.max(now_utc.saturating_add(1));
        Ok(())
    }

    pub fn complete(&mut self, succeeded: bool, now_utc: i64) {
        let Some(mut pending) = self.in_flight.take() else {
            return;
        };
        if !succeeded {
            pending.descriptor.failures = pending.descriptor.failures.saturating_add(1);
            let delay = ARCHIVE_RETRY_SECS
                .saturating_mul(1_i64 << pending.descriptor.failures.saturating_sub(1).min(6))
                .min(ARCHIVE_MAX_RETRY_SECS);
            pending.descriptor.retry_after = now_utc.saturating_add(delay);
            // 失败项移至队尾，已到期的新档案与积压均能继续推进。
            self.pending.push_back(pending);
            self.update_job_after();
        }
    }

    fn update_job_after(&mut self) {
        self.job_after = self
            .pending
            .iter()
            .map(|item| item.descriptor.retry_after)
            .min()
            .unwrap_or(i64::MAX);
    }

    pub fn purge_due(&mut self, now_utc: i64) -> bool {
        if now_utc < self.purge_after {
            return false;
        }
        self.purge_after = now_utc.saturating_add(3_600);
        true
    }

    pub fn complete_purge(&mut self, removed: Option<u64>, now_utc: i64) {
        self.purge_after = now_utc.saturating_add(match removed {
            Some(128) => 5,
            Some(_) => 3_600,
            None => ARCHIVE_RETRY_SECS,
        });
    }
}

pub struct ReportArchiveService;

impl ReportArchiveService {
    /// 自动清理按行数和执行时间双重限流；退出前归还共享 writer 的 handler。
    pub(crate) fn purge_expired_chunk(
        connection: &Connection,
        now_utc: i64,
        cancel: &Arc<AtomicBool>,
    ) -> Result<u64, ReportError> {
        poll_interrupt(cancel, "archive purge")?;
        let started = Instant::now();
        let deadline = Duration::from_millis(250);
        connection
            .busy_timeout(Duration::ZERO)
            .map_err(map_sqlite)?;
        let result = (|| {
            attach_cancel(connection, cancel, started, deadline)?;
            let removed = connection
                .execute(
                    "delete from report_archive where rowid in (
                       select rowid from report_archive
                       where (kind = 'hour' and range_end_utc < ?1)
                          or (kind = 'day' and range_end_utc < ?2)
                          or (kind = 'manual' and generated_utc < ?3)
                       limit 128)",
                    params![
                        now_utc.saturating_sub(ARCHIVE_HOUR_RETAIN_DAYS * 86_400),
                        now_utc.saturating_sub(DIMENSION_RETAIN_DAYS * 86_400),
                        now_utc.saturating_sub(ARCHIVE_MANUAL_RETAIN_DAYS * 86_400),
                    ],
                )
                .map_err(map_sqlite)?;
            Ok(removed as u64)
        })();
        let clear = connection.progress_handler(0, None::<fn() -> bool>);
        let timeout = connection.busy_timeout(Duration::from_millis(u64::from(
            crate::c0_contract::BUSY_TIMEOUT_MS,
        )));
        clear.map_err(map_sqlite)?;
        timeout.map_err(map_sqlite)?;
        match result {
            Err(_) if cancel.load(std::sync::atomic::Ordering::SeqCst) => {
                Err(ReportError::Cancelled("archive purge"))
            }
            Err(_) if started.elapsed() >= deadline => {
                Err(ReportError::DeadlineExceeded("archive purge"))
            }
            other => other,
        }
    }

    pub fn purge_expired(connection: &Connection, now_utc: i64) -> Result<u64, ReportError> {
        let hour_cut = now_utc.saturating_sub(ARCHIVE_HOUR_RETAIN_DAYS * 86_400);
        let day_cut = now_utc.saturating_sub(DIMENSION_RETAIN_DAYS * 86_400);
        let manual_cut = now_utc.saturating_sub(ARCHIVE_MANUAL_RETAIN_DAYS * 86_400);
        let n = connection
            .execute(
                "delete from report_archive
                  where (kind = 'hour' and range_end_utc < ?1)
                     or (kind = 'day' and range_end_utc < ?2)
                     or (kind = 'manual' and generated_utc < ?3)",
                params![hour_cut, day_cut, manual_cut],
            )
            .map_err(map_sqlite)?;
        Ok(n as u64)
    }

    pub fn next_job(
        connection: &Connection,
        now_utc: i64,
    ) -> Result<Option<ArchiveJob>, ReportError> {
        let ok = load_ok_keys(connection)?;
        for job in candidate_jobs(now_utc, "local")? {
            if !ok.contains(&(
                job.kind.as_sql().to_string(),
                job.range_start_utc,
                job.fingerprint.clone(),
            )) {
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
            Ok(result) => persist_ok(connection, job, result, now_utc, false),
            Err(error) => persist_failed(connection, job, &error, now_utc),
        }
    }

    pub fn persist_manual(
        connection: &Connection,
        result: ReportResult,
        now_utc: i64,
    ) -> Result<(), ReportError> {
        let query = result.query_echo.clone();
        let fingerprint = query_fingerprint(&query);
        let job = ArchiveJob {
            kind: ArchiveKind::Manual,
            range_start_utc: query.range_start_utc,
            range_end_utc: query.range_end_utc,
            query,
            fingerprint,
        };
        persist_ok(connection, &job, result, now_utc, true)?;
        let _ = Self::purge_expired(connection, now_utc);
        Ok(())
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

    pub fn is_residential_manual_report(result: &ReportResult) -> bool {
        result.query_echo.grouping == DimensionKind::Host
            && result.query_echo.filters.category.as_deref() == Some(RESIDENTIAL_ACCOUNTING_FILTER)
    }

    pub fn load_latest_residential_manual(
        connection: &Connection,
    ) -> Result<Option<ReportResult>, ReportError> {
        let mut statement = connection
            .prepare(
                "select result_json from report_archive
                  where kind = 'manual' and status = 'ok' and result_json is not null
                  order by generated_utc desc, archive_id desc",
            )
            .map_err(map_sqlite)?;
        let mut rows = statement.query([]).map_err(map_sqlite)?;
        while let Some(row) = rows.next().map_err(map_sqlite)? {
            let json: String = row.get(0).map_err(map_sqlite)?;
            let Ok(mut result) = serde_json::from_str::<ReportResult>(&json) else {
                continue;
            };
            if result.schema_version != REPORT_DTO_VERSION {
                continue;
            }
            result.reconcile_legacy_attribution_quality();
            if Self::is_residential_manual_report(&result) {
                return Ok(Some(result));
            }
        }
        Ok(None)
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
    overwrite_ok: bool,
) -> Result<(), ReportError> {
    if !overwrite_ok && existing_ok(connection, job)? {
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

fn candidate_jobs(now_utc: i64, timezone: &str) -> Result<Vec<ArchiveJob>, ReportError> {
    let hours = walk_closed_periods(
        timezone,
        now_utc,
        now_utc.saturating_sub(ARCHIVE_HOUR_RETAIN_DAYS * 86_400),
        local_hour_bounds,
        closed_local_hour_bounds,
    )?;
    let days = walk_closed_periods(
        timezone,
        now_utc,
        now_utc.saturating_sub(DIMENSION_RETAIN_DAYS * 86_400),
        local_day_bounds,
        closed_local_day_bounds,
    )?;
    Ok(hours
        .iter()
        .take(1)
        .map(|range| (ArchiveKind::Hour, range))
        .chain(days.iter().take(1).map(|range| (ArchiveKind::Day, range)))
        .chain(hours.iter().skip(1).map(|range| (ArchiveKind::Hour, range)))
        .chain(days.iter().skip(1).map(|range| (ArchiveKind::Day, range)))
        .map(|(kind, &(start, end))| {
            let mut query = default_auto_report_query(kind.granularity(), start, end);
            query.display_timezone = timezone.into();
            ArchiveJob {
                kind,
                range_start_utc: start,
                range_end_utc: end,
                fingerprint: query_fingerprint(&query),
                query,
            }
        })
        .collect())
}

type TimeBoundsFn = fn(&str, i64) -> Result<(i64, i64), ReportError>;

fn walk_closed_periods(
    timezone: &str,
    now_utc: i64,
    retain_end: i64,
    bounds: TimeBoundsFn,
    closed: TimeBoundsFn,
) -> Result<Vec<(i64, i64)>, ReportError> {
    let (mut start, mut end) = match closed(timezone, now_utc) {
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
        match bounds(timezone, start.saturating_sub(1)) {
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
    if crate::sqlite_probe::map_sqlite_error(&error) == "cancelled" {
        return ReportError::Cancelled("archive sqlite");
    }
    if crate::sqlite_probe::map_sqlite_error(&error) == "busy" {
        return ReportError::StorageBusy("archive sqlite");
    }
    ReportError::Failed("archive sqlite")
}

/// 测试与单连接路径：查询走独立 reader，写入走传入的 writer 连接。
pub fn run_one_archive_job(
    connection: &Connection,
    db_path: &Path,
    now_utc: i64,
    raw_retain_days: i64,
) -> Result<bool, ReportError> {
    ReportArchiveService::purge_expired(connection, now_utc)?;
    let Some(job) = ReportArchiveService::next_job(connection, now_utc)? else {
        return Ok(false);
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let outcome = run_uncached(
        db_path,
        job.query.clone(),
        now_utc,
        raw_retain_days,
        &cancel,
        None,
    );
    ReportArchiveService::persist_outcome(connection, &job, outcome, now_utc)?;
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

    #[test]
    fn automatic_purge_is_bounded_and_releases_the_writer_cancel_handler() {
        let dir = tempdir().expect("dir");
        let coordinator =
            StorageCoordinator::open(&dir.path().join("purge.sqlite3")).expect("open");
        let connection = coordinator.connection();
        let now = 500 * 86_400;
        for id in 0..131 {
            let generated = if id == 130 { now } else { now - 8 * 86_400 };
            connection
                .execute(
                    "insert into report_archive(archive_id,kind,range_start_utc,range_end_utc,
                     display_timezone,grouping,query_fingerprint,status,generated_utc)
                     values (?1,'manual',0,1,'UTC','hour',?1,'ok',?2)",
                    params![format!("purge-{id}"), generated],
                )
                .expect("seed");
        }
        let cancel = Arc::new(AtomicBool::new(false));
        assert_eq!(
            ReportArchiveService::purge_expired_chunk(connection, now, &cancel).expect("chunk"),
            128
        );
        cancel.store(true, std::sync::atomic::Ordering::SeqCst);
        let count: i64 = connection
            .query_row(
                "with recursive n(x) as (values(1) union all select x+1 from n where x<200)
                 select count(*) from n",
                [],
                |row| row.get(0),
            )
            .expect("later writer work has no stale cancellation hook");
        assert_eq!(count, 200);
        assert!(matches!(
            ReportArchiveService::purge_expired_chunk(connection, now, &cancel),
            Err(ReportError::Cancelled(_))
        ));
        cancel.store(false, std::sync::atomic::Ordering::SeqCst);
        assert_eq!(
            ReportArchiveService::purge_expired_chunk(connection, now, &cancel).expect("remainder"),
            2
        );
        assert_eq!(
            connection
                .query_row("select count(*) from report_archive", [], |row| row
                    .get::<_, i64>(0))
                .expect("retained"),
            1
        );
    }

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

    #[test]
    fn scheduler_complete_history_needs_no_reader_for_3600_ticks() {
        let dir = tempdir().expect("dir");
        let coordinator =
            StorageCoordinator::open(&dir.path().join("schedule.sqlite3")).expect("open");
        let now = 20 * 86_400;
        coordinator
            .connection()
            .execute_batch("begin immediate")
            .expect("begin");
        for job in candidate_jobs(now, "UTC").expect("jobs") {
            ReportArchiveService::persist_outcome(
                coordinator.connection(),
                &job,
                Ok(dummy_ok(job.query.clone(), now)),
                now,
            )
            .expect("persist");
        }
        coordinator
            .connection()
            .execute_batch("commit")
            .expect("commit");
        let cancel = Arc::new(AtomicBool::new(false));
        let mut scheduler = ArchiveScheduler::default();
        assert!(scheduler
            .next_job_in_zone(coordinator.path(), now, "UTC", &cancel)
            .expect("load")
            .is_none());
        // 移走表后仍应完成这一小时的全部空闲 tick：任何重新读历史都会报错。
        coordinator
            .connection()
            .execute_batch("drop table report_archive")
            .expect("drop");
        for tick in 0..3_600 {
            assert!(scheduler
                .next_job_in_zone(coordinator.path(), now + tick, "UTC", &cancel)
                .expect("idle")
                .is_none());
        }
        assert!(scheduler
            .next_job_in_zone(coordinator.path(), now + 3_600, "UTC", &cancel)
            .is_err());
        // 发现失败后也不会立即再读；先等待有界重试。
        assert!(scheduler
            .next_job_in_zone(coordinator.path(), now + 3_601, "UTC", &cancel)
            .expect("backoff")
            .is_none());
    }

    #[test]
    fn archive_descriptor_round_trip_keeps_dispatch_semantics_and_retry_state() {
        let mut job = job_at(ArchiveKind::Day, 1_700_000_000, 1_700_086_400);
        job.query.display_timezone = "America/New_York".into();
        job.fingerprint = query_fingerprint(&job.query);
        let descriptor = ArchiveDescriptor::from_job(&job, 1_700_100_000, 4);

        assert_eq!(descriptor.kind, job.kind);
        assert_eq!(
            (descriptor.range_start_utc, descriptor.range_end_utc),
            (job.range_start_utc, job.range_end_utc)
        );
        assert_eq!(descriptor.display_timezone, job.query.display_timezone);
        assert_eq!(descriptor.fingerprint, job.fingerprint);
        assert_eq!(descriptor.retry_after, 1_700_100_000);
        assert_eq!(descriptor.failures, 4);

        let dispatched = descriptor.to_job();
        assert_eq!(dispatched.kind, job.kind);
        assert_eq!(
            (dispatched.range_start_utc, dispatched.range_end_utc),
            (job.range_start_utc, job.range_end_utc)
        );
        assert_eq!(
            dispatched.query.display_timezone,
            job.query.display_timezone
        );
        assert_eq!(dispatched.fingerprint, job.fingerprint);
        assert_eq!(query_fingerprint(&dispatched.query), dispatched.fingerprint);
    }

    #[test]
    fn scheduler_pending_inventory_stays_bounded_and_query_free() {
        let dir = tempdir().expect("dir");
        let coordinator =
            StorageCoordinator::open(&dir.path().join("schedule.sqlite3")).expect("open");
        let cancel = Arc::new(AtomicBool::new(false));
        let mut scheduler = ArchiveScheduler::default();
        let _ = scheduler
            .next_job_in_zone(coordinator.path(), 20 * 86_400, "UTC", &cancel)
            .expect("load");

        assert!(scheduler.pending.len() <= 2_000);
        assert!(std::mem::size_of::<ArchiveDescriptor>() < std::mem::size_of::<ArchiveJob>());
        assert!(scheduler.pending.iter().all(|item| {
            item.descriptor.fingerprint.len() == 64 && item.descriptor.display_timezone == "UTC"
        }));
    }

    #[test]
    fn scheduler_failure_advances_backlog_and_survives_restart() {
        let dir = tempdir().expect("dir");
        let coordinator =
            StorageCoordinator::open(&dir.path().join("schedule.sqlite3")).expect("open");
        let now = 20 * 86_400;
        let cancel = Arc::new(AtomicBool::new(false));
        let mut scheduler = ArchiveScheduler::default();
        let failed = scheduler
            .next_job_in_zone(coordinator.path(), now, "UTC", &cancel)
            .expect("load")
            .expect("hour");
        assert!(scheduler
            .next_job_in_zone(coordinator.path(), now, "UTC", &cancel)
            .expect("in flight")
            .is_none());
        ReportArchiveService::persist_outcome(
            coordinator.connection(),
            &failed,
            Err(ReportError::DeadlineExceeded("fixture")),
            now,
        )
        .expect("failure");
        scheduler.complete(false, now);
        let next = scheduler
            .next_job_in_zone(coordinator.path(), now + 1, "UTC", &cancel)
            .expect("next")
            .expect("day");
        assert_eq!(next.kind, ArchiveKind::Day);
        assert_ne!(next.fingerprint, failed.fingerprint);
        let mut restarted = ArchiveScheduler::default();
        let next = restarted
            .next_job_in_zone(coordinator.path(), now + 2, "UTC", &cancel)
            .expect("restart")
            .expect("day");
        assert_eq!(next.kind, ArchiveKind::Day);
        assert!(restarted.pending.len() <= 2_000);
        scheduler.complete(true, now + 1);
        while let Some(job) = scheduler
            .next_job_in_zone(coordinator.path(), now + 2, "UTC", &cancel)
            .expect("drain")
        {
            assert_ne!(job.fingerprint, failed.fingerprint);
            scheduler.complete(true, now + 2);
        }
        assert!(scheduler
            .next_job_in_zone(coordinator.path(), now + 59, "UTC", &cancel)
            .expect("wait")
            .is_none());
        assert_eq!(
            scheduler
                .next_job_in_zone(coordinator.path(), now + 60, "UTC", &cancel)
                .expect("retry")
                .expect("failed")
                .fingerprint,
            failed.fingerprint
        );
        scheduler.complete(false, now + 60);
        assert!(scheduler
            .next_job_in_zone(coordinator.path(), now + 179, "UTC", &cancel)
            .expect("longer wait")
            .is_none());
    }

    #[test]
    fn scheduler_clock_rewind_pause_and_timezone_change_rebuild_boundaries() {
        let dir = tempdir().expect("dir");
        let coordinator =
            StorageCoordinator::open(&dir.path().join("schedule.sqlite3")).expect("open");
        let cancel = Arc::new(AtomicBool::new(false));
        let mut scheduler = ArchiveScheduler::default();
        let now = 20 * 86_400;
        for (utc, zone) in [
            (now, "UTC"),
            (now - 7_200, "UTC"),
            (now + 2 * 86_400, "Asia/Shanghai"),
        ] {
            let job = scheduler
                .next_job_in_zone(coordinator.path(), utc, zone, &cancel)
                .expect("load")
                .expect("hour");
            assert_eq!(
                (job.range_start_utc, job.range_end_utc),
                closed_local_hour_bounds(zone, utc).expect("bounds")
            );
            assert_eq!(
                scheduler.refresh_after,
                local_hour_bounds(zone, utc).expect("next boundary").1
            );
            scheduler.complete(true, utc);
        }
    }

    #[test]
    fn scheduler_dst_uses_closed_civil_periods() {
        let dir = tempdir().expect("dir");
        let coordinator =
            StorageCoordinator::open(&dir.path().join("schedule.sqlite3")).expect("open");
        let cancel = Arc::new(AtomicBool::new(false));
        for date in ["2024-03-11T04:00:00Z", "2024-11-04T05:00:00Z"] {
            let now = chrono::DateTime::parse_from_rfc3339(date)
                .expect("date")
                .timestamp();
            let mut scheduler = ArchiveScheduler::default();
            let _ = scheduler
                .next_job_in_zone(coordinator.path(), now, "America/New_York", &cancel)
                .expect("hour");
            scheduler.complete(true, now);
            let day = scheduler
                .next_job_in_zone(coordinator.path(), now, "America/New_York", &cancel)
                .expect("day")
                .expect("closed day");
            assert_eq!(day.kind, ArchiveKind::Day);
            let expected_hours = if date.contains("03-11") { 23 } else { 25 };
            assert_eq!(
                day.range_end_utc - day.range_start_utc,
                expected_hours * 3_600
            );
            assert!(scheduler.pending.len() <= 2_000);
        }
    }

    #[test]
    fn internal_archive_query_never_creates_spool_directory() {
        let dir = tempdir().expect("dir");
        let coordinator =
            StorageCoordinator::open(&dir.path().join("schedule.sqlite3")).expect("open");
        let spool = dir.path().join("report-spool");
        assert!(run_one_archive_job(
            coordinator.connection(),
            coordinator.path(),
            20 * 86_400,
            30
        )
        .expect("archive"));
        assert!(!spool.exists());
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
        let wrote = run_one_archive_job(coordinator.connection(), coordinator.path(), now, 30)
            .expect("run");
        assert!(wrote);
        let page = ReportArchiveService::list(coordinator.connection(), Some("hour"), None, None)
            .expect("list");
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].status, "ok");
        run_one_archive_job(coordinator.connection(), coordinator.path(), now, 30)
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

    #[test]
    fn persist_manual_overwrites_same_window() {
        let dir = tempdir().expect("dir");
        let coordinator = StorageCoordinator::open(&dir.path().join("a.sqlite3")).expect("open");
        let now = 20 * 86_400;
        let mut first = dummy_ok(
            default_auto_report_query(Granularity::Hour, now - 3_600, now),
            now,
        );
        first.query_echo.grouping = DimensionKind::Rule;
        ReportArchiveService::persist_manual(coordinator.connection(), first, now).expect("first");
        let page = ReportArchiveService::list(coordinator.connection(), Some("manual"), None, None)
            .expect("list");
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].kind, ArchiveKind::Manual);
        assert_eq!(page.items[0].grouping, "rule");
        let archive_id = page.items[0].archive_id.clone();
        let mut second = dummy_ok(
            default_auto_report_query(Granularity::Hour, now - 3_600, now),
            now + 1,
        );
        second.query_echo.grouping = DimensionKind::Rule;
        second.totals.download = 77;
        ReportArchiveService::persist_manual(coordinator.connection(), second, now + 1)
            .expect("overwrite");
        let page = ReportArchiveService::list(coordinator.connection(), Some("manual"), None, None)
            .expect("list2");
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].archive_id, archive_id);
        assert_eq!(page.items[0].totals_download, Some(77));
    }

    #[test]
    fn persist_manual_does_not_block_auto_jobs() {
        let dir = tempdir().expect("dir");
        let coordinator = StorageCoordinator::open(&dir.path().join("a.sqlite3")).expect("open");
        let now = chrono::Utc::now().timestamp();
        let (start, end) = closed_local_hour_bounds("local", now).expect("hour");
        let result = dummy_ok(
            default_auto_report_query(Granularity::Hour, start, end),
            now,
        );
        ReportArchiveService::persist_manual(coordinator.connection(), result, now)
            .expect("manual");
        let job = ReportArchiveService::next_job(coordinator.connection(), now)
            .expect("job")
            .expect("auto hour");
        assert_eq!(job.kind, ArchiveKind::Hour);
        assert_eq!(job.range_start_utc, start);
    }

    #[test]
    fn purge_expired_manual_by_generated_utc() {
        let dir = tempdir().expect("dir");
        let coordinator = StorageCoordinator::open(&dir.path().join("a.sqlite3")).expect("open");
        let now = 20 * 86_400;
        let old = dummy_ok(
            default_auto_report_query(Granularity::Hour, now - 3_600, now),
            now - 8 * 86_400,
        );
        ReportArchiveService::persist_manual(coordinator.connection(), old, now - 8 * 86_400)
            .expect("old");
        let live = dummy_ok(
            default_auto_report_query(Granularity::Hour, now - 7_200, now - 3_600),
            now,
        );
        ReportArchiveService::persist_manual(coordinator.connection(), live, now).expect("live");
        let page = ReportArchiveService::list(coordinator.connection(), Some("manual"), None, None)
            .expect("list");
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].generated_utc, now);
    }

    #[test]
    fn load_latest_residential_manual_skips_auto_and_other_manual() {
        let dir = tempdir().expect("dir");
        let coordinator = StorageCoordinator::open(&dir.path().join("a.sqlite3")).expect("open");
        let now = 20 * 86_400;
        let hour_job = job_at(ArchiveKind::Hour, now - 7_200, now - 3_600);
        let mut hour = dummy_ok(hour_job.query.clone(), now - 10);
        hour.query_echo.filters.category = Some(RESIDENTIAL_ACCOUNTING_FILTER.into());
        ReportArchiveService::persist_outcome(
            coordinator.connection(),
            &hour_job,
            Ok(hour),
            now - 10,
        )
        .expect("hour");
        let mut other = dummy_ok(
            default_auto_report_query(Granularity::Hour, now - 3_600, now),
            now - 5,
        );
        other.query_echo.grouping = DimensionKind::Host;
        ReportArchiveService::persist_manual(coordinator.connection(), other, now - 5)
            .expect("other");
        let mut residential = dummy_ok(
            default_auto_report_query(Granularity::Hour, now - 1_800, now),
            now,
        );
        residential.query_echo.grouping = DimensionKind::Host;
        residential.query_echo.filters.category = Some(RESIDENTIAL_ACCOUNTING_FILTER.into());
        residential.totals.download = 99;
        ReportArchiveService::persist_manual(coordinator.connection(), residential, now)
            .expect("residential");
        let loaded = ReportArchiveService::load_latest_residential_manual(coordinator.connection())
            .expect("load")
            .expect("hit");
        assert_eq!(loaded.totals.download, 99);
        assert!(ReportArchiveService::is_residential_manual_report(&loaded));
    }
}

//! 一次会话投影加一次分钟索引扫描，算出 raw 总量、归因、时间序列、排名和出口。
//!
//! 分组 SQL 仍是测试对照，不作为运行时回退。取消和整份报告 deadline 仍由连接上的
//! progress handler 执行，分桶不会重置预算。

use crate::c3::query::{
    AttributionQuality, DimensionKind, RankingRow, ReportError, ReportQuery, ReportTotals,
    SeriesPoint, SortField, UNKNOWN_LABEL_ZH,
};
use crate::c3::rule_name::{chain_identity, last_chain_hop};
use crate::c3::sql::{
    dimension_kind_sql, raw_session_projection_sql, RAW_MINUTE_SCAN, RESIDENTIAL_ACCOUNTING_FILTER,
    UNKNOWN_IDENTITY,
};
use rusqlite::types::ValueRef;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::{HashMap, HashSet};

const RAW_SESSION_PROJECTION_NAME: &str = "raw_session_projection";
const RAW_MINUTE_SCAN_NAME: &str = "raw_minute_scan";

pub(crate) struct SessionIndex {
    sessions: HashMap<i64, SessionFact>,
    pool: Pool,
    unknown: u32,
}

struct SessionFact {
    slot: u32,
    host_missing: bool,
    host_identity: u32,
    process_id: Option<i64>,
    process_missing: bool,
    process_identity: u32,
    network_id: Option<i64>,
    network_missing: bool,
    network_identity: u32,
    category_id: Option<i64>,
    category_missing: bool,
    category_identity: u32,
    rule_identity: u32,
    chain_identity: Option<u32>,
    chain_key: Option<u32>,
    residential: bool,
}

impl SessionIndex {
    fn get(&self, session_pk: i64) -> Option<&SessionFact> {
        self.sessions.get(&session_pk)
    }

    fn len(&self) -> usize {
        self.sessions.len()
    }
}

struct Pool {
    values: Vec<String>,
    ids: HashMap<String, u32>,
}

impl Pool {
    fn new() -> Self {
        Self {
            values: Vec::new(),
            ids: HashMap::new(),
        }
    }

    fn intern(&mut self, value: &str) -> u32 {
        if let Some(id) = self.ids.get(value) {
            return *id;
        }
        let id = self.values.len() as u32;
        self.values.push(value.to_string());
        self.ids.insert(value.to_string(), id);
        id
    }

    fn get(&self, id: u32) -> &str {
        &self.values[id as usize]
    }
}

struct Dict {
    process: HashMap<i64, String>,
    network: HashMap<i64, String>,
    category: HashMap<i64, String>,
    rule: HashMap<i64, String>,
}

pub(crate) struct FoldedRaw {
    pub totals: ReportTotals,
    pub attribution: AttributionQuality,
    pub series: Vec<SeriesPoint>,
    pub rankings: Vec<RankingRow>,
}

pub(crate) fn executed_names() -> Vec<String> {
    vec![
        RAW_SESSION_PROJECTION_NAME.to_string(),
        RAW_MINUTE_SCAN_NAME.to_string(),
    ]
}

pub(crate) fn load_sessions(
    connection: &Connection,
    query: &ReportQuery,
) -> Result<SessionIndex, ReportError> {
    let residential_only = query.filters.category.as_deref() == Some(RESIDENTIAL_ACCOUNTING_FILTER);
    let dict = load_dict(connection)?;
    let mut pool = Pool::new();
    let unknown = pool.intern(UNKNOWN_IDENTITY);
    let mut sessions = HashMap::new();
    let sql = raw_session_projection_sql(residential_only);
    let mut statement = connection.prepare(&sql).map_err(map_sqlite)?;
    let mut rows = statement.query(params![i64::MIN]).map_err(map_sqlite)?;
    while let Some(row) = rows.next().map_err(map_sqlite)? {
        let session_pk: i64 = row.get(0).map_err(map_sqlite)?;
        let host_missing: i64 = row.get(2).map_err(map_sqlite)?;
        let process_id: Option<i64> = row.get(3).map_err(map_sqlite)?;
        let network_id: Option<i64> = row.get(4).map_err(map_sqlite)?;
        let rule_id: Option<i64> = row.get(5).map_err(map_sqlite)?;
        let category_id: Option<i64> = row.get(6).map_err(map_sqlite)?;
        let residential: i64 = row.get(8).map_err(map_sqlite)?;
        let host_identity = intern_text(&mut pool, row.get_ref(1).map_err(map_sqlite)?)?;
        let chain_key = match row.get_ref(7).map_err(map_sqlite)? {
            ValueRef::Null => None,
            value => {
                let text = value
                    .as_str()
                    .map_err(|_| ReportError::Failed("session text"))?;
                if text.trim().is_empty() {
                    None
                } else {
                    Some(pool.intern(text))
                }
            }
        };
        let (process_identity, process_missing) = identity_from_dict(
            &mut pool,
            unknown,
            process_id,
            dict.process.get(&process_id.unwrap_or(i64::MIN)),
        );
        let (network_identity, network_missing) = identity_from_dict(
            &mut pool,
            unknown,
            network_id,
            dict.network.get(&network_id.unwrap_or(i64::MIN)),
        );
        let (category_identity, category_missing) = identity_from_dict(
            &mut pool,
            unknown,
            category_id,
            dict.category.get(&category_id.unwrap_or(i64::MIN)),
        );
        let chain_text = chain_key.map(|id| pool.get(id).to_string());
        let rule_identity = pool.intern(&rule_name(&dict, rule_id, chain_text.as_deref()));
        let chain_identity = chain_identity(chain_text.as_deref()).map(|value| pool.intern(&value));
        let slot = u32::try_from(sessions.len())
            .map_err(|_| ReportError::Failed("session projection overflow"))?;
        sessions.insert(
            session_pk,
            SessionFact {
                slot,
                host_missing: host_missing != 0,
                host_identity,
                process_id,
                process_missing,
                process_identity,
                network_id,
                network_missing,
                network_identity,
                category_id,
                category_missing,
                category_identity,
                rule_identity,
                chain_identity,
                chain_key,
                residential: residential != 0,
            },
        );
    }
    Ok(SessionIndex {
        sessions,
        pool,
        unknown,
    })
}

fn intern_text(pool: &mut Pool, value: ValueRef<'_>) -> Result<u32, ReportError> {
    Ok(pool.intern(
        value
            .as_str()
            .map_err(|_| ReportError::Failed("session text"))?,
    ))
}

pub(crate) fn fold_window(
    connection: &Connection,
    query: &ReportQuery,
    index: &SessionIndex,
    start_min: i64,
    end_min: i64,
) -> Result<FoldedRaw, ReportError> {
    if end_min <= start_min {
        return Ok(empty_fold());
    }
    let filters = resolve_filters(connection, query, index)?;
    let width = query.granularity.bucket_minutes();
    let exits_enabled = matches!(
        query.grouping,
        DimensionKind::Host | DimensionKind::Rule | DimensionKind::Process
    );
    let mut totals = Acc::new(start_min, end_min)?;
    let mut missing = Acc::new(start_min, end_min)?;
    let mut series: HashMap<i64, Acc> = HashMap::new();
    let mut groups: HashMap<u32, Acc> = HashMap::new();
    let mut exits: HashMap<u32, ExitAgg> = HashMap::new();
    let mut marks = vec![
        ScanMark {
            total: false,
            missing: false,
            group: false,
            bucket: None,
        };
        index.len()
    ];
    let mut statement = connection.prepare(RAW_MINUTE_SCAN).map_err(map_sqlite)?;
    let rows = statement
        .query_map(params![start_min, end_min], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .map_err(map_sqlite)?;
    for row in rows {
        let (minute, session_pk, upload, download) = row.map_err(map_sqlite)?;
        let Some(fact) = index.get(session_pk) else {
            continue;
        };
        if !filters.matches(fact) {
            continue;
        }
        let identity = identity_id(fact, query.grouping, index.unknown);
        let mark = &mut marks[fact.slot as usize];
        totals.add_bytes(minute, upload, download)?;
        if !mark.total {
            mark.total = true;
            totals.note_session();
        }
        if is_missing(fact, query.grouping) {
            missing.add_bytes(minute, upload, download)?;
            if !mark.missing {
                mark.missing = true;
                missing.note_session();
            }
        }
        let bucket = (minute / width) * width;
        let series_acc = series.entry(bucket).or_insert_with(Acc::bucket);
        series_acc.add_bytes(minute, upload, download)?;
        if mark.bucket != Some(bucket) {
            mark.bucket = Some(bucket);
            series_acc.note_session();
        }
        let group = groups
            .entry(identity)
            .or_insert_with(|| Acc::new(start_min, end_min).expect("window"));
        group.add_bytes(minute, upload, download)?;
        if !mark.group {
            mark.group = true;
            group.note_session();
        }
        if exits_enabled {
            if let Some(chain_key) = fact.chain_key {
                record_exit(&mut exits, identity, chain_key, download)?;
            }
        }
    }
    Ok(FoldedRaw {
        totals: totals.totals(),
        attribution: attribution(&totals, &missing),
        series: finish_series(&series),
        rankings: finish_rankings(query, index, groups, exits),
    })
}

#[derive(Clone)]
struct ScanMark {
    total: bool,
    missing: bool,
    group: bool,
    bucket: Option<i64>,
}

struct Acc {
    upload: i64,
    download: i64,
    session_count: i64,
    minutes: MinuteSet,
}

impl Acc {
    fn new(start_min: i64, end_min: i64) -> Result<Self, ReportError> {
        Ok(Self {
            upload: 0,
            download: 0,
            session_count: 0,
            minutes: MinuteSet::new(start_min, end_min)?,
        })
    }

    fn bucket() -> Self {
        Self {
            upload: 0,
            download: 0,
            session_count: 0,
            minutes: MinuteSet::sparse(),
        }
    }

    fn add_bytes(&mut self, minute: i64, upload: i64, download: i64) -> Result<(), ReportError> {
        self.upload = add_i64(self.upload, upload)?;
        self.download = add_i64(self.download, download)?;
        self.minutes.insert(minute);
        Ok(())
    }

    fn note_session(&mut self) {
        self.session_count += 1;
    }

    fn totals(&self) -> ReportTotals {
        ReportTotals {
            upload: self.upload,
            download: self.download,
            connection_count: self.session_count,
            active_duration_sec: self.minutes.count * 60,
            previous_upload: None,
            previous_download: None,
        }
    }
}

struct MinuteSet {
    origin: i64,
    words: Vec<u64>,
    sparse: HashSet<i64>,
    count: i64,
    dense: bool,
}

impl MinuteSet {
    fn new(start_min: i64, end_min: i64) -> Result<Self, ReportError> {
        let len = end_min
            .checked_sub(start_min)
            .ok_or(ReportError::Failed("raw minute window"))?;
        if len < 0 {
            return Err(ReportError::Failed("raw minute window"));
        }
        let words = usize::try_from(len)
            .ok()
            .map(|len| len.div_ceil(64))
            .ok_or(ReportError::Failed("raw minute window"))?;
        Ok(Self {
            origin: start_min,
            words: vec![0; words],
            sparse: HashSet::new(),
            count: 0,
            dense: true,
        })
    }

    fn sparse() -> Self {
        Self {
            origin: 0,
            words: Vec::new(),
            sparse: HashSet::new(),
            count: 0,
            dense: false,
        }
    }

    fn insert(&mut self, minute: i64) {
        if !self.dense {
            if self.sparse.insert(minute) {
                self.count += 1;
            }
            return;
        }
        let Some(index) = minute
            .checked_sub(self.origin)
            .and_then(|index| usize::try_from(index).ok())
        else {
            return;
        };
        let word = index / 64;
        let Some(slot) = self.words.get_mut(word) else {
            return;
        };
        let mask = 1_u64 << (index % 64);
        if *slot & mask == 0 {
            *slot |= mask;
            self.count += 1;
        }
    }
}

fn empty_fold() -> FoldedRaw {
    FoldedRaw {
        totals: ReportTotals {
            upload: 0,
            download: 0,
            connection_count: 0,
            active_duration_sec: 0,
            previous_upload: None,
            previous_download: None,
        },
        attribution: AttributionQuality::default(),
        series: Vec::new(),
        rankings: Vec::new(),
    }
}

struct ExitAgg {
    sums: HashMap<u32, i64>,
}

fn record_exit(
    exits: &mut HashMap<u32, ExitAgg>,
    identity: u32,
    chain_key: u32,
    download: i64,
) -> Result<(), ReportError> {
    let entry = exits.entry(identity).or_insert_with(|| ExitAgg {
        sums: HashMap::new(),
    });
    let sum = entry.sums.entry(chain_key).or_insert(0);
    *sum = add_i64(*sum, download)?;
    Ok(())
}

fn finish_series(series: &HashMap<i64, Acc>) -> Vec<SeriesPoint> {
    let mut labels: Vec<i64> = series.keys().copied().collect();
    labels.sort_unstable();
    labels
        .into_iter()
        .filter_map(|label| {
            let acc = series.get(&label)?;
            if acc.session_count == 0 {
                return None;
            }
            Some(SeriesPoint {
                bucket_utc: label * 60,
                upload: acc.upload,
                download: acc.download,
                connection_count: acc.session_count,
                active_duration_sec: acc.minutes.count * 60,
            })
        })
        .collect()
}

fn finish_rankings(
    query: &ReportQuery,
    index: &SessionIndex,
    groups: HashMap<u32, Acc>,
    exits: HashMap<u32, ExitAgg>,
) -> Vec<RankingRow> {
    let mut rows: Vec<(u32, Acc)> = groups.into_iter().collect();
    rows.sort_by(|left, right| compare_rank(query, index, left, right));
    rows.truncate(query.top_n as usize);
    rows.into_iter()
        .map(|(identity, acc)| {
            let name = index.pool.get(identity);
            let label = if name == UNKNOWN_IDENTITY {
                UNKNOWN_LABEL_ZH.to_string()
            } else {
                name.to_string()
            };
            let exit = exits
                .get(&identity)
                .and_then(|item| primary_exit(index, item));
            RankingRow {
                identity: name.to_string(),
                label,
                upload: acc.upload,
                download: acc.download,
                connection_count: acc.session_count,
                active_duration_sec: acc.minutes.count * 60,
                primary_exit: exit.map(|key| index.pool.get(key).to_string()),
                exit_mixed: exits.get(&identity).is_some_and(|item| item.sums.len() > 1),
            }
        })
        .collect()
}

fn primary_exit(index: &SessionIndex, agg: &ExitAgg) -> Option<u32> {
    let mut best: Option<(u32, i64)> = None;
    for (key, download) in &agg.sums {
        let replace = match best {
            None => true,
            Some((best_key, best_download)) => {
                *download > best_download
                    || (*download == best_download
                        && index.pool.get(*key) < index.pool.get(best_key))
            }
        };
        if replace {
            best = Some((*key, *download));
        }
    }
    best.map(|(key, _)| key)
}

fn compare_rank(
    query: &ReportQuery,
    index: &SessionIndex,
    left: &(u32, Acc),
    right: &(u32, Acc),
) -> std::cmp::Ordering {
    let name = |id: u32| index.pool.get(id);
    match query.sort.field {
        SortField::Upload | SortField::Download => {
            let value = |acc: &Acc| {
                if query.sort.field == SortField::Upload {
                    acc.upload
                } else {
                    acc.download
                }
            };
            let primary = if query.sort.descending {
                value(&right.1).cmp(&value(&left.1))
            } else {
                value(&left.1).cmp(&value(&right.1))
            };
            primary.then_with(|| name(left.0).cmp(name(right.0)))
        }
        SortField::Name | SortField::Identity => {
            if query.sort.descending {
                name(right.0).cmp(name(left.0))
            } else {
                name(left.0).cmp(name(right.0))
            }
        }
    }
}

fn attribution(totals: &Acc, missing: &Acc) -> AttributionQuality {
    AttributionQuality::from_parts(
        totals.upload - missing.upload,
        totals.download - missing.download,
        missing.upload,
        missing.download,
        totals.session_count - missing.session_count,
        missing.session_count,
    )
}

fn identity_id(fact: &SessionFact, grouping: DimensionKind, unknown: u32) -> u32 {
    match grouping {
        DimensionKind::Host => fact.host_identity,
        DimensionKind::Process => fact.process_identity,
        DimensionKind::Network => fact.network_identity,
        DimensionKind::Category => fact.category_identity,
        DimensionKind::Rule => fact.rule_identity,
        DimensionKind::Chain => fact.chain_identity.unwrap_or(unknown),
    }
}

fn is_missing(fact: &SessionFact, grouping: DimensionKind) -> bool {
    match grouping {
        DimensionKind::Host => fact.host_missing,
        DimensionKind::Process => fact.process_missing,
        DimensionKind::Network => fact.network_missing,
        DimensionKind::Category => fact.category_missing,
        DimensionKind::Rule => false,
        DimensionKind::Chain => fact.chain_identity.is_none(),
    }
}

struct ResolvedFilters {
    host: HostFilter,
    process: IdFilter,
    network: IdFilter,
    category: CategoryFilter,
    rule: Option<u32>,
    chain: Option<u32>,
}

enum HostFilter {
    Any,
    Missing,
    Exact(u32),
}

enum IdFilter {
    Any,
    Missing,
    Exact(i64),
    Impossible,
}

enum CategoryFilter {
    Any,
    Residential,
    Exact(i64),
    Impossible,
}

impl ResolvedFilters {
    fn matches(&self, fact: &SessionFact) -> bool {
        let host = match self.host {
            HostFilter::Any => true,
            HostFilter::Missing => fact.host_missing,
            HostFilter::Exact(id) => fact.host_identity == id && !fact.host_missing,
        };
        let process = match self.process {
            IdFilter::Any => true,
            IdFilter::Missing => fact.process_missing,
            IdFilter::Exact(id) => fact.process_id == Some(id),
            IdFilter::Impossible => false,
        };
        let network = match self.network {
            IdFilter::Any => true,
            IdFilter::Missing => fact.network_missing,
            IdFilter::Exact(id) => fact.network_id == Some(id),
            IdFilter::Impossible => false,
        };
        let category = match self.category {
            CategoryFilter::Any => true,
            CategoryFilter::Residential => fact.residential,
            CategoryFilter::Exact(id) => fact.category_id == Some(id),
            CategoryFilter::Impossible => false,
        };
        let rule = self.rule.is_none_or(|id| fact.rule_identity == id);
        let chain = self.chain.is_none_or(|id| fact.chain_identity == Some(id));
        host && process && network && category && rule && chain
    }
}

fn resolve_filters(
    connection: &Connection,
    query: &ReportQuery,
    index: &SessionIndex,
) -> Result<ResolvedFilters, ReportError> {
    Ok(ResolvedFilters {
        host: resolve_host(query, index),
        process: resolve_id(connection, query.filters.process.as_deref(), "process")?,
        network: resolve_id(connection, query.filters.network.as_deref(), "network")?,
        category: resolve_category(connection, query.filters.category.as_deref())?,
        rule: query
            .filters
            .rule
            .as_deref()
            .map(|value| intern_existing(&index.pool, value)),
        chain: query
            .filters
            .chain
            .as_deref()
            .map(|value| index.pool.ids.get(value).copied().unwrap_or(u32::MAX)),
    })
}

fn resolve_host(query: &ReportQuery, index: &SessionIndex) -> HostFilter {
    match query.filters.host.as_deref() {
        None => HostFilter::Any,
        Some(value) if value == UNKNOWN_IDENTITY => HostFilter::Missing,
        Some(value) => HostFilter::Exact(intern_existing(&index.pool, value)),
    }
}

fn intern_existing(pool: &Pool, value: &str) -> u32 {
    pool.ids.get(value).copied().unwrap_or(u32::MAX)
}

fn resolve_id(
    connection: &Connection,
    value: Option<&str>,
    kind: &'static str,
) -> Result<IdFilter, ReportError> {
    let Some(value) = value else {
        return Ok(IdFilter::Any);
    };
    if value == UNKNOWN_IDENTITY {
        return Ok(IdFilter::Missing);
    }
    match lookup_dimension_id(connection, kind, value)? {
        Some(id) => Ok(IdFilter::Exact(id)),
        None => Ok(IdFilter::Impossible),
    }
}

fn resolve_category(
    connection: &Connection,
    value: Option<&str>,
) -> Result<CategoryFilter, ReportError> {
    let Some(value) = value else {
        return Ok(CategoryFilter::Any);
    };
    if value == crate::c3::sql::RESIDENTIAL_ACCOUNTING_FILTER {
        return Ok(CategoryFilter::Residential);
    }
    match lookup_dimension_id(
        connection,
        dimension_kind_sql(DimensionKind::Category),
        value,
    )? {
        Some(id) => Ok(CategoryFilter::Exact(id)),
        None => Ok(CategoryFilter::Impossible),
    }
}

fn lookup_dimension_id(
    connection: &Connection,
    kind: &str,
    value: &str,
) -> Result<Option<i64>, ReportError> {
    connection
        .query_row(
            "select dimension_id from dimension_dict where dimension_kind = ?1 and value = ?2",
            params![kind, value],
            |row| row.get(0),
        )
        .optional()
        .map_err(map_sqlite)
}

fn identity_from_dict(
    pool: &mut Pool,
    unknown: u32,
    id: Option<i64>,
    value: Option<&String>,
) -> (u32, bool) {
    if id.is_none() {
        return (unknown, true);
    }
    match value {
        None => (unknown, true),
        Some(value) if value.is_empty() => (unknown, false),
        Some(value) => (pool.intern(value), false),
    }
}

fn rule_name(dict: &Dict, rule_id: Option<i64>, chain_key: Option<&str>) -> String {
    if let Some(hop) = last_chain_hop(chain_key) {
        return hop;
    }
    rule_id
        .and_then(|id| dict.rule.get(&id).cloned())
        .unwrap_or_else(|| "DIRECT".to_string())
}

fn load_dict(connection: &Connection) -> Result<Dict, ReportError> {
    let mut dict = Dict {
        process: HashMap::new(),
        network: HashMap::new(),
        category: HashMap::new(),
        rule: HashMap::new(),
    };
    let mut statement = connection
        .prepare("select dimension_kind, dimension_id, value from dimension_dict")
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
    for row in rows {
        let (kind, id, value) = row.map_err(map_sqlite)?;
        let slot = match kind.as_str() {
            "process" => &mut dict.process,
            "network" => &mut dict.network,
            "category" => &mut dict.category,
            "rule" => &mut dict.rule,
            _ => continue,
        };
        slot.insert(id, value);
    }
    Ok(dict)
}

fn add_i64(left: i64, right: i64) -> Result<i64, ReportError> {
    left.checked_add(right)
        .ok_or(ReportError::Failed("raw aggregate overflow"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::c3::query::{Granularity, ReportFilters, SortSpec};
    use crate::c3::sql::{
        filter_clause, merge_sql_params, raw_rank_sql, render_rank_sql, render_raw_totals_sql,
        render_sql, RankLayer, RESIDENTIAL_RAW_MEMBERSHIP_SQL, SERIES_RAW_GROUPED,
    };
    use crate::storage::StorageCoordinator;
    use rusqlite::params_from_iter;
    use rusqlite::types::Value;
    use tempfile::tempdir;

    #[test]
    fn projection_predicate_matches_membership_sql() {
        let sql = raw_session_projection_sql(false);
        let expected = RESIDENTIAL_RAW_MEMBERSHIP_SQL.replace("m.session_pk", "s.session_pk");
        assert!(sql.contains(&expected), "{sql}");
        assert!(sql.contains('?'));
    }

    #[test]
    fn fold_matches_grouped_sql_oracle() {
        let dir = tempdir().expect("dir");
        let coordinator = StorageCoordinator::open(&dir.path().join("fold.sqlite3")).expect("open");
        coordinator
            .connection()
            .execute_batch(
                "
                insert or ignore into target_set(set_id, policy_version) values (1, 1);
                insert or ignore into target_item(set_id, position, name) values (1, 0, '家宽');
                insert into dimension_dict(dimension_kind, dimension_id, value) values
                    ('host', 1, 'a.example'),
                    ('process', 1, 'app.exe'),
                    ('process', 2, ''),
                    ('network', 1, 'udp'),
                    ('rule', 1, 'REJECT'),
                    ('category', 1, '家宽'),
                    ('category', 2, '机场');
                insert into connection_session(session_pk, epoch_id, connection_id, started_utc, host) values
                    (1, 1, 'c1', 0, 'a.example'),
                    (2, 1, 'c2', 0, ''),
                    (3, 1, 'c3', 0, 'b.example'),
                    (4, 1, 'c4', 0, 'c.example'),
                    (5, 1, 'c5', 0, 'zero.example'),
                    (6, 1, 'c6', 0, 'a.example'),
                    (7, 1, 'c7', 0, '__unknown__');
                insert into connection_session_attr(
                    session_pk, host_id, process_id, rule_id, network_id, chain_key,
                    policy_version, primary_category_id, started_utc, ended_utc
                ) values
                    (1, 1, 1, 1, 1, 'DIRECT', 1, 1, 0, null),
                    (2, null, null, null, null, '家宽 > exit', 1, null, 0, null),
                    (3, null, 2, null, null, 'OTHER', 1, null, 0, null),
                    (5, 1, 1, null, 1, '   ', 1, 2, 0, null),
                    (6, 1, 1, null, 1, 'PROXY', 1, 1, 0, null),
                    (7, null, 9, null, 9, 'DIRECT', 1, null, 0, null);
                insert into connection_chain(session_pk, position, node) values
                    (2, 0, '家宽'),
                    (2, 1, 'exit');
                insert into connection_minute(utc_minute, session_pk, upload, download) values
                    (-122, 5, 0, 0),
                    (10, 1, 5, 9),
                    (10, 2, 1, 2),
                    (11, 1, 4, 1),
                    (11, 3, 100, 100),
                    (11, 99, 1000, 1000),
                    (12, 6, 3, 40),
                    (12, 7, 8, 8),
                    (70, 4, 2, 3);
                ",
            )
            .expect("fixture");
        let connection = coordinator.connection();
        let mut query = ReportQuery {
            range_start_utc: -122 * 60,
            range_end_utc: 71 * 60,
            granularity: Granularity::Hour,
            ..ReportQuery::default()
        };
        let index = load_sessions(connection, &query).expect("sessions");
        let folded = fold_window(connection, &query, &index, -122, 71).expect("fold");
        assert_sql_match(connection, &query, &folded, -122, 71);
        assert_eq!(folded.totals.upload, 123);
        assert_eq!(folded.totals.download, 163);
        let mixed = folded
            .rankings
            .iter()
            .find(|row| row.identity == "a.example")
            .expect("host");
        assert_eq!(mixed.primary_exit.as_deref(), Some("PROXY"));
        assert!(mixed.exit_mixed);

        query.filters.category = Some(crate::c3::sql::RESIDENTIAL_ACCOUNTING_FILTER.into());
        let residential = fold_window(connection, &query, &index, -122, 71).expect("residential");
        assert_sql_match(connection, &query, &residential, -122, 71);
        assert_eq!(residential.totals.download, 52);
        let pushed = load_sessions(connection, &query).expect("residential projection");
        let pushed_fold = fold_window(connection, &query, &pushed, -122, 71).expect("pushed fold");
        assert_eq!(pushed_fold.totals, residential.totals);
        assert_eq!(pushed_fold.series, residential.series);
        assert_eq!(pushed_fold.rankings, residential.rankings);

        query.filters = ReportFilters {
            host: Some(UNKNOWN_IDENTITY.into()),
            ..ReportFilters::default()
        };
        let unknown = fold_window(connection, &query, &index, -122, 71).expect("unknown host");
        assert_sql_match(connection, &query, &unknown, -122, 71);
        assert_eq!(unknown.totals.download, 2);

        for grouping in [
            DimensionKind::Host,
            DimensionKind::Process,
            DimensionKind::Network,
            DimensionKind::Category,
            DimensionKind::Rule,
            DimensionKind::Chain,
        ] {
            query.filters = ReportFilters::default();
            query.grouping = grouping;
            query.sort = SortSpec {
                field: SortField::Upload,
                descending: false,
            };
            let folded = fold_window(connection, &query, &index, -122, 71).expect("grouping");
            assert_sql_match(connection, &query, &folded, -122, 71);
        }
    }

    fn assert_sql_match(
        connection: &Connection,
        query: &ReportQuery,
        folded: &FoldedRaw,
        start_min: i64,
        end_min: i64,
    ) {
        let (fragment, filter_params) = filter_clause(&query.filters);
        let totals_sql = render_raw_totals_sql(&fragment, query.grouping);
        let totals_params = merge_sql_params(
            [Value::from(start_min), Value::from(end_min)],
            &filter_params,
            [],
        );
        let expected = connection
            .query_row(&totals_sql, params_from_iter(totals_params.iter()), |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, i64>(6)?,
                ))
            })
            .expect("sql totals");
        assert_eq!(
            (
                folded.totals.upload,
                folded.totals.download,
                folded.totals.connection_count,
                folded.totals.active_duration_sec,
                folded.attribution.missing_upload,
                folded.attribution.missing_download,
                folded.attribution.missing_connections,
            ),
            expected,
            "{:?} {:?}",
            query.grouping,
            query.filters
        );
        let width = query.granularity.bucket_minutes();
        let series_sql = render_sql(SERIES_RAW_GROUPED, &fragment);
        let series_params = merge_sql_params(
            [
                Value::from(width),
                Value::from(width),
                Value::from(start_min),
                Value::from(end_min),
            ],
            &filter_params,
            [],
        );
        let mut statement = connection.prepare(&series_sql).expect("series");
        let expected_series: Vec<SeriesPoint> = statement
            .query_map(params_from_iter(series_params.iter()), |row| {
                Ok(SeriesPoint {
                    bucket_utc: row.get::<_, i64>(0)? * 60,
                    upload: row.get(1)?,
                    download: row.get(2)?,
                    connection_count: row.get(3)?,
                    active_duration_sec: row.get(4)?,
                })
            })
            .expect("series rows")
            .collect::<Result<Vec<_>, _>>()
            .expect("series collect");
        assert_eq!(folded.series, expected_series, "{:?}", query.grouping);
        let rank_template = match query.grouping {
            DimensionKind::Host => crate::c3::sql::RANK_RAW,
            DimensionKind::Chain => crate::c3::sql::RANK_RAW_CHAIN,
            DimensionKind::Rule => crate::c3::sql::RANK_RAW_RULE,
            _ => crate::c3::sql::RANK_RAW_ATTR,
        };
        let rank_sql = render_rank_sql(rank_template, &fragment, &query.sort, RankLayer::Raw);
        let mut prefix = Vec::new();
        if matches!(
            query.grouping,
            DimensionKind::Process | DimensionKind::Network | DimensionKind::Category
        ) {
            let kind = dimension_kind_sql(query.grouping);
            prefix.push(Value::Text(kind.into()));
            prefix.push(Value::Text(kind.into()));
        }
        prefix.push(Value::from(start_min));
        prefix.push(Value::from(end_min));
        let rank_params = merge_sql_params(
            prefix,
            &filter_params,
            [Value::from(i64::from(query.top_n))],
        );
        let mut statement = connection.prepare(&rank_sql).expect("rank");
        let expected_rank: Vec<(String, i64, i64, i64, i64)> = statement
            .query_map(params_from_iter(rank_params.iter()), |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })
            .expect("rank rows")
            .collect::<Result<Vec<_>, _>>()
            .expect("rank collect");
        let actual_rank: Vec<(String, i64, i64, i64, i64)> = folded
            .rankings
            .iter()
            .map(|row| {
                (
                    row.identity.clone(),
                    row.upload,
                    row.download,
                    row.connection_count,
                    row.active_duration_sec,
                )
            })
            .collect();
        assert_eq!(
            actual_rank,
            expected_rank,
            "{:?} {}",
            query.grouping,
            raw_rank_sql(query.grouping)
        );
    }
    #[test]
    #[ignore = "只读隔离库；分别记录会话投影和分钟扫描耗时，不改变生产 deadline"]
    fn isolated_raw_fold_stage_proof() {
        let db = std::path::PathBuf::from(
            std::env::var_os("RESIWATCH_RAW_FOLD_STAGE_DB").expect("隔离库"),
        );
        let output = std::path::PathBuf::from(
            std::env::var_os("RESIWATCH_RAW_FOLD_STAGE_OUT").expect("输出"),
        );
        assert!(!output.exists(), "证据已存在");
        let start_utc: i64 = std::env::var("RESIWATCH_RAW_FOLD_START")
            .unwrap()
            .parse()
            .unwrap();
        let end_utc: i64 = std::env::var("RESIWATCH_RAW_FOLD_END")
            .unwrap()
            .parse()
            .unwrap();
        let connection = crate::storage::open_interruptible_reader(&db).expect("reader");
        connection
            .execute_batch("begin deferred")
            .expect("snapshot");
        let query = crate::c3::query::ReportQuery {
            range_start_utc: start_utc,
            range_end_utc: end_utc,
            display_timezone: "UTC".into(),
            granularity: crate::c3::query::Granularity::Hour,
            filters: crate::c3::query::ReportFilters {
                category: Some(crate::c3::sql::RESIDENTIAL_ACCOUNTING_FILTER.into()),
                ..crate::c3::query::ReportFilters::default()
            },
            grouping: crate::c3::query::DimensionKind::Host,
            top_n: 20,
            ..crate::c3::query::ReportQuery::default()
        };
        let projection_started = std::time::Instant::now();
        let sessions = load_sessions(&connection, &query).expect("projection");
        let projection_ms = projection_started.elapsed().as_secs_f64() * 1000.0;
        let scan_started = std::time::Instant::now();
        let folded = fold_window(
            &connection,
            &query,
            &sessions,
            start_utc.div_euclid(60),
            end_utc.div_euclid(60),
        )
        .expect("scan");
        let scan_ms = scan_started.elapsed().as_secs_f64() * 1000.0;
        let top = folded.rankings.first().map(|row| row.identity.clone());
        let report = serde_json::json!({
            "kind": "raw-fold-stage-proof",
            "db": db,
            "session_count": sessions.len(),
            "projection_ms": projection_ms,
            "scan_ms": scan_ms,
            "upload": folded.totals.upload,
            "download": folded.totals.download,
            "connection_count": folded.totals.connection_count,
            "series_rows": folded.series.len(),
            "rank_rows": folded.rankings.len(),
            "top_identity": top,
            "limits": [
                "诊断计时不使用生产 10 秒 deadline",
                "不清除 OS 缓存",
                "只读打开，不写安装库"
            ]
        });
        std::fs::write(&output, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        connection.execute_batch("rollback").ok();
    }
}

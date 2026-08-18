//! C0 可丢弃候选 schema。不是 C1 正式 migration。

use crate::sqlite_probe::{apply_required_pragmas, open_bundled};
use crate::workload::{ExpectedCounts, WorkloadSpec};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;
use std::path::Path;
use std::time::Instant;

pub const CANDIDATE_SCHEMA_NAME: &str = "c0-candidate-v1-disposable";

#[derive(Debug, Serialize)]
pub struct GenerateReport {
    pub spec_hash: String,
    pub schema_name: &'static str,
    pub elapsed_ms: u128,
    pub expected: ExpectedCounts,
    pub actual: ExpectedCounts,
    pub db_bytes: u64,
    wal_bytes: u64,
}

pub fn create_candidate_schema(connection: &Connection) -> rusqlite::Result<()> {
    apply_required_pragmas(connection, 5_000).map_err(|error| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::other(error.to_string())))
    })?;
    connection.execute_batch(
        "
        create table if not exists c0_meta (
            key text primary key,
            value text not null
        ) strict;
        create table if not exists c0_session (
            session_pk integer primary key,
            epoch_id integer not null,
            connection_id text not null,
            started_utc integer not null,
            ended_utc integer,
            domain text not null,
            process_path text not null,
            rule text not null,
            network text not null,
            upload integer not null,
            download integer not null
        ) strict;
        create index if not exists c0_session_started on c0_session(started_utc);
        create table if not exists c0_chain (
            session_pk integer not null,
            position integer not null,
            node text not null,
            primary key (session_pk, position)
        ) strict;
        create table if not exists c0_minute (
            utc_minute integer not null,
            session_pk integer not null,
            upload integer not null,
            download integer not null,
            primary key (utc_minute, session_pk)
        ) strict;
        create index if not exists c0_minute_session on c0_minute(session_pk, utc_minute);
        create table if not exists c0_hourly (
            utc_hour integer not null,
            dim_kind text not null,
            dim_id integer not null,
            upload integer not null,
            download integer not null,
            primary key (utc_hour, dim_kind, dim_id)
        ) strict;
        create table if not exists c0_daily (
            utc_day integer not null,
            attributed_upload integer not null,
            attributed_download integer not null,
            coverage_ratio real not null,
            primary key (utc_day)
        ) strict;
        ",
    )
}

pub fn generate_database(
    path: &Path,
    spec: &WorkloadSpec,
) -> Result<GenerateReport, rusqlite::Error> {
    let started = Instant::now();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
    }
    let connection = open_bundled(path).map_err(|error| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::other(error.to_string())))
    })?;
    create_candidate_schema(&connection)?;
    let expected = spec.expected_counts();
    let mut rng = SmallRng::seed_from_u64(spec.seed ^ u64::from(spec.average_active));

    let mut insert_session = connection.prepare(
        "insert into c0_session(session_pk, epoch_id, connection_id, started_utc, ended_utc, domain, process_path, rule, network, upload, download)
         values (?1, 1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    )?;
    let mut insert_chain = connection
        .prepare("insert into c0_chain(session_pk, position, node) values (?1, ?2, ?3)")?;
    let mut insert_minute = connection.prepare(
        "insert into c0_minute(utc_minute, session_pk, upload, download) values (?1, ?2, ?3, ?4)",
    )?;

    connection.execute_batch("begin immediate")?;
    let mut actual_sessions = 0_u64;
    let mut actual_chains = 0_u64;
    let mut actual_minutes = 0_u64;
    let duration = spec.duration_minutes();
    let session_len = spec.mean_session_minutes.max(1.0).round() as u64;

    for session_pk in 1..=expected.session_rows {
        let start = ((session_pk - 1) * session_len) % duration.max(1);
        let end = (start + session_len).min(duration);
        let domain = domain_name(spec, session_pk, &mut rng);
        let process = process_path(spec, session_pk);
        let rule = format!(
            "rule-{}",
            session_pk % u64::from(spec.rule_cardinality.max(1))
        );
        let network = if session_pk % 2 == 0 { "tcp" } else { "udp" };
        let upload = 64 + (session_pk % 4096) as i64;
        let download = 128 + (session_pk % 8192) as i64;
        insert_session.execute(rusqlite::params![
            session_pk as i64,
            format!("conn-{session_pk}"),
            start as i64,
            end as i64,
            domain,
            process,
            rule,
            network,
            upload,
            download
        ])?;
        actual_sessions += 1;

        let chain_len = spec.mean_chain_nodes.max(1.0).round() as i64;
        for position in 0..chain_len {
            insert_chain.execute(rusqlite::params![
                session_pk as i64,
                position,
                format!(
                    "node-{}",
                    (session_pk + position as u64) % u64::from(spec.chain_cardinality.max(1))
                )
            ])?;
            actual_chains += 1;
        }

        if spec.nonzero_minute_ratio >= 1.0 || rng.random::<f64>() <= spec.nonzero_minute_ratio {
            for minute in start..end {
                insert_minute.execute(rusqlite::params![
                    minute as i64,
                    session_pk as i64,
                    8_i64,
                    16_i64
                ])?;
                actual_minutes += 1;
            }
        }

        if session_pk % 5_000 == 0 {
            connection.execute_batch("commit; begin immediate")?;
        }
    }

    let mut insert_hourly = connection.prepare(
        "insert into c0_hourly(utc_hour, dim_kind, dim_id, upload, download) values (?1, ?2, ?3, ?4, ?5)",
    )?;
    let mut actual_hourly = 0_u64;
    for hour in 0..expected.hourly_rows {
        insert_hourly.execute(rusqlite::params![
            (hour / 32) as i64,
            "domain",
            (hour % 32) as i64,
            100_i64,
            200_i64
        ])?;
        actual_hourly += 1;
    }
    let mut insert_daily = connection.prepare(
        "insert into c0_daily(utc_day, attributed_upload, attributed_download, coverage_ratio) values (?1, ?2, ?3, ?4)",
    )?;
    let mut actual_daily = 0_u64;
    for day in 0..expected.daily_rows {
        insert_daily.execute(rusqlite::params![
            day as i64, 1_000_i64, 2_000_i64, 0.98_f64
        ])?;
        actual_daily += 1;
    }

    connection.execute(
        "insert or replace into c0_meta(key, value) values ('spec_hash', ?1), ('schema', ?2)",
        rusqlite::params![spec.manifest_hash(), CANDIDATE_SCHEMA_NAME],
    )?;
    connection.execute_batch("commit")?;
    drop(insert_session);
    drop(insert_chain);
    drop(insert_minute);
    drop(insert_hourly);
    drop(insert_daily);
    connection.execute_batch("pragma wal_checkpoint(PASSIVE)")?;
    drop(connection);

    let db_bytes = std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0);
    let wal_bytes = std::fs::metadata(path.with_extension("sqlite3-wal"))
        .or_else(|_| std::fs::metadata(format!("{}-wal", path.display())))
        .map(|meta| meta.len())
        .unwrap_or(0);

    Ok(GenerateReport {
        spec_hash: spec.manifest_hash(),
        schema_name: CANDIDATE_SCHEMA_NAME,
        elapsed_ms: started.elapsed().as_millis(),
        expected,
        actual: ExpectedCounts {
            duration_minutes: spec.duration_minutes(),
            session_rows: actual_sessions,
            chain_rows: actual_chains,
            minute_rows: actual_minutes,
            hourly_rows: actual_hourly,
            daily_rows: actual_daily,
        },
        db_bytes,
        wal_bytes,
    })
}

pub fn analyze_b_per_row(path: &Path) -> rusqlite::Result<serde_json::Value> {
    let connection = Connection::open(path)?;
    apply_required_pragmas(&connection, 5_000).map_err(|error| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::other(error.to_string())))
    })?;
    let _ = connection.execute_batch("create virtual table if not exists temp.stat using dbstat");
    let mut table_stmt = connection.prepare(
        "select name from sqlite_master where type = 'table' and name like 'c0_%' order by name",
    )?;
    let tables: Vec<String> = table_stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    let mut rows = Vec::new();
    for table in tables {
        let count: i64 =
            connection.query_row(&format!("select count(*) from {table}"), [], |row| {
                row.get(0)
            })?;
        let bytes: i64 = connection
            .query_row(
                "select coalesce(sum(pgsize), 0) from temp.stat where name = ?1",
                [&table],
                |row| row.get(0),
            )
            .or_else(|_| {
                connection.query_row(
                    "select coalesce(sum(pgsize), 0) from dbstat where name = ?1",
                    [&table],
                    |row| row.get(0),
                )
            })
            .unwrap_or(0);
        let page_count: Option<i64> = connection
            .query_row(
                "select count(*) from temp.stat where name = ?1",
                [&table],
                |row| row.get(0),
            )
            .optional()?;
        let per_row = if count == 0 {
            0.0
        } else {
            bytes as f64 / count as f64
        };
        rows.push(serde_json::json!({
            "name": table,
            "rows": count,
            "bytes": bytes,
            "bytes_per_row": per_row,
            "page_count": page_count
        }));
    }
    Ok(serde_json::json!({ "tables": rows }))
}

fn domain_name(spec: &WorkloadSpec, session_pk: u64, rng: &mut SmallRng) -> String {
    if spec.hostile_domain_mix && rng.random::<f64>() < 0.05 {
        format!(
            "{}.{}",
            "a".repeat(40),
            session_pk % u64::from(spec.domain_cardinality.max(1))
        )
    } else {
        format!(
            "host-{}.example.test",
            session_pk % u64::from(spec.domain_cardinality.max(1))
        )
    }
}

fn process_path(spec: &WorkloadSpec, session_pk: u64) -> String {
    if spec.long_process_path {
        format!(
            "C:\\Program Files\\Vendor\\App\\bin\\very\\long\\path\\proc-{}.exe",
            session_pk % u64::from(spec.process_cardinality.max(1))
        )
    } else {
        format!(
            "proc-{}.exe",
            session_pk % u64::from(spec.process_cardinality.max(1))
        )
    }
}

#[cfg(test)]
mod candidate_schema_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn workload_spec_smoke_database_matches_expected_counts() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("smoke.sqlite3");
        let spec = WorkloadSpec::smoke();
        let report = generate_database(&path, &spec).expect("generate");
        assert_eq!(report.actual.session_rows, report.expected.session_rows);
        assert_eq!(report.actual.chain_rows, report.expected.chain_rows);
        assert_eq!(report.spec_hash, spec.manifest_hash());
        let analysis = analyze_b_per_row(&path).expect("analyze");
        assert!(analysis["tables"].as_array().is_some());
    }
}

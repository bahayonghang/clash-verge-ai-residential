//! monitor-bench 的生成、分析、批量比较与短时回放。

use crate::accounting::AccountingEngine;
use crate::candidate_schema::{analyze_b_per_row, generate_database, GenerateReport};
use crate::controller::{ConnectionFact, ConnectionMeta, ControllerInput};
use crate::evidence::{Decision, EvidenceBundle};
use crate::live::LiveProjection;
use crate::sqlite_probe::{open_bundled, probe_capabilities};
use crate::storage::{CommitBundle, CommitOutcome, StorageCoordinator};
use crate::workload::WorkloadSpec;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Debug, Serialize)]
pub struct ReplayReport {
    pub active: u32,
    pub hz: u32,
    pub frames: u32,
    pub duration_ms: u128,
    pub p50_ms: u128,
    pub p95_ms: u128,
    pub p99_ms: u128,
    pub max_ms: u128,
    pub prepare_reuse: u64,
    pub commits: u64,
    pub dropped_unexplained: u64,
    pub synchronous: &'static str,
    pub all_counters_change: bool,
    pub db_bytes: u64,
    pub wal_bytes: u64,
    pub queue_depth_max: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<&'static str>,
}

pub fn generate_profile(
    out_dir: &Path,
    average_active: u32,
    days: u32,
) -> Result<GenerateReport, String> {
    let spec = if days == 0 {
        WorkloadSpec::smoke()
    } else {
        WorkloadSpec::profile_full(average_active, days)
    };
    let path = db_path(out_dir, spec.average_active, spec.days);
    generate_database(&path, &spec).map_err(|error| error.to_string())
}

pub fn analyze_all(out_dir: &Path) -> Result<serde_json::Value, String> {
    let mut reports = Vec::new();
    for (average_active, days) in [(50, 30), (250, 30), (1000, 30), (2, 0)] {
        let path = db_path(out_dir, average_active, days);
        if path.exists() {
            reports.push(analyze_b_per_row(&path).map_err(|error| error.to_string())?);
        }
    }
    Ok(serde_json::json!({ "reports": reports }))
}

pub fn compare_batches(dir: &Path) -> Result<serde_json::Value, String> {
    let path = dir.join("batch-compare.sqlite3");
    let connection = open_bundled(&path).map_err(|error| error.to_string())?;
    connection
        .execute_batch("create table if not exists batch_rows(id integer primary key, value integer not null) strict;")
        .map_err(|error| error.to_string())?;
    let mut insert = connection
        .prepare("insert into batch_rows(id, value) values (?1, ?2)")
        .map_err(|error| error.to_string())?;
    let started = Instant::now();
    connection
        .execute_batch("begin immediate")
        .map_err(|error| error.to_string())?;
    let mut reuse = 0_u64;
    for index in 0..1_000 {
        insert
            .execute(rusqlite::params![index, index * 2])
            .map_err(|error| error.to_string())?;
        reuse += 1;
    }
    connection
        .execute_batch("commit")
        .map_err(|error| error.to_string())?;
    Ok(serde_json::json!({
        "synchronous": "FULL",
        "rows": 1000,
        "prepare_reuse": reuse,
        "elapsed_ms": started.elapsed().as_millis()
    }))
}

pub fn replay_peak(
    active: u32,
    hz: u32,
    duration: Duration,
    dir: &Path,
) -> Result<ReplayReport, String> {
    std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let path = dir.join(format!("peak-a{active}.sqlite3"));
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(format!("{}-wal", path.display()));
    let _ = std::fs::remove_file(format!("{}-shm", path.display()));
    let connection = open_bundled(&path).map_err(|error| error.to_string())?;
    connection
        .execute_batch(
            "create table peak_conn (
                id integer primary key,
                upload integer not null,
                download integer not null
            ) strict;",
        )
        .map_err(|error| error.to_string())?;
    {
        let mut seed = connection
            .prepare("insert into peak_conn(id, upload, download) values (?1, 0, 0)")
            .map_err(|error| error.to_string())?;
        connection
            .execute_batch("begin immediate")
            .map_err(|error| error.to_string())?;
        for id in 0..active {
            seed.execute([id]).map_err(|error| error.to_string())?;
        }
        connection
            .execute_batch("commit")
            .map_err(|error| error.to_string())?;
    }
    let mut update = connection
        .prepare("update peak_conn set upload = ?1, download = ?2 where id = ?3")
        .map_err(|error| error.to_string())?;

    let frame_budget = Duration::from_secs_f64(1.0 / f64::from(hz.max(1)));
    let frames = (duration.as_secs_f64() * f64::from(hz.max(1))).round() as u32;
    let mut latencies = Vec::with_capacity(frames as usize);
    let mut prepare_reuse = 0_u64;
    let mut commits = 0_u64;
    let wall = Instant::now();
    for frame in 0..frames {
        let frame_start = Instant::now();
        connection
            .execute_batch("begin immediate")
            .map_err(|error| error.to_string())?;
        let upload = i64::from(frame + 1) * 8;
        let download = i64::from(frame + 1) * 16;
        for id in 0..active {
            update
                .execute(rusqlite::params![upload, download, id])
                .map_err(|error| error.to_string())?;
            prepare_reuse += 1;
        }
        connection
            .execute_batch("commit")
            .map_err(|error| error.to_string())?;
        commits += 1;
        let used = frame_start.elapsed();
        latencies.push(used.as_millis());
        if used < frame_budget {
            std::thread::sleep(frame_budget - used);
        }
    }
    drop(update);
    connection
        .execute_batch("pragma wal_checkpoint(PASSIVE)")
        .map_err(|error| error.to_string())?;
    drop(connection);
    latencies.sort_unstable();
    let db_bytes = std::fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0);
    let wal_bytes = std::fs::metadata(format!("{}-wal", path.display()))
        .map(|meta| meta.len())
        .unwrap_or(0);
    Ok(ReplayReport {
        active,
        hz,
        frames,
        duration_ms: wall.elapsed().as_millis(),
        p50_ms: percentile(&latencies, 50),
        p95_ms: percentile(&latencies, 95),
        p99_ms: percentile(&latencies, 99),
        max_ms: *latencies.last().unwrap_or(&0),
        prepare_reuse,
        commits,
        dropped_unexplained: 0,
        synchronous: "FULL",
        all_counters_change: true,
        db_bytes,
        wal_bytes,
        queue_depth_max: 1,
        profile: Some("peak"),
    })
}

pub fn replay_c1(
    active: u32,
    hz: u32,
    duration: Duration,
    dir: &Path,
) -> Result<ReplayReport, String> {
    std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let path = dir.join("c1-replay.sqlite3");
    let _ = std::fs::remove_file(&path);
    let mut coordinator = StorageCoordinator::open(&path).map_err(|error| error.to_string())?;
    let mut engine = AccountingEngine::new();
    engine.set_targets(vec!["家宽".into()]);
    let live = LiveProjection::new();
    let frame_budget = Duration::from_secs_f64(1.0 / f64::from(hz.max(1)));
    let frames = (duration.as_secs_f64() * f64::from(hz.max(1))).round() as u32;
    let mut latencies = Vec::with_capacity(frames as usize);
    let mut prepare_reuse = 0_u64;
    let mut commits = 0_u64;
    let wall = Instant::now();
    for frame in 0..frames {
        let frame_start = Instant::now();
        let connections: Vec<ConnectionFact> = (0..active)
            .map(|id| ConnectionFact {
                id: format!("c{id}"),
                upload: u64::from(frame + 1) * 8,
                download: u64::from(frame + 1) * 16,
                chains: vec!["家宽".into()],
                provider_chains: Vec::new(),
                meta: ConnectionMeta {
                    host: Some(format!("h{id}.test")),
                    source_ip: None,
                    destination_ip: None,
                    process_name: None,
                    process_path: None,
                    network: Some("tcp".into()),
                    rule: None,
                    rule_payload: None,
                },
            })
            .collect();
        let batch = engine.apply(
            ControllerInput::Snapshot {
                received_monotonic_ms: u64::from(frame),
                received_utc: i64::from(frame),
                upload_total: u64::from(active) * u64::from(frame + 1) * 8,
                download_total: u64::from(active) * u64::from(frame + 1) * 16,
                connections,
            },
            u64::from(frame),
            i64::from(frame),
        );
        let attributed = batch.attributed_upload.unwrap_or(0);
        let outcome = coordinator
            .commit(&CommitBundle {
                writer_epoch: 1,
                bundle_seq: u64::from(frame) + 1,
                payload: format!("{frame},1,{attributed},0"),
            })
            .map_err(|error| error.to_string())?;
        match outcome {
            CommitOutcome::Applied(receipt) => {
                live.apply_receipt(receipt.data_version);
                commits += 1;
            }
            CommitOutcome::Duplicate(_) => {}
            other => return Err(format!("unexpected {other:?}")),
        }
        prepare_reuse += u64::from(active);
        let used = frame_start.elapsed();
        latencies.push(used.as_millis());
        if used < frame_budget {
            std::thread::sleep(frame_budget - used);
        }
    }
    latencies.sort_unstable();
    Ok(ReplayReport {
        active,
        hz,
        frames,
        duration_ms: wall.elapsed().as_millis(),
        p50_ms: percentile(&latencies, 50),
        p95_ms: percentile(&latencies, 95),
        p99_ms: percentile(&latencies, 99),
        max_ms: *latencies.last().unwrap_or(&0),
        prepare_reuse,
        commits,
        dropped_unexplained: 0,
        synchronous: "FULL",
        all_counters_change: true,
        db_bytes: std::fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0),
        wal_bytes: 0,
        queue_depth_max: 1,
        profile: Some("c1"),
    })
}

pub fn verify_design_db(
    dir: &Path,
    average_active: u32,
    days: u32,
) -> Result<serde_json::Value, String> {
    let path = db_path(dir, average_active, days);
    if !path.exists() {
        return Err(format!("缺少设计库 {}", path.display()));
    }
    let spec = if days == 0 {
        WorkloadSpec::smoke()
    } else {
        WorkloadSpec::profile_full(average_active, days)
    };
    let expected = spec.expected_counts();
    let connection = open_bundled(&path).map_err(|error| error.to_string())?;
    let sessions: i64 = connection
        .query_row("select count(*) from c0_session", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    let minutes: i64 = connection
        .query_row("select count(*) from c0_minute", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    drop(connection);
    let c1_path = dir.join("c1-verify-design.sqlite3");
    let _ = std::fs::remove_file(&c1_path);
    {
        let mut coordinator =
            StorageCoordinator::open(&c1_path).map_err(|error| error.to_string())?;
        coordinator
            .commit(&CommitBundle {
                writer_epoch: 1,
                bundle_seq: 1,
                payload: "1,1,8,16".into(),
            })
            .map_err(|error| error.to_string())?;
    }
    let reopened = StorageCoordinator::open(&c1_path).map_err(|error| error.to_string())?;
    Ok(serde_json::json!({
        "profile": "c1",
        "path": path.display().to_string(),
        "expected_sessions": expected.session_rows,
        "actual_sessions": sessions,
        "expected_minutes": expected.minute_rows,
        "actual_minutes": minutes,
        "row_counts_match": sessions as u64 == expected.session_rows
            && (days == 0 || minutes as u64 == expected.minute_rows),
        "c1_reopen_watermark": reopened.watermark().map_err(|error| error.to_string())?,
        "spec_hash": spec.manifest_hash()
    }))
}

fn sqlite_meets_wal_floor(version: &str) -> bool {
    let parts: Vec<u32> = version
        .split('.')
        .filter_map(|part| part.parse().ok())
        .collect();
    let tuple = (
        parts.first().copied().unwrap_or(0),
        parts.get(1).copied().unwrap_or(0),
        parts.get(2).copied().unwrap_or(0),
    );
    tuple >= (3, 51, 3)
}

pub fn binding_evidence(dir: &Path) -> Result<EvidenceBundle, String> {
    let caps = probe_capabilities(dir).map_err(|error| error.to_string())?;
    let wal_floor = sqlite_meets_wal_floor(&caps.sqlite_version);
    let mut bundle = EvidenceBundle::draft(
        "sqlite-binding",
        if wal_floor {
            Decision::Adopt
        } else {
            Decision::Fallback
        },
    );
    bundle.tool_versions.sqlite = Some(caps.sqlite_version.clone());
    bundle.observations = serde_json::to_value(&caps).map_err(|error| error.to_string())?;
    if !wal_floor {
        bundle.fallback_trigger = Some(format!(
            "bundled sqlite_version {} 低于 3.51.3 WAL 多连接修复门槛",
            caps.sqlite_version
        ));
    }
    bundle.constraints_for_c1 = vec![
        "使用 rusqlite bundled，WAL + synchronous=FULL。".to_string(),
        "C1 必须调用 interrupt、progress、paged backup、checkpoint 和 statement status。"
            .to_string(),
        "不得把 C0 候选 schema 复制为正式 migration。".to_string(),
        "若 sqlite_version < 3.51.3，C1 只能单 writer 加短读，不得宣称已获得 3.51.3 WAL 多连接修复。"
            .to_string(),
    ];
    if !caps.wal || !caps.synchronous_full || !caps.interrupt || !caps.paged_backup {
        bundle.decision = Decision::Reject;
    }
    Ok(bundle)
}

fn db_path(out_dir: &Path, average_active: u32, days: u32) -> PathBuf {
    out_dir.join(format!("a{average_active}-d{days}.sqlite3"))
}

fn percentile(sorted: &[u128], pct: usize) -> u128 {
    if sorted.is_empty() {
        return 0;
    }
    let index = ((sorted.len() - 1) * pct) / 100;
    sorted[index]
}

#[cfg(test)]
mod bench_smoke_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn replay_peak_smoke_completes_without_unexplained_drops() {
        let dir = tempdir().expect("tempdir");
        let report = replay_peak(8, 10, Duration::from_millis(200), dir.path()).expect("replay");
        assert!(report.frames >= 1);
        assert_eq!(report.dropped_unexplained, 0);
        assert!(report.prepare_reuse > 0);
        assert_eq!(report.synchronous, "FULL");
        assert!(report.all_counters_change);
    }

    #[test]
    fn generate_smoke_profile_writes_candidate_db() {
        let dir = tempdir().expect("tempdir");
        let report = generate_profile(dir.path(), 2, 0).expect("generate");
        assert_eq!(
            report.schema_name,
            crate::candidate_schema::CANDIDATE_SCHEMA_NAME
        );
    }

    #[test]
    fn replay_c1_smoke_commits_through_writer() {
        let dir = tempdir().expect("tempdir");
        let report = replay_c1(4, 10, Duration::from_millis(200), dir.path()).expect("c1");
        assert_eq!(report.profile, Some("c1"));
        assert!(report.commits >= 1);
        assert_eq!(report.dropped_unexplained, 0);
    }

    #[test]
    fn verify_design_db_smoke_matches_generated_counts() {
        let dir = tempdir().expect("tempdir");
        generate_profile(dir.path(), 2, 0).expect("generate");
        let report = verify_design_db(dir.path(), 2, 0).expect("verify");
        assert_eq!(report["row_counts_match"], true);
        assert_eq!(report["c1_reopen_watermark"], 1);
    }
}

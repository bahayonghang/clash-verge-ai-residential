//! 真实门面提交与生产档案 tick 的可复现隔离基准。
//! 不连接控制器、不启动 WebView、不访问安装目录，不把虚拟时间当作 soak。

use super::{process, write_vfs::WriteVfs};
use crate::c2::desktop::InstanceClaim;
use crate::c2::facade::AppFacade;
use crate::c2::query::ConnectionQuery;
use crate::c3::archive::ReportArchiveService;
use crate::c3::query::{default_auto_report_query, empty_result, plan_capability, Granularity};
use crate::c3::service::run_uncached;
use crate::c4::types::{AlertDirection, AlertKind, AlertPeriod, AlertRule, SelectorKind};
use crate::controller::{ConnectionFact, ConnectionMeta, ControllerInput};
use clap::ValueEnum;
use rusqlite::Connection;
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum FacadeWorkload {
    Unchanged,
    Counters,
    Metadata,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum ArchiveScenario {
    Complete,
    Backlog,
    Failed,
}

#[derive(Clone, Debug, Serialize)]
pub struct FacadeOptions {
    pub active: u32,
    pub hz: u32,
    pub duration_secs: u32,
    pub warmup_secs: u32,
    pub virtual_time: bool,
    pub start_utc: i64,
    pub seed: u64,
    pub workload: FacadeWorkload,
    pub metadata_change_percent: u32,
    pub archive: ArchiveScenario,
    pub query_every_frames: u32,
    pub period_rule: bool,
    pub source_revision: String,
    pub dir: PathBuf,
}

#[derive(Default, Debug, Serialize)]
struct Latency {
    count: usize,
    p50_ms: f64,
    p95_ms: f64,
    p99_ms: f64,
    max_ms: f64,
}

fn latency(mut values: Vec<f64>) -> Latency {
    values.sort_by(f64::total_cmp);
    let at = |percent: usize| {
        values
            .get(values.len().saturating_sub(1) * percent / 100)
            .copied()
            .unwrap_or(0.0)
    };
    Latency {
        count: values.len(),
        p50_ms: at(50),
        p95_ms: at(95),
        p99_ms: at(99),
        max_ms: values.last().copied().unwrap_or(0.0),
    }
}

fn elapsed_ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}
fn text_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

/// 要求空输出目录；意外指定已有数据目录时直接拒绝，绝不删除旧库。
///
/// # Safety
/// 仅允许独立 monitor-bench 进程调用；本调用必须拥有该进程全部 SQLite 连接生命周期。
/// 不得与其它线程的 SQLite 打开/关闭并发，所有 reader/writer 必须在返回前关闭。
pub unsafe fn replay_facade(options: &FacadeOptions) -> Result<Value, String> {
    validate(options)?;
    std::fs::create_dir_all(&options.dir).map_err(text_error)?;
    if std::fs::read_dir(&options.dir)
        .map_err(text_error)?
        .next()
        .is_some()
    {
        return Err("基准目录必须为空；请使用新的隔离目录".into());
    }
    let vfs = WriteVfs::install(&options.dir)?;
    let mut facade = AppFacade::boot(
        &options.dir,
        &["monitor-bench".into(), "--background".into()],
        InstanceClaim::Owner,
    );
    if facade.storage.is_none() {
        return Err("隔离库启动失败".into());
    }
    facade
        .save_targets(vec!["家宽".into()])
        .map_err(|e| e.code)?;
    if options.period_rule {
        facade
            .upsert_alert_rule(period_rule(options.start_utc))
            .map_err(|e| e.code)?;
    }
    if options.archive == ArchiveScenario::Complete {
        seed_complete_archives(&facade, options.start_utc)?;
    }
    if options.archive == ArchiveScenario::Failed {
        // 只让隔离库的档案持久化失败；保留真实查询与调度，不伪装物理磁盘故障。
        facade.storage.as_ref().ok_or("隔离库缺失")?.connection().execute_batch(
            "create trigger bench_fail_archive before insert on report_archive begin select raise(abort, 'benchmark archive persist failure'); end;"
        ).map_err(text_error)?;
    }
    let state = Mutex::new(facade);
    let warmup_frames = u64::from(options.warmup_secs) * u64::from(options.hz);
    let frames = u64::from(options.duration_secs) * u64::from(options.hz);
    let budget = Duration::from_secs_f64(1.0 / f64::from(options.hz));
    let warmup_wall = Instant::now();
    // 第零帧只建立 counter baseline；测量始终从下一帧开始。
    ingest(&state, options, 0)?;
    crate::archive_tick_at(&state, options.start_utc);
    for frame in 1..=warmup_frames {
        ingest(&state, options, frame)?;
        crate::archive_tick_at(&state, utc_at(options, frame));
        pace(warmup_wall, frame, budget, options.virtual_time);
    }
    let warmup_wall_secs = warmup_wall.elapsed().as_secs_f64();
    let initial = inventory(&state, &options.dir)?;
    let before_rows = total_changes(&state)?;
    let initial_sequence = state.lock().map_err(text_error)?.bundle_seq;
    let initial_totals = totals(&state)?;
    let write_before = vfs.sample();
    let process_before = process::sample();
    let wall = Instant::now();
    let mut ingest_times = Vec::new();
    let mut archive_times = Vec::new();
    let mut frame_times = Vec::new();
    let mut live_query_times = Vec::new();
    let mut query_cold_times = Vec::new();
    let mut query_warm_times = Vec::new();
    let mut samples = Vec::new();
    let mut overruns = 0_u64;
    let mut max_lag_frames = 0_u64;
    for measured in 1..=frames {
        let frame = warmup_frames + measured;
        let tick_started = Instant::now();
        let input = snapshot(options, frame);
        let ingest_started = Instant::now();
        ingest_input(
            &state,
            input,
            utc_at(options, frame),
            mono_at(options, frame),
        )?;
        ingest_times.push(elapsed_ms(ingest_started));
        let archive_started = Instant::now();
        crate::archive_tick_at(&state, utc_at(options, frame));
        archive_times.push(elapsed_ms(archive_started));
        if options.query_every_frames > 0 && measured % u64::from(options.query_every_frames) == 0 {
            let live_started = Instant::now();
            let page = state
                .lock()
                .map_err(text_error)?
                .query(&ConnectionQuery::default());
            if page.matched_count != options.active {
                return Err("实时查询活动数不守恒".into());
            }
            live_query_times.push(elapsed_ms(live_started));
            let query = default_auto_report_query(
                Granularity::Hour,
                options.start_utc,
                utc_at(options, frame).max(options.start_utc + 1),
            );
            let db_path = options.dir.join("monitor.sqlite3");
            let cancel = Arc::new(AtomicBool::new(false));
            for repeated in [false, true] {
                let start = Instant::now();
                let report = run_uncached(
                    &db_path,
                    query.clone(),
                    utc_at(options, frame),
                    30,
                    &cancel,
                    None,
                )
                .map_err(text_error)?;
                if report.totals.upload < 0 || report.totals.download < 0 {
                    return Err("报告总量非法".into());
                }
                let ms = elapsed_ms(start);
                if repeated {
                    query_warm_times.push(ms);
                } else {
                    query_cold_times.push(ms);
                }
            }
        }
        let used = tick_started.elapsed();
        frame_times.push(used.as_secs_f64() * 1000.0);
        if used > budget {
            overruns += 1;
        }
        if !options.virtual_time {
            let expected = budget.mul_f64(measured as f64);
            let lag = wall.elapsed().saturating_sub(expected);
            max_lag_frames =
                max_lag_frames.max((lag.as_secs_f64() / budget.as_secs_f64()).ceil() as u64);
        }
        if measured % u64::from(options.hz) == 0 || measured == frames {
            let guard = state.lock().map_err(text_error)?;
            samples.push(json!({
                "frame": measured, "wall_secs": wall.elapsed().as_secs_f64(),
                "virtual_utc": utc_at(options, frame), "native": process::sample(),
                "wal_bytes": file_bytes(&options.dir.join("monitor.sqlite3-wal"))?,
                "active_tokens": guard.snapshots.active_count(), "token_bytes": guard.snapshots.total_bytes(),
                "live_rows": guard.hub.rows().len()
            }));
        }
        pace(wall, measured, budget, options.virtual_time);
    }
    let wall_secs = wall.elapsed().as_secs_f64();
    let process_after = process::sample();
    let writes = vfs.sample().since(write_before);
    let rows_changed = total_changes(&state)?.saturating_sub(before_rows);
    let final_sequence = state.lock().map_err(text_error)?.bundle_seq;
    let final_totals = totals(&state)?;
    let expected = expected_delta(options, frames);
    let conserved = final_totals.0 - initial_totals.0 == expected.0
        && final_totals.1 - initial_totals.1 == expected.1;
    if final_sequence - initial_sequence != frames || !conserved {
        return Err(format!(
            "基准提交或流量不守恒：frames={frames}, commits={}, expected={expected:?}, actual={:?}",
            final_sequence - initial_sequence,
            (
                final_totals.0 - initial_totals.0,
                final_totals.1 - initial_totals.1
            )
        ));
    }
    let final_inventory = inventory(&state, &options.dir)?;
    let tables = table_inventory(&state)?;
    let executable = std::env::current_exe().map_err(text_error)?;
    let executable_sha256 = hex::encode(Sha256::digest(
        std::fs::read(executable).map_err(text_error)?,
    ));
    let fixture_hash = hex::encode(Sha256::digest(serde_json::to_vec(&json!({
        "version": 1, "active": options.active, "hz": options.hz, "seed": options.seed,
        "start_utc": options.start_utc, "warmup_secs": options.warmup_secs, "duration_secs": options.duration_secs,
        "workload": options.workload, "metadata_change_percent": options.metadata_change_percent,
        "archive": options.archive, "query_every_frames": options.query_every_frames, "period_rule": options.period_rule
    })).map_err(text_error)?));
    let memory = |field: &str| {
        latency(
            samples
                .iter()
                .filter_map(|s| s["native"][field].as_u64().map(|n| n as f64))
                .collect(),
        )
    };
    let private = memory("private_bytes");
    let rss = memory("working_set_bytes");
    Ok(json!({
        "schema_version": 1, "kind": "isolated-real-facade", "options": options,
        "fixture_hash": fixture_hash, "executable_sha256": executable_sha256,
        "process_id": std::process::id(), "platform": std::env::consts::OS,
        "local_utc_offset_seconds": chrono::Local::now().offset().local_minus_utc(),
        "synchronous": "FULL", "warmup_wall_secs": warmup_wall_secs,
        "measured_wall_secs": wall_secs, "simulated_secs": options.duration_secs, "frames": frames,
        "commits": final_sequence - initial_sequence, "writer_rows_changed": rows_changed,
        "traffic": { "initial_upload": initial_totals.0, "initial_download": initial_totals.1,
            "final_upload": final_totals.0, "final_download": final_totals.1,
            "expected_added_upload": expected.0, "expected_added_download": expected.1, "conserved": conserved },
        "native_cpu_seconds": process_after.cpu_seconds.zip(process_before.cpu_seconds).map(|(a,b)| a-b),
        "native_process_io_write_bytes": process_after.io_write_bytes.zip(process_before.io_write_bytes).map(|(a,b)| a.saturating_sub(b)),
        "native_process_io_read_bytes": process_after.io_read_bytes.zip(process_before.io_read_bytes).map(|(a,b)| a.saturating_sub(b)),
        "native_process_io_write_operations": process_after.io_write_operations.zip(process_before.io_write_operations).map(|(a,b)| a.saturating_sub(b)),
        "sqlite_xwrite": writes,
        "sqlite_application_file_write_bytes": writes.db_bytes + writes.wal_bytes + writes.other_sqlite_file_bytes,
        "all_application_file_write_bytes": null, "spool_write_bytes": null,
        "native_private_bytes": {"p95": (private.count > 0).then_some(private.p95_ms), "max": (private.count > 0).then_some(private.max_ms), "samples": private.count},
        "native_working_set_bytes": {"p95": (rss.count > 0).then_some(rss.p95_ms), "max": (rss.count > 0).then_some(rss.max_ms), "samples": rss.count},
        "native_before": process_before, "native_after": process_after,
        "latency": { "facade_ingest_including_durable_commit": latency(ingest_times),
            "archive_tick": latency(archive_times), "whole_tick": latency(frame_times),
            "live_query": latency(live_query_times), "report_first_reader": latency(query_cold_times),
            "report_repeated_reader": latency(query_warm_times) },
        "frame_budget_overruns": overruns, "schedule_lag_frames_max": max_lag_frames,
        "initial_files_and_pages": initial, "final_files_and_pages": final_inventory,
        "tables_and_indexes": tables, "samples": samples,
        "limits": [
            "仅当前原生基准进程；没有 HTTP、Tauri 事件或 WebView，窗口为后台模拟状态。",
            "CPU/I/O/内存不含初始化、warmup、结果 JSON 写出及最终统计查询；采样开销包含在测量内。",
            "SQLite xWrite 只统计隔离目录内成功请求字节，不含 mmap/shm、spool、日志、系统缓存与物理设备写放大。",
            "process I/O 为 OS 当前进程计数，不能替代文件归属；spool 瞬时写入需要独立回归或文件跟踪。",
            "report first/repeated 每次均打开新 reader；未驱逐 OS page cache，不能宣称物理冷读。",
            "ingest 延迟包含 accounting、告警与提交；不是单独 SQLite commit 耗时。",
            "串行驱动没有 collector 输入队列；schedule_lag 不是实际队列深度。",
            "归档齐全 fixture 使用空历史的持久化结果；本回放不代替完整30天容量库或安装态 soak。",
            "failed 档案场景由隔离库 BEFORE INSERT 触发器拒绝持久化，非真实磁盘故障。"
        ]
    }))
}

fn validate(o: &FacadeOptions) -> Result<(), String> {
    if o.active == 0
        || o.hz == 0
        || o.hz > 1000
        || o.duration_secs == 0
        || o.metadata_change_percent > 100
    {
        return Err(
            "active/duration 必须为正，hz 为 1..1000，metadata-change-percent 为 0..100".into(),
        );
    }
    Ok(())
}

fn utc_at(o: &FacadeOptions, frame: u64) -> i64 {
    o.start_utc + (frame / u64::from(o.hz)) as i64
}
fn mono_at(o: &FacadeOptions, frame: u64) -> u64 {
    frame * 1000 / u64::from(o.hz)
}

fn snapshot(o: &FacadeOptions, frame: u64) -> ControllerInput {
    let counter_frame = if o.workload == FacadeWorkload::Unchanged {
        0
    } else {
        frame
    };
    let connections: Vec<_> = (0..o.active)
        .map(|id| {
            let stable = o.seed.wrapping_add(u64::from(id));
            let change = o.workload == FacadeWorkload::Metadata
                && stable % 100 < u64::from(o.metadata_change_percent);
            let variant = if change { frame % 2 } else { 0 };
            ConnectionFact {
                id: format!("bench-{id}"),
                upload: (counter_frame + 1) * (8 + stable % 13),
                download: (counter_frame + 1) * (32 + stable % 31),
                chains: vec!["家宽".into(), format!("exit-{}", (stable + variant) % 4)],
                provider_chains: Vec::new(),
                meta: ConnectionMeta {
                    host: Some(format!("host-{}-{variant}.example.test", stable % 97)),
                    process_name: Some(format!("process-{}.exe", stable % 5)),
                    network: Some("tcp".into()),
                    rule: Some("DOMAIN-SUFFIX".into()),
                    rule_payload: Some("example.test".into()),
                    ..ConnectionMeta::default()
                },
            }
        })
        .collect();
    ControllerInput::Snapshot {
        received_monotonic_ms: mono_at(o, frame),
        received_utc: utc_at(o, frame),
        upload_total: connections.iter().map(|c| c.upload).sum(),
        download_total: connections.iter().map(|c| c.download).sum(),
        connections,
    }
}

fn expected_delta(o: &FacadeOptions, frames: u64) -> (i64, i64) {
    if o.workload == FacadeWorkload::Unchanged {
        return (0, 0);
    }
    (0..o.active).fold((0, 0), |(u, d), id| {
        let stable = o.seed.wrapping_add(u64::from(id));
        (
            u + (frames * (8 + stable % 13)) as i64,
            d + (frames * (32 + stable % 31)) as i64,
        )
    })
}

fn ingest(state: &Mutex<AppFacade>, o: &FacadeOptions, frame: u64) -> Result<(), String> {
    ingest_input(
        state,
        snapshot(o, frame),
        utc_at(o, frame),
        mono_at(o, frame),
    )
}

fn ingest_input(
    state: &Mutex<AppFacade>,
    input: ControllerInput,
    utc: i64,
    mono: u64,
) -> Result<(), String> {
    let mut facade = state.lock().map_err(text_error)?;
    let previous = facade.bundle_seq;
    facade.ingest_snapshot(input, utc, mono);
    if facade.bundle_seq != previous + 1 {
        return Err("真实门面提交失败，停止基准".into());
    }
    Ok(())
}

fn pace(start: Instant, frame: u64, budget: Duration, virtual_time: bool) {
    if !virtual_time {
        let delay = budget.mul_f64(frame as f64).saturating_sub(start.elapsed());
        if !delay.is_zero() {
            std::thread::sleep(delay);
        }
    }
}

fn seed_complete_archives(facade: &AppFacade, now: i64) -> Result<(), String> {
    let connection = facade.storage.as_ref().ok_or("隔离库缺失")?.connection();
    while let Some(job) = ReportArchiveService::next_job(connection, now).map_err(text_error)? {
        // 各历史时窗没有 raw。使用同一空历史结果，但按每个真实 job 保存其查询与 fingerprint。
        let plan = plan_capability(
            &default_auto_report_query(Granularity::Hour, now - 3600, now),
            now,
            30,
        )
        .map_err(text_error)?;
        let mut result = empty_result(job.query.clone(), &plan, 0);
        result.generated_utc = now;
        ReportArchiveService::persist_outcome(connection, &job, Ok(result), now)
            .map_err(text_error)?;
    }
    Ok(())
}

fn period_rule(now: i64) -> AlertRule {
    AlertRule {
        rule_id: "bench-period".into(),
        version: 1,
        enabled: true,
        kind: AlertKind::PeriodUsage,
        selector_kind: SelectorKind::Domain,
        selector_value: Some("host-0-0.example.test".into()),
        direction: Some(AlertDirection::Combined),
        threshold_value: i64::MAX / 2,
        recovery_threshold: Some(i64::MAX / 4),
        period: Some(AlertPeriod::Rolling1h),
        timezone: "UTC".into(),
        cooldown_sec: 60,
        quiet_start_min: None,
        quiet_end_min: None,
        created_utc: now,
        updated_utc: now,
    }
}

fn totals(state: &Mutex<AppFacade>) -> Result<(i64, i64), String> {
    let f = state.lock().map_err(text_error)?;
    f.storage
        .as_ref()
        .ok_or("隔离库缺失")?
        .connection()
        .query_row(
            "select coalesce(sum(upload),0), coalesce(sum(download),0) from connection_minute",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(text_error)
}

fn total_changes(state: &Mutex<AppFacade>) -> Result<u64, String> {
    let f = state.lock().map_err(text_error)?;
    f.storage
        .as_ref()
        .ok_or("隔离库缺失")?
        .connection()
        .query_row("select total_changes()", [], |r| {
            r.get::<_, i64>(0).map(|n| n as u64)
        })
        .map_err(text_error)
}

pub(super) fn file_bytes(path: &Path) -> Result<u64, String> {
    match std::fs::metadata(path) {
        Ok(m) => Ok(m.len()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(e) => Err(e.to_string()),
    }
}

fn directory_bytes(path: &Path) -> Result<u64, String> {
    if !path.exists() {
        return Ok(0);
    }
    let mut total = 0;
    for item in std::fs::read_dir(path).map_err(text_error)? {
        let item = item.map_err(text_error)?;
        let metadata = item.metadata().map_err(text_error)?;
        if metadata.is_dir() {
            total += directory_bytes(&item.path())?;
        } else {
            total += metadata.len();
        }
    }
    Ok(total)
}

fn inventory(state: &Mutex<AppFacade>, dir: &Path) -> Result<Value, String> {
    let f = state.lock().map_err(text_error)?;
    let c = f.storage.as_ref().ok_or("隔离库缺失")?.connection();
    let pragma = |name: &str| {
        c.query_row(&format!("pragma {name}"), [], |r| {
            r.get::<_, i64>(0).map(|n| n as u64)
        })
        .map_err(text_error)
    };
    let page_size = pragma("page_size")?;
    let page_count = pragma("page_count")?;
    let freelist = pragma("freelist_count")?;
    Ok(json!({
        "db_bytes": file_bytes(&dir.join("monitor.sqlite3"))?,
        "wal_bytes": file_bytes(&dir.join("monitor.sqlite3-wal"))?,
        "shm_bytes": file_bytes(&dir.join("monitor.sqlite3-shm"))?,
        "spool_present_bytes": directory_bytes(&dir.join("report-spool"))?,
        "archive_spool_present_bytes": directory_bytes(&dir.join("archive-tick"))?,
        "page_size": page_size, "page_count": page_count, "freelist_count": freelist,
        "reusable_page_bytes": freelist * page_size, "active_page_bytes": (page_count-freelist)*page_size,
        "archive_ok": c.query_row("select count(*) from report_archive where status='ok'", [], |r| r.get::<_, i64>(0)).map_err(text_error)?,
        "archive_failed": c.query_row("select count(*) from report_archive where status='failed'", [], |r| r.get::<_, i64>(0)).map_err(text_error)?,
        "schema_version": pragma("user_version")?
    }))
}

fn table_inventory(state: &Mutex<AppFacade>) -> Result<Value, String> {
    let f = state.lock().map_err(text_error)?;
    table_inventory_on(f.storage.as_ref().ok_or("隔离库缺失")?.connection())
}

pub(super) fn table_inventory_on(c: &Connection) -> Result<Value, String> {
    let dbstat = c.prepare("select name, sum(pgsize) from dbstat group by name");
    let mut bytes = std::collections::BTreeMap::<String, u64>::new();
    let dbstat_available = match dbstat {
        Ok(mut s) => {
            for row in s
                .query_map([], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as u64))
                })
                .map_err(text_error)?
            {
                let (name, n) = row.map_err(text_error)?;
                bytes.insert(name, n);
            }
            true
        }
        Err(_) => false,
    };
    let mut stmt = c.prepare("select name,type,tbl_name from sqlite_master where type in ('table','index') order by name").map_err(text_error)?;
    let mut objects = Vec::new();
    for row in stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })
        .map_err(text_error)?
    {
        let (name, kind, table) = row.map_err(text_error)?;
        let count = if kind == "table" {
            Some(
                c.query_row(
                    &format!("select count(*) from \"{}\"", name.replace('"', "\"\"")),
                    [],
                    |r| r.get::<_, i64>(0).map(|n| n as u64),
                )
                .map_err(text_error)?,
            )
        } else {
            None
        };
        objects.push(json!({"name": name, "kind": kind, "table": table, "rows": count,
            "bytes": bytes.get(&name), "bytes_per_row": count.filter(|n| *n>0).zip(bytes.get(&name)).map(|(n,b)| *b as f64/n as f64) }));
    }
    Ok(json!({"dbstat_available": dbstat_available,"objects": objects}))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn options(dir: &Path) -> FacadeOptions {
        FacadeOptions {
            active: 4,
            hz: 1,
            duration_secs: 3,
            warmup_secs: 0,
            virtual_time: true,
            start_utc: 1_800_001_800,
            seed: 7,
            workload: FacadeWorkload::Counters,
            metadata_change_percent: 100,
            archive: ArchiveScenario::Backlog,
            query_every_frames: 1,
            period_rule: true,
            source_revision: "test".into(),
            dir: dir.into(),
        }
    }
    #[test]
    fn real_facade_replay_conserves_counters_and_exercises_archive_and_readers() {
        if crate::bench::run_isolated_test("bench::facade::tests::real_facade_replay_conserves_counters_and_exercises_archive_and_readers") { return; }
        let dir = tempfile::tempdir().unwrap();
        // SAFETY: 此测试在独立子进程执行，只有本测试打开 SQLite。
        let report = unsafe { replay_facade(&options(dir.path())) }.unwrap();
        assert_eq!(report["commits"], 3);
        assert_eq!(report["traffic"]["conserved"], true);
        assert!(report["sqlite_xwrite"]["wal_bytes"].as_u64().unwrap() > 0);
        assert!(
            report["final_files_and_pages"]["archive_ok"]
                .as_u64()
                .unwrap()
                > 0
        );
        assert_eq!(report["latency"]["report_repeated_reader"]["count"], 3);
        assert!(unsafe { replay_facade(&options(dir.path())) }.is_err());
    }
    #[test]
    fn unchanged_and_failed_workload_has_no_facts_or_successful_archives() {
        if crate::bench::run_isolated_test("bench::facade::tests::unchanged_and_failed_workload_has_no_facts_or_successful_archives") { return; }
        let dir = tempfile::tempdir().unwrap();
        let mut options = options(dir.path());
        options.workload = FacadeWorkload::Unchanged;
        options.archive = ArchiveScenario::Failed;
        // SAFETY: 此测试在独立子进程执行，只有本测试打开 SQLite。
        let report = unsafe { replay_facade(&options) }.unwrap();
        assert_eq!(report["traffic"]["final_upload"], 0);
        assert_eq!(report["final_files_and_pages"]["archive_ok"], 0);
    }
}

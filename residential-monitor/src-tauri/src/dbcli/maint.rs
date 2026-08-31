//! 维护子命令：包装既有服务，不新写业务 SQL。

use super::envelope::{
    CapabilityEcho, CoverageEcho, Envelope, TruncationEcho, WindowEcho, SCHEMA_VERSION,
};
use super::{map_report, open_reader, DbCliError};
use crate::c0_contract::{BUSY_TIMEOUT_MS, SCHEMA_VERSION as KNOWN_SCHEMA};
use crate::c3::backup::BackupRestoreService;
use crate::c3::query::{AUTO_DELETE_ENABLED, RAW_RETAIN_DAYS_DEFAULT};
use crate::c3::retention::{RetentionMode, RetentionService};
use crate::c3::space::SpaceBudget;
use crate::c5::{confirm_delete, preview_delete, run_user_vacuum};
use crate::identity::DELETE_CONFIRM_PHRASE;
use crate::storage::StorageCoordinator;
use rusqlite::ErrorCode;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

const VACUUM_LONG_BYTES: u64 = 1 << 30;
const OFFLINE_NOTE: &str = "CLI 不验证 ResiWatch 是否已退出，该前置条件由执行者保证。";

#[derive(Debug, Clone)]
pub struct MaintFlags {
    pub confirm: bool,
    pub offline_confirmed: bool,
    pub allow_long: bool,
    pub phrase: Option<String>,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PreviewItem {
    id: String,
    path: String,
    bytes: Option<u64>,
    byte_basis: String,
    extra: serde_json::Value,
}

pub fn run_status(db: &Path, now_utc: i64) -> Result<(Envelope, i32), DbCliError> {
    let reader = open_reader(db)?;
    let user_version: i32 = reader
        .query_row("pragma user_version", [], |row| row.get(0))
        .map_err(|_| DbCliError::FailClosed("读取 user_version 失败".into()))?;
    let page_size: i64 = reader
        .query_row("pragma page_size", [], |row| row.get(0))
        .unwrap_or(0);
    let freelist: i64 = reader
        .query_row("pragma freelist_count", [], |row| row.get(0))
        .unwrap_or(0);
    let watermarks = load_watermarks(&reader);
    drop(reader);
    let db_bytes = file_len(db);
    let wal_bytes = file_len(&sidecar(db, "-wal"));
    let result = serde_json::json!({
        "userVersion": user_version,
        "knownSchemaVersion": KNOWN_SCHEMA,
        "dbBytes": db_bytes,
        "walBytes": wal_bytes,
        "freelistCount": freelist,
        "pageSize": page_size,
        "reclaimableUpperBoundBytes": freelist.saturating_mul(page_size),
        "retentionWatermark": watermarks,
        "autoDeleteEnabled": AUTO_DELETE_ENABLED,
        "spaceBudget": "host",
    });
    Ok((
        maint_envelope(
            "maint.status",
            now_utc,
            result,
            vec!["status 只读。".into()],
        ),
        0,
    ))
}

pub fn run_retention(
    db: &Path,
    flags: &MaintFlags,
    now_utc: i64,
    cancel: &Arc<AtomicBool>,
) -> Result<(Envelope, i32), DbCliError> {
    let mut coordinator = acquire_exclusive(db)?;
    let preview =
        RetentionService::preview(&coordinator, RAW_RETAIN_DAYS_DEFAULT).map_err(map_report)?;
    let items = vec![PreviewItem {
        id: "retention".into(),
        path: db.display().to_string(),
        bytes: None,
        byte_basis: "unknown".into(),
        extra: serde_json::json!({
            "rawRows": preview.raw_rows,
            "hourlyRows": preview.hourly_rows,
            "dailyDimRows": preview.daily_dim_rows,
            "dailyCoreRows": preview.daily_core_rows,
        }),
    }];
    let mut notes = vec![
        format!("auto_delete_enabled={AUTO_DELETE_ENABLED}"),
        preview.note_zh.clone(),
        "过期 DELETE 处于关闭状态。".into(),
    ];
    if !flags.confirm {
        notes.push("缺少 --confirm，只预览不写库。".into());
        return Ok((
            maint_envelope(
                "maint.retention",
                now_utc,
                serde_json::json!({ "preview": true, "items": items, "autoDeleteEnabled": AUTO_DELETE_ENABLED }),
                notes,
            ),
            0,
        ));
    }
    let ran = RetentionService::run(
        &mut coordinator,
        now_utc,
        RAW_RETAIN_DAYS_DEFAULT,
        RetentionMode::MaterializeOnly,
        &SpaceBudget::default(),
        cancel,
    )
    .map_err(map_report)?;
    Ok((
        maint_envelope(
            "maint.retention",
            now_utc,
            serde_json::to_value(&ran).unwrap_or(serde_json::Value::Null),
            notes,
        ),
        0,
    ))
}

pub fn run_backup(
    db: &Path,
    flags: &MaintFlags,
    now_utc: i64,
    cancel: &Arc<AtomicBool>,
) -> Result<(Envelope, i32), DbCliError> {
    let dest = flags
        .path
        .clone()
        .ok_or_else(|| DbCliError::InvalidArgs("backup 需要 --path".into()))?;
    let db_bytes = file_len(db).unwrap_or(0);
    let wal_bytes = file_len(&sidecar(db, "-wal")).unwrap_or(0);
    let estimated = db_bytes.saturating_add(wal_bytes);
    let items = vec![PreviewItem {
        id: "backup-dest".into(),
        path: dest.display().to_string(),
        bytes: Some(estimated),
        byte_basis: "upperBound".into(),
        extra: serde_json::json!({ "dbBytes": db_bytes, "walBytes": wal_bytes }),
    }];
    if !flags.confirm {
        return Ok((
            maint_envelope(
                "maint.backup",
                now_utc,
                serde_json::json!({ "preview": true, "items": items }),
                vec!["缺少 --confirm，只预览不写备份文件。".into()],
            ),
            0,
        ));
    }
    let manifest =
        BackupRestoreService::create_backup(db, &dest, &SpaceBudget::default(), cancel, now_utc)
            .map_err(map_report)?;
    Ok((
        maint_envelope(
            "maint.backup",
            now_utc,
            serde_json::to_value(manifest).unwrap_or(serde_json::Value::Null),
            vec!["备份只写目标文件，不改当前库。".into()],
        ),
        0,
    ))
}

pub fn run_restore(
    db: &Path,
    flags: &MaintFlags,
    now_utc: i64,
    cancel: &Arc<AtomicBool>,
) -> Result<(Envelope, i32), DbCliError> {
    require_offline(flags)?;
    let candidate = flags
        .path
        .clone()
        .ok_or_else(|| DbCliError::InvalidArgs("restore 需要 --path".into()))?;
    let items = vec![
        sized_item("candidate", &candidate, "exact"),
        sized_item("live", db, "exact"),
        sized_item("live-wal", &sidecar(db, "-wal"), "exact"),
        sized_item("live-shm", &sidecar(db, "-shm"), "exact"),
    ];
    if !flags.confirm {
        return Ok((
            maint_envelope(
                "maint.restore",
                now_utc,
                serde_json::json!({ "preview": true, "items": items }),
                vec!["缺少 --confirm，只预览不写库。".into(), OFFLINE_NOTE.into()],
            ),
            0,
        ));
    }
    let _guard = acquire_exclusive(db)?;
    drop(_guard);
    BackupRestoreService::restore(db, &candidate, &SpaceBudget::default(), cancel)
        .map_err(map_report)?;
    Ok((
        maint_envelope(
            "maint.restore",
            now_utc,
            serde_json::json!({ "ok": true, "items": items }),
            vec![OFFLINE_NOTE.into()],
        ),
        0,
    ))
}

pub fn run_vacuum(
    db: &Path,
    flags: &MaintFlags,
    now_utc: i64,
) -> Result<(Envelope, i32), DbCliError> {
    require_offline(flags)?;
    let db_bytes = file_len(db).unwrap_or(0);
    if db_bytes > VACUUM_LONG_BYTES && !flags.allow_long {
        return Err(DbCliError::FailClosed(format!(
            "主库 {db_bytes} 字节超过 {}，VACUUM 需要 --allow-long",
            VACUUM_LONG_BYTES
        )));
    }
    let reader = open_reader(db)?;
    let page_size: i64 = reader
        .query_row("pragma page_size", [], |row| row.get(0))
        .unwrap_or(0);
    let freelist: i64 = reader
        .query_row("pragma freelist_count", [], |row| row.get(0))
        .unwrap_or(0);
    drop(reader);
    let items = vec![PreviewItem {
        id: "vacuum".into(),
        path: db.display().to_string(),
        bytes: Some(freelist.saturating_mul(page_size) as u64),
        byte_basis: "reclaimableUpperBound".into(),
        extra: serde_json::json!({ "dbBytes": db_bytes }),
    }];
    let mut notes = vec![
        OFFLINE_NOTE.into(),
        "vacuum 不可中断。中途终止进程后原库应仍可用；用 integrity_check 确认，失败则从备份恢复。"
            .into(),
        format!("预计时长随主库大小变化，当前 {db_bytes} 字节。"),
    ];
    if !flags.confirm {
        notes.push("缺少 --confirm，只预览不写库。".into());
        return Ok((
            maint_envelope(
                "maint.vacuum",
                now_utc,
                serde_json::json!({ "preview": true, "items": items, "interruptible": false }),
                notes,
            ),
            0,
        ));
    }
    let _guard = acquire_exclusive(db)?;
    drop(_guard);
    run_user_vacuum(db, &SpaceBudget::default()).map_err(map_report)?;
    Ok((
        maint_envelope(
            "maint.vacuum",
            now_utc,
            serde_json::json!({ "ok": true, "items": items, "interruptible": false }),
            notes,
        ),
        0,
    ))
}

pub fn run_purge(
    db: &Path,
    flags: &MaintFlags,
    now_utc: i64,
) -> Result<(Envelope, i32), DbCliError> {
    require_offline(flags)?;
    let data_dir = db.parent().unwrap_or(db);
    let log_dir = crate::app_log::resolve_dir();
    let preview = preview_delete(data_dir, &log_dir);
    let items: Vec<PreviewItem> = preview
        .items
        .iter()
        .map(|item| {
            let path = PathBuf::from(&item.path);
            let bytes = if item.exists { path_bytes(&path) } else { None };
            PreviewItem {
                id: item.id.clone(),
                path: item.path.clone(),
                bytes,
                byte_basis: if item.exists { "exact" } else { "unknown" }.into(),
                extra: serde_json::json!({
                    "kind": item.kind,
                    "exists": item.exists,
                    "noteZh": item.note_zh,
                }),
            }
        })
        .collect();
    let mut notes = vec![
        OFFLINE_NOTE.into(),
        "purge 不可中断、无回滚。失败后按 DeleteReport 分项确认哪些对象已删除。".into(),
        format!("确认短语: {DELETE_CONFIRM_PHRASE}"),
    ];
    if !flags.confirm {
        notes.push("缺少 --confirm，只预览不删除。".into());
        return Ok((
            maint_envelope(
                "maint.purge",
                now_utc,
                serde_json::json!({
                    "preview": true,
                    "items": items,
                    "interruptible": false,
                    "confirmPhrase": DELETE_CONFIRM_PHRASE
                }),
                notes,
            ),
            0,
        ));
    }
    let phrase = flags
        .phrase
        .as_deref()
        .ok_or_else(|| DbCliError::InvalidArgs("purge 需要 --phrase".into()))?;
    let _guard = acquire_exclusive(db)?;
    drop(_guard);
    let report =
        confirm_delete(data_dir, &log_dir, phrase, || Ok(())).map_err(DbCliError::FailClosed)?;
    Ok((
        maint_envelope(
            "maint.purge",
            now_utc,
            serde_json::to_value(&report).unwrap_or(serde_json::Value::Null),
            notes,
        ),
        0,
    ))
}

fn require_offline(flags: &MaintFlags) -> Result<(), DbCliError> {
    if flags.offline_confirmed {
        Ok(())
    } else {
        Err(DbCliError::FailClosed(
            "restore / vacuum / purge 需要 --offline-confirmed。CLI 不验证 ResiWatch 是否已退出。"
                .into(),
        ))
    }
}

pub(crate) fn acquire_exclusive(path: &Path) -> Result<StorageCoordinator, DbCliError> {
    let mut coordinator = StorageCoordinator::open(path).map_err(|error| {
        if error.to_string().contains("future schema") {
            DbCliError::FailClosed("schema user_version 高于本二进制已知版本".into())
        } else {
            DbCliError::FailClosed(error.to_string())
        }
    })?;
    let conn = coordinator.connection_mut();
    conn.busy_timeout(Duration::from_millis(0))
        .map_err(|_| DbCliError::FailClosed("设置 busy_timeout 失败".into()))?;
    conn.execute_batch("PRAGMA locking_mode = EXCLUSIVE")
        .map_err(|_| DbCliError::FailClosed("设置独占锁模式失败".into()))?;
    match conn.execute_batch("BEGIN IMMEDIATE; COMMIT;") {
        Ok(()) => {
            let _ = conn.busy_timeout(Duration::from_millis(BUSY_TIMEOUT_MS as u64));
            Ok(coordinator)
        }
        Err(rusqlite::Error::SqliteFailure(code, _))
            if code.code == ErrorCode::DatabaseBusy || code.code == ErrorCode::DatabaseLocked =>
        {
            Err(DbCliError::Busy("独占锁冲突".into()))
        }
        Err(error) => Err(DbCliError::FailClosed(error.to_string())),
    }
}

fn load_watermarks(connection: &rusqlite::Connection) -> serde_json::Value {
    let mut statement = match connection
        .prepare("select layer, watermark from retention_watermark order by layer")
    {
        Ok(statement) => statement,
        Err(_) => return serde_json::json!([]),
    };
    let rows = statement.query_map([], |row| {
        Ok(serde_json::json!({
            "layer": row.get::<_, String>(0)?,
            "watermark": row.get::<_, i64>(1)?,
        }))
    });
    match rows {
        Ok(rows) => serde_json::Value::Array(rows.filter_map(|row| row.ok()).collect()),
        Err(_) => serde_json::json!([]),
    }
}

fn file_len(path: &Path) -> Option<u64> {
    std::fs::metadata(path).ok().map(|meta| meta.len())
}

fn path_bytes(path: &Path) -> Option<u64> {
    if path.is_file() {
        file_len(path)
    } else if path.is_dir() {
        walkdir_len(path)
    } else {
        None
    }
}

fn walkdir_len(path: &Path) -> Option<u64> {
    let mut total = 0u64;
    let entries = std::fs::read_dir(path).ok()?;
    for entry in entries.flatten() {
        let child = entry.path();
        if child.is_dir() {
            total = total.saturating_add(walkdir_len(&child).unwrap_or(0));
        } else {
            total = total.saturating_add(file_len(&child).unwrap_or(0));
        }
    }
    Some(total)
}

fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut os = path.as_os_str().to_os_string();
    os.push(suffix);
    PathBuf::from(os)
}

fn sized_item(id: &str, path: &Path, basis: &str) -> PreviewItem {
    PreviewItem {
        id: id.into(),
        path: path.display().to_string(),
        bytes: file_len(path),
        byte_basis: basis.into(),
        extra: serde_json::json!({ "exists": path.exists() }),
    }
}

fn maint_envelope(
    command: &str,
    now_utc: i64,
    result: serde_json::Value,
    notes: Vec<String>,
) -> Envelope {
    Envelope {
        schema_version: SCHEMA_VERSION,
        command: command.into(),
        generated_utc: now_utc,
        window: WindowEcho {
            start_utc: 0,
            end_utc: 0,
            timezone: "UTC".into(),
            granularity: "none".into(),
        },
        data_version: 0,
        capability: CapabilityEcho {
            layer: "raw".into(),
            supported: true,
            reason: None,
        },
        coverage: CoverageEcho {
            observed_sec: None,
            gap_sec: None,
            status: "n/a".into(),
        },
        attribution_quality: super::envelope::AttributionEcho {
            status: "n/a".into(),
            known_bytes: None,
            missing_bytes: None,
        },
        truncation: TruncationEcho {
            status: "complete".into(),
            row_cap: 0,
            rows: 0,
        },
        named_sql: Vec::new(),
        result,
        notes,
    }
}

//! 本机文件日志。不读 SQLite，不桥接第三方 log 记录。

use crate::identity::IDENTIFIER;
use crate::redact::scan_text_for_secrets;
use serde_json::{Map, Value};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

pub const ENV_LOG_DIR: &str = "RESIDENTIAL_MONITOR_LOG_DIR";
pub const ENV_DATA_DIR: &str = "RESIDENTIAL_MONITOR_DATA_DIR";
pub const FILE_NAME: &str = "residential-monitor.log";
pub const DEFAULT_MAX_BYTES: u64 = 2 * 1024 * 1024;
const MAX_ROTATED: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Info,
    Warn,
    Error,
}

impl Level {
    fn as_str(self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

struct Inner {
    dir: PathBuf,
    max_bytes: u64,
    file: Option<File>,
}

static STATE: Mutex<Option<Inner>> = Mutex::new(None);
static PANIC_HOOK: OnceLock<()> = OnceLock::new();

#[cfg(test)]
static TEST: Mutex<()> = Mutex::new(());

#[cfg(test)]
pub fn exclusive_test() -> std::sync::MutexGuard<'static, ()> {
    TEST.lock().unwrap_or_else(|poison| poison.into_inner())
}

#[cfg(test)]
pub fn reset_for_test() {
    if let Ok(mut guard) = STATE.lock() {
        *guard = None;
    }
}

pub fn resolve_dir() -> PathBuf {
    resolve_dir_from(
        std::env::var_os(ENV_LOG_DIR).map(PathBuf::from),
        std::env::var_os(ENV_DATA_DIR).map(PathBuf::from),
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from),
    )
}

pub fn resolve_dir_from(
    log_override: Option<PathBuf>,
    _data_dir: Option<PathBuf>,
    local_app_data: Option<PathBuf>,
) -> PathBuf {
    if let Some(path) = log_override {
        return path;
    }
    let root = local_app_data.unwrap_or_else(std::env::temp_dir);
    root.join(IDENTIFIER).join("logs")
}

pub fn init() {
    init_at(resolve_dir(), DEFAULT_MAX_BYTES);
}

pub fn init_at(dir: PathBuf, max_bytes: u64) {
    let _ = fs::create_dir_all(&dir);
    let file = open_current(&dir);
    if let Ok(mut guard) = STATE.lock() {
        *guard = Some(Inner {
            dir,
            max_bytes,
            file,
        });
    }
    install_panic_hook();
}

pub fn dir() -> PathBuf {
    STATE
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().map(|inner| inner.dir.clone()))
        .unwrap_or_else(resolve_dir)
}

pub fn emit(level: Level, event: &str, fields: Value) {
    let line = format_line(level, event, fields);
    if cfg!(debug_assertions) {
        eprintln!("{line}");
    }
    let mut guard = match STATE.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    let Some(inner) = guard.as_mut() else {
        return;
    };
    maybe_rotate(inner);
    if write_line(inner, &line).is_err() {
        let _ = fs::create_dir_all(&inner.dir);
        inner.file = open_current(&inner.dir);
        let _ = write_line(inner, &line);
    }
}

pub fn open_in_explorer() -> Result<PathBuf, &'static str> {
    let dir = dir();
    let _ = fs::create_dir_all(&dir);
    let opened = spawn_explorer(&dir);
    emit(
        if opened { Level::Info } else { Level::Error },
        "open_log_dir",
        serde_json::json!({ "ok": opened }),
    );
    if opened {
        Ok(dir)
    } else {
        Err("explorer")
    }
}

fn spawn_explorer(dir: &Path) -> bool {
    #[cfg(windows)]
    {
        std::process::Command::new("explorer")
            .arg(dir)
            .spawn()
            .is_ok()
    }
    #[cfg(not(windows))]
    {
        dir.exists() || fs::create_dir_all(dir).is_ok()
    }
}

fn format_line(level: Level, event: &str, fields: Value) -> String {
    let json = sanitize_fields(fields);
    let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    format!("{} {} {event} {json}", ts, level.as_str())
}

fn sanitize_fields(fields: Value) -> String {
    let mut map = match fields {
        Value::Object(map) => map,
        _ => Map::new(),
    };
    for value in map.values_mut() {
        match value {
            Value::String(text) if scan_text_for_secrets(text) => {
                *text = "<redacted>".into();
            }
            Value::Number(_) | Value::Bool(_) | Value::String(_) | Value::Null => {}
            _ => *value = Value::Null,
        }
    }
    let encoded = Value::Object(map).to_string();
    if scan_text_for_secrets(&encoded) {
        "{}".into()
    } else {
        encoded
    }
}

fn open_current(dir: &Path) -> Option<File> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(FILE_NAME))
        .ok()
}

fn current_len(inner: &Inner) -> u64 {
    inner
        .file
        .as_ref()
        .and_then(|file| file.metadata().ok())
        .map(|meta| meta.len())
        .unwrap_or_else(|| {
            fs::metadata(inner.dir.join(FILE_NAME))
                .map(|meta| meta.len())
                .unwrap_or(0)
        })
}

fn maybe_rotate(inner: &mut Inner) {
    if current_len(inner) < inner.max_bytes {
        return;
    }
    inner.file = None;
    let oldest = inner.dir.join(format!("{FILE_NAME}.{MAX_ROTATED}"));
    let _ = fs::remove_file(&oldest);
    for index in (1..MAX_ROTATED).rev() {
        let from = inner.dir.join(format!("{FILE_NAME}.{index}"));
        let to = inner.dir.join(format!("{FILE_NAME}.{}", index + 1));
        let _ = fs::rename(&from, &to);
    }
    let _ = fs::rename(
        inner.dir.join(FILE_NAME),
        inner.dir.join(format!("{FILE_NAME}.1")),
    );
    inner.file = open_current(&inner.dir);
}

fn write_line(inner: &mut Inner, line: &str) -> std::io::Result<()> {
    let file = match inner.file.as_mut() {
        Some(file) => file,
        None => {
            inner.file = open_current(&inner.dir);
            inner
                .file
                .as_mut()
                .ok_or_else(|| std::io::Error::other("log file missing"))?
        }
    };
    writeln!(file, "{line}")?;
    Ok(())
}

fn install_panic_hook() {
    PANIC_HOOK.get_or_init(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let payload = info.to_string();
            let clipped: String = payload.chars().take(180).collect();
            emit(
                Level::Error,
                "panic",
                serde_json::json!({ "class": "panic", "note": clipped }),
            );
            previous(info);
        }));
    });
}

#[cfg(test)]
mod app_log_tests {
    use super::*;
    use tempfile::tempdir;

    fn with_log<R>(max_bytes: u64, body: impl FnOnce(&Path) -> R) -> R {
        let _lock = exclusive_test();
        let dir = tempdir().expect("dir");
        init_at(dir.path().to_path_buf(), max_bytes);
        let result = body(dir.path());
        reset_for_test();
        result
    }

    fn read_current(dir: &Path) -> String {
        fs::read_to_string(dir.join(FILE_NAME)).unwrap_or_default()
    }

    #[test]
    fn data_dir_does_not_move_log_dir() {
        let log = PathBuf::from("C:/tmp/monitor-logs");
        let data = PathBuf::from("C:/tmp/monitor-data");
        let resolved = resolve_dir_from(Some(log.clone()), Some(data), None);
        assert_eq!(resolved, log);
        let fallback = resolve_dir_from(None, Some(PathBuf::from("C:/not-logs")), None);
        assert!(fallback.ends_with("logs"));
        assert!(!fallback.starts_with("C:/not-logs"));
    }

    #[test]
    fn redacts_secret_password_and_host_payload() {
        with_log(DEFAULT_MAX_BYTES, |dir| {
            emit(
                Level::Info,
                "session",
                serde_json::json!({
                    "secret": "password=super-secret",
                    "host": "bearer abc",
                    "ok": true
                }),
            );
            let text = read_current(dir);
            assert!(!scan_text_for_secrets(&text));
            assert!(!text.to_ascii_lowercase().contains("super-secret"));
            assert!(text.contains("<redacted>"));
            assert!(text.contains("\"ok\":true"));
        });
    }

    #[test]
    fn rotation_keeps_at_most_five_files() {
        with_log(64, |dir| {
            for index in 0..20 {
                emit(
                    Level::Info,
                    "boot",
                    serde_json::json!({ "n": index, "pad": "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx" }),
                );
            }
            let mut names: Vec<_> = fs::read_dir(dir)
                .expect("list")
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .filter(|name| name.starts_with(FILE_NAME))
                .collect();
            names.sort();
            assert!(names.len() <= 5, "{names:?}");
            assert!(names.contains(&FILE_NAME.to_string()));
        });
    }

    #[test]
    fn emit_does_not_panic_when_uninitialized() {
        let _lock = exclusive_test();
        reset_for_test();
        emit(
            Level::Error,
            "storage_open",
            serde_json::json!({"class":"sqlite"}),
        );
    }
}

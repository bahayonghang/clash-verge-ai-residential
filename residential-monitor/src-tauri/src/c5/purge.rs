//! 应用内显式删除本地数据。二次确认。部分失败不得显示全部成功。
//! 不在本模块写入本机 Credential Manager。

use crate::identity::{CREDENTIAL_TARGET, DELETE_CONFIRM_PHRASE};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteItem {
    pub id: String,
    pub kind: String,
    pub path: String,
    pub exists: bool,
    pub note_zh: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletePreview {
    pub schema_version: u32,
    pub confirm_phrase: &'static str,
    pub items: Vec<DeleteItem>,
    pub note_zh: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteItemResult {
    pub id: String,
    pub ok: bool,
    pub existed: bool,
    pub message_zh: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteReport {
    pub schema_version: u32,
    pub all_declared_ok: bool,
    pub items: Vec<DeleteItemResult>,
    pub summary_zh: String,
}

pub fn preview_delete(data_dir: &Path) -> DeletePreview {
    DeletePreview {
        schema_version: 1,
        confirm_phrase: DELETE_CONFIRM_PHRASE,
        items: declared_items(data_dir),
        note_zh: "删除会停止采集并分项执行。普通卸载不走此路径，默认保留数据。凭据项只清理当前进程引用，不写本机 Credential Manager。".into(),
    }
}

pub fn confirm_delete(
    data_dir: &Path,
    phrase: &str,
    clear_credential: impl Fn() -> Result<(), String>,
) -> Result<DeleteReport, String> {
    if phrase != DELETE_CONFIRM_PHRASE {
        return Err("确认短语不匹配".into());
    }
    let preview = declared_items(data_dir);
    let mut items = Vec::new();
    for item in preview {
        if item.id == "credential-ref" {
            match clear_credential() {
                Ok(()) => items.push(DeleteItemResult {
                    id: item.id,
                    ok: true,
                    existed: item.exists,
                    message_zh: "已清除当前进程凭据引用。".into(),
                }),
                Err(message) => items.push(DeleteItemResult {
                    id: item.id,
                    ok: false,
                    existed: item.exists,
                    message_zh: message,
                }),
            }
            continue;
        }
        items.push(remove_path(&item));
    }
    let all_declared_ok = items.iter().all(|item| item.ok);
    let failed = items.iter().filter(|item| !item.ok).count();
    let summary_zh = if all_declared_ok {
        "已删除全部声明对象。".into()
    } else {
        format!("部分失败 {failed} 项。不是已全部删除。")
    };
    Ok(DeleteReport {
        schema_version: 1,
        all_declared_ok,
        items,
        summary_zh,
    })
}

fn declared_items(data_dir: &Path) -> Vec<DeleteItem> {
    let db = data_dir.join("monitor.sqlite3");
    vec![
        file_item("database", "sqlite", &db, "主数据库。"),
        file_item(
            "database-wal",
            "sqlite-sidecar",
            &sidecar(&db, "-wal"),
            "WAL 附属文件。",
        ),
        file_item(
            "database-shm",
            "sqlite-sidecar",
            &sidecar(&db, "-shm"),
            "SHM 附属文件。",
        ),
        file_item(
            "report-spool",
            "directory",
            &data_dir.join("report-spool"),
            "报告快照 spool，不持有 SQLite 读事务。",
        ),
        file_item(
            "migration-backup",
            "directory",
            &data_dir.join("migration-backup"),
            "迁移前备份目录（若存在）。",
        ),
        DeleteItem {
            id: "credential-ref".into(),
            kind: "credential-ref".into(),
            path: CREDENTIAL_TARGET.into(),
            exists: true,
            note_zh: "只清除当前进程 Fake / 会话引用。未授权前不写本机 Credential Manager。".into(),
        },
    ]
}

fn file_item(id: &str, kind: &str, path: &Path, note_zh: &str) -> DeleteItem {
    DeleteItem {
        id: id.into(),
        kind: kind.into(),
        path: path.to_string_lossy().into_owned(),
        exists: path.exists(),
        note_zh: note_zh.into(),
    }
}

fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}

fn remove_path(item: &DeleteItem) -> DeleteItemResult {
    let path = Path::new(&item.path);
    if !path.exists() {
        return DeleteItemResult {
            id: item.id.clone(),
            ok: true,
            existed: false,
            message_zh: "对象本来就不存在。".into(),
        };
    }
    let result = if path.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };
    match result {
        Ok(()) => DeleteItemResult {
            id: item.id.clone(),
            ok: true,
            existed: true,
            message_zh: "已删除。".into(),
        },
        Err(_) => DeleteItemResult {
            id: item.id.clone(),
            ok: false,
            existed: true,
            message_zh: "删除失败，对象仍在。".into(),
        },
    }
}

#[cfg(test)]
mod purge_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn wrong_phrase_does_not_delete() {
        let dir = tempdir().expect("dir");
        let db = dir.path().join("monitor.sqlite3");
        std::fs::write(&db, b"db").expect("db");
        let error = confirm_delete(dir.path(), "delete", || Ok(())).expect_err("phrase");
        assert_eq!(error, "确认短语不匹配");
        assert!(db.exists());
    }

    #[test]
    fn confirm_deletes_files_and_reports_partial_failure() {
        let dir = tempdir().expect("dir");
        std::fs::write(dir.path().join("monitor.sqlite3"), b"db").expect("db");
        std::fs::create_dir_all(dir.path().join("report-spool")).expect("spool");
        let ok = confirm_delete(dir.path(), DELETE_CONFIRM_PHRASE, || Ok(())).expect("ok");
        assert!(ok.all_declared_ok);
        assert!(!dir.path().join("monitor.sqlite3").exists());
        assert!(ok.summary_zh.contains("全部声明"));

        std::fs::write(dir.path().join("monitor.sqlite3"), b"db").expect("db2");
        let failed = confirm_delete(dir.path(), DELETE_CONFIRM_PHRASE, || {
            Err("凭据引用未能清除。".into())
        })
        .expect("partial");
        assert!(!failed.all_declared_ok);
        assert!(failed.summary_zh.contains("部分失败"));
        assert!(!failed.summary_zh.contains("已删除全部声明对象"));
    }
}

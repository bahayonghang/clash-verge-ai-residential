use residential_monitor_lib::storage::{CommitBundle, CommitOutcome, StorageCoordinator};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::tempdir;

fn wait_ready(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if path.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("子进程未写出 ready 文件");
}

fn spawn_kill_child(mode: &str, db: &Path, ready: &Path) -> std::process::Child {
    Command::new(env!("CARGO_BIN_EXE_monitor-bench"))
        .args([
            "kill-child",
            "--mode",
            mode,
            "--db",
            db.to_str().expect("db"),
            "--ready",
            ready.to_str().expect("ready"),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn")
}

fn bundle() -> CommitBundle {
    CommitBundle {
        writer_epoch: 1,
        bundle_seq: 1,
        payload: "1,1,8,16".into(),
    }
}

#[test]
fn crash_before_commit_isolated_process_leaves_no_receipt() {
    let dir = tempdir().expect("tempdir");
    let db = dir.path().join("before.sqlite3");
    let ready = dir.path().join("ready");
    let mut child = spawn_kill_child("before-commit", &db, &ready);
    wait_ready(&ready);
    child.kill().expect("kill");
    let _ = child.wait();
    let coordinator = StorageCoordinator::open(&db).expect("reopen");
    assert_eq!(coordinator.receipt_count().expect("count"), 0);
}

#[test]
fn crash_after_commit_before_receipt_isolated_process_replays_once() {
    let dir = tempdir().expect("tempdir");
    let db = dir.path().join("after.sqlite3");
    let ready = dir.path().join("ready");
    let mut child = spawn_kill_child("after-commit", &db, &ready);
    wait_ready(&ready);
    child.kill().expect("kill");
    let _ = child.wait();
    let mut coordinator = StorageCoordinator::open(&db).expect("reopen");
    let outcome = coordinator.commit(&bundle()).expect("retry");
    assert!(matches!(outcome, CommitOutcome::Duplicate(_)));
    assert_eq!(coordinator.watermark().expect("wm"), 1);
}

#[test]
fn crash_commit_unknown_isolated_process_retries_same_bundle() {
    let dir = tempdir().expect("tempdir");
    let db = dir.path().join("unknown.sqlite3");
    let ready = dir.path().join("ready");
    let status = spawn_kill_child("commit-unknown", &db, &ready)
        .wait()
        .expect("wait");
    assert!(status.success());
    wait_ready(&ready);
    let mut coordinator = StorageCoordinator::open(&db).expect("reopen");
    let outcome = coordinator.commit(&bundle()).expect("retry");
    assert!(matches!(outcome, CommitOutcome::Duplicate(_)));
    assert_eq!(coordinator.watermark().expect("wm"), 1);
}

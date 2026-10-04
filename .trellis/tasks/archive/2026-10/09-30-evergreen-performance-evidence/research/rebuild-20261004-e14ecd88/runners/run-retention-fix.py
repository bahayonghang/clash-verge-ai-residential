"""辅助游标相位修复后的复测（retention_day.rs cleanup_expired 两游标同轮结束）；程序为修复后 release lib 测试程序，其它参数不变。
在 A50/A250 语料副本上运行隔离回收容量测试（first-day 口径，rebuild-20261004 binding）。

口径沿用 09-19 benchmark-harness：维护 now = manifest.start_utc + 31 天，chunks=100000，
调用 cfg(test) 隔离删除门 `bench::corpus::tests::isolated_corpus_retention_capacity_gate`。
只写语料副本；原语料 hash 在前后核对。生产 AUTO_DELETE_ENABLED 不变。
"""

import sys

sys.dont_write_bytecode = True
from datetime import datetime, timezone
from pathlib import Path
import gzip
import hashlib
import json
import os
import shutil
import subprocess
import time

EVIDENCE = Path(__file__).resolve().parents[1]
STATE = json.loads((EVIDENCE / "state.json").read_text(encoding="utf-8"))
assert STATE["status"].startswith("P2_GENERATION_ORACLE_PROBES_ENDED"), STATE["status"]
ASSET = Path(STATE["asset_root"])
OUT = EVIDENCE / "retention-fix"
DATA = ASSET / "retention-fix"
FIX_EXE = ASSET / "executables" / "candidate-fix-library-tests.exe"
TESTS = {"path": str(FIX_EXE), "sha256": hashlib.sha256(FIX_EXE.read_bytes()).hexdigest().upper(), "source": "working tree after auxiliary cursor fix; cargo +1.98.0 test --release --lib --no-run"}
TEST = "bench::corpus::tests::isolated_corpus_retention_capacity_gate"
DAY = 86400


def utc():
    return datetime.now(timezone.utc).isoformat()


def sha(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as stream:
        while block := stream.read(4 * 1024 * 1024):
            digest.update(block)
    return digest.hexdigest().upper()


def save(path, value):
    Path(path).write_text(
        json.dumps(value, ensure_ascii=False, indent=2) + chr(10), encoding="utf-8"
    )


def competing_load():
    import psutil

    skip = {os.getpid()}
    try:
        skip.update(parent.pid for parent in psutil.Process().parents())
    except (psutil.NoSuchProcess, psutil.AccessDenied):
        pass
    names = ("cargo.exe", "rustc.exe", "monitor-bench.exe", "monitor-db.exe")
    suffixes = ("monitor-bench.exe", "monitor-db.exe", "library-tests.exe")
    scripts = (
        "run-formal-replay.py",
        "run-capacity.py",
        "run-retention.py",
        "run-heap-diagnostics.py",
        "builder.py",
        "generator.py",
        "read-corpus-oracle.py",
    )
    found = []
    for process in psutil.process_iter(["pid", "name", "cmdline"]):
        if process.pid in skip:
            continue
        try:
            name = (process.info["name"] or "").lower()
            command = " ".join(process.info["cmdline"] or [])
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            continue
        if (
            "ferrots" in command
            or name in names
            or name.endswith(suffixes)
            or (
                name.startswith("python")
                and any(script in command for script in scripts)
            )
        ):
            found.append(
                {
                    "pid": process.info["pid"],
                    "name": process.info["name"],
                    "cmdline_head": command[:160],
                }
            )
    return found


def percentile(values, percent):
    values = sorted(values)
    return values[(len(values) - 1) * percent // 100] if values else None


INITIAL_LOAD = competing_load()
assert not INITIAL_LOAD, "competing load before retention start: " + json.dumps(
    INITIAL_LOAD, ensure_ascii=False
)
assert not OUT.exists() and not DATA.exists()
assert sha(TESTS["path"]) == TESTS["sha256"], "library tests executable drift"
OUT.mkdir()
DATA.mkdir()
save(
    OUT / "execution-contract.json",
    {
        "argv": sys.argv,
        "driver_sha256": sha(Path(__file__)),
        "runner_pid": os.getpid(),
        "identity": TESTS,
        "test": TEST,
        "expiry": "first-day: now = manifest.start_utc + 31 days",
        "chunks": 100000,
        "copies_only": True,
        "initial_competing_load": INITIAL_LOAD,
    },
)
summary = {"cases": [], "initial_competing_load": INITIAL_LOAD}
for active in (50, 250):
    bound = STATE["corpus_identities"][str(active)]
    source = Path(bound["db"]).parent
    db, marker = source / "monitor.sqlite3", source / "production-corpus.json"
    assert sha(db) == bound["db_sha256"] and sha(marker) == bound["marker_sha256"], (
        f"A{active} corpus identity drift"
    )
    wal = db.with_name(db.name + "-wal")
    assert not wal.exists() or wal.stat().st_size == 0, "nonempty WAL in source corpus"
    target = DATA / f"a{active}-first-day"
    target.mkdir()
    for path in (db, marker):
        with path.open("rb") as stream, (target / path.name).open("xb") as output:
            shutil.copyfileobj(stream, output)
    assert (
        sha(target / db.name) == bound["db_sha256"]
        and sha(target / marker.name) == bound["marker_sha256"]
    )
    manifest = json.loads(marker.read_text(encoding="utf-8"))
    now = manifest["start_utc"] + 31 * DAY
    label = f"a{active}-first-day"
    environment = dict(
        os.environ,
        RESIWATCH_BENCH_CORPUS_DIR=str(target),
        RESIWATCH_BENCH_CORPUS_NOW_UTC=str(now),
        RESIWATCH_BENCH_CORPUS_CHUNKS="100000",
    )
    argv = [
        TESTS["path"],
        "--exact",
        TEST,
        "--ignored",
        "--nocapture",
        "--test-threads=1",
    ]
    receipt = {
        "argv": argv,
        "environment_override": {
            "RESIWATCH_BENCH_CORPUS_DIR": str(target),
            "RESIWATCH_BENCH_CORPUS_NOW_UTC": str(now),
            "RESIWATCH_BENCH_CORPUS_CHUNKS": "100000",
        },
        "started_utc": utc(),
        "status": "running",
        "competing_load_at_start": competing_load(),
    }
    started = time.perf_counter()
    process = subprocess.Popen(
        argv, env=environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE
    )
    receipt["pid"] = process.pid
    save(OUT / (label + ".receipt.json"), receipt)
    print(json.dumps({"start": label, "pid": process.pid, "now_utc": now}), flush=True)
    stdout, stderr = process.communicate()
    for stream, data in (("stdout", stdout), ("stderr", stderr)):
        (OUT / (label + "." + stream + ".raw.gz")).write_bytes(
            gzip.compress(data, mtime=0)
        )
        (OUT / (label + "." + stream + ".log")).write_text(
            data.decode("utf-8", errors="replace"), encoding="utf-8"
        )
    receipt.update(
        {
            "status": "finished",
            "exit_code": process.returncode,
            "finished_utc": utc(),
            "wall_seconds": time.perf_counter() - started,
            "competing_load_at_end": competing_load(),
        }
    )
    save(OUT / (label + ".receipt.json"), receipt)
    result_path = target / "retention-test-result.json"
    case = {
        "case": label,
        "exit_code": process.returncode,
        "test_passed": "test result: ok. 1 passed"
        in stdout.decode("utf-8", errors="replace"),
        "now_utc": now,
    }
    if result_path.is_file():
        result = json.loads(result_path.read_text(encoding="utf-8"))
        save(OUT / (label + ".result.json"), result)
        chunks = result.get("chunks", [])
        walls = [item["wall_ms"] for item in chunks if "wall_ms" in item]
        case.update(
            {
                "complete": result.get("complete"),
                "quick_check": result.get("quick_check"),
                "conservation": result.get("conservation"),
                "wall_secs": result.get("wall_secs"),
                "chunks": len(chunks),
                "chunk_errors": [item for item in chunks if "error" in item],
                "chunk_wall_ms_p95": percentile(walls, 95),
                "chunk_wall_ms_max": max(walls) if walls else None,
                "retry_attempts": result.get("retry_attempts"),
                "ledger_after": result.get("ledger_after"),
            }
        )
    else:
        case["result"] = "MISSING"
    assert sha(db) == bound["db_sha256"], f"A{active} source corpus changed"
    case["source_corpus_unchanged"] = True
    summary["cases"].append(case)
    save(OUT / "summary.json", summary)
    print(
        json.dumps(
            {
                k: case.get(k)
                for k in (
                    "case",
                    "exit_code",
                    "test_passed",
                    "complete",
                    "wall_secs",
                    "chunks",
                    "chunk_wall_ms_max",
                    "retry_attempts",
                )
            }
        ),
        flush=True,
    )
passed = all(
    case["exit_code"] == 0 and case["test_passed"] and case.get("complete") is True
    for case in summary["cases"]
)
summary["status"] = "PASS" if passed else "FAIL"
save(OUT / "summary.json", summary)
sys.exit(0 if passed else 2)

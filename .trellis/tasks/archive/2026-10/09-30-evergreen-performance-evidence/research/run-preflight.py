"""Run isolated smoke and ordered report diagnostics with immutable receipts."""
from datetime import datetime, timezone
from pathlib import Path
import gzip
import hashlib
import json
import math
import os
import sqlite3
import subprocess
import time

ROOT = Path(__file__).resolve().parent
BUILD = ROOT / "build-20260930"
STATE = json.loads((BUILD / "state.json").read_text())
IDENTITY = json.loads((BUILD / "executable-identity.json").read_text())
OUT = ROOT / "stages-20260930-initial"
RUN = Path(STATE["run_root"])
DB = Path("C:/Users/lyh/AppData/Local/Temp/resiwatch-resource-measure-20260924-raw-fold/corpus-a250-30d/monitor.sqlite3")
START = 1787184000
END = 1789776000


def save(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + chr(10), encoding="utf-8")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest().upper()


def call(label, argv, overrides=None):
    receipt = OUT / (label + ".receipt.json")
    assert not receipt.exists()
    start = datetime.now(timezone.utc).isoformat()
    metadata = {"argv": argv, "started_utc": start, "cwd": STATE["repo"], "environment_overrides": overrides or {}, "status": "running"}
    save(receipt, metadata)
    print(json.dumps({"start": label}), flush=True)
    clock = time.perf_counter()
    result = subprocess.run(argv, cwd=STATE["repo"], capture_output=True, env=dict(os.environ, **(overrides or {})))
    logs = {}
    for stream, data in [("stdout", result.stdout), ("stderr", result.stderr)]:
        zipped = OUT / (label + "." + stream + ".raw.gz")
        zipped.write_bytes(gzip.compress(data, mtime=0))
        readable = OUT / (label + "." + stream + ".log")
        readable.write_text(chr(10).join(x.rstrip() for x in data.decode("utf-8", errors="replace").splitlines()) + chr(10), encoding="utf-8")
        logs[stream] = {"gzip": zipped.name, "raw_sha256": hashlib.sha256(data).hexdigest().upper(), "readable": readable.name}
    metadata.update({"finished_utc": datetime.now(timezone.utc).isoformat(), "wall_seconds": time.perf_counter() - clock, "exit_code": result.returncode, "status": "finished", "logs": logs})
    save(receipt, metadata)
    print(json.dumps({"finished": label, "exit_code": result.returncode, "wall_seconds": metadata["wall_seconds"]}), flush=True)
    return result


assert not OUT.exists(), "Do not replace evidence"
OUT.mkdir()
for value in IDENTITY.values():
    assert sha(Path(value["path"])) == value["sha256"]
smokes = {}
for variant in ["baseline", "candidate"]:
    entry = IDENTITY[variant + "-monitor-bench.exe"]
    data = RUN / ("smoke-" + variant)
    assert not data.exists()
    label = "smoke-" + variant
    args = [entry["path"], "replay-facade", "--active", "8", "--hz", "1", "--duration-secs", "3", "--warmup-secs", "0", "--workload", "counters", "--metadata-change-percent", "100", "--archive", "complete", "--query-every-frames", "0", "--seed", "20260919", "--start-utc", "1800001800", "--source-revision", STATE["baseline_revision"] if variant == "baseline" else "d3a25b4+candidate-source-before.json", "--dir", str(data), "--virtual-time"]
    result = call(label, args)
    assert result.returncode == 0, label
    report = json.loads(result.stdout)
    save(OUT / (label + ".json"), report)
    assert report["commits"] == 3 and report["frames"] == 3 and report["traffic"]["conserved"]
    assert report["synchronous"] == "FULL"
    assert report["executable_sha256"].upper() == entry["sha256"]
    with sqlite3.connect((data / "monitor.sqlite3").as_uri() + "?mode=ro", uri=True) as connection:
        wal = connection.execute("pragma journal_mode").fetchone()[0]
    assert wal.lower() == "wal"
    smokes[variant] = {"fixture_hash": report["fixture_hash"], "commits": report["commits"], "conserved": report["traffic"]["conserved"], "journal_mode_after_run": wal, "synchronous_reported": report["synchronous"]}
assert smokes["baseline"]["fixture_hash"] == smokes["candidate"]["fixture_hash"]
save(OUT / "smoke-validation.json", smokes)

marker = DB.parent / "production-corpus.json"
assert sha(marker) == "667A21E9FBE3FCF100974411D8E714E1589907E6458902AFD50683CEE68A3D42"
corpus = json.loads(marker.read_text())
assert corpus["kind"] == "production-corpus" and corpus["full_30_day_input"]
assert corpus["start_utc"] == START and corpus["end_utc"] == END
assert corpus["workload"]["average_active"] == 250 and corpus["workload"]["seed"] == 20260919
assert DB.stat().st_size == 1697222656
save(OUT / "first-reader-preconditions.json", {"database": str(DB.resolve()), "marker_sha256": sha(marker), "database_bytes": DB.stat().st_size, "database_read_hash_copy_integrity_check_before_first_reader": False, "cache_statement": "本轮未主动预热；没有物理冷缓存证明", "recorded_utc": datetime.now(timezone.utc).isoformat(), "smoke_databases": "separate new isolated databases"})
test = IDENTITY["candidate-library-tests.exe"]["path"]
first_path = OUT / "a250-minute-production-first.json"
result = call("a250-minute-production-first", [test, "--exact", "c3::service::tests::isolated_nonempty_report_stage_proof", "--ignored", "--nocapture", "--test-threads=1"], {"RESIWATCH_NONEMPTY_STAGE_DB": str(DB), "RESIWATCH_NONEMPTY_STAGE_OUT": str(first_path), "RESIWATCH_NONEMPTY_STAGE_START": str(START), "RESIWATCH_NONEMPTY_STAGE_END": str(START + 60)})
first = json.loads(first_path.read_text()) if first_path.exists() else {}
production = first.get("production_first_read", {})
elapsed = first.get("production_first_read_ms")
checks = {"native_exit_zero": result.returncode == 0, "identity": first.get("kind") == "nonempty-minute-report-stages" and first.get("query") == "default_auto_report_query", "window": first.get("range_start_utc") == START and first.get("range_end_utc") == START + 60, "production_ok_raw": production.get("status") == "ok" and production.get("tier") == "Raw", "wall_valid_below_deadline": isinstance(elapsed, (float, int)) and math.isfinite(elapsed) and 0 <= elapsed < 10000, "improved_from_historical_minute": isinstance(elapsed, (float, int)) and elapsed < 7690.685, "series_nonempty": production.get("series_rows", 0) > 0 and first.get("stage_series_rows", 0) > 0}
for name, value in {"connection_count": 250, "upload": 3497, "download": 11725}.items():
    checks[name + "_oracle"] = production.get(name) == value and first.get("stage_" + name) == value
save(OUT / "production-first-validation.json", {"checks": checks, "pass": all(checks.values()), "production_first_read": production, "production_first_read_ms": elapsed, "query_timezone": "local", "previous_equal_window": True, "now_utc": START + 60, "cache": "本轮未主动预热；没有物理冷缓存证明"})
print(json.dumps({"production_first": production, "wall_ms": elapsed, "checks": checks}), flush=True)
stage_summaries = []
for window, end in [("minute", START + 60), ("30d", END)]:
    for filter_name in ["all", "residential"]:
        label = "a250-" + window + "-" + filter_name
        path = OUT / (label + ".json")
        result = call(label, [test, "--exact", "c3::raw_fold::tests::isolated_raw_fold_stage_proof", "--ignored", "--nocapture", "--test-threads=1"], {"RESIWATCH_RAW_FOLD_STAGE_DB": str(DB), "RESIWATCH_RAW_FOLD_STAGE_OUT": str(path), "RESIWATCH_RAW_FOLD_START": str(START), "RESIWATCH_RAW_FOLD_END": str(end), "RESIWATCH_RAW_FOLD_FILTER": filter_name})
        report = json.loads(path.read_text()) if path.exists() else {}
        stage_summaries.append({"label": label, "exit_code": result.returncode, "query_timezone": "local" if filter_name == "all" else "UTC", "previous_equal_window": filter_name == "all", "production_deadline": False, "cache": "after earlier production/stage reads; page cache not dropped", "report": report})
        save(OUT / "stage-summary.json", stage_summaries)
        print(json.dumps(stage_summaries[-1]), flush=True)
for label, pattern in [("raw-fold-tests", "c3::raw_fold::"), ("report-service-tests", "c3::service::"), ("bench-instrumentation-tests", "bench::")]:
    result = call(label, [test, pattern, "--nocapture", "--test-threads=1"])
    assert result.returncode == 0, label
print(json.dumps({"preflight_complete": True, "production_minute_pass": all(checks.values()), "out": str(OUT)}), flush=True)

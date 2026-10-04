"""Correct the recorded zero-test invocation without replacing its evidence."""
from pathlib import Path
from datetime import datetime, timezone
import gzip
import hashlib
import json
import math
import os
import re
import subprocess
import time

ROOT = Path(__file__).resolve().parent
IDENTITY = json.loads((ROOT / "build-20260930/executable-identity.json").read_text())
ENTRY = IDENTITY["candidate-library-tests.exe"]
EXE = ENTRY["path"]
DB = Path("C:/Users/lyh/AppData/Local/Temp/resiwatch-resource-measure-20260924-raw-fold/corpus-a250-30d/monitor.sqlite3")
OUT = ROOT / "stages-20260930-corrected"
START, END = 1787184000, 1789776000
PROBE = "c3::service::raw_stage_probe_tests::isolated_nonempty_report_stage_proof"
FOLD = "c3::raw_fold::tests::isolated_raw_fold_stage_proof"


def save(path, obj):
    path.write_text(json.dumps(obj, ensure_ascii=False, indent=2) + chr(10), encoding="utf-8")


def call(label, args, env=None):
    assert not (OUT / (label + ".receipt.json")).exists()
    receipt = {"argv": args, "started_utc": datetime.now(timezone.utc).isoformat(), "environment_overrides": env or {}, "status": "running"}
    save(OUT / (label + ".receipt.json"), receipt)
    print(json.dumps({"start": label}), flush=True)
    clock = time.perf_counter()
    result = subprocess.run(args, capture_output=True, env=dict(os.environ, **(env or {})))
    logs = {}
    for name, data in [("stdout", result.stdout), ("stderr", result.stderr)]:
        path = OUT / (label + "." + name + ".raw.gz")
        path.write_bytes(gzip.compress(data, mtime=0))
        readable = OUT / (label + "." + name + ".log")
        readable.write_text(chr(10).join(x.rstrip() for x in data.decode("utf-8", errors="replace").splitlines()) + chr(10), encoding="utf-8")
        logs[name] = {"gzip": path.name, "raw_sha256": hashlib.sha256(data).hexdigest(), "readable": readable.name}
    receipt.update({"finished_utc": datetime.now(timezone.utc).isoformat(), "wall_seconds": time.perf_counter() - clock, "exit_code": result.returncode, "logs": logs, "status": "finished"})
    save(OUT / (label + ".receipt.json"), receipt)
    print(json.dumps({"finished": label, "exit_code": result.returncode, "wall_seconds": receipt["wall_seconds"]}), flush=True)
    return result


assert not OUT.exists()
OUT.mkdir()
assert hashlib.sha256(Path(EXE).read_bytes()).hexdigest().upper() == ENTRY["sha256"]
listing = call("test-list", [EXE, "--list"])
assert listing.returncode == 0
names = listing.stdout.decode().splitlines()
assert PROBE + ": test" in names and FOLD + ": test" in names
save(OUT / "execution-boundary.json", {"previous_receipts": "../stages-20260930-initial", "previous_production_probe": "zero tests; no JSON; FAIL", "database_first_read_this_round": "initial a250-minute-all diagnostic", "cache": "此前阶段已读取数据库；本次补测已受页缓存读取影响，没有物理冷缓存证明", "production_probe": PROBE, "list_verified": True, "executable_sha256": ENTRY["sha256"]})
summaries = []
for window, end, oracle in [("minute", START + 60, {"connection_count": 250, "upload": 3497, "download": 11725}), ("30d", END, {"connection_count": 2160000, "upload": 151200035, "download": 507599545})]:
    label = "a250-" + window + "-production"
    path = OUT / (label + ".json")
    result = call(label, [EXE, "--exact", PROBE, "--ignored", "--nocapture", "--test-threads=1"], {"RESIWATCH_NONEMPTY_STAGE_DB": str(DB), "RESIWATCH_NONEMPTY_STAGE_OUT": str(path), "RESIWATCH_NONEMPTY_STAGE_START": str(START), "RESIWATCH_NONEMPTY_STAGE_END": str(end)})
    checks = {"native_exit_zero": result.returncode == 0, "ran_exactly_one_test": bool(re.search(rb"(?m)^running 1 test\r?$", result.stdout)), "json_exists": path.exists()}
    assert checks["ran_exactly_one_test"] and checks["json_exists"], checks
    data = json.loads(path.read_text())
    first = data.get("production_first_read", {})
    wall = data.get("production_first_read_ms")
    checks.update({"query_identity": data.get("query") == "default_auto_report_query" and data.get("range_start_utc") == START and data.get("range_end_utc") == end, "production_ok_raw": first.get("status") == "ok" and first.get("tier") == "Raw", "production_under_10s": isinstance(wall, (int, float)) and math.isfinite(wall) and 0 <= wall < 10000, "nonempty_series": first.get("series_rows", 0) > 0 and data.get("stage_series_rows", 0) > 0})
    for key, value in oracle.items():
        checks[key + "_oracle"] = first.get(key) == value and data.get("stage_" + key) == value
    if window == "minute":
        checks["improvement_from_7690_685_ms"] = wall < 7690.685
    item = {"label": label, "checks": checks, "pass": all(checks.values()), "production_first_read": first, "production_first_read_ms": wall, "query_timezone": "local", "previous_equal_window": True, "now_utc": end, "cache": "already read by preceding probes", "oracle_source": "fixed 09-24 minute values" if window == "minute" else "initial 30d diagnostic; independent SQL oracle still required"}
    summaries.append(item)
    save(OUT / "production-validation.json", summaries)
    print(json.dumps(item), flush=True)
for filter_name in ["all", "residential"]:
    label = "a250-30d-" + filter_name
    path = OUT / (label + ".json")
    result = call(label, [EXE, "--exact", FOLD, "--ignored", "--nocapture", "--test-threads=1"], {"RESIWATCH_RAW_FOLD_STAGE_DB": str(DB), "RESIWATCH_RAW_FOLD_STAGE_OUT": str(path), "RESIWATCH_RAW_FOLD_START": str(START), "RESIWATCH_RAW_FOLD_END": str(END), "RESIWATCH_RAW_FOLD_FILTER": filter_name})
    assert re.search(rb"(?m)^running 1 test\r?$", result.stdout) and path.exists()
    data = json.loads(path.read_text())
    save(OUT / (label + ".validation.json"), {"native_exit_zero": result.returncode == 0, "ran_exactly_one_test": True, "json_exists": True, "timezone": "local" if filter_name == "all" else "UTC", "previous_equal_window": filter_name == "all", "cache": "already read by preceding probes", "production_deadline": False})
    print(json.dumps({"label": label, "projection_ms": data["projection_ms"], "scan_ms": data["scan_ms"], "totals": {k:data[k] for k in ["connection_count", "upload", "download"]}}), flush=True)

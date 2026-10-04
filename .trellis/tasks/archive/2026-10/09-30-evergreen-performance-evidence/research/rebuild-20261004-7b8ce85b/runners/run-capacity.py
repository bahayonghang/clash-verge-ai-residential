"""Run the approved A50/A250 capacity cases without changing database contents (rebuild-20261004 binding)."""
import sys
sys.dont_write_bytecode = True
from datetime import datetime, timezone
from contextlib import closing
from pathlib import Path
import gzip
import hashlib
import json
import math
import os
import sqlite3
import subprocess
import sys
import time
import traceback

# 新绑定：新 state、新 candidate-monitor-db、新 A50/A250 语料与 marker/DB hash；验证与门槛语义沿用原 runner。
EVIDENCE = Path(__file__).resolve().parents[1]
STATE = json.loads((EVIDENCE / "state.json").read_text(encoding="utf-8"))
assert STATE["status"].startswith("P2_GENERATION_ORACLE_PROBES_ENDED"), STATE["status"]
OUT = EVIDENCE / "capacity"
CORPUS = Path(STATE["asset_root"])
IDENTITY = STATE["executable_identities"]["candidate-monitor-db.exe"]
PREVIOUS = json.loads((EVIDENCE.parent / "corpus-a250-verification.json").read_text(encoding="utf-8-sig"))
QUERIES = {item["name"]: item for item in PREVIOUS["queries"]}
BOUND = STATE["corpus_identities"]


def competing_load():
    # 只记录竞争负载，不停止用户程序；是否作废由主会话审查。排除本进程及其祖先（包装器与启动 shell）。
    import psutil
    skip = {os.getpid()}
    try:
        skip.update(parent.pid for parent in psutil.Process().parents())
    except (psutil.NoSuchProcess, psutil.AccessDenied):
        pass
    names = ("cargo.exe", "rustc.exe", "monitor-bench.exe", "monitor-db.exe")
    suffixes = ("monitor-bench.exe", "monitor-db.exe", "library-tests.exe")
    scripts = ("run-formal-replay.py", "run-capacity.py", "builder.py", "generator.py", "read-corpus-oracle.py", "p2-completion.py")
    found = []
    for process in psutil.process_iter(["pid", "name", "cmdline"]):
        if process.pid in skip:
            continue
        try:
            name = (process.info["name"] or "").lower()
            command = " ".join(process.info["cmdline"] or [])
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            continue
        if "ferrots" in command or name in names or name.endswith(suffixes) or (name.startswith("python") and any(script in command for script in scripts)):
            found.append({"pid": process.info["pid"], "name": process.info["name"], "cmdline_head": command[:160]})
    return found


INITIAL_LOAD = competing_load()
assert not INITIAL_LOAD, "competing load before capacity start: " + json.dumps(INITIAL_LOAD, ensure_ascii=False)
assert not OUT.exists()
OUT.mkdir()

def save(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + chr(10), encoding="utf-8")

def sha(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        while block := stream.read(4 * 1024 * 1024):
            digest.update(block)
    return digest.hexdigest()

failure_context = {"stage": "source_and_executable_validation"}
preflight = None
results = []

def retain_unhandled_failure(error_type, error, trace):
    failure = {"status": "FAIL", "context": failure_context, "error": repr(error), "traceback": traceback.format_exception(error_type, error, trace), "utc": datetime.now(timezone.utc).isoformat()}
    save(OUT / "execution-failure.json", failure)
    if failure_context.get("active") is not None:
        failed_preflight = preflight if preflight is not None else dict(failure_context)
        failed_preflight.update({"status": "FAIL", "failure": failure})
        save(OUT / ("preflight-a" + str(failure_context["active"]) + ".json"), failed_preflight)
    save(OUT / "summary.json", {"status": "FAIL", "execution_failure": failure, "expected_native_invocations": 106, "recorded_native_invocations": len(results), "partial_results": results, "A1000": "NOT_RUN: stage prerequisite unmet", "AC3": "NOT_COMPLETE"})
    sys.__excepthook__(error_type, error, trace)

sys.excepthook = retain_unhandled_failure

assert sha(Path(IDENTITY["path"])).upper() == IDENTITY["sha256"]
for relative, expected in STATE["source_manifest"].items():
    assert sha(Path(STATE["candidate_source"]) / relative).upper() == expected["sha256"], relative
save(OUT / "execution-contract.json", {"argv": sys.argv, "driver_sha256": sha(Path(__file__)), "runner_pid": os.getpid(), "identity": IDENTITY, "native_clock": "monitor-db actual Utc::now; no override", "cache": "preflight hash/integrity/oracle reads occur before native queries; first process is not a cold-page proof", "cases": "A50/A250 host 30d and 1d each 21; A250 network 30d 21; A50 network 30d one original diagnostic observation", "expected_native_invocations": 106, "A1000": "NOT_RUN: approved stage prerequisite unmet; original complete 30d/21-exit0/10000ms gate retained"})
preflights = []
for active in [50, 250]:
    directory = CORPUS / f"corpus-a{active}-30d"
    db, marker = directory / "monitor.sqlite3", directory / "production-corpus.json"
    failure_context = {"stage": "database_preflight", "active": active, "database": str(db)}
    preflight = None
    expected_marker = BOUND[str(active)]["marker_sha256"]
    failure_context["marker_sha256_actual"] = sha(marker).upper()
    failure_context["marker_sha256_expected"] = expected_marker
    assert failure_context["marker_sha256_actual"] == expected_marker, "synthetic corpus marker identity mismatch"
    manifest = json.loads(marker.read_text(encoding="utf-8"))
    assert manifest["kind"] == "production-corpus" and manifest["full_30_day_input"] is True
    assert manifest["workload"]["average_active"] == active and manifest["workload"]["seed"] == 20260919 and manifest["workload"]["days"] == 30
    assert (manifest["start_utc"], manifest["end_utc"]) == (1787184000, 1789776000)
    wal = db.with_name(db.name + "-wal")
    assert not wal.exists() or wal.stat().st_size == 0, "nonempty WAL before capacity"
    digest = sha(db)
    failure_context["database_sha256_actual"] = digest
    assert digest.upper() == BOUND[str(active)]["db_sha256"], "corpus DB identity differs from P2 generator binding"
    preflight = {"active": active, "database": str(db), "database_sha256_before": digest, "database_bytes": db.stat().st_size, "marker_sha256": expected_marker, "started_utc": datetime.now(timezone.utc).isoformat(), "read_only_uri": True, "sqlite_version": sqlite3.sqlite_version, "oracles": {}}
    save(OUT / f"preflight-a{active}.json", preflight)
    print(json.dumps({"preflight_start": active}), flush=True)
    def check(name, actual, expected):
        preflight.setdefault("checks", {})[name] = {"actual": actual, "expected": expected}
        save(OUT / f"preflight-a{active}.json", preflight)
        assert actual == expected, name + " mismatch: " + repr(actual)
    with closing(sqlite3.connect(db.as_uri() + "?mode=ro", uri=True)) as connection:
        connection.execute("pragma query_only = on")
        connection.execute("begin deferred")
        check("quick_check", connection.execute("pragma quick_check").fetchall(), [("ok",)])
        check("schema_version", connection.execute("pragma user_version").fetchone(), (5,))
        check("journal_mode", connection.execute("pragma journal_mode").fetchone(), ("wal",))
        check("migration_identity", connection.execute("select checksum from schema_migration where version=5").fetchone(), ("ledger-lifecycle-v5-layout4",))
        counts = {}
        for name, table in [("minutes", "connection_minute"), ("sessions", "connection_session"), ("chains", "connection_chain"), ("receipts", "committed_bundle")]:
            counts[name] = connection.execute("select count(*) from " + table).fetchone()[0]
        check("row_counts_expected", counts, manifest["expected"])
        check("row_counts_original", counts, manifest["actual"])
        preflight["counts"] = counts
        for kind, window, top in [("host", "30d", 20), ("host", "1d", 20), ("network", "30d", 100)]:
            start = manifest["start_utc"] if window == "30d" else manifest["end_utc"] - 86400
            oracle = QUERIES["residential_" + kind + "_rank_oracle"]
            params = [start // 60, manifest["end_utc"] // 60, top]
            failure_context.update({"stage": "oracle_validation", "oracle_kind": kind, "oracle_window": window})
            oracle_record = {"sql": oracle["sql"], "params": params, "status": "running", "expected_rows": oracle["result"] if active == 250 and window == "30d" else None}
            preflight["oracles"][kind + "-" + window] = oracle_record
            save(OUT / f"preflight-a{active}.json", preflight)
            started = time.perf_counter()
            rows = connection.execute(oracle["sql"], params).fetchall()
            oracle_record.update({"rows": rows, "wall_seconds": time.perf_counter() - started})
            shape_ok = bool(rows) and all(len(row) == 5 for row in rows)
            values_ok = oracle_record["expected_rows"] is None or [list(row) for row in rows] == oracle_record["expected_rows"]
            oracle_record.update({"shape_ok": shape_ok, "values_ok": values_ok, "status": "PASS" if shape_ok and values_ok else "FAIL"})
            save(OUT / f"preflight-a{active}.json", preflight)
            assert shape_ok, "SQL oracle returned an empty or malformed result"
            assert values_ok, "SQL oracle differs from independently verified A250 results"
        connection.execute("commit")
    preflight["status"] = "PASS"
    save(OUT / f"preflight-a{active}.json", preflight)
    preflights.append(preflight)
    failure_context["stage"] = "native_capacity_queries"
    failure_context.pop("oracle_kind", None)
    failure_context.pop("oracle_window", None)
    for kind, window, top, rounds in [("host", "30d", 20, 21), ("host", "1d", 20, 21), ("network", "30d", 100, 21 if active == 250 else 1)]:
        start = manifest["start_utc"] if window == "30d" else manifest["end_utc"] - 86400
        expected_rows = [list(row[:4]) for row in preflight["oracles"][kind + "-" + window]["rows"]]
        for round_number in range(1, rounds + 1):
            label = f"a{active}-{kind}-{window}-r{round_number:02d}"
            argv = [IDENTITY["path"], "--db", str(db), "--since", str(start), "--until", str(manifest["end_utc"]), "--tz", "UTC", "rank", "--by", kind, "--top", str(top)]
            load_before = competing_load()
            receipt = {"argv": argv, "started_utc": datetime.now(timezone.utc).isoformat(), "status": "running", "case": f"a{active}-{kind}-{window}", "round": round_number, "expected_rounds": rounds, "first_process_in_series": round_number == 1, "physical_cold_pages": False, "residential_filter": True, "competing_load_at_start": load_before}
            started = time.perf_counter()
            process = subprocess.Popen(argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            receipt["pid"] = process.pid
            save(OUT / (label + ".receipt.json"), receipt)
            stdout, stderr = process.communicate()
            wall_ms = (time.perf_counter() - started) * 1000
            finished_utc = datetime.now(timezone.utc)
            for stream, data in [("stdout", stdout), ("stderr", stderr)]:
                (OUT / (label + "." + stream + ".raw.gz")).write_bytes(gzip.compress(data, mtime=0))
                (OUT / (label + "." + stream + ".log")).write_text(data.decode("utf-8", errors="replace"), encoding="utf-8")
                receipt[stream + "_sha256"] = hashlib.sha256(data).hexdigest()
            errors = []
            if process.returncode != 0:
                errors.append("native exit " + str(process.returncode))
            if not math.isfinite(wall_ms) or wall_ms > 10000:
                errors.append("process wall exceeds 10000ms")
            try:
                data = json.loads(stdout)
                save(OUT / (label + ".json"), data)
                assert type(data["schemaVersion"]) is int and data["schemaVersion"] == 1 and data["command"] == "rank"
                assert data["window"] == {"startUtc": start, "endUtc": manifest["end_utc"], "timezone": "UTC", "granularity": "hour"}
                assert data["capability"]["supported"] is True and data["capability"]["layer"] == "raw"
                assert type(data["dataVersion"]) is int and data["dataVersion"] == counts["receipts"]
                rows = data["result"]["rankings"]
                assert [[row[key] for key in ["identity", "upload", "download", "connectionCount"]] for row in rows] == expected_rows
                assert all(type(row[key]) is int for row in rows for key in ["upload", "download", "connectionCount"])
                assert all(row["unknown"] is (row["identity"] == "__unknown__") and row["zeroFlow"] is (row["upload"] == 0 and row["download"] == 0) for row in rows)
                assert data["truncation"] == {"status": "complete", "rowCap": top, "rows": len(expected_rows)}
                assert type(data["generatedUtc"]) is int, "generatedUtc must be an integer"
                assert int(datetime.fromisoformat(receipt["started_utc"]).timestamp()) <= data["generatedUtc"] <= int(finished_utc.timestamp()), "generatedUtc is outside this native process wall interval"
                receipt["generated_utc"] = data["generatedUtc"]
            except (AssertionError, KeyError, TypeError, ValueError) as error:
                errors.append("report/oracle validation: " + repr(error))
            receipt.update({"status": "finished", "finished_utc": finished_utc.isoformat(), "exit_code": process.returncode, "wall_ms": wall_ms, "validation_errors": errors, "pass": not errors})
            save(OUT / (label + ".receipt.json"), receipt)
            results.append(receipt)
            save(OUT / "results.json", results)
            print(json.dumps({"finished": label, "exit_code": process.returncode, "wall_ms": wall_ms, "validation_errors": errors}), flush=True)
    failure_context["stage"] = "database_postflight"
    preflight["database_sha256_after"] = sha(db)
    preflight["wal_bytes_after"] = wal.stat().st_size if wal.exists() else 0
    preflight["postflight_status"] = "PASS" if preflight["database_sha256_after"] == digest and preflight["wal_bytes_after"] == 0 else "FAIL"
    preflight["status"] = preflight["postflight_status"]
    save(OUT / f"preflight-a{active}.json", preflight)
    assert preflight["database_sha256_after"] == digest, "database bytes changed during capacity"
    assert preflight["wal_bytes_after"] == 0, "nonempty WAL after capacity"
    save(OUT / f"preflight-a{active}.json", preflight)
cases = []
for name in dict.fromkeys(item["case"] for item in results):
    samples = [item for item in results if item["case"] == name]
    expected = samples[0]["expected_rounds"]
    cases.append({"case": name, "required_samples": expected, "recorded_samples": len(samples), "passing_samples": sum(item["pass"] for item in samples), "max_wall_ms": max(item["wall_ms"] for item in samples), "status": "PASS" if len(samples) == expected and all(item["pass"] for item in samples) else "FAIL", "boundary": "original one-sample diagnostic; not a 21-run gate" if expected == 1 else "21-process exact-oracle capacity gate"})
summary = {"cases": cases, "expected_native_invocations": 106, "recorded_native_invocations": len(results), "A1000": {"status": "NOT_RUN", "reason": "stage prerequisite for generation not met; 30d/21/10000ms requirement remains"}, "initial_competing_load": INITIAL_LOAD, "competing_load_observed": [item["case"] + "/r" + str(item["round"]) for item in results if item.get("competing_load_at_start")], "cache": "all native queries follow preflight database reads; no physical cold-page proof", "AC3": "NOT_COMPLETE"}
save(OUT / "summary.json", summary)
print(json.dumps(summary), flush=True)
sys.exit(2 if len(results) != 106 or any(case["status"] == "FAIL" for case in cases) else 3)

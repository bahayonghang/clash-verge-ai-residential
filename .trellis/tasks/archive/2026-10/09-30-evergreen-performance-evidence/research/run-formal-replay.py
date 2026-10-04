"""Run the original real-time matrix or primary gate with per-command receipts."""
from datetime import datetime, timezone
from pathlib import Path
import argparse
import gzip
import hashlib
import json
import math
import os
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent
parser = argparse.ArgumentParser()
parser.add_argument("--candidate-directory", required=True)
parser.add_argument("--output-directory", required=True)
parser.add_argument("--set", choices=["matrix", "primary"], required=True)
args = parser.parse_args()
BUILD = ROOT / "build-20260930"
STATE = json.loads((BUILD / "state.json").read_text())
candidate_dir = ROOT / args.candidate_directory
frozen = json.loads((candidate_dir / "executable-identity.json").read_text())
baseline = frozen["baseline-monitor-bench.exe"]
candidate = frozen["candidate-monitor-bench.exe"]
source_manifest = candidate_dir / "candidate-source-before.json"
source = json.loads(source_manifest.read_text())
identities = {"baseline": baseline, "candidate": candidate}
for variant, identity in identities.items():
    manifest_path = candidate_dir / (variant + "-source-before.json")
    manifest = json.loads(manifest_path.read_text())
    assert manifest == json.loads((candidate_dir / (variant + "-source-after.json")).read_text())
    assert hashlib.sha256(manifest_path.read_bytes()).hexdigest().upper() == identity["source_manifest_sha256"]
    root = Path(STATE["baseline_source"] if variant == "baseline" else STATE["repo"])
    for relative, expected in manifest.items():
        path = root / relative
        assert path.stat().st_size == expected["bytes"], relative
        assert hashlib.sha256(path.read_bytes()).hexdigest().upper() == expected["sha256"], relative
    assert hashlib.sha256(Path(identity["path"]).read_bytes()).hexdigest().upper() == identity["sha256"]
OUT = ROOT / args.output_directory
DATA = Path(STATE["run_root"]) / args.output_directory
assert not OUT.exists() and not DATA.exists()
OUT.mkdir(); DATA.mkdir()


def save(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + chr(10), encoding="utf-8")


def ratio(before, after):
    if not number(before) or not number(after) or before <= 0:
        return None
    value = after / before
    return value if math.isfinite(value) else None


def number(value):
    return type(value) in (int, float) and math.isfinite(value) and value >= 0


def validate_report(data, expected, identity, pid):
    errors = []
    def get(*keys):
        value = data
        for key in keys:
            if not isinstance(value, dict) or key not in value:
                errors.append("missing field: " + ".".join(keys))
                return None
            value = value[key]
        return value
    def exact(keys, value):
        actual = get(*keys)
        if type(actual) is not type(value) or actual != value:
            errors.append("mismatch: " + ".".join(keys))
    for key, value in expected.items():
        exact(("options", key), value)
    for key, value in [("schema_version", 1), ("kind", "isolated-real-facade"), ("platform", "windows"), ("process_id", pid), ("frames", expected["duration_secs"]), ("commits", expected["duration_secs"]), ("synchronous", "FULL")]:
        exact((key,), value)
    exact(("traffic", "conserved"), True)
    executable = get("executable_sha256")
    if not isinstance(executable, str) or executable.upper() != identity["sha256"]:
        errors.append("executable identity mismatch")
    fixture = get("fixture_hash")
    if not isinstance(fixture, str) or len(fixture) != 64 or any(c not in "0123456789abcdefABCDEF" for c in fixture):
        errors.append("invalid fixture_hash")
    if type(get("local_utc_offset_seconds")) is not int:
        errors.append("invalid local UTC offset")
    for inventory in ["initial_files_and_pages", "final_files_and_pages"]:
        exact((inventory, "journal_mode"), "wal")
        exact((inventory, "synchronous"), 2)
    duration, queries = expected["duration_secs"], expected["query_every_frames"]
    for field, count in [("facade_ingest_including_durable_commit", duration), ("report_first_reader", duration // queries if queries else 0), ("report_repeated_reader", duration // queries if queries else 0)]:
        exact(("latency", field, "count"), count)
        for metric in ["p50_ms", "p95_ms", "p99_ms", "max_ms"]:
            if not number(get("latency", field, metric)):
                errors.append("invalid latency: " + field + "." + metric)
    for field in ["native_private_bytes", "native_working_set_bytes"]:
        exact((field, "samples"), duration)
        for metric in ["p95", "max"]:
            if not number(get(field, metric)):
                errors.append("invalid memory: " + field + "." + metric)
    samples = get("samples")
    if not isinstance(samples, list) or len(samples) != duration:
        errors.append("invalid raw sample count")
    for field in ["native_cpu_seconds", "sqlite_application_file_write_bytes", "measured_wall_secs", "warmup_wall_secs"]:
        if not number(get(field)):
            errors.append("invalid numeric field: " + field)
    for field in ["all_application_file_write_bytes", "spool_write_bytes"]:
        value = get(field)
        if value is not None and not number(value):
            errors.append("invalid optional numeric field: " + field)
    return errors


def measure(name, active, workload, archive, duration, warmup, queries, period_rule, variant):
    label = name + "-" + variant
    identity = identities[variant]
    source_label = STATE["baseline_revision"] if variant == "baseline" else "manifest-sha256:" + hashlib.sha256(source_manifest.read_bytes()).hexdigest()
    argv = [identity["path"], "replay-facade", "--active", str(active), "--hz", "1", "--duration-secs", str(duration), "--warmup-secs", str(warmup), "--workload", workload, "--metadata-change-percent", "100", "--archive", archive, "--query-every-frames", str(queries), "--seed", "20260919", "--start-utc", "1800001800", "--source-revision", source_label, "--dir", str(DATA / label)]
    if period_rule:
        argv.append("--period-rule")
    receipt = {"argv": argv, "started_utc": datetime.now(timezone.utc).isoformat(), "cwd": STATE["repo"], "status": "running", "name": name, "variant": variant}
    path = OUT / (label + ".receipt.json")
    clock = time.perf_counter()
    process = subprocess.Popen(argv, cwd=STATE["repo"], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    receipt["pid"] = process.pid
    save(path, receipt)
    print(json.dumps({"start": label, "pid": process.pid}), flush=True)
    stdout, stderr = process.communicate()
    for stream, data in [("stdout", stdout), ("stderr", stderr)]:
        (OUT / (label + "." + stream + ".raw.gz")).write_bytes(gzip.compress(data, mtime=0))
        (OUT / (label + "." + stream + ".log")).write_text(chr(10).join(line.rstrip() for line in data.decode("utf-8", errors="replace").splitlines()) + chr(10), encoding="utf-8")
        receipt[stream + "_raw_sha256"] = hashlib.sha256(data).hexdigest()
    receipt.update({"status": "finished", "exit_code": process.returncode, "finished_utc": datetime.now(timezone.utc).isoformat(), "wall_seconds": time.perf_counter() - clock})
    data = None
    errors = []
    if process.returncode != 0:
        errors.append("native exit " + str(process.returncode))
    try:
        data = json.loads(stdout)
        save(OUT / (label + ".json"), data)
        expected = {"active": active, "hz": 1, "duration_secs": duration, "warmup_secs": warmup, "workload": workload, "archive": archive, "metadata_change_percent": 100, "query_every_frames": queries, "seed": 20260919, "start_utc": 1800001800, "period_rule": period_rule, "virtual_time": False, "source_revision": source_label, "dir": str(DATA / label)}
        errors.extend(validate_report(data, expected, identity, process.pid))
    except (ValueError, KeyError, TypeError, OverflowError, AttributeError) as error:
        errors.append("invalid report: " + str(error))
    receipt["validation_errors"] = errors
    save(path, receipt)
    print(json.dumps({"finished": label, "exit_code": process.returncode, "validation_errors": errors, "wall_seconds": receipt["wall_seconds"]}), flush=True)
    return {"receipt": receipt, "report": data}


cases = []
if args.set == "matrix":
    for active in [50, 250, 1000]:
        for workload in ["unchanged", "counters", "metadata"]:
            cases.append((f"matrix-a{active}-{workload}", active, workload, "complete", 30, 5, 5, False))
    for archive in ["backlog", "failed"]:
        cases.append(("matrix-a250-" + archive, 250, "metadata", archive, 30, 5, 5, True))
else:
    cases = [(f"primary-r{round_number}", 250, "counters", "complete", 300, 30, 0, False) for round_number in [1, 2, 3]]
save(OUT / "execution-contract.json", {"driver_argv": sys.argv, "driver_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(), "runner_pid": os.getpid(), "identities": identities, "source_manifest": str(source_manifest), "source_manifest_sha256": hashlib.sha256(source_manifest.read_bytes()).hexdigest(), "cases": cases, "variant_order": ["baseline", "candidate"], "expected_native_invocations": len(cases) * 2, "all_application_file_attribution": "UNVERIFIED; no system trace; null is not zero", "failure_policy": "retain every native exit; continue remaining fixed cases; final summary and nonzero exit report failures"})
pairs = {}
for case in cases:
    pair = {variant: measure(*case, variant) for variant in ["baseline", "candidate"]}
    pairs[case[0]] = pair
    save(OUT / "results.json", pairs)
failures = []
for name, pair in pairs.items():
    for variant, item in pair.items():
        failures.extend(name + "/" + variant + ": " + error for error in item["receipt"]["validation_errors"])
    a, b = pair["baseline"]["report"], pair["candidate"]["report"]
    if isinstance(a, dict) and isinstance(b, dict):
        for key in ["fixture_hash", "local_utc_offset_seconds", "platform"]:
            if a.get(key) != b.get(key):
                failures.append(name + " pair mismatch: " + key)
gates = []
def gate(label, before, after, limit):
    value = ratio(before, after)
    gates.append({"gate": label, "before": before, "candidate": after, "ratio": value, "limit": limit, "status": "UNVERIFIED" if value is None else ("PASS" if value <= limit else "FAIL")})
if not failures and args.set == "matrix":
    definitions = [("F1", "a1000-metadata", "ingest"), ("F2", "a250-backlog", "ingest"), ("F3", "a50-unchanged", "ingest"), ("F4", "a250-failed", "ingest"), ("F5", "a250-metadata", "ingest"), ("F6", "a1000-metadata", "private"), ("F7", "a1000-unchanged", "private"), ("F8", "a50-metadata", "writes"), ("F9", "a250-failed", "writes"), ("F10", "a1000-metadata", "writes"), ("F11", "a250-backlog", "cpu")]
    def metric(report, kind):
        if kind == "ingest": return report["latency"]["facade_ingest_including_durable_commit"]["p95_ms"]
        if kind == "private": return report["native_private_bytes"]["p95"]
        if kind == "writes": return report["sqlite_application_file_write_bytes"]
        return report["native_cpu_seconds"]
    for label, scene, kind in definitions:
        pair = pairs["matrix-" + scene]
        gate(label, metric(pair["baseline"]["report"], kind), metric(pair["candidate"]["report"], kind), 1.10)
    for name, pair in pairs.items():
        gate("F12/" + name, pair["baseline"]["report"]["latency"]["report_first_reader"]["p95_ms"], pair["candidate"]["report"]["latency"]["report_first_reader"]["p95_ms"], 1.10)
    gates.append({"gate": "F12 nonempty production oracle", "status": "UNVERIFIED", "reason": "Original facade reader windows have no reported row counts or exact totals. Separate final-identity nonempty production evidence must be reviewed; parameter fixture_hash is not a result oracle."})
elif not failures:
    for field, limit, label in [("native_cpu_seconds", 0.70, "AC7 CPU"), ("sqlite_application_file_write_bytes", 0.50, "AC7 SQLite subset only"), ("all_application_file_write_bytes", 0.50, "AC7 all application files")]:
        totals = {}
        for variant in identities:
            values = [pair[variant]["report"].get(field) for pair in pairs.values()]
            totals[variant] = sum(values) if all(number(value) for value in values) else None
        gate(label, totals["baseline"], totals["candidate"], limit)
summary = {"set": args.set, "expected_invocations": len(cases) * 2, "recorded_invocations": sum(len(pair) for pair in pairs.values()), "execution_failures": failures, "gates": gates, "completed_utc": datetime.now(timezone.utc).isoformat(), "limits": ["original matrix starts on a minute boundary and queries at +10..+35 seconds; default_auto_report_query retains endpoints and service div_euclid(60) makes the current Raw minute window empty; separate nonempty evidence required", "native measurements do not cover WebView or installed background workers"]}
save(OUT / "summary.json", summary)
print(json.dumps(summary), flush=True)
sys.exit(2 if failures or any(g["status"] == "FAIL" for g in gates) else 3 if any(g["status"] == "UNVERIFIED" for g in gates) else 0)

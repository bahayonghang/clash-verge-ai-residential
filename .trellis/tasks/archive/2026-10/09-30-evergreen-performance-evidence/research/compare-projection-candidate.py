"""Compare frozen readers on the same already-read generated database."""
from datetime import datetime, timezone
from pathlib import Path
import argparse
import gzip
import hashlib
import json
import os
import re
import statistics
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent
parser = argparse.ArgumentParser()
parser.add_argument("--candidate-directory", default="projection-cache-20260930")
parser.add_argument("--before-directory", default="build-20260930")
parser.add_argument("--difference-description", default="network unknown contract restoration plus projection cache and regression fixture")
parser.add_argument("--output-directory", default="projection-cache-comparison-20260930")
parser.add_argument("--rounds", type=int, choices=[1, 2, 3], default=3)
parser.add_argument("--skip-production", action="store_true")
args = parser.parse_args()
OUT = ROOT / args.output_directory
IDENTITIES = {variant: json.loads((ROOT / directory / "executable-identity.json").read_text())["candidate-library-tests.exe"] for variant, directory in [("before", args.before_directory), ("cache", args.candidate_directory)]}
DB = Path("C:/Users/lyh/AppData/Local/Temp/resiwatch-resource-measure-20260924-raw-fold/corpus-a250-30d/monitor.sqlite3")
START, END = 1787184000, 1789776000
PROBE = "c3::service::raw_stage_probe_tests::isolated_nonempty_report_stage_proof"
FOLD = "c3::raw_fold::tests::isolated_raw_fold_stage_proof"


def save(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + chr(10), encoding="utf-8")


def run(label, variant, name, overrides, output):
    argv = [IDENTITIES[variant]["path"], "--exact", name, "--ignored", "--nocapture", "--test-threads=1"]
    receipt = {"argv": argv, "environment_overrides": overrides, "started_utc": datetime.now(timezone.utc).isoformat(), "status": "running", "variant": variant}
    save(OUT / (label + ".receipt.json"), receipt)
    print(json.dumps({"start": label}), flush=True)
    clock = time.perf_counter()
    result = subprocess.run(argv, capture_output=True, env=dict(os.environ, **overrides))
    for stream, data in [("stdout", result.stdout), ("stderr", result.stderr)]:
        (OUT / (label + "." + stream + ".raw.gz")).write_bytes(gzip.compress(data, mtime=0))
        (OUT / (label + "." + stream + ".log")).write_text(chr(10).join(line.rstrip() for line in data.decode("utf-8", errors="replace").splitlines()) + chr(10), encoding="utf-8")
        receipt[stream + "_raw_sha256"] = hashlib.sha256(data).hexdigest()
    executed = bool(re.search(rb"(?m)^running 1 test\r?$", result.stdout))
    receipt.update({"status": "finished", "finished_utc": datetime.now(timezone.utc).isoformat(), "wall_seconds": time.perf_counter() - clock, "exit_code": result.returncode, "ran_exactly_one_test": executed, "json_exists": output.exists()})
    save(OUT / (label + ".receipt.json"), receipt)
    assert result.returncode == 0 and executed and output.exists(), receipt
    data = json.loads(output.read_text())
    print(json.dumps({"finished": label, "wall_seconds": receipt["wall_seconds"], "projection_ms": data.get("projection_ms"), "scan_ms": data.get("scan_ms"), "production_first_read_ms": data.get("production_first_read_ms"), "production_first_read": data.get("production_first_read")}), flush=True)
    return data


assert not OUT.exists()
OUT.mkdir()
save(OUT / "driver-identity.json", {"argv": sys.argv, "sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest()})
for variant, identity in IDENTITIES.items():
    assert hashlib.sha256(Path(identity["path"]).read_bytes()).hexdigest().upper() == identity["sha256"]
    listing = subprocess.run([identity["path"], "--list"], capture_output=True)
    assert listing.returncode == 0 and (PROBE + ": test").encode() in listing.stdout and (FOLD + ": test").encode() in listing.stdout
    save(OUT / (variant + "-test-list.json"), {"argv": [identity["path"], "--list"], "exit_code": listing.returncode, "stdout": listing.stdout.decode(), "stderr": listing.stderr.decode()})
save(OUT / "boundary.json", {"identities": IDENTITIES, "database": str(DB), "cache": "database already read by previous production/stage/oracle/hash checks; no cache eviction", "source_difference": args.difference_description, "all_query": {"timezone": "local", "previous_equal_window": True}, "residential_query": {"timezone": "UTC", "previous_equal_window": False}, "production_now_utc": "window end", "stage_deadline_enabled": False, "formal_matrix_or_capacity_gate": False})
stages = []
for round_number in range(1, args.rounds + 1):
    order = ["before", "cache"] if round_number % 2 else ["cache", "before"]
    for filter_name in ["all", "residential"]:
        for variant in order:
            label = f"r{round_number}-{filter_name}-{variant}"
            output = OUT / (label + ".json")
            data = run(label, variant, FOLD, {"RESIWATCH_RAW_FOLD_STAGE_DB": str(DB), "RESIWATCH_RAW_FOLD_STAGE_OUT": str(output), "RESIWATCH_RAW_FOLD_START": str(START), "RESIWATCH_RAW_FOLD_END": str(END), "RESIWATCH_RAW_FOLD_FILTER": filter_name}, output)
            expected = (2160000, 151200035, 507599545) if filter_name == "all" else (1620000, 113399990, 380699625)
            assert tuple(data[key] for key in ["connection_count", "upload", "download"]) == expected
            stages.append({"round": round_number, "variant": variant, "filter": filter_name, "report": data})
            save(OUT / "stage-summary.json", stages)
comparisons = []
for filter_name in ["all", "residential"]:
    item = {"filter": filter_name, "samples_per_side": args.rounds}
    for field in ["projection_ms", "scan_ms"]:
        values = {variant: [x["report"][field] for x in stages if x["variant"] == variant and x["filter"] == filter_name] for variant in IDENTITIES}
        before, after = statistics.median(values["before"]), statistics.median(values["cache"])
        item[field] = {"samples": values, "before_median": before, "cache_median": after, "ratio": after / before}
    comparisons.append(item)
save(OUT / "comparison.json", comparisons)
productions = []
production_windows = [] if args.skip_production else [("minute", START + 60, (250, 3497, 11725)), ("30d", END, (2160000, 151200035, 507599545))]
for window, end, oracle in production_windows:
    for variant in ["before", "cache"]:
        label = f"production-{window}-{variant}"
        output = OUT / (label + ".json")
        data = run(label, variant, PROBE, {"RESIWATCH_NONEMPTY_STAGE_DB": str(DB), "RESIWATCH_NONEMPTY_STAGE_OUT": str(output), "RESIWATCH_NONEMPTY_STAGE_START": str(START), "RESIWATCH_NONEMPTY_STAGE_END": str(end)}, output)
        first = data["production_first_read"]
        passed = first.get("status") == "ok" and first.get("tier") == "Raw" and data["production_first_read_ms"] < 10000 and tuple(first.get(key) for key in ["connection_count", "upload", "download"]) == oracle and first.get("series_rows", 0) > 0
        productions.append({"window": window, "variant": variant, "pass": passed, "production_first_read": first, "production_first_read_ms": data["production_first_read_ms"]})
        save(OUT / "production-validation.json", productions)
print(json.dumps({"comparisons": comparisons, "productions": productions}), flush=True)

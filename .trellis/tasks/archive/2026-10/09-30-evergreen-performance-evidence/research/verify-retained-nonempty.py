"""Bind exact nonempty production checks to the retained candidate identity."""
from pathlib import Path
import hashlib
import json
import os
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
OUT = ROOT / "retained-nonempty-20260930"
assert not OUT.exists()
OUT.mkdir()
identity = json.loads((ROOT / "candidate-retained-20260930/executable-identity.json").read_text())["candidate-library-tests.exe"]
assert hashlib.sha256(Path(identity["path"]).read_bytes()).hexdigest().upper() == identity["sha256"]
probe = "c3::service::raw_stage_probe_tests::isolated_nonempty_report_stage_proof"
listing = subprocess.run([identity["path"], "--list"], capture_output=True)
assert listing.returncode == 0 and (probe + ": test").encode() in listing.stdout
summaries = []
for label, end, oracle in [("minute", 1787184060, (250, 3497, 11725)), ("30d", 1789776000, (2160000, 151200035, 507599545))]:
    output = OUT / (label + ".json")
    overrides = {"RESIWATCH_NONEMPTY_STAGE_DB": "C:/Users/lyh/AppData/Local/Temp/resiwatch-resource-measure-20260924-raw-fold/corpus-a250-30d/monitor.sqlite3", "RESIWATCH_NONEMPTY_STAGE_OUT": str(output), "RESIWATCH_NONEMPTY_STAGE_START": "1787184000", "RESIWATCH_NONEMPTY_STAGE_END": str(end)}
    (OUT / (label + ".contract.json")).write_text(json.dumps({"identity": identity, "overrides": overrides, "cache": "same database read by earlier production, stage, oracle and hash; no eviction", "now_utc": end}, indent=2) + chr(10), encoding="utf-8")
    argv = [identity["path"], "--exact", probe, "--ignored", "--nocapture", "--test-threads=1"]
    result = subprocess.run([sys.executable, "-X", "utf8", str(ROOT / "run-recorded.py"), str(OUT), label, *argv], env=dict(os.environ, **overrides))
    stdout = (OUT / (label + ".stdout.log")).read_text()
    executed = re.search(r"(?m)^running 1 test$", stdout) is not None
    assert result.returncode == 0 and executed and output.exists()
    data = json.loads(output.read_text())
    first = data["production_first_read"]
    passed = first.get("status") == "ok" and first.get("tier") == "Raw" and data["production_first_read_ms"] < 10000 and tuple(first.get(key) for key in ["connection_count", "upload", "download"]) == oracle and first.get("series_rows", 0) > 0
    summaries.append({"window": label, "pass": passed, "production_first_read_ms": data["production_first_read_ms"], "production_first_read": first})
    (OUT / "summary.json").write_text(json.dumps(summaries, indent=2) + chr(10), encoding="utf-8")
print(json.dumps(summaries), flush=True)
sys.exit(0 if all(item["pass"] for item in summaries) else 2)

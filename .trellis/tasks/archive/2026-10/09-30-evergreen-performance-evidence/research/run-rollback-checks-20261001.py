"""Record the approved rollback checks against the fixed source manifest."""
from datetime import datetime, timezone
from pathlib import Path
import gzip
import hashlib
import json
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent
OUT = ROOT / "rollback-cache-20261001"
REPO = ROOT.parents[3]
MANIFEST = OUT / "source-manifest-before-checks.json"
source = json.loads(MANIFEST.read_text())
source_hash = hashlib.sha256(MANIFEST.read_bytes()).hexdigest().upper()
for relative, expected in source.items():
    path = REPO / relative
    assert path.stat().st_size == expected["bytes"]
    assert hashlib.sha256(path.read_bytes()).hexdigest().upper() == expected["sha256"]

commands = [
    ("rust-fmt", ["cargo", "fmt", "--manifest-path", "residential-monitor/src-tauri/Cargo.toml", "--check"]),
    ("raw-fold-tests", ["cargo", "test", "--locked", "--manifest-path", "residential-monitor/src-tauri/Cargo.toml", "--lib", "c3::raw_fold::"]),
    ("report-service-tests", ["cargo", "test", "--locked", "--manifest-path", "residential-monitor/src-tauri/Cargo.toml", "--lib", "c3::service::"]),
    ("rust-clippy", ["cargo", "clippy", "--locked", "--manifest-path", "residential-monitor/src-tauri/Cargo.toml", "--workspace", "--all-targets", "--", "-D", "warnings"]),
    ("just-ci", ["just", "ci"]),
]


def save(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


results = []
for label, argv in commands:
    path = OUT / (label + ".receipt.json")
    assert not path.exists(), path
    receipt = {"argv": argv, "resolved_executable": shutil.which(argv[0]), "cwd": str(REPO), "source_manifest_sha256": source_hash, "started_utc": datetime.now(timezone.utc).isoformat(), "status": "running"}
    started = time.perf_counter()
    process = subprocess.Popen(argv, cwd=REPO, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    receipt["pid"] = process.pid
    save(path, receipt)
    print(json.dumps({"start": label, "pid": process.pid}), flush=True)
    stdout, stderr = process.communicate()
    for stream, data in [("stdout", stdout), ("stderr", stderr)]:
        (OUT / (label + "." + stream + ".raw.gz")).write_bytes(gzip.compress(data, mtime=0))
        (OUT / (label + "." + stream + ".log")).write_text(data.decode("utf-8", errors="replace"), encoding="utf-8")
        receipt[stream + "_raw_sha256"] = hashlib.sha256(data).hexdigest()
    receipt.update(status="finished", finished_utc=datetime.now(timezone.utc).isoformat(), wall_seconds=time.perf_counter() - started, exit_code=process.returncode)
    save(path, receipt)
    results.append(receipt)
    save(OUT / "checks-results.json", results)
    print(json.dumps({"finished": label, "exit_code": process.returncode, "wall_seconds": receipt["wall_seconds"]}), flush=True)
    if process.returncode:
        print(stdout.decode("utf-8", errors="replace")[-2500:], flush=True)
        print(stderr.decode("utf-8", errors="replace")[-2500:], flush=True)

summary = {"completed_utc": datetime.now(timezone.utc).isoformat(), "source_manifest_sha256": source_hash, "results": [{key: item[key] for key in ["argv", "exit_code", "wall_seconds"]} for item in results], "status": "PASS" if all(item["exit_code"] == 0 for item in results) else "FAIL", "limits": ["No performance, capacity, installed application, hosted CI, or physical cold-cache result applies to this rollback identity."]}
save(OUT / "checks-summary.json", summary)
print(json.dumps(summary), flush=True)
sys.exit(0 if summary["status"] == "PASS" else 2)

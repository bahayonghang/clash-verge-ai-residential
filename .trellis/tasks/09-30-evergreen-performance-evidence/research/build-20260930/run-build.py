"""Build frozen benchmark inputs and retain every command receipt."""
from pathlib import Path
from datetime import datetime, timezone
import gzip
import hashlib
import json
import os
import shutil
import subprocess
import sys
import time

EVIDENCE = Path(__file__).resolve().parent
STATE = json.loads((EVIDENCE / "state.json").read_text(encoding="utf-8"))
RUN = Path(STATE["run_root"])
REPO = Path(STATE["repo"])
BASE = Path(STATE["baseline_source"])
TARGET = "x86_64-pc-windows-msvc"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest().upper()


def save(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + chr(10), encoding="utf-8")


def run(label, argv):
    receipt_path = EVIDENCE / (label + ".json")
    assert not receipt_path.exists(), f"Refuse to overwrite {receipt_path}"
    out_path = RUN / (label + ".stdout.raw")
    err_path = RUN / (label + ".stderr.raw")
    started = datetime.now(timezone.utc).isoformat()
    receipt = {"label": label, "argv": argv, "cwd": str(REPO), "started_utc": started, "status": "running", "environment_override": {"RUSTUP_AUTO_INSTALL": "0"}}
    save(receipt_path, receipt)
    print(json.dumps({"start": label, "argv": argv}), flush=True)
    clock = time.perf_counter()
    with out_path.open("xb") as stdout, err_path.open("xb") as stderr:
        result = subprocess.run(argv, cwd=REPO, stdout=stdout, stderr=stderr, env=dict(os.environ, RUSTUP_AUTO_INSTALL="0"))
    logs = {}
    for stream, path in [("stdout", out_path), ("stderr", err_path)]:
        data = path.read_bytes()
        zipped = EVIDENCE / (label + "." + stream + ".raw.gz")
        zipped.write_bytes(gzip.compress(data, mtime=0))
        readable = EVIDENCE / (label + "." + stream + ".log")
        readable.write_text(chr(10).join(line.rstrip() for line in data.decode("utf-8", errors="replace").splitlines()) + chr(10), encoding="utf-8")
        logs[stream] = {"raw_path": str(path), "raw_sha256": sha(path), "gzip": zipped.name, "gzip_sha256": sha(zipped), "readable": readable.name, "readable_sha256": sha(readable)}
    receipt.update({"status": "finished", "finished_utc": datetime.now(timezone.utc).isoformat(), "wall_seconds": time.perf_counter() - clock, "exit_code": result.returncode, "logs": logs})
    save(receipt_path, receipt)
    print(json.dumps({"finished": label, "exit_code": result.returncode, "wall_seconds": receipt["wall_seconds"]}), flush=True)
    if result.returncode:
        print(err_path.read_text(encoding="utf-8", errors="replace")[-12000:], flush=True)
        sys.exit(result.returncode)
    return out_path


baseline_args = ["cargo", "+1.98.0", "build", "--locked", "--release", "--target", TARGET, "--manifest-path", str(BASE / "residential-monitor/src-tauri/Cargo.toml"), "--target-dir", STATE["baseline_target"], "--bin", "monitor-bench"]
candidate_args = ["cargo", "+1.98.0", "build", "--locked", "--release", "--target", TARGET, "--manifest-path", str(REPO / "residential-monitor/src-tauri/Cargo.toml"), "--target-dir", STATE["candidate_target"], "--bin", "monitor-bench", "--bin", "monitor-db"]
test_args = ["cargo", "+1.98.0", "test", "--locked", "--release", "--target", TARGET, "--manifest-path", str(REPO / "residential-monitor/src-tauri/Cargo.toml"), "--target-dir", STATE["candidate_target"], "--lib", "--no-run", "--message-format=json"]
run("baseline-build", baseline_args)
run("candidate-build", candidate_args)
test_output = run("candidate-test-build", test_args)
test_artifacts = []
for line in test_output.read_text(encoding="utf-8").splitlines():
    try:
        item = json.loads(line)
    except ValueError:
        continue
    if item.get("reason") == "compiler-artifact" and item.get("profile", {}).get("test") and item.get("executable"):
        test_artifacts.append(item)
assert len(test_artifacts) == 1, test_artifacts
destination = Path(STATE["executables"])
destination.mkdir()
sources = {"baseline-monitor-bench.exe": Path(STATE["baseline_target"]) / TARGET / "release/monitor-bench.exe", "candidate-monitor-bench.exe": Path(STATE["candidate_target"]) / TARGET / "release/monitor-bench.exe", "candidate-monitor-db.exe": Path(STATE["candidate_target"]) / TARGET / "release/monitor-db.exe", "candidate-library-tests.exe": Path(test_artifacts[0]["executable"])}
executables = {}
for name, source in sources.items():
    target = destination / name
    shutil.copy2(source, target)
    assert sha(source) == sha(target)
    executables[name] = {"source": str(source), "path": str(target), "bytes": target.stat().st_size, "sha256": sha(target)}
save(EVIDENCE / "test-compiler-artifact.json", test_artifacts[0])
save(EVIDENCE / "executable-identity.json", executables)
for label, source, before in [("baseline", BASE, "baseline-instrumented-manifest.json"), ("candidate", REPO, "candidate-source-before.json")]:
    expected = json.loads((EVIDENCE / before).read_text(encoding="utf-8"))
    actual = {rel: {"bytes": (source / rel).stat().st_size, "sha256": sha(source / rel)} for rel in expected}
    save(EVIDENCE / (label + "-source-after.json"), actual)
    assert actual == expected, f"{label} source changed during build"
print(json.dumps({"builds_complete": True, "executables": executables}), flush=True)

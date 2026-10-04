"""Check and freeze the approved first projection candidate."""
from pathlib import Path
import hashlib
import json
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
BUILD = ROOT / "build-20260930"
STATE = json.loads((BUILD / "state.json").read_text())
REPO = Path(STATE["repo"])
OUT = ROOT / "projection-cache-20260930"
TARGET = "x86_64-pc-windows-msvc"
RECORDER = ROOT / "run-recorded.py"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest().upper()


def save(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + chr(10), encoding="utf-8")


def call(label, argv):
    result = subprocess.run([sys.executable, "-X", "utf8", str(RECORDER), str(OUT), label, *argv], cwd=REPO)
    if result.returncode:
        sys.exit(result.returncode)


prior = json.loads((BUILD / "candidate-source-before.json").read_text())
before = {name: {"bytes": (REPO / name).stat().st_size, "sha256": sha(REPO / name)} for name in prior}
changed = [name for name in prior if prior[name] != before[name]]
assert changed == ["residential-monitor/src-tauri/src/c3/raw_fold.rs"], changed
save(OUT / "candidate-source-before.json", before)
call("cache-fmt", ["cargo", "+1.98.0", "fmt", "--manifest-path", "residential-monitor/src-tauri/Cargo.toml", "--check"])
common = ["--locked", "--release", "--target", TARGET, "--manifest-path", "residential-monitor/src-tauri/Cargo.toml", "--target-dir", STATE["candidate_target"]]
call("cache-clippy", ["cargo", "+1.98.0", "clippy", *common, "--workspace", "--all-targets", "--", "-D", "warnings"])
call("cache-bin-build", ["cargo", "+1.98.0", "build", *common, "--bin", "monitor-bench", "--bin", "monitor-db"])
call("cache-test-build", ["cargo", "+1.98.0", "test", *common, "--lib", "--no-run", "--message-format=json"])
artifacts = []
for line in (OUT / "cache-test-build.stdout.log").read_text().splitlines():
    try:
        item = json.loads(line)
    except ValueError:
        continue
    if item.get("reason") == "compiler-artifact" and item.get("profile", {}).get("test") and item.get("executable"):
        artifacts.append(item)
assert len(artifacts) == 1
save(OUT / "test-compiler-artifact.json", artifacts[0])
destination = Path(STATE["executables"]) / "projection-cache-v1"
assert not destination.exists()
destination.mkdir()
sources = {"candidate-monitor-bench.exe": Path(STATE["candidate_target"]) / TARGET / "release/monitor-bench.exe", "candidate-monitor-db.exe": Path(STATE["candidate_target"]) / TARGET / "release/monitor-db.exe", "candidate-library-tests.exe": Path(artifacts[0]["executable"])}
identity = {}
for name, source in sources.items():
    target = destination / name
    shutil.copy2(source, target)
    assert sha(source) == sha(target)
    identity[name] = {"path": str(target), "source": str(source), "bytes": target.stat().st_size, "sha256": sha(target)}
save(OUT / "executable-identity.json", identity)
after = {name: {"bytes": (REPO / name).stat().st_size, "sha256": sha(REPO / name)} for name in before}
save(OUT / "candidate-source-after.json", after)
assert before == after
test = identity["candidate-library-tests.exe"]["path"]
call("cache-service-tests", [test, "c3::service::", "--nocapture", "--test-threads=1"])
call("cache-bench-tests", [test, "bench::", "--nocapture", "--test-threads=1"])
call("cache-diff-check", ["git", "diff", "--check"])
print(json.dumps({"cache_candidate_frozen": True, "executables": identity}), flush=True)

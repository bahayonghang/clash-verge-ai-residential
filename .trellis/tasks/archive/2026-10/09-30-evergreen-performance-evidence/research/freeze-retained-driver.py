"""Verify the approved candidate, synchronize measurement code, and freeze both binaries."""
from pathlib import Path
import difflib
import hashlib
import json
import os
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
BUILD = ROOT / "build-20260930"
STATE = json.loads((BUILD / "state.json").read_text())
REPO, BASE = Path(STATE["repo"]), Path(STATE["baseline_source"])
OUT = ROOT / "candidate-retained-20260930"
TARGET = "x86_64-pc-windows-msvc"
RECORDER = ROOT / "run-recorded.py"
assert not OUT.exists()
OUT.mkdir()

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest().upper()

def save(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + chr(10), encoding="utf-8")

def manifest(root, prior):
    return {name: {"bytes": (root / name).stat().st_size, "sha256": sha(root / name)} for name in prior}

def call(label, argv, env=None):
    result = subprocess.run([sys.executable, "-X", "utf8", str(RECORDER), str(OUT), label, *argv], cwd=REPO, env=env)
    if result.returncode:
        sys.exit(result.returncode)

baseline_prior = json.loads((ROOT / "candidate-final-20260930/baseline-source-after.json").read_text())
assert manifest(BASE, baseline_prior) == baseline_prior
candidate_prior = json.loads((BUILD / "candidate-source-before.json").read_text())
candidate_before = manifest(REPO, candidate_prior)
changed = {name for name in candidate_prior if candidate_prior[name] != candidate_before[name]}
assert changed == {"residential-monitor/src-tauri/src/c3/raw_fold.rs", "residential-monitor/src-tauri/src/bench/facade.rs"}, changed
relative = "residential-monitor/src-tauri/src/bench/facade.rs"
new = (REPO / relative).read_bytes()
assert (BASE / relative).read_bytes() == new
baseline_before = baseline_prior
save(OUT / "baseline-source-before.json", baseline_before)
save(OUT / "candidate-source-before.json", candidate_before)
(OUT / "raw_fold-final.rs").write_bytes((REPO / "residential-monitor/src-tauri/src/c3/raw_fold.rs").read_bytes())
(OUT / "facade-final.rs").write_bytes(new)
call("final-fmt", ["cargo", "+1.98.0", "fmt", "--manifest-path", "residential-monitor/src-tauri/Cargo.toml", "--check"])
common = ["--locked", "--release", "--target", TARGET, "--manifest-path", "residential-monitor/src-tauri/Cargo.toml", "--target-dir", STATE["candidate_target"]]
call("final-release-clippy", ["cargo", "+1.98.0", "clippy", *common, "--workspace", "--all-targets", "--", "-D", "warnings"])
ci_steps = OUT / "just-ci-steps"
call("final-just-ci", ["just", "--shell", sys.executable, "--clear-shell-args", "--shell-arg", str(ROOT / "record-just-shell.py"), "ci"], env=dict(os.environ, RESIWATCH_T05_CI_LOG_DIR=str(ci_steps)))
step_receipts = [json.loads(path.read_text()) for path in sorted(ci_steps.glob("*.json"))]
assert [item["argv"][-1] for item in step_receipts] == [
    "node scripts/sync-monitor-version.js --check",
    "npm --prefix residential-monitor ci",
    "npm --prefix residential-monitor run check",
    "cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check",
    "cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings",
    "cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace",
    "npm run check:secrets",
    "npm run ci",
]
assert all(item["exit_code"] == 0 for item in step_receipts)
save(OUT / "just-ci-step-summary.json", step_receipts)
call("final-candidate-build", ["cargo", "+1.98.0", "build", *common, "--bin", "monitor-bench", "--bin", "monitor-db"])
call("final-test-build", ["cargo", "+1.98.0", "test", *common, "--lib", "--no-run", "--message-format=json"])
artifacts = []
for line in (OUT / "final-test-build.stdout.log").read_text().splitlines():
    try:
        item = json.loads(line)
    except ValueError:
        continue
    if item.get("reason") == "compiler-artifact" and item.get("profile", {}).get("test") and item.get("executable"):
        artifacts.append(item)
assert len(artifacts) == 1
save(OUT / "test-compiler-artifact.json", artifacts[0])
destination = Path(STATE["executables"]) / "candidate-retained-20260930"
assert not destination.exists()
destination.mkdir()
sources = {"baseline-monitor-bench.exe": Path(json.loads((ROOT / "candidate-final-20260930/executable-identity.json").read_text())["baseline-monitor-bench.exe"]["path"]), "candidate-monitor-bench.exe": Path(STATE["candidate_target"]) / TARGET / "release/monitor-bench.exe", "candidate-monitor-db.exe": Path(STATE["candidate_target"]) / TARGET / "release/monitor-db.exe", "candidate-library-tests.exe": Path(artifacts[0]["executable"])}
identity = {}
for name, source in sources.items():
    target = destination / name
    shutil.copy2(source, target)
    assert sha(source) == sha(target)
    variant = "baseline" if name.startswith("baseline-") else "candidate"
    identity[name] = {"path": str(target), "source": str(source), "bytes": target.stat().st_size, "sha256": sha(target), "source_manifest_sha256": sha(OUT / (variant + "-source-before.json"))}
save(OUT / "executable-identity.json", identity)
for label, root, before in [("baseline", BASE, baseline_before), ("candidate", REPO, candidate_before)]:
    after = manifest(root, before)
    save(OUT / (label + "-source-after.json"), after)
    assert before == after
test = identity["candidate-library-tests.exe"]["path"]
for label, selector in [("raw-fold", "c3::raw_fold::"), ("service", "c3::service::"), ("bench", "bench::")]:
    call("final-release-" + label, [test, selector, "--nocapture", "--test-threads=1"])
print(json.dumps({"final_candidate_frozen": True, "identities": identity}), flush=True)

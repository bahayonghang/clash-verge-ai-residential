"""Verify matched frozen executables and actual writer pragmas before formal load."""
import argparse
import ast
import hashlib
import json
import math
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
parser = argparse.ArgumentParser()
parser.add_argument("--candidate-directory", required=True)
parser.add_argument("--output-directory", required=True)
args = parser.parse_args()
frozen = ROOT / args.candidate_directory
OUT = ROOT / args.output_directory
STATE = json.loads((ROOT / "build-20260930/state.json").read_text())
DATA = Path(STATE["run_root"]) / args.output_directory
assert not OUT.exists() and not DATA.exists()
OUT.mkdir(); DATA.mkdir()
tree = ast.parse((ROOT / "run-formal-replay.py").read_text())
scope = {"math": math}
exec(compile(ast.Module(body=[node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name in ["number", "validate_report"]], type_ignores=[]), "run-formal-replay.py", "exec"), scope)
identities = json.loads((frozen / "executable-identity.json").read_text())
reports = {}
for variant in ["baseline", "candidate"]:
    identity = identities[variant + "-monitor-bench.exe"]
    assert hashlib.sha256(Path(identity["path"]).read_bytes()).hexdigest().upper() == identity["sha256"]
    source = STATE["baseline_revision"] if variant == "baseline" else "manifest-sha256:" + hashlib.sha256((frozen / "candidate-source-before.json").read_bytes()).hexdigest()
    options = {"active": 8, "hz": 1, "duration_secs": 3, "warmup_secs": 0, "workload": "counters", "archive": "complete", "metadata_change_percent": 100, "query_every_frames": 1, "seed": 20260919, "start_utc": 1800001800, "period_rule": False, "virtual_time": False, "source_revision": source, "dir": str(DATA / variant)}
    argv = [identity["path"], "replay-facade"]
    for name, value in options.items():
        if type(value) is not bool:
            argv += ["--" + name.replace("_", "-"), str(value)]
    result = subprocess.run([sys.executable, "-X", "utf8", str(ROOT / "run-recorded.py"), str(OUT), variant, *argv], cwd=STATE["repo"])
    assert result.returncode == 0
    report = json.loads((OUT / (variant + ".stdout.log")).read_text(encoding="utf-8"))
    errors = scope["validate_report"](report, options, identity, report["process_id"])
    assert not errors, errors
    reports[variant] = report
for key in ["fixture_hash", "platform", "local_utc_offset_seconds"]:
    assert reports["baseline"][key] == reports["candidate"][key]
summary = {"status": "PASS", "identities": identities, "reports": reports, "boundary": "A8 real-time 3-tick instrumentation smoke; not a formal performance or nonempty oracle gate"}
(OUT / "summary.json").write_text(json.dumps(summary, ensure_ascii=False, indent=2) + chr(10), encoding="utf-8")
print(json.dumps({"status": "PASS", "initial_and_final_writer": "WAL/FULL", "samples_per_side": 3, "fixture_equal": True}), flush=True)

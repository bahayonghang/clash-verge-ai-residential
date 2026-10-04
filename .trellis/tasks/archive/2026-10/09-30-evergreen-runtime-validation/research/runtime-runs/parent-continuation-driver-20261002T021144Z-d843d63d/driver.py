from datetime import datetime, timezone
import json
from pathlib import Path
import subprocess
import sys

sys.stdout.reconfigure(encoding="utf-8")
ROOT = Path(__file__).resolve().parents[6]
RESEARCH = ROOT / ".trellis/tasks/09-30-evergreen-runtime-validation/research"
CAPTURE = RESEARCH / "runtime-capture.py"
DESTINATION = Path(__file__).resolve().parent
HARNESS = sys.argv[1]
if HARNESS not in ("kimi", "omp"):
    raise SystemExit("Only separately approved unfinished harnesses")
receipt = {"harness": HARNESS, "phase": "parent", "child_authorized": False, "started_utc": datetime.now(timezone.utc).isoformat(), "steps": []}

def save():
    (DESTINATION / (HARNESS + ".driver.json")).write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

def run(label, args):
    started = datetime.now(timezone.utc).isoformat()
    argv = [sys.executable, str(CAPTURE), *args]
    item = {"label": label, "argv": ["python", CAPTURE.relative_to(ROOT).as_posix(), *args], "started_utc": started, "native_exit": "RUNNING"}
    receipt["steps"].append(item)
    save()
    print(json.dumps({"phase": label, "status": "STARTED"}), flush=True)
    with (DESTINATION / (HARNESS + "." + label + ".stdout.txt")).open("wb") as stdout, (DESTINATION / (HARNESS + "." + label + ".stderr.txt")).open("wb") as stderr:
        process = subprocess.Popen(argv, cwd=ROOT, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=stderr, shell=False)
        item["driver_child_pid"] = process.pid
        save()
        for line in iter(process.stdout.readline, b""):
            stdout.write(line)
            stdout.flush()
            if label == "execute":
                print(line.decode("utf-8", errors="replace").rstrip(), flush=True)
        process.stdout.close()
        exit_code = process.wait()
    item.update(native_exit=exit_code, ended_utc=datetime.now(timezone.utc).isoformat())
    save()
    print(json.dumps({"phase": label, "native_exit": exit_code}), flush=True)
    return exit_code

frozen = json.loads((RESEARCH / "runtime-candidate.json").read_text(encoding="utf-8"))
identity = frozen["snapshot"]["source_snapshot_id"]
if identity != "9a31d79c85465e3213e6f364f670ebac4a45911897e385c74e5909a740778cf9":
    raise SystemExit("Frozen manifest no longer matches authorized snapshot")
receipt["source_snapshot_id"] = identity
receipt["head_is_baseline_only"] = frozen["snapshot"]["head"]
save()
print(json.dumps({"source_snapshot_id": identity, "phase": "parent", "head_is_baseline_only": receipt["head_is_baseline_only"]}), flush=True)
code = run("execute", ["execute", "--phase", "parent", "--harness", HARNESS, "--go", "GO:" + identity])
receipt.update(status="EXECUTOR_ENDED_REQUIRES_EVENT_REVIEW", ended_utc=datetime.now(timezone.utc).isoformat(), outer_exit=code)
save()
raise SystemExit(code)

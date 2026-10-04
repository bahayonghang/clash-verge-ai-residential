"""Keep live driver output while retaining its native exit across shell wrappers."""
from datetime import datetime, timezone
from pathlib import Path
import json
import subprocess
import sys
import time

path = Path(sys.argv[1])
argv = sys.argv[2:]
assert argv and not path.exists()
receipt = {"argv": argv, "cwd": str(Path.cwd()), "status": "running", "started_utc": datetime.now(timezone.utc).isoformat()}
def save():
    path.write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + chr(10), encoding="utf-8")
save()
started = time.perf_counter()
process = subprocess.Popen(argv)
receipt["pid"] = process.pid
save()
code = process.wait()
receipt.update({"status": "finished", "exit_code": code, "finished_utc": datetime.now(timezone.utc).isoformat(), "wall_seconds": time.perf_counter() - started})
save()
print(json.dumps({"driver_exit_code": code, "receipt": str(path)}), flush=True)
sys.exit(code)

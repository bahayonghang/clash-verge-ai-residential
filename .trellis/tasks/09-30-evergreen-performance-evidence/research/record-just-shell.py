"""Preserve the configured PowerShell invocation and record each Just recipe exit."""
from datetime import datetime, timezone
from pathlib import Path
import json
import os
import subprocess
import sys
import time

assert len(sys.argv) == 2, sys.argv
directory = Path(os.environ["RESIWATCH_T05_CI_LOG_DIR"])
directory.mkdir(exist_ok=True)
sequence = len(list(directory.glob("*.json"))) + 1
path = directory / f"{sequence:02d}.json"
assert not path.exists()
argv = ["powershell.exe", "-NoLogo", "-NoProfile", "-Command", sys.argv[1]]
receipt = {"argv": argv, "cwd": str(Path.cwd()), "started_utc": datetime.now(timezone.utc).isoformat(), "status": "running"}
def save():
    path.write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + chr(10), encoding="utf-8")
save()
started = time.perf_counter()
result = subprocess.run(argv)
receipt.update({"status": "finished", "exit_code": result.returncode, "wall_seconds": time.perf_counter() - started, "finished_utc": datetime.now(timezone.utc).isoformat()})
save()
sys.exit(result.returncode)

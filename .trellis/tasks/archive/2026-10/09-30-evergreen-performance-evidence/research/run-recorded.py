"""Record one command without hiding earlier failures."""
from pathlib import Path
from datetime import datetime, timezone
import argparse
import gzip
import hashlib
import json
import os
import subprocess
import sys
import time

parser = argparse.ArgumentParser()
parser.add_argument("directory")
parser.add_argument("label")
parser.add_argument("command", nargs=argparse.REMAINDER)
args = parser.parse_args()
command = args.command[1:] if args.command[:1] == ["--"] else args.command
directory = Path(args.directory).resolve()
directory.mkdir(parents=True, exist_ok=True)
receipt_path = directory / (args.label + ".receipt.json")
assert not receipt_path.exists(), receipt_path
receipt = {"argv": command, "cwd": str(Path.cwd()), "started_utc": datetime.now(timezone.utc).isoformat(), "status": "running", "environment_overrides": {"RUSTUP_AUTO_INSTALL": "0"}}
def save():
    receipt_path.write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + chr(10), encoding="utf-8")
save()
print(json.dumps({"start": args.label, "argv": command}), flush=True)
clock = time.perf_counter()
result = subprocess.run(command, capture_output=True, env=dict(os.environ, RUSTUP_AUTO_INSTALL="0"))
logs = {}
for stream, data in [("stdout", result.stdout), ("stderr", result.stderr)]:
    zipped = directory / (args.label + "." + stream + ".raw.gz")
    zipped.write_bytes(gzip.compress(data, mtime=0))
    readable = directory / (args.label + "." + stream + ".log")
    readable.write_text(chr(10).join(line.rstrip() for line in data.decode("utf-8", errors="replace").splitlines()) + chr(10), encoding="utf-8")
    logs[stream] = {"raw_sha256": hashlib.sha256(data).hexdigest(), "gzip": zipped.name, "readable": readable.name}
receipt.update({"exit_code": result.returncode, "finished_utc": datetime.now(timezone.utc).isoformat(), "wall_seconds": time.perf_counter() - clock, "status": "finished", "logs": logs})
save()
print(json.dumps({"finished": args.label, "exit_code": result.returncode, "wall_seconds": receipt["wall_seconds"]}), flush=True)
print(result.stdout.decode("utf-8", errors="replace")[-7000:], flush=True)
if result.returncode:
    print(result.stderr.decode("utf-8", errors="replace")[-12000:], flush=True)
sys.exit(result.returncode)

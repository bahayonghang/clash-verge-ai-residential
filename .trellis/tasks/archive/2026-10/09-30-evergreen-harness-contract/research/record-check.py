"""记录 T03 单条本地命令的退出码和原始输出。"""

import datetime
import gzip
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

sys.stdout.reconfigure(encoding="utf-8", errors="replace")
root = Path(__file__).resolve().parents[4]
name, command = sys.argv[1:3]
research = Path(__file__).resolve().parent
started = datetime.datetime.now(datetime.timezone.utc).isoformat()
environment = os.environ.copy()
environment["PYTHONIOENCODING"] = "utf-8"
result = subprocess.run(
    ["powershell.exe", "-NoLogo", "-NoProfile", "-Command", command],
    cwd=root, env=environment, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, check=False
)
finished = datetime.datetime.now(datetime.timezone.utc).isoformat()
raw = result.stdout
with gzip.open(research / f"{name}.log.gz", "wb") as output:
    output.write(raw)
readable = re.sub(r"\x1b\[[0-9;?]*[ -/]*[@-~]", "", raw.decode("utf-8", errors="replace"))
readable = "\n".join(line.rstrip() for line in readable.splitlines()) + "\n"
(research / f"{name}.log").write_text(readable, encoding="utf-8", newline="\n")
receipt = {
    "command": command, "cwd": str(root), "started_utc": started,
    "finished_utc": finished, "exit_code": result.returncode,
    "raw_sha256": hashlib.sha256(raw).hexdigest(),
    "readable_normalization": "ANSI removed; trailing line whitespace removed; LF",
}
(research / f"{name}.result.json").write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps(receipt, ensure_ascii=False))
print("\n".join(readable.splitlines()[-18:]))
sys.exit(result.returncode)

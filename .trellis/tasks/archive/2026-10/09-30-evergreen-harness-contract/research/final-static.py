"""核对 T03 正式门之后的文件、空白和任务上下文。"""

import hashlib
import json
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[4]
research = Path(__file__).resolve().parent
snapshot = json.loads((research / "scope-integrity.json").read_text(encoding="utf-8"))
for file, digest in snapshot["sha256"].items():
    assert hashlib.sha256((root / file).read_bytes()).hexdigest() == digest, file

commands = [
    ["git", "diff", "--check"],
    ["python", ".trellis/scripts/task.py", "validate", ".trellis/tasks/09-30-evergreen-harness-contract"],
]
checks = []
for command in commands:
    result = subprocess.run(command, cwd=root, capture_output=True, text=True, encoding="utf-8")
    print(result.stdout)
    print(result.stderr)
    checks.append({"command": command, "exit_code": result.returncode})
    assert result.returncode == 0, command

files = [root / "scripts/check-agent-contract.js", root / "tests/check-agent-contract.test.js"]
files.extend(file for file in research.iterdir() if file.suffix in {".md", ".py", ".json", ".log"})
findings = []
for file in files:
    for number, line in enumerate(file.read_text(encoding="utf-8").splitlines(), start=1):
        if line != line.rstrip():
            findings.append(f"{file.relative_to(root)}:{number}")
assert not findings, findings
record = {
    "formal_gate_hashes_still_match": True,
    "checks": checks,
    "untracked_text_files_checked": len(files),
    "whitespace_findings": findings,
}
(research / "final-integrity.json").write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps(record, ensure_ascii=False))

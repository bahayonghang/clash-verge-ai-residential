"""T06 捕获器定向合成验证；不启动客户端，不扫描真实工作树。"""
import contextlib
import hashlib
import importlib.util
import io
import json
from pathlib import Path
from types import SimpleNamespace
import sys
import tempfile
from unittest.mock import patch

sys.dont_write_bytecode = True
RESEARCH = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("t06_capture", RESEARCH / "runtime-capture.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
checks = []


def require(name, result):
    checks.append({"case": name, "pass": bool(result)})
    if not result:
        raise AssertionError(name)


def fake_bound():
    return {name: {"path": f"synthetic/{name}", "status": "READY", "sha256": f"hash-{name}",
                   "version_source": "synthetic_only", "recorded_version_output": "synthetic_only"}
            for name in (*module.HARNESS, "pwsh", "omp_bun", "omp_cli")}


def batch_case(directory, name, bound, frozen_bound, expected_launched, expected_blocked):
    root = directory / name
    root.mkdir()
    manifest = root / "candidate.json"
    frozen = {"source_snapshot_id": "synthetic-candidate", "files": {}, "index": {},
              "head": "synthetic-head", "branch": "synthetic-branch"}
    manifest.write_text(json.dumps({"snapshot": frozen, "executables": module.public_bindings(frozen_bound)}), encoding="utf-8")
    launched = []

    def fake_capture(argv, destination, timeout):
        launched.append(Path(argv[0]).name)
        return {"exit_code": 0, "capture_stop_reason": None, "synthetic_only": True}

    def fake_command(harness, phase, current_bound):
        return [current_bound[harness]["path"], module.prompt_for(harness, phase)]

    args = SimpleNamespace(go="GO:synthetic-candidate", harness="all", phase="parent", child_readiness=None, timeout=180)
    with patch.object(module, "MANIFEST", manifest), patch.object(module, "RUNS", root / "runs"), \
         patch.object(module, "snapshot", return_value=frozen), patch.object(module, "bindings", return_value=bound), \
         patch.object(module, "capture", side_effect=fake_capture), patch.object(module, "command_for", side_effect=fake_command), \
         contextlib.redirect_stdout(io.StringIO()):
        module.execute(args)
    batches = list((root / "runs").glob("*/batch.json"))
    require(name + "_one_batch", len(batches) == 1)
    outcomes = json.loads(batches[0].read_text(encoding="utf-8"))["outcomes"]
    require(name + "_five_independent_receipts", len(outcomes) == 5)
    require(name + "_only_expected_launches", launched == expected_launched)
    observed = {item["harness"]: item["status"] for item in outcomes if item["status"].startswith("BLOCKED")}
    require(name + "_blocked_states", observed == expected_blocked)


def main():
    for case in json.loads((RESEARCH / "runtime-private-path-cases.json").read_text(encoding="utf-8"))["cases"]:
        require("private_path:" + case["path"], module.private_path(case["path"]) == case["expected_private"])
    for value in ("authorization=never", "approval_policy=never", "普通授权范围与权限说明"):
        changes = []
        require("ordinary_language:" + value, module.redact(value, 1, changes) == value and not changes)
    for value in ("Authorization: Bearer " + "synthetic-test-" * 2, "authorization=basic " + "YQ" * 12):
        changes = []
        cleaned = module.redact(value, 1, changes)
        require("synthetic_auth_header", "[REDACTED:" in cleaned and any(item["category"] == "authorization_header" for item in changes))
    bound = fake_bound()
    for phase in ("parent", "child"):
        for harness in module.HARNESS:
            argv = module.command_for(harness, phase, bound)
            require(f"single_prompt:{phase}:{harness}", argv[-1].startswith("Active task: " + module.TASK + "\n") and sum(arg.startswith("Active task:") for arg in argv) == 1)
            require(f"no_model_override:{phase}:{harness}", "--model" not in argv and "--yolo" not in argv and "--plan-yolo" not in argv)
    with tempfile.TemporaryDirectory(prefix="t06-synthetic-", dir=module.RUNS) as temporary:
        directory = Path(temporary)
        missing = fake_bound()
        missing["codex"].update(status="BLOCKED_EXECUTABLE_MISSING")
        batch_case(directory, "missing_codex", missing, missing, ["claude", "grok", "kimi", "omp"], {"codex": "BLOCKED_EXECUTABLE_OR_DEPENDENCY"})
        changed = fake_bound()
        changed["codex"]["sha256"] = "changed-hash"
        batch_case(directory, "changed_codex", changed, fake_bound(), ["claude", "grok", "kimi", "omp"], {"codex": "BLOCKED_EXECUTABLE_CHANGED"})
        dependency = fake_bound()
        dependency["omp_bun"].update(status="BLOCKED_EXECUTABLE_MISSING")
        batch_case(directory, "missing_omp_dependency", dependency, dependency, ["claude", "codex", "grok", "kimi"], {"omp": "BLOCKED_EXECUTABLE_OR_DEPENDENCY"})
        unverified = fake_bound()
        unverified["codex"].update(status="BLOCKED_BINDING_NOT_VERIFIED")
        batch_case(directory, "unverified_codex", unverified, unverified, ["claude", "grok", "kimi", "omp"], {"codex": "BLOCKED_EXECUTABLE_OR_DEPENDENCY"})
    print(json.dumps({"synthetic_only": True, "clients_started": 0, "real_worktree_snapshots": 0,
                      "capture_sha256": hashlib.sha256((RESEARCH / "runtime-capture.py").read_bytes()).hexdigest(), "checks": checks}, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    main()

"""T06 基本读取捕获入口；复用原捕获器，保留独立候选与退出码。"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
RESEARCH = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("t06_original_capture_operational", RESEARCH / "runtime-capture.py")
base = importlib.util.module_from_spec(spec)
spec.loader.exec_module(base)
MANIFEST = RESEARCH / "operational-candidate.json"
RUNS = RESEARCH / "operational-runs"
DRIVER = RESEARCH / "operational-kimi-acp.py"
PROMPT = RESEARCH / "operational-parent-prompt.txt"
GO = None
TIMEOUT = 180
OUTCOMES = None
original_output_path = base.output_path
original_command_for = base.command_for
original_bindings = base.bindings
original_write_json = base.write_json
original_capture = base.capture


def output_path(name):
    return (original_output_path(name)
            or name.startswith(f"{base.TASK}/research/operational-runs/")
            or name == f"{base.TASK}/research/operational-candidate.json")


def prompt_for(name, phase):
    if phase != "parent":
        raise RuntimeError("本轮只允许基本 parent 读取")
    prompt = PROMPT.read_text(encoding="utf-8")
    if not prompt.startswith(f"Active task: {base.TASK}\n") or "{{" in prompt:
        raise RuntimeError("基本读取 prompt 首行不正确")
    if not prompt.isascii():
        raise RuntimeError("基本读取 prompt 必须为 ASCII；具名文件按 UTF-8 读取")
    return prompt


def bindings(hash_files=False):
    bound = original_bindings(hash_files)
    bound["kimi"].update(operational_route="native_acp_plan_parent_only",
                         driver_file=DRIVER.relative_to(base.ROOT).as_posix(),
                         driver_sha256=base.digest_file(DRIVER),
                         driver_python_sha256=base.digest_file(Path(sys.executable)))
    return bound


def command_for(name, phase, bound):
    if phase != "parent":
        raise RuntimeError("本轮不派发 child")
    if name != "kimi":
        return original_command_for(name, phase, bound)
    if bound[name].get("status") != "READY":
        raise RuntimeError("Kimi 原生 ACP 入口未就绪")
    return [sys.executable, "-B", "-X", "utf8", str(DRIVER),
            "--kimi", bound[name]["path"], "--manifest", str(MANIFEST),
            "--go", GO or "GO_REQUIRED", "--timeout", str(TIMEOUT - 15),
            "--prompt", prompt_for(name, phase)]


def write_json(path, value):
    global OUTCOMES
    if path.name in ("invocation.json", "receipt.json"):
        value = {**value, "protocol": "basic_parent_read_only_not_full_T06_acceptance"}
    original_write_json(path, value)
    if path.name == "batch.json":
        OUTCOMES = value["outcomes"]


def capture(argv, destination, timeout_s):
    result = original_capture(argv, destination, timeout_s)
    if str(DRIVER) not in argv:
        return result
    result["stream_hash_scope"] = "driver_forwarded_bytes_not_native_kimi_bytes"
    result["native_kimi_exit_code"] = "UNKNOWN"
    for line in (destination / "events.redacted.jsonl").read_text(encoding="utf-8").splitlines():
        event = json.loads(line).get("payload")
        if not isinstance(event, dict):
            continue
        if event.get("type") == "operational.acp.exit":
            result["native_kimi_exit_code"] = event.get("native_exit_code", "UNKNOWN")
            result["acp_protocol_result"] = event.get("protocol_result", "UNKNOWN")
        if event.get("type") == "operational.acp.stop":
            result["capture_stop_reason"] = result.get("capture_stop_reason") or event["reason"]
            if event["reason"] == "SENSITIVE_OUTPUT_REDACTED":
                result["sensitive_output_detected"] = True
                result.setdefault("redactions", []).extend(event.get("redactions", []))
    return result


base.MANIFEST, base.RUNS = MANIFEST, RUNS
base.LOCAL_BINDINGS = RESEARCH / "operational-bindings.json"
base.output_path, base.prompt_for = output_path, prompt_for
base.bindings, base.command_for = bindings, command_for
base.write_json, base.capture = write_json, capture
base.ASSETS += [path.relative_to(base.ROOT).as_posix() for path in (
    Path(__file__), PROMPT, DRIVER, RESEARCH / "operational-verification.py",
    RESEARCH / "operational-kimi-source.json", base.LOCAL_BINDINGS)]


def execute(args):
    global GO, TIMEOUT, OUTCOMES
    GO, TIMEOUT, OUTCOMES = args.go, args.timeout, None
    selected = base.HARNESS if args.harness == "all" else (args.harness,)
    # 每工具只执行一次基本调用；保留已经存在的首份真实收据。
    for name in selected:
        if any(RUNS.glob(f"*/{name}/invocation.json")):
            raise RuntimeError(f"已有本轮基本调用：{name}；不自动重试")
    args.phase, args.child_readiness = "parent", None
    base.execute(args)
    # 捕获器、native 和协议状态独立；该退出码不表示完成完整验收。
    return 0 if (OUTCOMES and len(OUTCOMES) == len(selected)
                 and all(row["status"] == "CAPTURED_REQUIRES_REVIEW" and row["exit_code"] == 0
                         for row in OUTCOMES)) else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("plan", "freeze", "execute"), nargs="?", default="plan")
    parser.add_argument("--harness", choices=(*base.HARNESS, "all"), default="all")
    parser.add_argument("--timeout", type=int, default=180)
    parser.add_argument("--go")
    args = parser.parse_args()
    if not 30 <= args.timeout <= 300:
        parser.error("每进程超时须在 30–300 秒")
    if args.mode == "execute":
        return execute(args)
    if args.mode == "freeze":
        write_json(MANIFEST, {"created_utc": base.utc(), "snapshot": base.snapshot(),
                             "executables": base.public_bindings(bindings(True)),
                             "meaning": "basic_read_candidate_only_not_approval_or_full_acceptance"})
        print("基本读取候选已保存；未启动客户端")
        return 0
    bound = bindings()
    for name in (base.HARNESS if args.harness == "all" else (args.harness,)):
        try:
            argv = command_for(name, "parent", bound)
            print(json.dumps({"harness": name, "status": "PREPARED_NOT_RUN",
                              "argv": base.redact(argv[:-1], 0, []) + ["<BASIC_PROMPT>"],
                              "protocol": "basic_parent_read_only"}, ensure_ascii=False))
        except RuntimeError as error:
            print(json.dumps({"harness": name, "status": "BLOCKED_UNPREPARED_ROUTE", "reason": str(error)}, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    sys.stderr.reconfigure(encoding="utf-8")
    try:
        raise SystemExit(main())
    except Exception as error:
        print(json.dumps({"status": "STOPPED", "error_type": type(error).__name__,
                          "error": base.redact(str(error), 0, [])}, ensure_ascii=False), file=sys.stderr)
        raise SystemExit(2)

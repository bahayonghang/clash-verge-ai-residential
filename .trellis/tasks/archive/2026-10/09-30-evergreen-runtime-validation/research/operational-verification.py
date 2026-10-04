"""T06 本轮必要合成回归；不启动模型客户端，不冻结真实候选。"""
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
from types import SimpleNamespace
from unittest.mock import patch

sys.dont_write_bytecode = True
RESEARCH = Path(__file__).resolve().parent


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


op = load("operational_verification_target", RESEARCH / "runtime-capture-operational.py")
acp = load("operational_acp_verification_target", RESEARCH / "operational-kimi-acp.py")
checks = []
native_fixtures = []


def require(name, condition):
    checks.append({"case": name, "pass": bool(condition)})
    if not condition:
        raise AssertionError(name)


def reply(identifier, result):
    return {"jsonrpc": "2.0", "id": identifier, "result": result}


def start_protocol():
    protocol = acp.Protocol("synthetic-cwd", "synthetic-prompt", lambda params: "公开文本\n")
    require("initialize_has_no_write_terminal_auth", protocol.start()["params"]["clientCapabilities"] ==
            {"fs": {"readTextFile": True, "writeTextFile": False}, "terminal": False, "auth": {"terminal": False}})
    require("initialize_then_new", protocol.receive(reply(1, {"protocolVersion": 1}))[0]["method"] == "session/new")
    require("new_then_plan", protocol.receive(reply(2, {"sessionId": "synthetic-id", "modes": {"availableModes": [{"id": "plan"}]}}))[0]["params"]["modeId"] == "plan")
    return protocol


def mode_update(mode="plan"):
    return {"jsonrpc": "2.0", "method": "session/update", "params": {"sessionId": "synthetic-id",
            "update": {"sessionUpdate": "current_mode_update", "currentModeId": mode}}}


def expect_stop(name, function, expected):
    try:
        function()
    except acp.ProtocolStop as error:
        require(name, str(error) == expected)
    else:
        require(name, False)


def main():
    bound = {name: {"path": "synthetic/" + name, "status": "READY"} for name in (*op.base.HARNESS, "pwsh", "omp_bun", "omp_cli")}
    for name in op.base.HARNESS:
        argv = op.command_for(name, "parent", bound)
        require("prompt_whole_argument:" + name, argv[-1] == op.prompt_for(name, "parent") and sum(value.startswith("Active task:") for value in argv) == 1)
        require("no_model_or_auto_override:" + name, not any(value in argv for value in ("--model", "--yolo", "--auto", "--plan-yolo")))
        if name != "kimi":
            require("unchanged_parent_limits:" + name, argv == op.original_command_for(name, "parent", bound))
        else:
            require("kimi_plan_prompt_not_combined", "--plan" not in argv and "-p" not in argv and str(op.DRIVER) in argv)
    require("prompt_ascii_utf8_instruction", op.PROMPT.read_text(encoding="utf-8").isascii() and "-Encoding utf8" in op.PROMPT.read_text(encoding="utf-8"))
    require("snapshot_sources_included_outputs_excluded", all(not op.output_path(f"{op.base.TASK}/research/{name}") for name in
            ("runtime-capture-operational.py", "operational-parent-prompt.txt", "operational-verification.py", "operational-kimi-acp.py", "operational-kimi-source.json"))
            and op.output_path(f"{op.base.TASK}/research/operational-runs/fixture.json")
            and op.output_path(f"{op.base.TASK}/research/operational-candidate.json"))
    require("private_db_local_remain_excluded", all(op.base.private_path(name) for name in ("monitor.sqlite3", "monitor.sqlite3-wal", "x.local.toml", "credentials.json")))
    protocol = start_protocol()
    require("no_prompt_before_mode_notification", protocol.receive(reply(3, {})) == [])
    require("prompt_only_after_both_confirmations", protocol.receive(mode_update())[0]["method"] == "session/prompt")
    require("prompt_completion_then_close", protocol.receive(reply(4, {"stopReason": "end_turn"}))[0]["method"] == "session/close")
    require("close_completion", protocol.receive(reply(5, {})) == [] and protocol.completed and protocol.closed)
    protocol = start_protocol()
    require("notification_alone_no_prompt", protocol.receive(mode_update()) == [])
    require("reverse_confirmation_order", protocol.receive(reply(3, {}))[0]["method"] == "session/prompt")
    permission = protocol.receive({"jsonrpc": "2.0", "id": 10, "method": "session/request_permission", "params": {"sessionId": "synthetic-id"}})
    require("permission_cancel_never_selected", permission[0]["result"] == {"outcome": {"outcome": "cancelled"}} and permission[1]["method"] == "session/cancel" and protocol.permission_refused)
    expect_stop("mode_loss_stops", lambda: protocol.receive(mode_update("auto")), "PLAN_MODE_LOST")
    expect_stop("write_callback_refused", lambda: protocol.receive({"jsonrpc": "2.0", "id": 11, "method": "fs/write_text_file"}), "UNSUPPORTED_SERVER_METHOD")
    expect_stop("private_read_refused_before_open", lambda: acp.read_public({"path": "monitor.sqlite3"}), "READ_PATH_NOT_ALLOWED")
    for payload in ({"credentialId": "SYNTHETIC-NONSECRET"}, {"authorization": "Bearer " + "synthetic-value-" * 2}):
        changes = []
        cleaned = op.base.redact(payload, 1, changes)
        require("unchanged_sensitive_redaction", cleaned != payload and any(row["category"] != "home_path" for row in changes))
    frozen = {"snapshot": {"source_snapshot_id": "synthetic-id"}, "executables": {}}
    args = SimpleNamespace(go=None, harness="claude", timeout=180)
    with patch.object(op.base.Path, "read_text", return_value=json.dumps(frozen)), patch.object(op.base, "capture") as capture, patch.object(op.base, "snapshot") as snapshot:
        try:
            op.execute(args)
        except RuntimeError:
            pass
        require("no_go_no_capture_or_snapshot", not capture.called and not snapshot.called)
    acp_args = SimpleNamespace(manifest=str(op.MANIFEST), go="NO_GO", kimi="synthetic", prompt="synthetic", timeout=1)
    with patch.object(acp.operational.MANIFEST.__class__, "read_text", return_value=json.dumps(frozen)), patch.object(acp.subprocess, "Popen") as native:
        expect_stop("acp_no_go_before_native", lambda: acp.validate(acp_args), "NO_MATCHING_MAIN_GO")
        require("acp_no_go_native_not_started", not native.called)
    # 自有 Python fixture 只模拟 ACP；从未调用 Kimi/其它模型客户端。
    fixture = '''import json, sys
def send(value):
 print(json.dumps(value, ensure_ascii=False), flush=True)
print("UTF8-公开文本", file=sys.stderr, flush=True)
for line in sys.stdin:
 v=json.loads(line); i=v.get("id"); m=v.get("method")
 if m=="initialize": r={"protocolVersion":1}
 elif m=="session/new": r={"sessionId":"synthetic-id","modes":{"availableModes":[{"id":"plan"}]}}
 elif m=="session/set_mode":
  send({"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"synthetic-id","update":{"sessionUpdate":"current_mode_update","currentModeId":"plan"}}}); r={}
 elif m=="session/prompt": r={"stopReason":"end_turn"}
 elif m=="session/close":
  send({"jsonrpc":"2.0","id":i,"result":{}}); break
 else: raise RuntimeError(m)
 send({"jsonrpc":"2.0","id":i,"result":r})
'''
    real_popen = subprocess.Popen
    synthetic_args = SimpleNamespace(kimi="synthetic-no-native-client", timeout=5, prompt="synthetic-public-prompt")
    output = io.StringIO()
    with patch.object(acp, "validate"), patch.object(acp.subprocess, "Popen", side_effect=lambda *a, **kw: real_popen([sys.executable, "-B", "-X", "utf8", "-c", fixture], **kw)), contextlib.redirect_stdout(output):
        driver_exit = acp.run(synthetic_args)
    rows = [json.loads(line) for line in output.getvalue().splitlines()]
    exit_row = next(row for row in rows if row.get("type") == "operational.acp.exit")
    native_fixtures.append({"case": "acp_success", "driver_exit": driver_exit, **exit_row})
    require("synthetic_native_driver_independent_exit_zero", driver_exit == 0 and exit_row["native_exit_code"] == 0 and exit_row["protocol_result"] == "COMPLETED_REQUIRES_REVIEW")
    require("native_utf8_forwarding", any(row.get("payload") == "UTF8-公开文本" for row in rows))
    for label, replacement, timeout, expected in (
            ("sensitive", 'send({"jsonrpc":"2.0","credentialId":"SYNTHETIC-NONSECRET"}); __import__("time").sleep(10); r={}', 3, "SENSITIVE_OUTPUT_REDACTED"),
            ("permission", 'send({"jsonrpc":"2.0","id":10,"method":"session/request_permission","params":{"sessionId":"synthetic-id"}}); __import__("time").sleep(10); r={}', 3, "PERMISSION_REQUEST_REFUSED"),
            ("timeout", '__import__("time").sleep(10); r={}', 0.2, "TIMEOUT")):
        variant = fixture.replace('r={"stopReason":"end_turn"}', replacement)
        output = io.StringIO()
        synthetic_args.timeout = timeout
        with patch.object(acp, "validate"), patch.object(acp.subprocess, "Popen", side_effect=lambda *a, **kw: real_popen([sys.executable, "-B", "-X", "utf8", "-c", variant], **kw)), contextlib.redirect_stdout(output):
            driver_exit = acp.run(synthetic_args)
        rows = [json.loads(line) for line in output.getvalue().splitlines()]
        exit_row = next(row for row in rows if row.get("type") == "operational.acp.exit")
        native_fixtures.append({"case": "acp_" + label, "driver_exit": driver_exit, **exit_row})
        require("acp_native_fixture_stop:" + label, driver_exit == 1 and exit_row["stop_reason"] == expected
                and type(exit_row["native_exit_code"]) is int and exit_row["protocol_result"] == "BLOCKED_OR_FAILED")
        if label == "sensitive":
            require("acp_sensitive_never_forwarded_raw", "SYNTHETIC-NONSECRET" not in output.getvalue())
        if label == "permission":
            require("acp_permission_refusal_sent", any(row.get("payload", {}).get("result") == {"outcome": {"outcome": "cancelled"}} for row in rows if isinstance(row.get("payload"), dict)))
    with patch.object(op.base, "execute", side_effect=lambda args: setattr(op, "OUTCOMES", [{"status": "BLOCKED_OR_FAILED_REQUIRES_REVIEW", "exit_code": 7}])):
        require("outer_failure_exit_independent", op.execute(SimpleNamespace(go="GO:synthetic", harness="claude", timeout=180)) == 1)
    op.RUNS.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="preparation-synthetic-", dir=op.RUNS) as temporary:
        root = Path(temporary)
        for label, code, timeout, reason in (
                ("sensitive", 'import json, time; print(json.dumps({"credentialId":"SYNTHETIC-NONSECRET"}), flush=True); time.sleep(10)', 3, "SENSITIVE_OUTPUT_REDACTED"),
                ("timeout", 'import time; time.sleep(10)', 0.2, "TIMEOUT"),
                ("native_exit", 'raise SystemExit(7)', 3, None)):
            destination = root / label
            destination.mkdir()
            result = op.original_capture([sys.executable, "-B", "-X", "utf8", "-c", code], destination, timeout)
            native_fixtures.append({"case": "capture_" + label, "native_exit": result["exit_code"],
                                    "stop_reason": result["capture_stop_reason"], "termination_requested": result["termination_requested"]})
            require("native_fixture_capture:" + label, result["capture_stop_reason"] == reason and (result["exit_code"] == 7 if label == "native_exit" else result["termination_requested"]))
            if label == "sensitive":
                require("secret_never_saved_raw", "SYNTHETIC-NONSECRET" not in (destination / "events.redacted.jsonl").read_text(encoding="utf-8"))
    print(json.dumps({"status": "SYNTHETIC_PASS", "model_clients_started": 0, "real_candidate_freezes": 0,
                      "checks": checks, "native_fixtures": native_fixtures}, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    main()

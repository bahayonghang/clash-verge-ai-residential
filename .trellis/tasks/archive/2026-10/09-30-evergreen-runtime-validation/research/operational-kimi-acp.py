"""Kimi 原生 ACP plan 基本读取 driver；没有 GO 不启动原生进程。"""
from __future__ import annotations

import argparse
import importlib.util
import json
from pathlib import Path
import queue
import subprocess
import sys
import threading
import time

sys.dont_write_bytecode = True
RESEARCH = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("t06_operational_capture_acp", RESEARCH / "runtime-capture-operational.py")
operational = importlib.util.module_from_spec(spec)
spec.loader.exec_module(operational)
base = operational.base


class ProtocolStop(RuntimeError):
    pass


def request(identifier, method, params):
    return {"jsonrpc": "2.0", "id": identifier, "method": method, "params": params}


class Protocol:
    """固定顺序；plan 响应和本次 mode 更新同时到达后才发 prompt。"""
    def __init__(self, cwd, prompt, read_file):
        self.cwd, self.prompt, self.read_file = cwd, prompt, read_file
        self.session_id = None
        self.expected = 1
        self.plan_response = False
        self.plan_notification = False
        self.completed = False
        self.closed = False
        self.permission_refused = False

    def start(self):
        return request(1, "initialize", {"protocolVersion": 1,
                       "clientCapabilities": {"fs": {"readTextFile": True, "writeTextFile": False},
                                              "terminal": False, "auth": {"terminal": False}},
                       "clientInfo": {"name": "T06 basic read driver", "version": "1"}})

    def cancel(self):
        return {"jsonrpc": "2.0", "method": "session/cancel", "params": {"sessionId": self.session_id}} if self.session_id else None

    def receive(self, value):
        if not isinstance(value, dict) or value.get("jsonrpc") != "2.0":
            raise ProtocolStop("INVALID_ACP_MESSAGE")
        method, identifier = value.get("method"), value.get("id")
        if method == "session/request_permission":
            self.permission_refused = True
            return [{"jsonrpc": "2.0", "id": identifier, "result": {"outcome": {"outcome": "cancelled"}}}, self.cancel()]
        if method == "fs/read_text_file" and identifier is not None:
            if self.expected != 4 or not self.plan_response or not self.plan_notification:
                raise ProtocolStop("READ_BEFORE_PLAN_CONFIRMATION")
            params = value.get("params", {})
            if params.get("sessionId") != self.session_id:
                raise ProtocolStop("READ_SESSION_MISMATCH")
            content = self.read_file(params)
            return [{"jsonrpc": "2.0", "id": identifier, "result": {"content": content}}]
        if method == "session/update":
            params = value.get("params", {})
            if params.get("sessionId") != self.session_id:
                raise ProtocolStop("UPDATE_SESSION_MISMATCH")
            update = params.get("update", {})
            if update.get("sessionUpdate") == "current_mode_update":
                if update.get("currentModeId") != "plan":
                    raise ProtocolStop("PLAN_MODE_LOST")
                self.plan_notification = True
            return self.maybe_prompt()
        if method is not None:
            raise ProtocolStop("UNSUPPORTED_SERVER_METHOD")
        if "error" in value:
            raise ProtocolStop("ACP_RESPONSE_ERROR")
        if identifier != self.expected or "result" not in value:
            raise ProtocolStop("UNEXPECTED_ACP_RESPONSE")
        result = value["result"]
        if identifier == 1:
            if not isinstance(result, dict) or result.get("protocolVersion") != 1:
                raise ProtocolStop("UNSUPPORTED_ACP_VERSION")
            self.expected = 2
            return [request(2, "session/new", {"cwd": self.cwd, "mcpServers": [], "additionalDirectories": []})]
        if identifier == 2:
            if not isinstance(result, dict) or not isinstance(result.get("sessionId"), str) or not result["sessionId"]:
                raise ProtocolStop("MISSING_NEW_SESSION_ID")
            self.session_id = result["sessionId"]
            modes = result.get("modes", {}).get("availableModes", [])
            if not any(mode.get("id") == "plan" for mode in modes if isinstance(mode, dict)):
                raise ProtocolStop("NATIVE_PLAN_UNAVAILABLE")
            self.expected = 3
            return [request(3, "session/set_mode", {"sessionId": self.session_id, "modeId": "plan"})]
        if identifier == 3:
            self.plan_response = True
            return self.maybe_prompt()
        if identifier == 4:
            if not isinstance(result, dict) or result.get("stopReason") != "end_turn":
                raise ProtocolStop("PROMPT_NOT_COMPLETED")
            self.completed = True
            self.expected = 5
            return [request(5, "session/close", {"sessionId": self.session_id})]
        if identifier == 5:
            self.closed = True
            return []
        raise ProtocolStop("UNEXPECTED_PROTOCOL_STATE")

    def maybe_prompt(self):
        if self.expected == 3 and self.plan_response and self.plan_notification:
            self.expected = 4
            return [request(4, "session/prompt", {"sessionId": self.session_id,
                           "prompt": [{"type": "text", "text": self.prompt}]})]
        return []


def read_public(params):
    requested = Path(params.get("path", ""))
    path = requested if requested.is_absolute() else base.ROOT / requested
    allowed = {(base.ROOT / name).resolve() for name in ("AGENTS.md", "CONTEXT.md")}
    if path.is_symlink() or path.resolve() not in allowed:
        raise ProtocolStop("READ_PATH_NOT_ALLOWED")
    text = path.read_text(encoding="utf-8")
    line, limit = params.get("line"), params.get("limit")
    if line is not None or limit is not None:
        if ((line is not None and (type(line) is not int or line < 1))
                or (limit is not None and (type(limit) is not int or limit < 1))):
            raise ProtocolStop("READ_RANGE_NOT_ALLOWED")
        lines = text.splitlines(keepends=True)
        start = (line or 1) - 1
        text = "".join(lines[start:start + limit] if limit is not None else lines[start:])
    return text


def validate(args):
    if Path(args.manifest).resolve() != operational.MANIFEST.resolve():
        raise ProtocolStop("MANIFEST_PATH_NOT_ALLOWED")
    frozen = json.loads(operational.MANIFEST.read_text(encoding="utf-8"))
    identity = frozen["snapshot"]["source_snapshot_id"]
    if args.go != "GO:" + identity:
        raise ProtocolStop("NO_MATCHING_MAIN_GO")
    if not base.diff_snapshot(frozen["snapshot"], base.snapshot())["same"]:
        raise ProtocolStop("PROTECTED_CANDIDATE_CHANGED")
    binding = frozen["executables"]["kimi"]
    if (base.digest_file(Path(args.kimi)) != binding["sha256"]
            or base.digest_file(Path(__file__)) != binding["driver_sha256"]
            or base.digest_file(Path(sys.executable)) != binding["driver_python_sha256"]):
        raise ProtocolStop("ACP_ENTRY_IDENTITY_CHANGED")
    if args.prompt != operational.prompt_for("kimi", "parent"):
        raise ProtocolStop("PROMPT_IDENTITY_CHANGED")


def emit(value):
    print(json.dumps(value, ensure_ascii=False), flush=True)


def run(args):
    validate(args)
    events = queue.Queue()
    stats = {stream: 0 for stream in ("stdout", "stderr")}
    job, process, reason = base.OwnedJob(), None, None
    protocol = Protocol(str(base.ROOT), args.prompt, read_public)

    def reader(pipe, stream):
        try:
            for raw in iter(lambda: pipe.readline(base.MAX_EVENT + 1), b""):
                stats[stream] += len(raw)
                if len(raw) > base.MAX_EVENT or stats[stream] > base.MAX_STREAM:
                    events.put((stream, None))
                    break
                events.put((stream, raw))
        finally:
            pipe.close()
            events.put((stream, b""))

    def send(value):
        if value is not None:
            process.stdin.write((json.dumps(value, ensure_ascii=False) + "\n").encode("utf-8"))
            process.stdin.flush()
            # 请求内容只含本轮公开 prompt、限定文件与原生新会话 ID。
            clean = base.redact(value, 0, [])
            emit({"type": "operational.acp.sent", "payload": clean})

    try:
        process = subprocess.Popen([args.kimi, "acp"], cwd=base.ROOT, shell=False,
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                   creationflags=subprocess.CREATE_NEW_PROCESS_GROUP)
        try:
            job.attach(process)
        except Exception:
            process.terminate()
            process.wait(timeout=5)
            raise
        threads = [threading.Thread(target=reader, args=(getattr(process, stream), stream), daemon=True)
                   for stream in stats]
        for thread in threads:
            thread.start()
        send(protocol.start())
        deadline = time.monotonic() + args.timeout
        shutdown_deadline = None
        ended = set()
        while len(ended) < 2:
            if time.monotonic() >= deadline:
                raise ProtocolStop("TIMEOUT")
            if shutdown_deadline is not None and time.monotonic() >= shutdown_deadline:
                raise ProtocolStop("NATIVE_POST_PROTOCOL_EXIT_TIMEOUT")
            try:
                stream, raw = events.get(timeout=0.1)
            except queue.Empty:
                if process.poll() is not None and not any(thread.is_alive() for thread in threads) and not protocol.closed:
                    raise ProtocolStop("NATIVE_EXIT_BEFORE_PROTOCOL_COMPLETE")
                continue
            if raw is None:
                raise ProtocolStop("OUTPUT_LIMIT")
            if not raw:
                ended.add(stream)
                if len(ended) == 2 and not protocol.closed:
                    raise ProtocolStop("NATIVE_EOF_BEFORE_PROTOCOL_COMPLETE")
                continue
            decoded = raw.decode("utf-8", errors="strict")
            try:
                payload = json.loads(decoded)
            except json.JSONDecodeError:
                if stream == "stdout":
                    raise ProtocolStop("NON_JSON_ACP_STDOUT")
                payload = decoded.rstrip("\r\n")
            changes = []
            clean = base.redact(payload, 0, changes)
            emit({"type": "operational.acp.received", "native_stream": stream,
                  "representation": "redacted_native_event", "payload": clean})
            if any(change["category"] != "home_path" for change in changes):
                emit({"type": "operational.acp.stop", "reason": "SENSITIVE_OUTPUT_REDACTED", "redactions": changes})
                raise ProtocolStop("SENSITIVE_OUTPUT_REDACTED")
            if base.blocking_error(payload, stream):
                raise ProtocolStop("CLIENT_REPORTED_BLOCKER")
            if stream == "stdout" and not protocol.closed:
                outgoing = protocol.receive(payload)
                for value in outgoing:
                    send(value)
                if protocol.permission_refused:
                    raise ProtocolStop("PERMISSION_REQUEST_REFUSED")
                if protocol.closed:
                    process.stdin.close()
                    shutdown_deadline = time.monotonic() + 5
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            raise ProtocolStop("NATIVE_POST_PROTOCOL_EXIT_TIMEOUT")
    except Exception as error:
        reason = str(error) if isinstance(error, ProtocolStop) else type(error).__name__
        emit({"type": "operational.acp.stop", "reason": reason})
        if process is not None and process.poll() is None:
            try:
                send(protocol.cancel())
            except (OSError, ValueError):
                pass
    finally:
        job.close()
        if process is not None:
            if process.stdin and not process.stdin.closed:
                process.stdin.close()
            if process.poll() is None:
                process.terminate()
            native_exit = process.wait(timeout=5)
        else:
            native_exit = "NOT_STARTED"
        emit({"type": "operational.acp.exit", "native_exit_code": native_exit,
              "protocol_result": "COMPLETED_REQUIRES_REVIEW" if protocol.completed and protocol.closed and reason is None else "BLOCKED_OR_FAILED",
              "stop_reason": reason, "native_bytes": stats,
              "job_cleanup": "best_effort_owned_job; post_Popen_attach_race; no_permission_sandbox"})
    return 0 if reason is None and native_exit == 0 and protocol.completed and protocol.closed else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--kimi", required=True)
    parser.add_argument("--manifest", required=True)
    parser.add_argument("--go", required=True)
    parser.add_argument("--timeout", type=int, required=True)
    parser.add_argument("--prompt", required=True)
    args = parser.parse_args()
    if not 15 <= args.timeout <= 285:
        parser.error("原生 ACP 超时须在 15–285 秒")
    return run(args)


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    sys.stderr.reconfigure(encoding="utf-8")
    try:
        raise SystemExit(main())
    except Exception as error:
        emit({"type": "operational.acp.stop", "reason": str(error) if isinstance(error, ProtocolStop) else type(error).__name__})
        raise SystemExit(2)

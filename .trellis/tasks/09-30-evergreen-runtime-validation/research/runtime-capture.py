"""T06 一次性捕获器；plan 不执行客户端，execute 需要主会话 GO。"""
from __future__ import annotations

import argparse
import ctypes
from ctypes import wintypes
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import queue
import re
import shutil
import subprocess
import sys
import threading
import time
import uuid

ROOT = Path(__file__).resolve().parents[4]
TASK = ".trellis/tasks/09-30-evergreen-runtime-validation"
RESEARCH = ROOT / TASK / "research"
MANIFEST = RESEARCH / "runtime-candidate.json"
RUNS = RESEARCH / "runtime-runs"
VERSIONS = ROOT / ".trellis/tasks/09-30-evergreen-five-harness-audit/research/tool-versions.json"
LOCAL_BINDINGS = RESEARCH / "runtime-bindings-20261001.json"
HARNESS = ("claude", "codex", "grok", "kimi", "omp")
MAX_EVENT = 1024 * 1024
MAX_STREAM = 16 * 1024 * 1024
ROLES = {
    "claude": ("Agent + trellis-research", ".claude/agents/trellis-research.md"),
    "codex": ("spawn_agent + trellis-research；V1 非 full-history；不覆盖模型或 V1/V2", ".codex/agents/trellis-research.toml"),
    "grok": ("spawn_subagent(subagent_type=trellis-research)", ".grok/agents/trellis-research.md"),
    "kimi": ("Agent + built-in coder + research role skill", ".kimi-code/skills/trellis-research/SKILL.md"),
    "omp": ("task + 项目 trellis-research；字段以本次实际 schema 为准", ".omp/agents/trellis-research.md"),
}
ASSETS = [
    ".claude/settings.json", ".codex/config.toml", ".codex/hooks.json",
    ".grok/commands/trellis-start.md", ".omp/extensions/trellis/index.ts",
    ".kimi-code/skills/trellis-implement/SKILL.md", ".kimi-code/skills/trellis-check/SKILL.md",
] + [item[1] for item in ROLES.values()]
for platform in (".agents", ".claude", ".codex", ".cursor", ".omp", ".grok", ".kimi-code"):
    for name in ("SKILL.md", "reference.md", "scripts/build-inputs.js"):
        ASSETS.append(f"{platform}/skills/residential-rule-tuning/{name}")
for platform in (".claude", ".codex"):
    for name in ("session-start.py", "inject-workflow-state.py", "inject-subagent-context.py", "inject-spec-context.py"):
        ASSETS.append(f"{platform}/hooks/{name}")
SECRET_KEYS = {"token", "access_token", "refresh_token", "api_key", "apikey", "secret",
               "client_secret", "password", "passwd", "cookie", "set_cookie", "credential", "credentialid"}
PATTERNS = (
    ("authorization_header", re.compile(r"(?i)\b(?:bearer\s+[A-Za-z0-9._~+/=-]{8,}|authorization\s*[:=]\s*basic\s+[A-Za-z0-9+/=]{8,})")),
    ("credential_url", re.compile(r"(?i)https?://[^\s/:@]+:[^\s/@]+@")),
    ("api_key_shape", re.compile(r"\b(?:sk-(?:proj-|ant-)?|ghp_|github_pat_)[A-Za-z0-9_-]{16,}")),
    ("secret_assignment", re.compile(r'''(?i)\b(?:api[_-]?key|access[_-]?token|refresh[_-]?token|password|client[_-]?secret)\s*[=:]\s*["']?[^\s,"'\r\n]{8,}''')),
)
ERROR_TERMS = re.compile(r"(?i)authentication|unauthorized|invalid.{0,4}(?:api|key|token)|quota|credit|rate.limit|project.{0,8}trust|trust.{0,12}(?:project|hook)|unknown (?:option|argument)|unrecognized (?:option|argument)|unsupported (?:option|argument)|permission denied")


def utc():
    return datetime.now(timezone.utc).isoformat()


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")


def digest_file(path):
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def write_json(path, value):
    if not path.resolve().is_relative_to(RESEARCH.resolve()):
        raise RuntimeError("输出路径不在 T06 research")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def git(*args):
    result = subprocess.run(["git", *args], cwd=ROOT, stdin=subprocess.DEVNULL, capture_output=True, timeout=30, check=False)
    if result.returncode:
        raise RuntimeError(f"Git 只读元数据命令失败：{args[0]}，exit={result.returncode}")
    return result.stdout.decode("utf-8", errors="strict")


def private_path(name):
    lower = name.replace("\\", "/").lower()
    parts, base = lower.split("/"), lower.split("/")[-1]
    # 按已知私有文件名识别原件、备份和压缩副本；不读取文件内容。
    # .example、.test.js、.rs 等公开示例/源码后缀不属于副本后缀。
    copies = (r"(?:$|~.*$|[._ -]+(?:(?:bak|backup|old|orig|save|copy|tmp|temp|swp|swo|"
              r"gz|gzip|zip|bz2|xz|zst|7z|rar|tar)(?=$|[._~ -]|\d)|\d|\((?:copy|\d+)\))[^/]*$)")
    private_file = any(re.search(pattern + copies, base) for pattern in (
        r"\.local\.(?:toml|js|ya?ml)",
        r"\.(?:sqlite3?|db3?|s3db)(?:-(?:wal|shm|journal))?",
        r"\.(?:pem|pfx|p12|p8|key|keystore)",
        r"^\.?(?:auth|credentials?|secrets?|tokens?|api[-_]keys?)\.(?:json|toml|ya?ml|ini|conf|txt)",
        r"^(?:id_rsa|id_ed25519|id_ecdsa|id_dsa|\.npmrc|\.netrc|\.pypirc|\.git-credentials|history\.jsonl)",
    ))
    return (base.startswith(".env") or private_file
            or any(x in parts for x in (".git", "node_modules", "target", "__pycache__"))
            or lower.startswith(".trellis/.runtime/")
            or any(lower.startswith(f"{p}/{n}/") for p in (".claude", ".codex", ".grok", ".kimi", ".kimi-code", ".omp") for n in ("sessions", "history", "projects", "logs", "cache")))


def output_path(name):
    return name.startswith(f"{TASK}/research/runtime-runs/") or name == f"{TASK}/research/runtime-candidate.json"


def snapshot():
    tracked = set(filter(None, git("ls-files", "-z", "--cached").split("\0")))
    added = set(filter(None, git("ls-files", "-z", "--others", "--exclude-standard").split("\0")))
    excluded = sorted(n for n in tracked | added if private_path(n))
    names = sorted(n for n in tracked | added | set(ASSETS) if not private_path(n) and not output_path(n))
    files = {}
    for name in names:
        path = ROOT / name
        if not path.exists():
            files[name] = {"state": "missing"}
            continue
        if path.is_symlink() or not path.resolve().is_relative_to(ROOT.resolve()) or private_path(path.resolve().relative_to(ROOT.resolve()).as_posix()):
            raise RuntimeError(f"保护文件需要额外路径审查：{name}")
        if not path.is_file():
            raise RuntimeError(f"保护清单包含非普通文件：{name}")
        files[name] = {"state": "file", "sha256": digest_file(path), "bytes": path.stat().st_size}
    index = {}
    for record in filter(None, git("ls-files", "--stage", "-z").split("\0")):
        meta, name = record.split("\t", 1)
        if name in files:
            index.setdefault(name, []).append(meta)
    value = {"head": git("rev-parse", "HEAD").strip(), "branch": git("branch", "--show-current").strip(),
             "files": files, "index": index, "tracked": sorted(tracked & files.keys()),
             "untracked_public": sorted(added & files.keys()), "excluded_private_path_count": len(excluded)}
    value["source_snapshot_id"] = hashlib.sha256(canonical(value)).hexdigest()
    return value


def diff_snapshot(before, after):
    old, new = before["files"], after["files"]
    return {"same": before["source_snapshot_id"] == after["source_snapshot_id"],
            "changed": sorted(n for n in old.keys() & new.keys() if old[n] != new[n]),
            "added": sorted(new.keys() - old.keys()), "removed": sorted(old.keys() - new.keys()),
            "index_changed": before["index"] != after["index"],
            "head_changed": before["head"] != after["head"], "branch_changed": before["branch"] != after["branch"]}


def bindings(hash_files=False):
    records = json.loads(VERSIONS.read_text(encoding="utf-8-sig"))
    local = json.loads(LOCAL_BINDINGS.read_text(encoding="utf-8")) if LOCAL_BINDINGS.is_file() else {}
    result = {}
    for name in HARNESS:
        key = "codex-app-binary" if name == "codex" else name
        record = next(item for item in records if item.get("tool") == key)
        path = Path(record["executable"])
        source = VERSIONS.relative_to(ROOT).as_posix() + "#" + key
        version = record.get("output", "UNKNOWN")
        replacement = local.get("codex_app_binary") if name == "codex" else None
        if replacement:
            relative = Path(replacement["relative_to_userprofile"])
            app_bin = Path.home() / "AppData/Local/OpenAI/Codex/bin"
            path = Path.home() / relative
            if relative.is_absolute() or path.name != "codex.exe" or not path.resolve().is_relative_to(app_bin.resolve()):
                result[name] = {"status": "BLOCKED_BINDING_PATH", "reason": "T06 入口不在已有 Codex app bin 范围"}
                continue
            source = LOCAL_BINDINGS.relative_to(ROOT).as_posix() + "#codex_app_binary"
            version = replacement.get("version_output", "UNKNOWN")
        item = {"path": str(path), "version_source": source, "recorded_version_output": version,
                "status": "READY" if path.is_file() else "BLOCKED_EXECUTABLE_MISSING"}
        if replacement and replacement.get("verified_for_runtime") is not True and hash_files and item["status"] == "READY":
            item.update(status="BLOCKED_BINDING_NOT_VERIFIED", reason="新入口仅有元数据；版本、help 与哈希尚待核对")
        if hash_files and item["status"] == "READY":
            try:
                item["sha256"] = digest_file(path)
            except OSError as error:
                item.update(status="BLOCKED_EXECUTABLE_READ", reason=type(error).__name__)
            if replacement and item.get("sha256") != replacement.get("sha256"):
                item.update(status="BLOCKED_EXECUTABLE_IDENTITY", reason="T06 已核对入口的哈希不匹配")
        result[name] = item
    pwsh = shutil.which("pwsh")
    result["pwsh"] = {"path": pwsh or "", "status": "READY" if pwsh else "BLOCKED_EXECUTABLE_MISSING"}
    omp_root = Path(result["omp"]["path"]).parent
    bun = omp_root / "bun.exe"
    if not bun.is_file():
        discovered_bun = shutil.which("bun.exe")
        bun = Path(discovered_bun) if discovered_bun else bun
    for name, path in {"omp_bun": bun, "omp_cli": omp_root / "node_modules/@oh-my-pi/pi-coding-agent/dist/cli.js"}.items():
        result[name] = {"path": str(path), "status": "READY" if path.is_file() else "BLOCKED_EXECUTABLE_MISSING"}
    if hash_files:
        for name in ("pwsh", "omp_bun", "omp_cli"):
            if result[name]["status"] == "READY":
                try:
                    result[name]["sha256"] = digest_file(Path(result[name]["path"]))
                except OSError as error:
                    result[name].update(status="BLOCKED_EXECUTABLE_READ", reason=type(error).__name__)
    return result


def public_bindings(bound):
    return {name: {k: v for k, v in value.items() if k != "path"} for name, value in bound.items()}


def prompt_for(name, phase):
    text = (RESEARCH / f"runtime-{phase}-prompt.txt").read_text(encoding="utf-8")
    if phase == "child":
        text = text.replace("{{NATIVE_ROUTE}}", ROLES[name][0]).replace("{{ROLE_FILE}}", ROLES[name][1])
    if not text.startswith(f"Active task: {TASK}\n") or "{{" in text:
        raise RuntimeError("prompt 首行或模板替换不正确")
    return text


def command_for(name, phase, bound):
    for dependency in (name, "pwsh", "omp_bun", "omp_cli") if name == "omp" else (name,):
        if bound[dependency].get("status") != "READY":
            raise RuntimeError(f"入口或依赖未就绪：{dependency}；{bound[dependency].get('status')}")
    prompt = prompt_for(name, phase)
    common = {
        "claude": ["--restricted", "--strict-mcp-config", "--permission-mode", "plan", "--permission-prompts", "none",
                   "--tools", "Read,Glob,Grep" + (",Agent" if phase == "child" else ""), "--no-session-persistence",
                   "--output-format", "stream-json", "--verbose", "--include-hook-events"],
        "codex": ["exec", "--sandbox", "read-only", "-c", 'approval_policy="never"', "--ephemeral", "--json"],
        "grok": ["--permission-mode", "plan", "--disable-web-search", "--output-format", "streaming-messages-json", "--max-turns", "8"],
        "kimi": ["--plan", "--output-format", "stream-json"],
        "omp": ["--mode", "json", "--no-session", "--no-title", "--no-prewalk", "--no-lsp", "--no-pty",
                "--no-extensions", "--tools", "read,grep,glob", "--approval-mode", "write"],
    }
    if name == "omp" and phase == "child":
        # 独立预选的正常原生路线；保持已有工具、审批及 extension 配置。
        common[name] = ["--mode", "json", "--no-session", "--no-title", "--no-prewalk", "--no-lsp", "--no-pty"]
    if name == "claude" and phase == "child":
        common[name].append("--forward-subagent-text")
    suffix = [prompt] if name == "codex" else ["-p", prompt]
    argv = [bound[name]["path"], *common[name], *suffix]
    if name == "omp":
        argv = [bound["pwsh"]["path"], "-NoLogo", "-NoProfile", "-NonInteractive", "-File", *argv]
    return argv


def redact(value, event_index, changes, field="$", key=""):
    normalized = key.lower().replace("-", "_")
    if normalized in SECRET_KEYS or (normalized == "authorization" and isinstance(value, str) and value.lower().startswith(("bearer ", "basic "))):
        changes.append({"event_index": event_index, "field": field, "category": "secret_key"})
        return "[REDACTED:secret_key]"
    if isinstance(value, dict):
        return {k: redact(v, event_index, changes, f"{field}.{k}", str(k)) for k, v in value.items()}
    if isinstance(value, list):
        return [redact(v, event_index, changes, f"{field}[{i}]") for i, v in enumerate(value)]
    if isinstance(value, str):
        for category, pattern in PATTERNS:
            if pattern.search(value):
                changes.append({"event_index": event_index, "field": field, "category": category})
                value = pattern.sub(f"[REDACTED:{category}]", value)
        for pattern in (r"(?i)[A-Z]:[\\/]Users[\\/][^\\/\s\"']+", r"(?i)[A-Z]:[\\/]home[\\/][^\\/\s\"']+"):
            if re.search(pattern, value):
                changes.append({"event_index": event_index, "field": field, "category": "home_path"})
                value = re.sub(pattern, "<HOME>", value)
        return value
    return value


def blocking_error(payload, stream):
    if isinstance(payload, dict):
        kind = str(payload.get("type", payload.get("event", ""))).lower()
        if kind == "error" or kind.endswith(".error") or isinstance(payload.get("error"), dict):
            return bool(ERROR_TERMS.search(json.dumps(payload, ensure_ascii=False)))
    return isinstance(payload, str) and stream == "stderr" and bool(re.match(r"(?i)^\s*(?:error|fatal|usage:)", payload)) and bool(ERROR_TERMS.search(payload))


class OwnedJob:
    """仅清理本次创建的进程树；不提供文件系统或权限沙箱。"""
    def __init__(self):
        if os.name != "nt":
            raise RuntimeError("此一次性捕获器只核验 Windows 入口")
        class IO(ctypes.Structure):
            _fields_ = [(n, ctypes.c_uint64) for n in ("ReadOperationCount", "WriteOperationCount", "OtherOperationCount", "ReadTransferCount", "WriteTransferCount", "OtherTransferCount")]
        class BASIC(ctypes.Structure):
            _fields_ = [("PerProcessUserTimeLimit", ctypes.c_int64), ("PerJobUserTimeLimit", ctypes.c_int64),
                        ("LimitFlags", wintypes.DWORD), ("MinimumWorkingSetSize", ctypes.c_size_t), ("MaximumWorkingSetSize", ctypes.c_size_t),
                        ("ActiveProcessLimit", wintypes.DWORD), ("Affinity", ctypes.c_size_t), ("PriorityClass", wintypes.DWORD), ("SchedulingClass", wintypes.DWORD)]
        class EXT(ctypes.Structure):
            _fields_ = [("BasicLimitInformation", BASIC), ("IoInfo", IO), ("ProcessMemoryLimit", ctypes.c_size_t),
                        ("JobMemoryLimit", ctypes.c_size_t), ("PeakProcessMemoryUsed", ctypes.c_size_t), ("PeakJobMemoryUsed", ctypes.c_size_t)]
        self.api = ctypes.WinDLL("kernel32", use_last_error=True)
        self.api.CreateJobObjectW.argtypes = (ctypes.c_void_p, wintypes.LPCWSTR)
        self.api.CreateJobObjectW.restype = wintypes.HANDLE
        self.api.SetInformationJobObject.argtypes = (wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p, wintypes.DWORD)
        self.api.SetInformationJobObject.restype = wintypes.BOOL
        self.api.AssignProcessToJobObject.argtypes = (wintypes.HANDLE, wintypes.HANDLE)
        self.api.AssignProcessToJobObject.restype = wintypes.BOOL
        self.api.CloseHandle.argtypes = (wintypes.HANDLE,)
        self.api.CloseHandle.restype = wintypes.BOOL
        self.handle = self.api.CreateJobObjectW(None, None)
        if not self.handle:
            raise OSError(ctypes.get_last_error(), "CreateJobObjectW 失败")
        limits = EXT()
        limits.BasicLimitInformation.LimitFlags = 0x2000
        if not self.api.SetInformationJobObject(self.handle, 9, ctypes.byref(limits), ctypes.sizeof(limits)):
            code = ctypes.get_last_error()
            self.close()
            raise OSError(code, "Job 退出清理配置失败")

    def attach(self, process):
        if not self.api.AssignProcessToJobObject(self.handle, wintypes.HANDLE(int(process._handle))):
            raise OSError(ctypes.get_last_error(), "本次进程无法加入退出清理 Job")

    def close(self):
        if self.handle:
            if not self.api.CloseHandle(self.handle):
                raise OSError(ctypes.get_last_error(), "关闭本次 Job 失败")
            self.handle = None


def capture(argv, destination, timeout_s):
    events = queue.Queue()
    stats = {s: {"sha256": hashlib.sha256(), "bytes": 0, "records": 0} for s in ("stdout", "stderr")}
    changes, marker, received = [], None, 0
    started, stop_deadline, termination_requested = utc(), None, False
    job, process = OwnedJob(), None

    def reader(pipe, stream):
        try:
            for raw in iter(lambda: pipe.readline(MAX_EVENT + 1), b""):
                stats[stream]["sha256"].update(raw)
                stats[stream]["bytes"] += len(raw)
                stats[stream]["records"] += 1
                if stats[stream]["bytes"] <= MAX_STREAM:
                    events.put((stream, stats[stream]["records"], raw, utc()))
                else:
                    events.put((stream, stats[stream]["records"], None, utc()))
                    break
        except OSError as error:
            stats[stream]["reader_error_type"] = type(error).__name__
        finally:
            pipe.close()
            events.put((stream, -1, None, utc()))

    try:
        process = subprocess.Popen(argv, cwd=ROOT, shell=False, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE, creationflags=subprocess.CREATE_NEW_PROCESS_GROUP)
        try:
            job.attach(process)
        except Exception:
            process.terminate()
            process.wait(timeout=10)
            raise
        threads = [threading.Thread(target=reader, args=(getattr(process, s), s), daemon=True) for s in stats]
        for thread in threads:
            thread.start()
        deadline, ended = time.monotonic() + timeout_s, set()
        with (destination / "events.redacted.jsonl").open("x", encoding="utf-8", newline="\n") as sink:
            while len(ended) < 2:
                now = time.monotonic()
                if now >= deadline and marker is None:
                    marker, stop_deadline = "TIMEOUT", now
                if stop_deadline is not None and now >= stop_deadline and not termination_requested:
                    job.close()
                    termination_requested = True
                if stop_deadline is not None and now >= stop_deadline + 10:
                    marker = marker or "DRAIN_TIMEOUT"
                    break
                try:
                    stream, sequence, raw, observed = events.get(timeout=0.1)
                except queue.Empty:
                    if process.poll() is not None and not any(t.is_alive() for t in threads):
                        break
                    continue
                if sequence == -1:
                    ended.add(stream)
                    continue
                received += 1
                if raw is None or len(raw) > MAX_EVENT:
                    payload = {"capture_error": "OUTPUT_LIMIT", "omitted_bytes": len(raw) if raw else "remaining"}
                    marker, stop_deadline = marker or "OUTPUT_LIMIT", time.monotonic()
                else:
                    decoded = raw.decode("utf-8", errors="replace")
                    try:
                        payload = json.loads(decoded)
                    except json.JSONDecodeError:
                        payload = decoded.rstrip("\r\n")
                    if blocking_error(payload, stream) and marker is None:
                        marker, stop_deadline = "CLIENT_REPORTED_BLOCKER", time.monotonic() + 5
                first_change = len(changes)
                clean = redact(payload, received, changes)
                if any(x["category"] != "home_path" for x in changes[first_change:]):
                    marker, stop_deadline = marker or "SENSITIVE_OUTPUT_REDACTED", time.monotonic()
                sink.write(json.dumps({"event_index": received, "stream": stream, "stream_sequence": sequence, "received_utc": observed, "payload": clean}, ensure_ascii=False) + "\n")
                sink.flush()
        try:
            exit_code = process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            marker = marker or "POST_OUTPUT_EXIT_TIMEOUT"
            termination_requested = True
            job.close()
            exit_code = process.wait(timeout=10)
        job.close()
        for thread in threads:
            thread.join(timeout=2)
        return {"started_utc": started, "ended_utc": utc(), "pid": process.pid, "exit_code": exit_code,
                "capture_stop_reason": marker, "termination_requested": termination_requested,
                "capture_representation": "normalized_redacted_events_not_original_bytes",
                "stream_hash_scope": "captured_bytes_only_if_limit_hit",
                "streams": {s: {**{k: v for k, v in st.items() if k != "sha256"}, "sha256": st["sha256"].hexdigest()} for s, st in stats.items()},
                "redactions": changes, "sensitive_output_detected": any(x["category"] != "home_path" for x in changes),
                "events": received, "cross_stream_order": "receiver_observation_only",
                "job_cleanup": "best_effort_owned_job; post_Popen_attach_race; no_permission_sandbox; no_absolute_no_orphan_claim"}
    except Exception as error:
        return {"started_utc": started, "ended_utc": utc(), "pid": process.pid if process else "NOT_STARTED",
                "exit_code": process.poll() if process and process.poll() is not None else "UNKNOWN",
                "capture_stop_reason": "CAPTURE_ERROR", "error_type": type(error).__name__,
                "error": redact(str(error), received, changes), "redactions": changes,
                "job_cleanup": "best_effort_owned_job; post_Popen_attach_race; no_permission_sandbox"}
    finally:
        job.close()
        if process is not None and process.poll() is None:
            process.terminate()
            process.wait(timeout=10)


def execute(args):
    frozen = json.loads(MANIFEST.read_text(encoding="utf-8"))
    identity = frozen["snapshot"]["source_snapshot_id"]
    if args.go != "GO:" + identity:
        raise RuntimeError("缺少与冻结候选匹配的主会话 GO；不会启动客户端")
    current = snapshot()
    if not diff_snapshot(frozen["snapshot"], current)["same"]:
        raise RuntimeError("候选已变化；不会启动客户端")
    bound = bindings(hash_files=True)
    selected = HARNESS if args.harness == "all" else (args.harness,)
    readiness = {}
    if args.phase == "child":
        if not args.child_readiness:
            raise RuntimeError("child 阶段需要主会话审核的 schema 依据与当前授权收据")
        path = Path(args.child_readiness).resolve()
        if not path.is_relative_to(RUNS.resolve()):
            raise RuntimeError("child 审核收据须在 T06 research/runtime-runs 控制记录目录")
        readiness = json.loads(path.read_text(encoding="utf-8"))
        if readiness.get("source_snapshot_id") != identity or readiness.get("reviewed_by_main") is not True:
            raise RuntimeError("child 审核收据未绑定当前候选或未经主会话确认")
    if not RUNS.resolve().is_relative_to(RESEARCH.resolve()):
        raise RuntimeError("运行输出目录越界")
    batch = RUNS / (datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ") + "-" + uuid.uuid4().hex[:8])
    batch.mkdir(parents=True, exist_ok=False)
    outcomes = []

    def blocked(destination, name, status, reason):
        item = {"invocation_id": f"{batch.name}/{name}/{args.phase}", "harness": name, "phase": args.phase,
                "source_snapshot_id": identity, "status": status, "reason": reason, "exit_code": "NOT_STARTED",
                "effective_model": "UNKNOWN", "permissions": "UNVERIFIED", "native_child": "UNVERIFIED"}
        write_json(destination / "receipt.json", item)
        outcomes.append(item)
        print(json.dumps(item, ensure_ascii=False), flush=True)

    for name in selected:
        destination = batch / name
        destination.mkdir()
        before = snapshot()
        if not diff_snapshot(frozen["snapshot"], before)["same"]:
            blocked(destination, name, "BLOCKED_CANDIDATE_CHANGED", diff_snapshot(frozen["snapshot"], before))
            break
        dependencies = (name, "pwsh", "omp_bun", "omp_cli") if name == "omp" else (name,)
        unavailable = [dependency for dependency in dependencies if bound[dependency].get("status") != "READY"]
        if unavailable:
            blocked(destination, name, "BLOCKED_EXECUTABLE_OR_DEPENDENCY", {dependency: public_bindings(bound)[dependency] for dependency in unavailable})
            continue
        changed = [dependency for dependency in dependencies if public_bindings(bound)[dependency] != frozen["executables"].get(dependency)]
        if changed:
            blocked(destination, name, "BLOCKED_EXECUTABLE_CHANGED", changed)
            continue
        if args.phase == "child":
            item = readiness.get("harnesses", {}).get(name, {})
            basis_kind = item.get("basis_kind")
            reference = item.get("basis_reference", {})
            basis_path = (ROOT / str(reference.get("path", ""))).resolve()
            if (item.get("approved_route") != ROLES[name][0]
                    or basis_kind not in ("installed_source_schema", "runtime_schema_event", "reviewed_local_prerequisite")
                    or not basis_path.is_relative_to(RESEARCH.resolve()) or not basis_path.is_file()
                    or not reference.get("locator")):
                blocked(destination, name, "BLOCKED_CHILD_PREPARATION_OR_AUTHORIZATION", "缺少主会话审核的路线与可追溯 schema 依据；不等于工具不支持")
                continue
        try:
            argv = command_for(name, args.phase, bound)
        except RuntimeError as error:
            blocked(destination, name, "BLOCKED_UNPREPARED_ROUTE", str(error))
            continue
        prompt = argv[-1]
        display_args = [a.replace(str(ROOT), "<REPO>") for a in argv[:-1]] + ["<PROMPT_SHA256:" + hashlib.sha256(prompt.encode("utf-8")).hexdigest() + ">"]
        public_args = redact(display_args, 0, [])
        write_json(destination / "before.json", before)
        with (destination / "prompt.txt").open("x", encoding="utf-8", newline="\n") as prompt_file:
            prompt_file.write(prompt)
        write_json(destination / "invocation.json", {"invocation_id": f"{batch.name}/{name}/{args.phase}", "harness": name, "phase": args.phase, "argv": public_args, "shell": False, "timeout_seconds": args.timeout,
                  "source_snapshot_id": identity, "version_source": bound[name]["version_source"], "recorded_version_output": bound[name]["recorded_version_output"],
                  "executable_sha256": bound[name]["sha256"], "prompt_sha256": hashlib.sha256(prompt.encode("utf-8")).hexdigest(),
                  "child_preparation_basis": readiness.get("harnesses", {}).get(name, "NOT_APPLICABLE"),
                  "basis_is_runtime_proof": False})
        try:
            captured = capture(argv, destination, args.timeout)
        except Exception as error:
            captured = {"exit_code": "UNKNOWN", "capture_stop_reason": "CAPTURE_ERROR", "error_type": type(error).__name__, "error": redact(str(error), 0, [])}
        after = snapshot()
        delta = diff_snapshot(before, after)
        write_json(destination / "after.json", after)
        receipt = {"invocation_id": f"{batch.name}/{name}/{args.phase}", "harness": name, "phase": args.phase, "parent_only": args.phase == "parent", "source_snapshot_id": identity,
                   "executable_binding": name, "executable_sha256": bound[name]["sha256"], "version_source": bound[name]["version_source"],
                   "status": "CAPTURED_REQUIRES_REVIEW", "effective_model": "UNKNOWN", "model_evidence": "UNVERIFIED", "rules": "UNVERIFIED",
                   "planning": "UNVERIFIED", "permissions": "UNVERIFIED", "runtime_enforcement": "UNVERIFIED", "native_child": "UNVERIFIED",
                   "role_persistence": "NOT_COVERED_PURE_OUTPUT", "hook_extension_pull": "UNVERIFIED", "negative_case": "UNVERIFIED",
                   "product_hash_unchanged": delta["same"], "protected_snapshot_diff": delta, **captured}
        if captured.get("capture_stop_reason") or captured.get("exit_code") != 0:
            receipt["status"] = "BLOCKED_OR_FAILED_REQUIRES_REVIEW"
        if not delta["same"]:
            receipt["status"] = "STOPPED_PROTECTED_FILES_CHANGED"
        write_json(destination / "receipt.json", receipt)
        outcomes.append({"harness": name, "status": receipt["status"], "exit_code": receipt["exit_code"]})
        print(json.dumps(outcomes[-1], ensure_ascii=False), flush=True)
        if not delta["same"] or captured.get("sensitive_output_detected") or captured.get("capture_stop_reason") in ("TIMEOUT", "OUTPUT_LIMIT", "CAPTURE_ERROR", "SENSITIVE_OUTPUT_REDACTED", "POST_OUTPUT_EXIT_TIMEOUT"):
            break
    write_json(batch / "batch.json", {"source_snapshot_id": identity, "phase": args.phase, "outcomes": outcomes})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("plan", "freeze", "execute"), nargs="?", default="plan")
    parser.add_argument("--phase", choices=("parent", "child"), default="parent")
    parser.add_argument("--harness", choices=(*HARNESS, "all"), default="all")
    parser.add_argument("--timeout", type=int, default=180)
    parser.add_argument("--go")
    parser.add_argument("--child-readiness")
    args = parser.parse_args()
    if not 30 <= args.timeout <= 300:
        parser.error("每进程超时须在 30–300 秒；默认 180 秒")
    if args.mode == "freeze":
        write_json(MANIFEST, {"created_utc": utc(), "snapshot": snapshot(), "executables": public_bindings(bindings(hash_files=True)), "meaning": "local_candidate_only_not_hosted_or_approval"})
        print("候选 manifest 已写入；没有启动客户端")
    elif args.mode == "execute":
        execute(args)
    else:
        bound = bindings()
        for name in (HARNESS if args.harness == "all" else (args.harness,)):
            try:
                argv = command_for(name, args.phase, bound)
                print(json.dumps({"harness": name, "phase": args.phase, "argv": redact(argv[:-1], 0, []) + [f"<PROMPT:{args.phase}>"], "timeout_seconds": args.timeout, "status": "NOT_RUN"}, ensure_ascii=False))
            except RuntimeError as error:
                print(json.dumps({"harness": name, "phase": args.phase, "status": "BLOCKED_UNPREPARED_ROUTE", "reason": str(error)}, ensure_ascii=False))


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    try:
        main()
    except Exception as error:
        print(json.dumps({"status": "STOPPED", "error_type": type(error).__name__, "error": redact(str(error), 0, [])}, ensure_ascii=False), file=sys.stderr)
        raise SystemExit(2)

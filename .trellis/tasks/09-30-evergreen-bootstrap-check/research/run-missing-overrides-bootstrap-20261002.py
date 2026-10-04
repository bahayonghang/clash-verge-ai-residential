"""执行已批准的缺失可选覆盖隔离初始化，保留逐步原始收据。"""

import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import time
import traceback


ROOT = Path(__file__).resolve().parents[4]
RESEARCH = Path(__file__).resolve().parent
OUT = RESEARCH / "bootstrap-missing-overrides-20261002"
NODE = Path(r"D:\GreenSoftware\node\node.EXE")
NPM = Path(r"C:\home\lyh\.npm-global\node_modules\npm\bin\npm-cli.js")
LOCK = RESEARCH / "isolated-package-lock.json"
LOCK_HASH = "fd71957d4e2f20706bf8499f8cbe91e188acabb86fd94ff78febd06e94c4cfa7"
OVERRIDES = [".codex/config.toml"] + [
    f".kimi-code/skills/trellis-{role}/SKILL.md"
    for role in ("implement", "check", "research")
]
ASSETS = [
    ".claude/agents/trellis-implement.md", ".claude/settings.json",
    ".codex/hooks.json", ".grok/agents/trellis-implement.md",
    ".kimi-code/skills/trellis-start/SKILL.md",
    ".omp/agents/trellis-implement.md", ".omp/extensions/trellis/index.ts",
]
EXPLICIT = [
    ".github/workflows/ci.yml", ".gitignore", ".trellis/.version",
    ".trellis/config.yaml", ".trellis/workflow.md", "AGENTS.md", "CLAUDE.md",
    "CONTEXT.md", "docs/agents/harnesses.md",
    "docs/agents/residential-rule-tuning.md", "justfile", "package.json",
    "scripts/check-agent-contract.js", "scripts/check-harness-environment.js",
    "skills/residential-rule-tuning/SKILL.md",
]
REFERENCES = [
    ".trellis/scripts/common/workflow_phase.py",
    ".trellis/tasks/09-30-evergreen-five-harness-audit/research/audit.md",
    ".trellis/tasks/archive/2026-09/09-07-evergreen-harness-audit/research/harness-audit.md",
]
ENV = os.environ.copy()
ENV["PYTHONDONTWRITEBYTECODE"] = "1"
ENV["PYTHONIOENCODING"] = "utf-8"
ENV["NO_COLOR"] = "1"
STEPS = []
SUMMARY = {"scenario": "缺失四可选覆盖的最小公开合同隔离 init", "fresh_checkout": False,
           "native_loading": "UNVERIFIED", "started_utc": None, "steps": STEPS}


def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write(name, data):
    payload = json.dumps(data, ensure_ascii=False, indent=2) + "\n"
    (OUT / name).write_text(payload, encoding="utf-8")
    if "isolated_evidence" in SUMMARY:
        (Path(SUMMARY["isolated_evidence"]) / name).write_text(payload, encoding="utf-8")


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT, env=ENV).decode("utf-8").strip()


def state(rel, base=ROOT):
    path = base / rel
    return {"present": path.is_file(), "sha256": sha(path) if path.is_file() else None}


def protected():
    paths = [".trellis/.template-hashes.json", ".trellis/.version", *OVERRIDES,
             "AGENTS.md", "CLAUDE.md", ".gitignore"]
    index = Path(git("rev-parse", "--git-path", "index"))
    if not index.is_absolute():
        index = ROOT / index
    branch = subprocess.run(["git", "symbolic-ref", "--short", "-q", "HEAD"], cwd=ROOT,
                            env=ENV, capture_output=True)
    return {"files": {p: state(p) for p in paths}, "HEAD": git("rev-parse", "HEAD"),
            "branch": branch.stdout.decode("utf-8").strip(),
            "index": {"path": str(index.resolve()), "sha256": sha(index)},
            "global_config_private_and_trust": "未读取内容；无修改这些对象的命令"}


def run(name, argv, cwd, timeout=300):
    start = time.monotonic()
    receipt = {"name": name, "argv": [str(v) for v in argv], "cwd": str(cwd),
               "started_utc": utc(), "native_exit_code": None, "timed_out": False}
    stdout = OUT / f"{name}.stdout.raw.log"
    stderr = OUT / f"{name}.stderr.raw.log"
    with stdout.open("wb") as out, stderr.open("wb") as err:
        process = subprocess.Popen(receipt["argv"], cwd=cwd, env=ENV, stdout=out, stderr=err)
        receipt["pid"] = process.pid
        try:
            receipt["native_exit_code"] = process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            receipt["timed_out"] = True
            process.kill()
            receipt["native_exit_code"] = process.wait()
    receipt.update({"finished_utc": utc(), "wall_seconds": time.monotonic() - start,
                    "stdout": str(stdout), "stderr": str(stderr),
                    "stdout_sha256": sha(stdout), "stderr_sha256": sha(stderr)})
    STEPS.append(receipt)
    write(f"{name}.receipt.json", receipt)
    for path in (stdout, stderr):
        shutil.copyfile(path, Path(SUMMARY["isolated_evidence"]) / path.name)
    write("driver-progress.json", SUMMARY)
    print(json.dumps({"milestone": name, "exit": receipt["native_exit_code"],
                      "wall_seconds": receipt["wall_seconds"]}), flush=True)
    return receipt


def manifest(base):
    return {p.relative_to(base).as_posix(): {"sha256": sha(p), "bytes": p.stat().st_size}
            for p in sorted(base.rglob("*")) if p.is_file()}


def main():
    OUT.mkdir(exist_ok=False)
    SUMMARY["started_utc"] = utc()
    SUMMARY["driver_pid"] = os.getpid()
    SUMMARY["driver_sha256"] = sha(__file__)
    before = protected()
    write("protected-before.json", before)
    result = 2
    try:
        if not NODE.is_file() or not NPM.is_file():
            raise RuntimeError("明确 Node/npm-cli 入口缺失")
        if sha(LOCK) != LOCK_HASH:
            raise RuntimeError("固定依赖 lock 身份不匹配")
        lock_data = json.loads(LOCK.read_text(encoding="utf-8"))
        if len(lock_data["packages"]) != 59:
            raise RuntimeError("固定 lock 节点数量不匹配")
        tracked = set(git("ls-files").splitlines())
        files = sorted(set(EXPLICIT + REFERENCES + [p for p in tracked
                       if p.startswith((".trellis/scripts/", ".trellis/spec/"))
                       and "__pycache__" not in p and not p.endswith((".pyc", ".pyo"))]))
        if len(files) != 67:
            raise RuntimeError(f"明确复制清单数量变化：{len(files)}")
        actual_refs = set()
        for rel in (".trellis/workflow.md", "docs/agents/harnesses.md"):
            for ref in re.findall(r"`(\.trellis/(?:tasks|scripts)/[^`]+)`", (ROOT / rel).read_text(encoding="utf-8")):
                if not re.search(r"[<>*{}\s]", ref):
                    actual_refs.add(ref)
        if actual_refs != set(REFERENCES):
            raise RuntimeError("checker 具体引用发生变化")
        for rel in files:
            source = ROOT / rel
            if not source.is_file() or not source.resolve().is_relative_to(ROOT.resolve()):
                raise RuntimeError(f"公开来源不存在或越界：{rel}")
        base = Path(tempfile.mkdtemp(prefix="trellis-t04-missing-overrides-20261002-"))
        tool, cache, candidate, evidence = [base / name for name in ("tool", "npm-cache", "candidate", "evidence")]
        for path in (tool, cache, candidate, evidence):
            path.mkdir()
        SUMMARY.update({"isolated_root": str(base), "tool": str(tool), "cache": str(cache),
                        "candidate": str(candidate), "isolated_evidence": str(evidence)})
        write("isolated-paths.json", SUMMARY)
        copy_records = []
        for rel in files:
            destination = candidate / rel
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / rel, destination)
            copy_records.append({"path": rel, "source": str(ROOT / rel),
                                 "tracked_in_current_index": rel in tracked,
                                 "selection": "explicit_public" if rel in EXPLICIT else
                                              "checker_reference" if rel in REFERENCES else "tracked_scripts_or_spec",
                                 "source_sha256": sha(ROOT / rel), "candidate_sha256": sha(destination)})
        write("copy-manifest.json", {"count": len(copy_records), "files": copy_records,
                                     "scenario": SUMMARY["scenario"], "fresh_checkout": False})
        write("protected-before.json", before)
        write("overrides-before.json", {p: {**state(p, candidate), "selected": False, "copied": False} for p in OVERRIDES})
        if any((candidate / p).exists() for p in OVERRIDES):
            raise RuntimeError("可选覆盖在 init 前意外存在")
        (tool / "package.json").write_text(json.dumps(lock_data["packages"][""], indent=2) + "\n", encoding="utf-8")
        shutil.copyfile(LOCK, tool / "package-lock.json")
        for config in ("user.npmrc", "global.npmrc"):
            (base / config).write_bytes(b"")
        write("executor-before.json", {"node": {"path": str(NODE), "sha256": sha(NODE)},
                                       "npm_cli": {"path": str(NPM), "sha256": sha(NPM)},
                                       "lock": {"sha256": LOCK_HASH, "nodes": 59},
                                       "npmrc": [state(p, base) | {"path": str(base / p)} for p in ("user.npmrc", "global.npmrc")]})
        pre_contract = run("contract-before-init", [NODE, candidate / "scripts/check-agent-contract.js"], candidate)
        if pre_contract["native_exit_code"] != 0:
            raise RuntimeError("缺失覆盖候选的共享合同在 init 前失败；保留失败并停止")
        install = run("npm-ci-attempt1", [NODE, NPM, "ci", "--prefix", tool, "--cache", cache,
                      "--userconfig", base / "user.npmrc", "--globalconfig", base / "global.npmrc",
                      "--global=false", "--ignore-scripts", "--no-audit", "--no-fund",
                      "--registry=https://registry.npmjs.org"], tool, timeout=600)
        if install["native_exit_code"] != 0:
            raise RuntimeError("固定工具依赖获取失败；不回落全局版本")
        dependency_identity = []
        for rel, item in lock_data["packages"].items():
            if not rel:
                continue
            package = tool / rel / "package.json"
            installed = json.loads(package.read_text(encoding="utf-8"))
            dependency_identity.append({"path": rel, "version_locked": item.get("version"),
                                        "version_installed": installed.get("version"),
                                        "name_installed": installed.get("name"), "package_sha256": sha(package),
                                        "resolved": item.get("resolved"), "integrity": item.get("integrity")})
        dep_ok = all(v["version_locked"] == v["version_installed"] for v in dependency_identity)
        write("dependency-identity.json", {"nodes_including_root": len(dependency_identity) + 1,
                                           "all_versions_match": dep_ok, "packages": dependency_identity,
                                           "lock_before": LOCK_HASH, "lock_after": sha(tool / "package-lock.json")})
        if not dep_ok or sha(tool / "package-lock.json") != LOCK_HASH:
            raise RuntimeError("工具 lock 或依赖版本身份发生变化")
        entry = tool / "node_modules/@mindfoldhq/trellis/bin/trellis.js"
        SUMMARY["cli_entry"] = str(entry)
        SUMMARY["cli_sha256"] = sha(entry)
        version = run("selected-cli-version", [NODE, entry, "--version"], candidate)
        version_text = Path(version["stdout"]).read_text(encoding="utf-8").strip()
        expected = (candidate / ".trellis/.version").read_text(encoding="utf-8").strip()
        SUMMARY["version"] = {"expected": expected, "actual_stdout": version_text,
                              "same_entry_for_init": True, "matches": version_text == expected}
        if version["native_exit_code"] != 0 or version_text != expected:
            raise RuntimeError("所选 CLI 版本门失败；不执行 init")
        for phase in ("before-init",):
            env_result = run("environment-" + phase, [NODE, candidate / "scripts/check-harness-environment.js",
                             "--root", candidate, "--entry", "trellis=" + str(entry)], candidate, timeout=120)
            env_json = json.loads(Path(env_result["stdout"]).read_text(encoding="utf-8"))
            write("environment-" + phase + ".json", env_json)
            if env_json["bootstrap"]["status"] != "READY":
                raise RuntimeError("候选同入口环境诊断未满足 bootstrap 前提")
        pre_manifest = manifest(candidate)
        write("candidate-before-init-manifest.json", pre_manifest)
        init = run("isolated-init", [NODE, entry, "init", "--claude", "--codex", "--grok", "--kimi", "--omp", "--skip-existing", "-y"], candidate)
        post_manifest = manifest(candidate)
        write("candidate-after-init-manifest.json", post_manifest)
        write("generated-assets.json", {p: post_manifest[p] for p in post_manifest if p not in pre_manifest})
        write("necessary-assets.json", {p: state(p, candidate) for p in ASSETS})
        write("overrides-after.json", {p: {**state(p, candidate), "selected": False, "copied": False,
                                         "origin": "isolated_init_default" if (candidate / p).is_file() else "missing"}
                                       for p in OVERRIDES})
        for rel in OVERRIDES:
            generated = candidate / rel
            if generated.is_file():
                dest = OUT / "generated-overrides" / rel
                dest.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(generated, dest)
                dest = evidence / "generated-overrides" / rel
                dest.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(generated, dest)
        copied_changes = [p for p in pre_manifest if pre_manifest[p] != post_manifest.get(p)]
        write("copied-file-preservation.json", {"changed_paths": copied_changes, "unchanged": not copied_changes})
        post_contract = run("contract-after-init", [NODE, candidate / "scripts/check-agent-contract.js"], candidate)
        post_env = run("environment-after-init", [NODE, candidate / "scripts/check-harness-environment.js",
                       "--root", candidate, "--entry", "trellis=" + str(entry)], candidate, timeout=120)
        write("environment-after-init.json", json.loads(Path(post_env["stdout"]).read_text(encoding="utf-8")))
        ignore = run("candidate-ignore", ["git", "-c", "core.excludesFile=", "-c", "core.bare=false",
                     "--git-dir=" + str(ROOT / ".git"), "--work-tree=" + str(candidate),
                     "check-ignore", "--no-index", "-v", *ASSETS, *OVERRIDES], candidate)
        SUMMARY.update({"init_exit_code": init["native_exit_code"],
                        "pre_contract_exit_code": pre_contract["native_exit_code"],
                        "post_contract_exit_code": post_contract["native_exit_code"],
                        "generated_file_count": len(set(post_manifest) - set(pre_manifest)),
                        "copied_files_unchanged": not copied_changes,
                        "necessary_assets_present": all((candidate / p).is_file() for p in ASSETS),
                        "ignore_check_exit_code": ignore["native_exit_code"],
                        "no_host_overrides_copied_or_defaults_repaired": True})
        result = 0 if init["native_exit_code"] == 0 and post_contract["native_exit_code"] == 0 and not copied_changes else 1
    except Exception as error:
        SUMMARY["failure"] = {"message": str(error), "traceback": traceback.format_exc(), "cause_inferred": False}
        print(json.dumps({"milestone": "blocked", "message": str(error)}, ensure_ascii=False), flush=True)
    finally:
        after = protected()
        write("protected-after.json", after)
        SUMMARY["protected_unchanged"] = before == after
        if before != after:
            result = 2
        SUMMARY.update({"finished_utc": utc(), "driver_intended_exit_code": result,
                        "native_exit_codes": {v["name"]: v["native_exit_code"] for v in STEPS},
                        "formal_product_ci_docs": "本轮无产品源码改动；不从最小候选运行产品/文档完整门"})
        write("result.json", SUMMARY)
        print(json.dumps({"milestone": "complete", "research": str(OUT), "driver_exit": result,
                          "protected_unchanged": SUMMARY["protected_unchanged"]}, ensure_ascii=False), flush=True)
    return result


if __name__ == "__main__":
    sys.exit(main())

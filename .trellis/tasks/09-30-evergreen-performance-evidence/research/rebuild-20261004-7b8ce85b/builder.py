"""P0/P1 only (rebuild-20261004): heap counter + chain index layout4.

Baseline 来自 rebuild-20261002 已审查的 baseline 副本，再加同一 heap 计数器补丁；candidate 从仓库按扩展后的清单复制。
工具链、构建参数、产物复制与 smoke 校验沿用 rebuild-20261002 builder。
"""
import sys
sys.dont_write_bytecode = True

from datetime import datetime, timezone
import difflib
import gzip
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import subprocess
import tarfile
import time
import psutil

EVIDENCE = Path(__file__).resolve().parent
REPO = EVIDENCE.parents[4]
TASK_RESEARCH = EVIDENCE.parent
ASSET = REPO / "bench-data/t05-rebuild-20261004-7b8ce85b"
BASELINE = ASSET / "sources/baseline"
CANDIDATE = ASSET / "sources/candidate"
BASE_TARGET = ASSET / "targets/baseline"
CAND_TARGET = ASSET / "targets/candidate"
TARGET = "x86_64-pc-windows-msvc"
REVISION = "c278bb7b56603001e32e353d2ee589dccef0bfe9"
PREFIX = Path("residential-monitor/src-tauri/src")
PREVIOUS_EVIDENCE = TASK_RESEARCH / "rebuild-20261002-4f8a8c68"
PREVIOUS_STATE_SHA = "F53BCF2C81CE7BA6FCA38A4E477BCF8FC76E4A0005E18838B995F9B8CC5A0B02"
DIST_PREFIX = "residential-monitor/dist/"
# 2026-10-04 用户批准的文件范围（design.md「2026-10-04 范围变更」）。
APPROVED_CHANGES = ("c3/schema.rs", "c3/retention_day.rs", "storage.rs", "c0_contract.rs", "bench/heap.rs", "bench.rs", "bin/monitor-bench.rs", "bench/facade.rs")
ENV_KEYS = ("VCToolsInstallDir", "VSINSTALLDIR", "WindowsSdkDir", "WindowsSDKVersion", "LIB", "INCLUDE", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER", "CARGO_BUILD_TARGET", "CARGO_TARGET_DIR")


def utc():
    return datetime.now(timezone.utc).isoformat()


def sha(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest().upper()


def load(path):
    return json.loads(Path(path).read_text(encoding="utf-8-sig"))


def save(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def safe(path, root):
    path = Path(path)
    assert path.resolve().is_relative_to(root.resolve()), str(path)
    for ancestor in (path, *path.parents):
        if ancestor.exists():
            assert not ancestor.is_symlink() and not ancestor.is_junction(), str(ancestor)
        if ancestor == root:
            break
    return path


def manifest(root, keys):
    result = {}
    for relative in sorted(keys):
        path = safe(root / relative, root)
        assert path.is_file(), str(path)
        result[relative] = {"bytes": path.stat().st_size, "sha256": sha(path)}
    return result


def tree_keys(root):
    paths = []
    for directory, subdirs, files in os.walk(root, followlinks=False):
        for name in subdirs + files:
            path = safe(Path(directory) / name, root)
            if path.is_file():
                paths.append(path.relative_to(root).as_posix())
    return sorted(paths)


def run(label, argv, cwd=REPO):
    directory = EVIDENCE / "build"
    directory.mkdir(exist_ok=True)
    receipt_path = directory / (label + ".receipt.json")
    assert not receipt_path.exists(), str(receipt_path)
    stdout_path = directory / (label + ".stdout.raw")
    stderr_path = directory / (label + ".stderr.raw")
    environment = dict(os.environ, RUSTUP_AUTO_INSTALL="0")
    value = {"label": label, "argv": argv, "cwd": str(cwd), "started_utc": utc(), "status": "RUNNING", "environment_override": {"RUSTUP_AUTO_INSTALL": "0"}, "build_environment": {key: environment.get(key) for key in ENV_KEYS}, "executable_resolved": shutil.which(argv[0]), "shell": False, "native_exit": "RUNNING", "pid": None}
    save(receipt_path, value)
    began = time.perf_counter()
    observed = {}
    with stdout_path.open("xb") as out, stderr_path.open("xb") as err:
        process = subprocess.Popen(argv, cwd=cwd, env=environment, stdin=subprocess.DEVNULL, stdout=out, stderr=err, shell=False)
        value["pid"] = process.pid
        save(receipt_path, value)
        print(json.dumps({"phase": label, "pid": process.pid, "status": "STARTED", "utc": utc()}), flush=True)
        last_progress = began
        while process.poll() is None:
            try:
                for child in psutil.Process(process.pid).children(recursive=True):
                    try:
                        item = {"pid": child.pid, "name": child.name(), "executable": child.exe()}
                        observed[(child.pid, child.create_time())] = item
                    except (psutil.NoSuchProcess, psutil.AccessDenied):
                        pass
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                pass
            if time.perf_counter() - last_progress >= 55:
                with stderr_path.open("rb") as current:
                    current.seek(max(0, stderr_path.stat().st_size - 2000))
                    lines = current.read().decode("utf-8", errors="replace").splitlines()
                print(json.dumps({"phase": label, "pid": process.pid, "elapsed_seconds": round(time.perf_counter() - began, 3), "stderr_tail": lines[-2:], "status": "RUNNING"}), flush=True)
                last_progress = time.perf_counter()
            time.sleep(0.5)
        native_exit = process.wait()
    logs = {}
    for stream, path in (("stdout", stdout_path), ("stderr", stderr_path)):
        data = path.read_bytes()
        zipped = directory / (label + "." + stream + ".raw.gz")
        zipped.write_bytes(gzip.compress(data, mtime=0))
        readable = directory / (label + "." + stream + ".log")
        readable.write_text(data.decode("utf-8", errors="replace"), encoding="utf-8")
        logs[stream] = {"raw": str(path), "bytes": len(data), "raw_sha256": sha(path), "gzip": str(zipped), "gzip_sha256": sha(zipped), "readable": str(readable), "readable_sha256": sha(readable)}
    value.update(status="ENDED", ended_utc=utc(), wall_seconds=time.perf_counter() - began, native_exit=native_exit, logs=logs, owned_descendants_observed=list(observed.values()), process_cleanup="native process waited; descendant metadata observations limited to own subtree; no absolute no-orphan claim")
    save(receipt_path, value)
    print(json.dumps({"phase": label, "native_exit": native_exit, "wall_seconds": value["wall_seconds"], "status": "ENDED"}), flush=True)
    if native_exit:
        raise RuntimeError(f"{label}: native exit {native_exit}; original raw logs retained")
    return stdout_path


def copy_file(source, target, root):
    target = safe(target, root)
    target.parent.mkdir(parents=True, exist_ok=True)
    with Path(source).open("rb") as stream, target.open("xb") as output:
        shutil.copyfileobj(stream, output)


def strip_corpus(module, cli):
    """与 rebuild-20261002 builder 相同的 baseline 去 corpus 变换，用于核对补丁后的 baseline。"""
    assert module.count("pub mod corpus;\n") == 1
    module = module.replace("pub mod corpus;\n", "", 1)
    corpus_import = "use residential_monitor_lib::bench::corpus::{generate_corpus, retain_corpus};\n"
    assert cli.count(corpus_import) == 1
    cli = cli.replace(corpus_import, "", 1)
    first = cli.index("    /// 批量生成生产 schema")
    last = cli.index("    /// 真实门面 + 档案 tick", first)
    cli = cli[:first] + cli[last:]
    first = cli.index("        Commands::GenerateCorpus {")
    last = cli.index("        Commands::ReplayFacade {", first)
    cli = cli[:first] + cli[last:]
    assert "GenerateCorpus" not in cli and "RetainCorpus" not in cli and "pub mod corpus;" not in module
    return module, cli


def prepare():
    assert not ASSET.exists(), "Asset root already exists; no continuation or overwrite"
    for path in (ASSET, EVIDENCE):
        assert path.parent.is_dir()
        for ancestor in (path.parent, *path.parent.parents):
            assert not ancestor.is_symlink() and not ancestor.is_junction(), str(ancestor)
    save(EVIDENCE / "pre-creation.json", {"observed_utc": utc(), "asset_root": str(ASSET), "asset_existed_before_creation": False})
    assert sha(PREVIOUS_EVIDENCE / "state.json") == PREVIOUS_STATE_SHA, "previous state identity drift"
    previous = load(PREVIOUS_EVIDENCE / "state.json")
    old_source, old_baseline, old_dist = previous["source_manifest"], previous["baseline_manifest"], previous["dist_manifest"]
    previous_baseline = Path(previous["baseline_source"])
    assert len(old_source) == 108 and len(old_baseline) == 103
    assert manifest(previous_baseline, old_baseline) == old_baseline, "previous baseline source drift"
    assert manifest(previous_baseline, old_dist) == old_dist, "previous baseline dist drift"
    for name in ("bench/facade.rs", "bench/process.rs", "bench/write_vfs.rs"):
        key = (PREFIX / name).as_posix()
        assert old_baseline[key] == old_source[key], "previous harness file not identical: " + name
    heap_key = (PREFIX / "bench/heap.rs").as_posix()
    keys = sorted(set(old_source) | {heap_key})
    candidate_expected = manifest(REPO, keys)
    changed = sorted(key for key in keys if old_source.get(key) != candidate_expected[key])
    approved = {(PREFIX / name).as_posix() for name in APPROVED_CHANGES}
    assert set(changed) <= approved, "candidate change outside approved scope: " + json.dumps(sorted(set(changed) - approved))
    save(EVIDENCE / "inputs/candidate-change-set.json", {"previous_state": str(PREVIOUS_EVIDENCE / "state.json"), "previous_state_sha256": PREVIOUS_STATE_SHA, "changed": {key: {"previous": old_source.get(key), "current": candidate_expected[key]} for key in changed}, "approved_scope": sorted(approved)})
    dist_keys = [(Path("residential-monitor/dist") / path).as_posix() for path in tree_keys(REPO / "residential-monitor/dist")]
    assert dist_keys, "Missing public dist input"
    dist_before = manifest(REPO, dist_keys)
    save(EVIDENCE / "inputs/repository-source-before.json", candidate_expected)
    save(EVIDENCE / "inputs/dist-copy-before.json", dist_before)
    ignore = run("preflight-ignore", ["git", "check-ignore", "-v", "--no-index", str(ASSET)])
    assert "bench-data/" in ignore.read_text(encoding="utf-8"), "Ignore boundary changed"
    rustc = run("rustc-version", ["rustc", "+1.98.0", "-vV"])
    cargo = run("cargo-version", ["cargo", "+1.98.0", "-vV"])
    installed = run("installed-targets", ["rustup", "target", "list", "--installed", "--toolchain", "1.98.0"])
    assert "88d9e12ae178fab0fb5cc050a94da85685d449ea" in rustc.read_text(encoding="utf-8")
    assert "797e8a9bca276c1c9f9f738d2a20f484fa4eea9d" in cargo.read_text(encoding="utf-8")
    assert TARGET in installed.read_text(encoding="utf-8")
    object_type = run("baseline-object", ["git", "cat-file", "-t", REVISION])
    assert object_type.read_text(encoding="utf-8").strip() == "commit"
    vswhere = "C:/Program Files (x86)/Microsoft Visual Studio/Installer/vswhere.exe"
    vs = run("vswhere", [vswhere, "-latest", "-products", "*", "-requires", "Microsoft.VisualStudio.Component.VC.Tools.x86.x64", "-property", "installationPath"])
    vs_path = Path(vs.read_text(encoding="utf-8-sig").strip())
    assert vs_path.is_dir()
    linker_candidates = []
    for version in sorted((vs_path / "VC/Tools/MSVC").iterdir()):
        if version.is_dir():
            linker = version / "bin/Hostx64/x64/link.exe"
            compiler = version / "bin/Hostx64/x64/cl.exe"
            if linker.is_file() and compiler.is_file():
                linker_candidates.append({"version": version.name, "linker": str(linker), "linker_sha256": sha(linker), "compiler": str(compiler), "compiler_sha256": sha(compiler)})
    sdk = Path("C:/Program Files (x86)/Windows Kits/10/Lib/10.0.26100.0")
    assert linker_candidates and (sdk / "um/x64/kernel32.lib").is_file() and (sdk / "ucrt/x64/ucrt.lib").is_file()
    preflight = {"status": "PASSED_SOURCE_PATH_TOOL_METADATA; ACTUAL_BUILD_NOT_RUN", "observed_utc": utc(), "source_files": len(candidate_expected), "previous_baseline_files": len(old_baseline), "instrumented_baseline_files": len(old_baseline) + 1, "dist_files": len(dist_keys), "disk_free_bytes": {drive: shutil.disk_usage(drive + ":/").free for drive in ("C", "D")}, "environment_override": {"RUSTUP_AUTO_INSTALL": "0"}, "environment_observed": {key: os.environ.get(key) for key in ENV_KEYS}, "path_linker": shutil.which("link.exe"), "path_compiler": shutil.which("cl.exe"), "linker_discovery_candidates": linker_candidates, "sdk_candidate": str(sdk), "actual_linker_and_sdk": "NOT_YET_OBSERVED; Rust/Cargo automatic discovery kept unchanged", "cargo_configuration": "No explicit global/private configuration reads or writes. Inherited environment and Cargo defaults; effects beyond observed argv/output remain unverified.", "resource_boundary": "Current free bytes recorded; peak target/build resource demand unknown; no disk cleanup", "builder_sha256": sha(__file__)}
    save(EVIDENCE / "preflight.json", preflight)
    ASSET.mkdir()
    previous_tree = [key for key in tree_keys(previous_baseline) if not key.startswith(DIST_PREFIX)]
    previous_tree_manifest = manifest(previous_baseline, previous_tree)
    for relative in previous_tree:
        copy_file(previous_baseline / relative, BASELINE / relative, BASELINE)
    assert manifest(BASELINE, previous_tree) == previous_tree_manifest
    save(EVIDENCE / "inputs/baseline-previous-tree.json", previous_tree_manifest)
    module_path = safe(BASELINE / PREFIX / "bench.rs", BASELINE)
    cli_path = safe(BASELINE / PREFIX / "bin/monitor-bench.rs", BASELINE)
    module_before, cli_before = module_path.read_text(encoding="utf-8"), cli_path.read_text(encoding="utf-8")
    module = module_before
    assert module.count("pub mod facade;\n") == 1
    module = module.replace("pub mod facade;\n", "pub mod facade;\npub mod heap;\n", 1)
    cli = cli_before
    for old_text, new_text in (
        ("use residential_monitor_lib::bench::{\n", "use residential_monitor_lib::bench::heap::{enable_from_env, CountingAlloc};\nuse residential_monitor_lib::bench::{\n"),
        ("#[derive(Parser)]\n", "#[global_allocator]\nstatic GLOBAL: CountingAlloc = CountingAlloc;\n\n#[derive(Parser)]\n"),
        ("fn main() {\n    let cli = Cli::parse();\n", "fn main() {\n    enable_from_env();\n    let cli = Cli::parse();\n"),
    ):
        assert cli.count(old_text) == 1, old_text
        cli = cli.replace(old_text, new_text, 1)
    expected_module, expected_cli = strip_corpus((REPO / PREFIX / "bench.rs").read_text(encoding="utf-8"), (REPO / PREFIX / "bin/monitor-bench.rs").read_text(encoding="utf-8"))
    assert module == expected_module, "patched baseline bench.rs differs from candidate without corpus"
    assert cli == expected_cli, "patched baseline monitor-bench.rs differs from candidate without corpus"
    outputs = {"bench.rs": module.encode("utf-8"), "bin/monitor-bench.rs": cli.encode("utf-8"), "bench/facade.rs": (REPO / PREFIX / "bench/facade.rs").read_bytes(), "bench/heap.rs": (REPO / PREFIX / "bench/heap.rs").read_bytes()}
    patch = []
    for name, data in outputs.items():
        key = (PREFIX / name).as_posix()
        assert hashlib.sha256(data).hexdigest().upper() == candidate_expected[key]["sha256"] or name in ("bench.rs", "bin/monitor-bench.rs"), name
        target = safe(BASELINE / PREFIX / name, BASELINE)
        before = target.read_bytes() if target.exists() else b""
        patch.extend(difflib.unified_diff(before.decode("utf-8").splitlines(keepends=True), data.decode("utf-8").splitlines(keepends=True), fromfile="a/" + key, tofile="b/" + key))
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
    (EVIDENCE / "inputs/baseline-instrumentation.patch").write_text("".join(patch), encoding="utf-8", newline="")
    baseline_expected = manifest(BASELINE, sorted(set(old_baseline) | {heap_key}))
    save(EVIDENCE / "inputs/baseline-instrumented-source.json", baseline_expected)
    save(EVIDENCE / "inputs/instrumentation-identity.json", {"base": "rebuild-20261002 reviewed baseline copy (c278 + corpus-free instrumentation)", "outputs": {name: {"bytes": len(data), "sha256": hashlib.sha256(data).hexdigest().upper()} for name, data in outputs.items()}, "patch_sha256": sha(EVIDENCE / "inputs/baseline-instrumentation.patch"), "production_or_cargo_changes": False})
    CANDIDATE.mkdir(parents=True)
    assert manifest(REPO, keys) == candidate_expected, "Repository changed before copy"
    for relative in keys:
        copy_file(REPO / relative, CANDIDATE / relative, CANDIDATE)
    assert manifest(CANDIDATE, keys) == candidate_expected
    save(EVIDENCE / "inputs/candidate-copy-after.json", manifest(CANDIDATE, keys))
    for root, label in ((BASELINE, "baseline"), (CANDIDATE, "candidate")):
        for relative in dist_keys:
            copy_file(REPO / relative, root / relative, root)
        copied = manifest(root, dist_keys)
        assert copied == dist_before
        save(EVIDENCE / ("inputs/" + label + "-dist-copy-after.json"), copied)
    assert manifest(REPO, keys) == candidate_expected and manifest(REPO, dist_keys) == dist_before
    state = {"status": "PREPARED_NOT_BUILT", "created_utc": utc(), "repo": str(REPO), "asset_root": str(ASSET), "evidence_root": str(EVIDENCE), "previous_evidence": str(PREVIOUS_EVIDENCE), "baseline_revision": REVISION, "baseline_source": str(BASELINE), "candidate_source": str(CANDIDATE), "baseline_target": str(BASE_TARGET), "candidate_target": str(CAND_TARGET), "executables": str(ASSET / "executables"), "toolchain": "1.98.0", "target": TARGET, "environment_override": {"RUSTUP_AUTO_INSTALL": "0"}, "source_manifest": candidate_expected, "baseline_manifest": baseline_expected, "dist_manifest": dist_before, "candidate_changed_files": changed, "heap_counter": "monitor-bench only; RESIWATCH_BENCH_HEAP=1 enables; formal runs keep it disabled", "builder_sha256": sha(__file__), "formal_loads": "NOT_RUN", "corpus_generation": "NOT_RUN", "scope": "P0/P1 only; fresh source and two release target roots; no product or previous evidence writes"}
    save(EVIDENCE / "state.json", state)
    return state


def stable_inputs(state, label):
    checks = {
        "repository_source": manifest(REPO, state["source_manifest"]) == state["source_manifest"],
        "baseline_source": manifest(BASELINE, state["baseline_manifest"]) == state["baseline_manifest"],
        "candidate_source": manifest(CANDIDATE, state["source_manifest"]) == state["source_manifest"],
        "repository_dist": manifest(REPO, state["dist_manifest"]) == state["dist_manifest"],
        "baseline_dist": manifest(BASELINE, state["dist_manifest"]) == state["dist_manifest"],
        "candidate_dist": manifest(CANDIDATE, state["dist_manifest"]) == state["dist_manifest"],
        "builder_identity": sha(__file__) == state["builder_sha256"],
    }
    save(EVIDENCE / ("inputs/" + label + "-validation.json"), {"observed_utc": utc(), "checks": checks})
    assert all(checks.values()), f"Input drift during {label}"
    for root, keys, name in ((BASELINE, state["baseline_manifest"], "baseline"), (CANDIDATE, state["source_manifest"], "candidate")):
        save(EVIDENCE / ("inputs/" + name + "-" + label + "-source.json"), manifest(root, keys))
        save(EVIDENCE / ("inputs/" + name + "-" + label + "-dist.json"), manifest(root, state["dist_manifest"]))


def artifact(output, name, is_test=False):
    found = []
    for line in output.read_text(encoding="utf-8").splitlines():
        try:
            value = json.loads(line)
        except json.JSONDecodeError:
            continue
        if value.get("reason") != "compiler-artifact" or not value.get("executable"):
            continue
        target = value.get("target", {})
        if target.get("name") == name and bool(value.get("profile", {}).get("test")) == is_test:
            if is_test or "bin" in target.get("kind", []):
                found.append(value)
    assert len(found) == 1, f"Expected one compiler-artifact for {name}; got {len(found)}"
    return found[0]


def build_and_smoke(state):
    stable_inputs(state, "before-build")
    options = ["--locked", "--release", "--target", TARGET]
    baseline_args = ["cargo", "+1.98.0", "build", *options, "--manifest-path", str(BASELINE / "residential-monitor/src-tauri/Cargo.toml"), "--target-dir", str(BASE_TARGET), "--bin", "monitor-bench", "--message-format=json"]
    candidate_args = ["cargo", "+1.98.0", "build", *options, "--manifest-path", str(CANDIDATE / "residential-monitor/src-tauri/Cargo.toml"), "--target-dir", str(CAND_TARGET), "--bin", "monitor-bench", "--bin", "monitor-db", "--message-format=json"]
    test_args = ["cargo", "+1.98.0", "test", *options, "--manifest-path", str(CANDIDATE / "residential-monitor/src-tauri/Cargo.toml"), "--target-dir", str(CAND_TARGET), "--lib", "--no-run", "--message-format=json"]
    state["status"] = "BASELINE_BUILD_RUNNING"
    save(EVIDENCE / "state.json", state)
    baseline_out = run("baseline-build", baseline_args)
    baseline_artifact = artifact(baseline_out, "monitor-bench")
    state["status"] = "CANDIDATE_BUILD_RUNNING"
    save(EVIDENCE / "state.json", state)
    candidate_out = run("candidate-build", candidate_args)
    candidate_bench_artifact = artifact(candidate_out, "monitor-bench")
    candidate_db_artifact = artifact(candidate_out, "monitor-db")
    state["status"] = "CANDIDATE_LIBRARY_TEST_BUILD_RUNNING"
    save(EVIDENCE / "state.json", state)
    test_out = run("candidate-library-test-build", test_args)
    library_artifact = artifact(test_out, "residential_monitor_lib", is_test=True)
    stable_inputs(state, "after-build")
    executable_directory = ASSET / "executables"
    executable_directory.mkdir()
    identities = {}
    for name, item, expected_root in (("baseline-monitor-bench.exe", baseline_artifact, BASE_TARGET), ("candidate-monitor-bench.exe", candidate_bench_artifact, CAND_TARGET), ("candidate-monitor-db.exe", candidate_db_artifact, CAND_TARGET), ("candidate-library-tests.exe", library_artifact, CAND_TARGET)):
        source = safe(Path(item["executable"]), expected_root)
        assert source.is_file()
        target = safe(executable_directory / name, ASSET)
        with source.open("rb") as stream, target.open("xb") as output:
            shutil.copyfileobj(stream, output)
        assert sha(source) == sha(target)
        identities[name] = {"source": str(source), "path": str(target), "bytes": target.stat().st_size, "sha256": sha(target), "compiler_artifact": item}
    save(EVIDENCE / "build/executable-identity.json", identities)
    state["executable_identities"] = identities
    state["status"] = "BUILT_SMOKE_NOT_RUN"
    save(EVIDENCE / "state.json", state)
    smokes = {}
    for side in ("baseline", "candidate"):
        directory = ASSET / "data/smoke" / side
        assert not directory.exists()
        directory.parent.mkdir(parents=True, exist_ok=True)
        identity = identities[side + "-monitor-bench.exe"]
        assert sha(identity["path"]) == identity["sha256"]
        source_label = REVISION if side == "baseline" else "rebuild-20261004-candidate"
        argv = [identity["path"], "replay-facade", "--active", "8", "--hz", "1", "--duration-secs", "3", "--warmup-secs", "0", "--virtual-time", "--start-utc", "1800001800", "--seed", "20260919", "--workload", "counters", "--metadata-change-percent", "100", "--archive", "complete", "--query-every-frames", "0", "--source-revision", source_label, "--dir", str(directory)]
        state["status"] = side.upper() + "_SMOKE_RUNNING"
        save(EVIDENCE / "state.json", state)
        output = run(side + "-smoke", argv)
        result = load(output)
        save(EVIDENCE / ("build/" + side + "-smoke-result.json"), result)
        assert result["kind"] == "isolated-real-facade"
        assert result["frames"] == 3 and result["commits"] == 3 and result["traffic"]["conserved"] is True
        assert result["options"]["duration_secs"] == 3 and result["options"]["hz"] == 1
        assert result["options"]["virtual_time"] is True and result["options"]["active"] == 8
        for key, expected in {"warmup_secs": 0, "start_utc": 1800001800, "seed": 20260919, "workload": "counters", "metadata_change_percent": 100, "archive": "complete", "query_every_frames": 0}.items():
            assert result["options"][key] == expected, key
        assert result["executable_sha256"].upper() == identity["sha256"]
        assert result["process_id"] == load(EVIDENCE / ("build/" + side + "-smoke.receipt.json"))["pid"]
        assert result["synchronous"] == "FULL"
        for field in ("initial_files_and_pages", "final_files_and_pages"):
            assert result[field]["journal_mode"] == "wal" and result[field]["synchronous"] == 2
        assert sha(identity["path"]) == identity["sha256"]
        smokes[side] = result
    assert smokes["baseline"]["fixture_hash"] == smokes["candidate"]["fixture_hash"]
    stable_inputs(state, "after-smoke")
    state.update(status="P1_BUILD_AND_VIRTUAL_SMOKE_VALIDATED; WAITING_INDEPENDENT_REVIEW", completed_utc=utc(), virtual_smoke_fixture_hash=smokes["baseline"]["fixture_hash"], corpus_generation="NOT_RUN", formal_matrix="NOT_RUN", formal_primary="NOT_RUN", formal_capacity="NOT_RUN")
    save(EVIDENCE / "state.json", state)
    save(ASSET / "manifest.json", {"asset_root": str(ASSET), "evidence_root": str(EVIDENCE), "baseline_revision": REVISION, "source_manifests": {"baseline": str(EVIDENCE / "inputs/baseline-instrumented-source.json"), "candidate": str(EVIDENCE / "inputs/candidate-copy-after.json")}, "dist_manifest": str(EVIDENCE / "inputs/dist-copy-before.json"), "executables": identities, "virtual_smoke": {"fixture_hash": state["virtual_smoke_fixture_hash"], "commits_each": 3, "conserved_each": True, "writer_each_initial_final": "WAL/FULL"}, "formal_loads": "NOT_RUN", "asset_preservation": "No automatic cleanup; new stable ignored root; old assets and evidence unchanged"})


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    driver_path = EVIDENCE / "builder-driver.receipt.json"
    assert not driver_path.exists(), "Driver already ran; do not rerun or overwrite"
    driver = {"argv": [sys.executable, "-B", str(Path(__file__).resolve())], "cwd": str(Path.cwd()), "pid": os.getpid(), "started_utc": utc(), "status": "RUNNING", "scope": "P0/P1 only (20261004 heap counter + layout4); no corpus generation or formal load", "python_dont_write_bytecode": sys.dont_write_bytecode}
    save(driver_path, driver)
    code = 0
    try:
        state = prepare()
        build_and_smoke(state)
        driver["status"] = "P1_ENDED_WAITING_INDEPENDENT_REVIEW"
    except Exception as error:
        code = 1
        driver.update(status="STOPPED_FIRST_FAILURE", error_type=type(error).__name__, error=str(error))
        print(json.dumps({"status": driver["status"], "error_type": type(error).__name__, "error": str(error)}), flush=True)
    driver.update(ended_utc=utc(), driver_exit=code)
    save(driver_path, driver)
    raise SystemExit(code)

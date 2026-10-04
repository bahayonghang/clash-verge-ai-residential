"""P2 only: full A50/A250 30-day corpus generation, SQL oracle and diagnostic stage probes."""

import sys

sys.dont_write_bytecode = True

from datetime import datetime, timezone
import gzip
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
import psutil

EVIDENCE = Path(__file__).resolve().parent
REPO = EVIDENCE.parents[4]
TASK_RESEARCH = EVIDENCE.parent
ASSET = REPO / "bench-data/t05-rebuild-20261002-4f8a8c68"
OUT = EVIDENCE / "generator"
ORACLE = EVIDENCE / "oracle"
START, DAY_START, END = 1787184000, 1789689600, 1789776000
SEED = 20260919
HISTORICAL_ORACLE = TASK_RESEARCH / "corpus-a250-verification.json"
HISTORICAL_ORACLE_SHA = (
    "684240C5352CB3182D8B066F987ED5664787901FB9C842F5E4E065B1D1CC64EB"
)
NONEMPTY_TEST = (
    "c3::service::raw_stage_probe_tests::isolated_nonempty_report_stage_proof"
)
RAW_FOLD_TEST = "c3::raw_fold::tests::isolated_raw_fold_stage_proof"
COUNTS = {
    50: {
        "sessions": 432000,
        "minutes": 2160000,
        "chains": 1296000,
        "receipts": 2592000,
    },
    250: {
        "sessions": 2160000,
        "minutes": 10800000,
        "chains": 6480000,
        "receipts": 2592000,
    },
}


def utc():
    return datetime.now(timezone.utc).isoformat()


def sha(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest().upper()


def load(path):
    return json.loads(Path(path).read_text(encoding="utf-8-sig"))


def save(path, value, exclusive=False):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("x" if exclusive else "w", encoding="utf-8") as stream:
        stream.write(json.dumps(value, ensure_ascii=False, indent=2) + "\n")


def require(condition, detail):
    if not condition:
        raise RuntimeError(detail)


def sidecars(db):
    return {
        suffix: (
            Path(str(db) + suffix).stat().st_size
            if Path(str(db) + suffix).exists()
            else None
        )
        for suffix in ("-wal", "-shm")
    }


def competing_load():
    items = []
    for process in psutil.process_iter(["pid", "name", "cmdline"]):
        try:
            command = " ".join(process.info["cmdline"] or [])
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            continue
        if "ferrots" in command or process.info["name"] in (
            "cargo.exe",
            "rustc.exe",
            "monitor-bench.exe",
            "monitor-db.exe",
        ):
            items.append(
                {
                    "pid": process.info["pid"],
                    "name": process.info["name"],
                    "cmdline_head": command[:160],
                }
            )
    return items


def run(label, argv, environment_override=None, cwd=REPO):
    receipt_path = OUT / (label + ".receipt.json")
    require(not receipt_path.exists(), "receipt already exists: " + str(receipt_path))
    stdout_path = OUT / (label + ".stdout.raw")
    stderr_path = OUT / (label + ".stderr.raw")
    override = dict(environment_override or {})
    environment = dict(os.environ, **override)
    value = {
        "label": label,
        "argv": argv,
        "cwd": str(cwd),
        "environment_override": override,
        "shell": False,
        "started_utc": utc(),
        "status": "RUNNING",
        "native_exit": "RUNNING",
        "pid": None,
        "competing_load_at_start": competing_load(),
        "disk_free_bytes_before": {
            drive: shutil.disk_usage(drive + ":/").free for drive in ("C", "D")
        },
    }
    save(receipt_path, value, exclusive=True)
    began = time.perf_counter()
    peak_rss = 0
    peak_cpu_times = None
    with stdout_path.open("xb") as out, stderr_path.open("xb") as err:
        process = subprocess.Popen(
            argv,
            cwd=cwd,
            env=environment,
            stdin=subprocess.DEVNULL,
            stdout=out,
            stderr=err,
            shell=False,
        )
        value["pid"] = process.pid
        save(receipt_path, value)
        print(
            json.dumps(
                {"phase": label, "pid": process.pid, "status": "STARTED", "utc": utc()}
            ),
            flush=True,
        )
        last_progress = began
        while process.poll() is None:
            try:
                handle = psutil.Process(process.pid)
                peak_rss = max(peak_rss, handle.memory_info().rss)
                peak_cpu_times = handle.cpu_times()._asdict()
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                pass
            if time.perf_counter() - last_progress >= 300:
                print(
                    json.dumps(
                        {
                            "phase": label,
                            "pid": process.pid,
                            "elapsed_seconds": round(time.perf_counter() - began, 1),
                            "peak_rss_bytes": peak_rss,
                            "status": "RUNNING",
                        }
                    ),
                    flush=True,
                )
                last_progress = time.perf_counter()
            time.sleep(0.5)
        native_exit = process.wait()
    logs = {}
    for stream, path in (("stdout", stdout_path), ("stderr", stderr_path)):
        data = path.read_bytes()
        zipped = OUT / (label + "." + stream + ".raw.gz")
        zipped.write_bytes(gzip.compress(data, mtime=0))
        readable = OUT / (label + "." + stream + ".log")
        readable.write_text(data.decode("utf-8", errors="replace"), encoding="utf-8")
        logs[stream] = {
            "raw": str(path),
            "bytes": len(data),
            "raw_sha256": sha(path),
            "gzip_sha256": sha(zipped),
            "readable_sha256": sha(readable),
        }
    value.update(
        status="ENDED",
        ended_utc=utc(),
        wall_seconds=time.perf_counter() - began,
        native_exit=native_exit,
        logs=logs,
        peak_rss_bytes_sampled=peak_rss,
        last_cpu_times_sampled=peak_cpu_times,
        disk_free_bytes_after={
            drive: shutil.disk_usage(drive + ":/").free for drive in ("C", "D")
        },
        sampling="0.5 s psutil polling of the direct native process; not a complete peak proof",
    )
    save(receipt_path, value)
    print(
        json.dumps(
            {
                "phase": label,
                "native_exit": native_exit,
                "wall_seconds": round(value["wall_seconds"], 3),
                "status": "ENDED",
            }
        ),
        flush=True,
    )
    require(
        native_exit == 0,
        f"{label}: native exit {native_exit}; original raw logs retained",
    )
    return stdout_path


def preflight():
    state = load(EVIDENCE / "state.json")
    require(
        state["status"].startswith("P1_BUILD_AND_VIRTUAL_SMOKE_VALIDATED"),
        "P1 not validated: " + state["status"],
    )
    require(
        load(EVIDENCE / "builder-driver.receipt.json")["driver_exit"] == 0,
        "P1 driver exit is not 0",
    )
    identities = state["executable_identities"]
    observed = {}
    for name, identity in identities.items():
        observed[name] = sha(identity["path"])
        require(observed[name] == identity["sha256"], "executable drift: " + name)
    require(manifest_matches(state), "candidate source drift")
    for active in (50, 250):
        require(
            not (ASSET / f"corpus-a{active}-30d").exists(),
            f"corpus-a{active}-30d already exists; no continuation or overwrite",
        )
    require(not OUT.exists(), "generator evidence directory already exists")
    require(
        sha(HISTORICAL_ORACLE) == HISTORICAL_ORACLE_SHA,
        "historical oracle identity drift",
    )
    value = {
        "status": "PASSED_GENERATOR_PREFLIGHT; GENERATION_NOT_RUN",
        "observed_utc": utc(),
        "executable_sha256": observed,
        "disk_free_bytes": {
            drive: shutil.disk_usage(drive + ":/").free for drive in ("C", "D")
        },
        "competing_load": competing_load(),
        "generator_sha256": sha(__file__),
        "competing_load_policy": "Recorded only; generation wall time is not a formal gate. User processes are not stopped.",
        "fixed_arguments": {
            "days": 30,
            "seed": SEED,
            "start_utc": START,
            "expected_end_utc": END,
        },
    }
    save(OUT / "preflight.json", value, exclusive=True)
    return state


def manifest_matches(state):
    candidate = Path(state["candidate_source"])
    for relative, item in state["source_manifest"].items():
        path = candidate / relative
        if (
            not path.is_file()
            or path.stat().st_size != item["bytes"]
            or sha(path) != item["sha256"]
        ):
            return False
    return True


def generate(state, active):
    directory = ASSET / f"corpus-a{active}-30d"
    bench = state["executable_identities"]["candidate-monitor-bench.exe"]
    argv = [
        bench["path"],
        "generate-corpus",
        "--average-active",
        str(active),
        "--days",
        "30",
        "--seed",
        str(SEED),
        "--start-utc",
        str(START),
        "--dir",
        str(directory),
    ]
    output = run(f"generate-a{active}", argv)
    require(
        sha(bench["path"]) == bench["sha256"],
        "candidate monitor-bench drift after generation",
    )
    report = load(output)
    db = directory / "monitor.sqlite3"
    marker = directory / "production-corpus.json"
    require(db.is_file() and marker.is_file(), "generated DB or marker missing")
    manifest = load(marker)
    require(manifest == report, "stdout report differs from marker")
    checks = {
        "kind": manifest["kind"] == "production-corpus"
        and manifest["schema_version"] == 1,
        "window": (manifest["start_utc"], manifest["end_utc"], manifest["minutes"])
        == (START, END, 43200),
        "full_30_day_input": manifest["full_30_day_input"] is True,
        "counts": manifest["counts_match"] is True
        and manifest["expected"] == COUNTS[active]
        and manifest["actual"] == COUNTS[active],
        "schema": manifest["sqlite_schema_version"] == 5,
        "seed_and_active": manifest["workload"]["seed"] == SEED
        and manifest["workload"]["average_active"] == active,
        "generation_cache_kib": manifest["generation_cache_kib"] == 65536,
        "zero_wal": sidecars(db)["-wal"] in (None, 0),
    }
    identity = {
        "active": active,
        "db": str(db),
        "marker": str(marker),
        "db_bytes": db.stat().st_size,
        "db_sha256": sha(db),
        "marker_sha256": sha(marker),
        "sidecars": sidecars(db),
        "checks": checks,
        "generation_wall_secs_reported": manifest["generation_wall_secs"],
        "observed_utc": utc(),
    }
    save(OUT / f"a{active}-identity.json", identity, exclusive=True)
    require(all(checks.values()), f"A{active} marker checks failed: {checks}")
    return identity


def sql_contract():
    path = ORACLE / "sql-contract.json"
    require(not path.exists(), "sql-contract.json already exists")
    historical = load(HISTORICAL_ORACLE)
    queries = {item["name"]: item for item in historical["queries"]}
    templates = {}
    for name in (
        "all_and_residential_totals_oracle",
        "residential_host_rank_oracle",
        "residential_network_rank_oracle",
    ):
        item = queries[name]
        require(item["status"] == "ok", name)
        templates[name] = {
            "sql": item["sql"],
            "historical_params": item["params"],
            "historical_rows": item["result"],
        }
    contract = {
        "source": str(HISTORICAL_ORACLE),
        "source_sha256": HISTORICAL_ORACLE_SHA,
        "derivation": "SQL text, params and result rows copied byte-for-byte from the historical A250 oracle; no hand edits",
        "templates": templates,
    }
    save(path, contract, exclusive=True)
    return sha(path)


def oracle(identities):
    script = ORACLE / "read-corpus-oracle.py"
    output = ORACLE / "run-20261002"
    argv = [sys.executable, "-B", str(script)]
    for active in (50, 250):
        item = identities[active]
        argv += [
            f"--a{active}-db",
            item["db"],
            f"--a{active}-db-sha256",
            item["db_sha256"],
            f"--a{active}-marker-sha256",
            item["marker_sha256"],
        ]
    argv += ["--output-dir", str(output)]
    run("sql-oracle", argv)
    summary = load(output / "summary.json")
    require(
        summary["status"] == "PASS_SQL_ORACLE_NATIVE_NOT_RUN",
        "oracle summary: " + summary["status"],
    )
    return {active: load(output / f"a{active}.json") for active in (50, 250)}


def probes(state, identities, oracles):
    tests = state["executable_identities"]["candidate-library-tests.exe"]
    listing = run("test-list", [tests["path"], "--list", "--ignored"]).read_text(
        encoding="utf-8"
    )
    for name in (NONEMPTY_TEST, RAW_FOLD_TEST):
        require(
            f"{name}: test" in listing.splitlines(),
            "exact ignored test missing: " + name,
        )
    db = identities[250]["db"]
    probe_dir = OUT / "probes"
    probe_dir.mkdir()
    totals = {
        row["name"]: row["rows"][0]
        for row in oracles[250]["queries"]
        if row["name"].startswith("all_and_residential")
    }
    results = {}
    cases = [
        (
            "a250-minute-production",
            NONEMPTY_TEST,
            {
                "RESIWATCH_NONEMPTY_STAGE_START": str(START),
                "RESIWATCH_NONEMPTY_STAGE_END": str(START + 60),
            },
            "NONEMPTY",
        ),
        (
            "a250-30d-production",
            NONEMPTY_TEST,
            {
                "RESIWATCH_NONEMPTY_STAGE_START": str(START),
                "RESIWATCH_NONEMPTY_STAGE_END": str(END),
            },
            "NONEMPTY",
        ),
        (
            "a250-30d-all",
            RAW_FOLD_TEST,
            {
                "RESIWATCH_RAW_FOLD_START": str(START),
                "RESIWATCH_RAW_FOLD_END": str(END),
                "RESIWATCH_RAW_FOLD_FILTER": "all",
            },
            "RAW_FOLD",
        ),
        (
            "a250-30d-residential",
            RAW_FOLD_TEST,
            {
                "RESIWATCH_RAW_FOLD_START": str(START),
                "RESIWATCH_RAW_FOLD_END": str(END),
                "RESIWATCH_RAW_FOLD_FILTER": "residential",
            },
            "RAW_FOLD",
        ),
    ]
    for label, test, window, prefix in cases:
        result_path = probe_dir / (label + ".json")
        override = dict(
            window,
            **{
                f"RESIWATCH_{prefix}_STAGE_DB": db,
                f"RESIWATCH_{prefix}_STAGE_OUT": str(result_path),
            },
        )
        stdout = run(
            "probe-" + label,
            [
                tests["path"],
                "--exact",
                test,
                "--ignored",
                "--nocapture",
                "--test-threads=1",
            ],
            override,
        ).read_text(encoding="utf-8")
        require(
            "test result: ok. 1 passed" in stdout,
            label + ": exactly one test did not pass",
        )
        require(result_path.is_file(), label + ": JSON missing")
        value = load(result_path)
        checks = {}
        if prefix == "NONEMPTY":
            production = value["production_first_read"]
            checks["production_status_ok"] = production["status"] == "ok"
            checks["tier_raw"] = production.get("tier") == "Raw"
            checks["nonempty"] = production.get("connection_count", 0) > 0
            checks["deadline_ms_le_10000"] = value["production_first_read_ms"] <= 10000
            if label == "a250-30d-production":
                expected = totals["all_and_residential_totals_30d"]
                checks["totals_match_oracle"] = (
                    production["upload"],
                    production["download"],
                    production["connection_count"],
                ) == tuple(expected[:3])
                checks["series_rows_720"] = production["series_rows"] == 720
        results[label] = {
            "result": str(result_path),
            "checks": checks,
            "production_first_read_ms": value.get("production_first_read_ms"),
        }
    require(sha(tests["path"]) == tests["sha256"], "library tests executable drift")
    after = {
        active: {
            "db_sha256": sha(identities[active]["db"]),
            "sidecars": sidecars(identities[active]["db"]),
        }
        for active in (50, 250)
    }
    save(
        OUT / "probe-summary.json",
        {
            "results": results,
            "db_after": after,
            "boundary": "Diagnostic only. Pages already cached by generation, hash and oracle; not physical cold reads. Does not close F12 non-empty same-window gate or capacity gate.",
        },
        exclusive=True,
    )
    require(
        all(
            after[active]["db_sha256"] == identities[active]["db_sha256"]
            for active in (50, 250)
        ),
        "DB hash changed after probes",
    )
    return results


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    driver_path = EVIDENCE / "generator-driver.receipt.json"
    require(
        not driver_path.exists(),
        "generator driver already ran; do not rerun or overwrite",
    )
    driver = {
        "argv": [sys.executable, "-B", str(Path(__file__).resolve())],
        "pid": os.getpid(),
        "started_utc": utc(),
        "status": "RUNNING",
        "generator_sha256": sha(__file__),
        "scope": "P2 only; no matrix/primary/capacity",
    }
    save(driver_path, driver, exclusive=True)
    code = 0
    state = None
    try:
        state = preflight()
        identities = {active: generate(state, active) for active in (50, 250)}
        driver["sql_contract_sha256"] = sql_contract()
        oracles = oracle(identities)
        probe_results = probes(state, identities, oracles)
        failed = {
            label: checks
            for label, item in probe_results.items()
            for checks in [item["checks"]]
            if not all(checks.values())
        }
        state = load(EVIDENCE / "state.json")
        state.update(
            corpus_generation="PASS_A50_A250_FULL_30D",
            sql_oracle="PASS_SQL_ORACLE_NATIVE_NOT_RUN",
            corpus_identities={
                str(active): {
                    key: identities[active][key]
                    for key in (
                        "db",
                        "marker",
                        "db_sha256",
                        "marker_sha256",
                        "db_bytes",
                    )
                }
                for active in identities
            },
            stage_probes="PASS_DIAGNOSTIC"
            if not failed
            else "FAIL_DIAGNOSTIC_RETAINED",
            stage_probe_failures=failed,
            generator_sha256=sha(__file__),
            status="P2_GENERATION_ORACLE_PROBES_ENDED; WAITING_P3_RUNNER_BINDING",
        )
        save(EVIDENCE / "state.json", state)
        driver["status"] = "P2_ENDED"
        driver["stage_probe_failures"] = failed
    except Exception as error:
        code = 1
        driver.update(
            status="STOPPED_FIRST_FAILURE",
            error_type=type(error).__name__,
            error=str(error),
        )
        print(
            json.dumps(
                {"status": driver["status"], "error": str(error)}, ensure_ascii=False
            ),
            flush=True,
        )
    driver.update(ended_utc=utc(), driver_exit=code)
    save(driver_path, driver)
    return code


if __name__ == "__main__":
    raise SystemExit(main())

"""P2 completion after generator.py stopped on its own probe-check KeyError.

--bind: no native execution. Retain the generator failure, record probe diagnostics, bind corpus identities into state.json.
--quiet-probes: run the two raw_fold stage probes not yet run; refuses to start while competing load exists.
"""

import sys

sys.dont_write_bytecode = True

import argparse
import json
import os
from pathlib import Path

import generator as g

EVIDENCE = g.EVIDENCE
OUT = EVIDENCE / "p2-completion"


def bind():
    driver = g.load(EVIDENCE / "generator-driver.receipt.json")
    g.require(
        driver["status"] == "STOPPED_FIRST_FAILURE" and driver["driver_exit"] == 1,
        "unexpected generator driver state",
    )
    identities = {
        active: g.load(g.OUT / f"a{active}-identity.json") for active in (50, 250)
    }
    for active, item in identities.items():
        g.require(
            all(item["checks"].values()), f"A{active} identity checks not all true"
        )
        g.require(
            g.sha(item["db"]) == item["db_sha256"]
            and g.sha(item["marker"]) == item["marker_sha256"],
            f"A{active} corpus drift",
        )
    oracle = g.load(g.ORACLE / "run-20261002/summary.json")
    g.require(oracle["status"] == "PASS_SQL_ORACLE_NATIVE_NOT_RUN", "oracle not passed")
    probes = {}
    for label in ("a250-minute-production", "a250-30d-production"):
        receipt = g.load(g.OUT / f"probe-{label}.receipt.json")
        value = g.load(g.OUT / "probes" / f"{label}.json")
        production = value["production_first_read"]
        stdout = (g.OUT / f"probe-{label}.stdout.log").read_text(encoding="utf-8")
        checks = {
            "native_exit_0": receipt["native_exit"] == 0,
            "exactly_one_test_passed": "test result: ok. 1 passed" in stdout,
            "production_status_ok": production["status"] == "ok",
            "tier_raw": production.get("tier") == "Raw",
            "nonempty": production.get("connection_count", 0) > 0,
            "deadline_ms_le_10000": value["production_first_read_ms"] <= 10000,
        }
        if label == "a250-30d-production":
            expected = [
                q
                for q in g.load(g.ORACLE / "run-20261002/a250.json")["queries"]
                if q["name"] == "all_and_residential_totals_30d"
            ][0]["rows"][0]
            checks["totals_match_oracle"] = (
                production.get("upload"),
                production.get("download"),
                production.get("connection_count"),
            ) == tuple(expected[:3])
            checks["series_rows_720"] = production.get("series_rows") == 720
            checks["stage_connection_count_matches_oracle"] = (
                value.get("stage_connection_count") == expected[2]
            )
        probes[label] = {
            "checks": checks,
            "status": "PASS_DIAGNOSTIC" if all(checks.values()) else "FAIL_DIAGNOSTIC",
            "production_first_read": production,
            "production_first_read_ms": value["production_first_read_ms"],
            "raw_query_ms": value.get("raw_query_ms"),
            "competing_load_at_start": receipt["competing_load_at_start"],
        }
    after = {
        str(active): {
            "db_sha256": g.sha(identities[active]["db"]),
            "sidecars": g.sidecars(identities[active]["db"]),
        }
        for active in (50, 250)
    }
    g.require(
        all(
            after[str(active)]["db_sha256"] == identities[active]["db_sha256"]
            for active in (50, 250)
        ),
        "DB changed after probes",
    )
    record = {
        "observed_utc": g.utc(),
        "generator_failure": {
            "status": driver["status"],
            "error_type": driver.get("error_type"),
            "error": driver.get("error"),
            "cause": "generator.py probe check read production['upload'] after production status=error; driver-side KeyError, not a native failure",
        },
        "probes": probes,
        "db_after": after,
        "raw_fold_probes": "NOT_RUN by generator; run later with quiet-probes",
        "boundary": "Diagnostic only. Competing ferrots CUDA load was present; pages cached by generation/hash/oracle. Does not replace or close any formal gate.",
        "completion_sha256": g.sha(__file__),
    }
    g.save(OUT / "bind.json", record, exclusive=True)
    state = g.load(EVIDENCE / "state.json")
    g.require(
        state["status"].startswith("P1_BUILD_AND_VIRTUAL_SMOKE_VALIDATED"),
        "state status drift",
    )
    state.update(
        corpus_generation="PASS_A50_A250_FULL_30D",
        sql_oracle="PASS_SQL_ORACLE_NATIVE_NOT_RUN",
        corpus_identities={
            str(active): {
                key: identities[active][key]
                for key in ("db", "marker", "db_sha256", "marker_sha256", "db_bytes")
            }
            for active in identities
        },
        stage_probes={label: item["status"] for label, item in probes.items()}
        | {"a250-30d-all": "NOT_RUN", "a250-30d-residential": "NOT_RUN"},
        generator_driver="STOPPED_FIRST_FAILURE (driver KeyError after production DeadlineExceeded); see p2-completion/bind.json",
        status="P2_GENERATION_ORACLE_PROBES_ENDED; WAITING_P3_RUNNER_BINDING",
    )
    g.save(EVIDENCE / "state.json", state)
    print(
        json.dumps({label: item["status"] for label, item in probes.items()}),
        flush=True,
    )


def quiet_probes():
    load = g.competing_load()
    g.require(
        not load, "competing load present: " + json.dumps(load, ensure_ascii=False)
    )
    state = g.load(EVIDENCE / "state.json")
    g.require(
        state["status"].startswith("P2_GENERATION_ORACLE_PROBES_ENDED"), "P2 not bound"
    )
    g.OUT = OUT
    tests = state["executable_identities"]["candidate-library-tests.exe"]
    g.require(g.sha(tests["path"]) == tests["sha256"], "library tests drift")
    db = state["corpus_identities"]["250"]["db"]
    results = {}
    for filter_name in ("all", "residential"):
        label = "a250-30d-" + filter_name
        result_path = OUT / "probes" / (label + ".json")
        result_path.parent.mkdir(exist_ok=True)
        override = {
            "RESIWATCH_RAW_FOLD_STAGE_DB": db,
            "RESIWATCH_RAW_FOLD_STAGE_OUT": str(result_path),
            "RESIWATCH_RAW_FOLD_START": str(g.START),
            "RESIWATCH_RAW_FOLD_END": str(g.END),
            "RESIWATCH_RAW_FOLD_FILTER": filter_name,
        }
        stdout = g.run(
            "probe-" + label,
            [
                tests["path"],
                "--exact",
                g.RAW_FOLD_TEST,
                "--ignored",
                "--nocapture",
                "--test-threads=1",
            ],
            override,
        ).read_text(encoding="utf-8")
        g.require(
            "test result: ok. 1 passed" in stdout,
            label + ": exactly one test did not pass",
        )
        results[label] = g.load(result_path)
    g.require(
        g.sha(db) == state["corpus_identities"]["250"]["db_sha256"], "A250 DB changed"
    )
    g.save(
        OUT / "quiet-probes.json",
        {
            "observed_utc": g.utc(),
            "results": results,
            "boundary": "Diagnostic stage split; pages already cached; not a formal gate",
        },
        exclusive=True,
    )
    state["stage_probes"].update({label: "RECORDED_DIAGNOSTIC" for label in results})
    g.save(EVIDENCE / "state.json", state)


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    parser = argparse.ArgumentParser()
    parser.add_argument("mode", choices=["bind", "quiet-probes"])
    os.chdir(EVIDENCE)
    mode = parser.parse_args().mode
    bind() if mode == "bind" else quiet_probes()

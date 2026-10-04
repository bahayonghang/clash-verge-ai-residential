"""重跑：首次 heap-diagnostics 的最后 4 次运行在后台 shell 达到 2 小时时限被停止后以 0xC0000142 启动失败；本副本只改输出目录，参数不变。
F6/F7 分阶段堆峰值诊断（非正式计量，rebuild-20261004 binding）。

使用与正式 matrix 相同的程序与 F6/F7 场景参数，只额外设置 RESIWATCH_BENCH_HEAP=1。
每个场景 baseline/candidate 交替各 3 轮。结果只用于归属，不替代正式门。
"""

import sys

sys.dont_write_bytecode = True
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json
import os
import subprocess
import time

EVIDENCE = Path(__file__).resolve().parents[1]
STATE = json.loads((EVIDENCE / "state.json").read_text(encoding="utf-8"))
assert STATE["status"].startswith("P2_GENERATION_ORACLE_PROBES_ENDED"), STATE["status"]
OUT = EVIDENCE / "heap-diagnostics-rerun"
DATA = Path(STATE["asset_root"]) / "data" / "heap-diagnostics-rerun"
identities = {
    side: STATE["executable_identities"][side + "-monitor-bench.exe"]
    for side in ("baseline", "candidate")
}
ROUNDS = 3


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest().upper()


def save(path, value):
    Path(path).write_text(
        json.dumps(value, ensure_ascii=False, indent=2) + chr(10), encoding="utf-8"
    )


for identity in identities.values():
    assert sha(identity["path"]) == identity["sha256"]
assert not OUT.exists() and not DATA.exists()
OUT.mkdir()
DATA.mkdir(parents=True)
runs = []
for workload in ("unchanged", "metadata"):
    for round_number in range(1, ROUNDS + 1):
        for side in ("baseline", "candidate"):
            label = f"a1000-{workload}-r{round_number}-{side}"
            source_label = (
                STATE["baseline_revision"]
                if side == "baseline"
                else "heap-diagnostics-candidate"
            )
            argv = [
                identities[side]["path"],
                "replay-facade",
                "--active",
                "1000",
                "--hz",
                "1",
                "--duration-secs",
                "30",
                "--warmup-secs",
                "5",
                "--workload",
                workload,
                "--metadata-change-percent",
                "100",
                "--archive",
                "complete",
                "--query-every-frames",
                "5",
                "--seed",
                "20260919",
                "--start-utc",
                "1800001800",
                "--source-revision",
                source_label,
                "--dir",
                str(DATA / label),
            ]
            started = time.perf_counter()
            process = subprocess.run(
                argv,
                env=dict(os.environ, RESIWATCH_BENCH_HEAP="1"),
                capture_output=True,
            )
            report = json.loads(process.stdout) if process.returncode == 0 else None
            record = {
                "label": label,
                "workload": workload,
                "round": round_number,
                "side": side,
                "argv": argv,
                "exit_code": process.returncode,
                "wall_seconds": time.perf_counter() - started,
                "stderr_tail": process.stderr.decode("utf-8", errors="replace")[-2000:],
            }
            if report is not None:
                save(OUT / (label + ".json"), report)
                record.update(
                    {
                        "native_private_p95": report["native_private_bytes"]["p95"],
                        "native_private_max": report["native_private_bytes"]["max"],
                        "heap_phases": report["heap_phases"],
                        "executable_sha256": report["executable_sha256"],
                    }
                )
            runs.append(record)
            save(OUT / "runs.json", runs)
            print(
                json.dumps(
                    {
                        "finished": label,
                        "exit_code": process.returncode,
                        "private_p95": record.get("native_private_p95"),
                    }
                ),
                flush=True,
            )


def median(values):
    values = sorted(values)
    return values[len(values) // 2] if values else None


summary = {"rounds": ROUNDS, "scenes": {}}
for workload in ("unchanged", "metadata"):
    scene = {}
    for side in ("baseline", "candidate"):
        items = [
            run
            for run in runs
            if run["workload"] == workload
            and run["side"] == side
            and run.get("heap_phases")
        ]
        phases = {}
        for kind in ("rust", "sqlite"):
            names = items[0]["heap_phases"][kind].keys() if items else []
            phases[kind] = {
                name: {
                    "median_p95": median(
                        [item["heap_phases"][kind][name]["p95"] for item in items]
                    ),
                    "median_max": median(
                        [item["heap_phases"][kind][name]["max"] for item in items]
                    ),
                }
                for name in names
            }
        scene[side] = {
            "runs": len(items),
            "private_p95_median": median(
                [item["native_private_p95"] for item in items]
            ),
            "rust_window_peak_median": median(
                [item["heap_phases"]["rust_window_peak"] for item in items]
            ),
            "sqlite_window_peak_median": median(
                [item["heap_phases"]["sqlite_window_peak"] for item in items]
            ),
            "phases": phases,
        }
    summary["scenes"]["a1000-" + workload] = scene
summary["boundary"] = (
    "Diagnostic only; heap counter adds per-allocation atomics; not a formal F6/F7 measurement."
)
save(OUT / "summary.json", summary)
sys.exit(0 if all(run["exit_code"] == 0 for run in runs) else 2)

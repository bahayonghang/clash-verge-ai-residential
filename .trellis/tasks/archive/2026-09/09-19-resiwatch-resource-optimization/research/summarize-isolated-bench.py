"""汇总已完成的真实回放；不把 SQLite 子集或 native 指标升级为完整应用验收。"""

import json
import sys
from pathlib import Path


def reduction(before, after):
    return None if before in (None, 0) or after is None else 100 * (1 - after / before)


def latency(result, name, field):
    sample = result["latency"][name]
    return sample[field] if sample["count"] else None


root = Path(sys.argv[1])
comparisons = []
for baseline_path in sorted(root.glob("*-baseline.json")):
    candidate_path = baseline_path.with_name(baseline_path.name.replace("-baseline.json", "-candidate.json"))
    if not candidate_path.exists() or candidate_path.stat().st_size == 0:
        continue
    baseline = json.loads(baseline_path.read_text(encoding="utf-8-sig"))
    candidate = json.loads(candidate_path.read_text(encoding="utf-8-sig"))
    if baseline["fixture_hash"] != candidate["fixture_hash"]:
        raise ValueError(f"输入不一致: {baseline_path.name}")
    if not all(x["traffic"]["conserved"] for x in (baseline, candidate)):
        raise ValueError(f"流量不守恒: {baseline_path.name}")
    row = {
        "name": baseline_path.name.removesuffix("-baseline.json"),
        "fixture_hash": baseline["fixture_hash"],
        "virtual_time": baseline["options"]["virtual_time"],
        "cpu_seconds": [baseline["native_cpu_seconds"], candidate["native_cpu_seconds"]],
        "cpu_reduction_percent": reduction(baseline["native_cpu_seconds"], candidate["native_cpu_seconds"]),
        "sqlite_xwrite_bytes": [baseline["sqlite_application_file_write_bytes"], candidate["sqlite_application_file_write_bytes"]],
        "sqlite_xwrite_reduction_percent": reduction(baseline["sqlite_application_file_write_bytes"], candidate["sqlite_application_file_write_bytes"]),
        "native_private_p95_bytes": [baseline["native_private_bytes"]["p95"], candidate["native_private_bytes"]["p95"]],
        "ingest_p95_ms": [latency(x, "facade_ingest_including_durable_commit", "p95_ms") for x in (baseline, candidate)],
        "ingest_max_ms": [latency(x, "facade_ingest_including_durable_commit", "max_ms") for x in (baseline, candidate)],
        "report_first_reader_p95_ms": [latency(x, "report_first_reader", "p95_ms") for x in (baseline, candidate)],
        "frame_budget_overruns": [x["frame_budget_overruns"] for x in (baseline, candidate)],
    }
    comparisons.append(row)

primary = [row for row in comparisons if row["name"].startswith("primary-r")]
complete = len(primary) == 3 and not any(row["virtual_time"] for row in primary)
result = {
    "comparisons": comparisons,
    "primary_three_rounds_complete": complete,
    "primary_cpu_reduction_percent": reduction(*[sum(row["cpu_seconds"][i] for row in primary) for i in range(2)]) if complete else None,
    "primary_sqlite_xwrite_reduction_percent": reduction(*[sum(row["sqlite_xwrite_bytes"][i] for row in primary) for i in range(2)]) if complete else None,
    "all_application_file_writes_gate": "UNVERIFIED: SQLite xWrite is only a subset",
    "native_plus_webview_memory_gate": "UNVERIFIED: this harness has no WebView",
    "collector_background_worker_gate": "UNVERIFIED: synchronous facade/archive driver",
}
output = root / "comparison-summary.json"
output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result, ensure_ascii=False, indent=2))

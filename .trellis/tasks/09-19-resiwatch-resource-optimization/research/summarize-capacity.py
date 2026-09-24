"""汇总原始容量证据；生成成功、查询成功和保留守恒分别报告。"""

import json
import math
import sys
from pathlib import Path


root = Path(sys.argv[1])
corpora = []
for path in sorted(root.glob("corpus-a*-30d.json")):
    if path.stat().st_size:
        result = json.loads(path.read_text(encoding="utf-8-sig"))
        corpora.append({
            "name": path.stem,
            "generation_wall_secs": result["generation_wall_secs"],
            "counts_match": result["counts_match"],
            "rows": result["actual"],
            "files_and_pages": result["files_and_pages"],
        })

index_path = root / "capacity-query-index.jsonl"
rows = [json.loads(line) for line in index_path.read_text(encoding="utf-8-sig").splitlines() if line.strip()]
queries = []
for active in (50, 250, 1000):
    for window in ("30d", "1d"):
        prefix = f"query-a{active}-{window}-"
        runs = [row for row in rows if row["name"].startswith(prefix)]
        repeated = sorted(row["wall_ms"] for row in runs if not row["first_reader"] and row["exit_code"] == 0)
        queries.append({
            "average_active": active,
            "window": window,
            "runs": len(runs),
            "successful_runs": sum(row["exit_code"] == 0 for row in runs),
            "first_reader_ms": runs[0]["wall_ms"] if runs else None,
            "first_reader_exit": runs[0]["exit_code"] if runs else None,
            "repeated_success_p50_ms": repeated[math.ceil(len(repeated) * .5) - 1] if repeated else None,
            "repeated_success_p95_ms": repeated[math.ceil(len(repeated) * .95) - 1] if repeated else None,
            "repeated_success_max_ms": max(repeated) if repeated else None,
        })

result = {
    "corpora": corpora,
    "queries": queries,
    "network_count_checks": [row for row in rows if row["name"].startswith("network-")],
    "boundaries": [
        "Query timings include CLI startup; OS cache was not flushed.",
        "Nonzero query exits remain failures, not zero counts.",
        "Generation success does not establish retention, soak or total application memory acceptance.",
    ],
}
output = root / "capacity-summary.json"
output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result, ensure_ascii=False, indent=2))

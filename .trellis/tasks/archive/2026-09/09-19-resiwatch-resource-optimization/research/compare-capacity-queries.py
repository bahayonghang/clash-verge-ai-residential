"""对比同一fixture的完整成功CLI输出，仅排除生成时刻。"""

import json
import sys
from pathlib import Path


before, after = map(Path, sys.argv[1:3])
checks = []
for name in [f"query-a{active}-{window}-0.json" for active, window in [(50, "30d"), (50, "1d"), (250, "1d"), (1000, "1d")]] + ["network-a50-30d.json"]:
    pair = []
    for root in (before, after):
        data = json.loads((root / name).read_text(encoding="utf-8-sig"))
        data.pop("generatedUtc", None)
        pair.append(data)
    checks.append({"name": name, "equal_except_generated_utc": pair[0] == pair[1]})
result = {"before": str(before), "after": str(after), "checks": checks}
(after / "query-equivalence.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result, ensure_ascii=False, indent=2))
if not all(check["equal_except_generated_utc"] for check in checks):
    raise SystemExit("成功查询结果不同；检查原始JSON")

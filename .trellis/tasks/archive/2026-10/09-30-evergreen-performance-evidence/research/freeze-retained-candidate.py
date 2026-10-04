"""Repeat the full verified gate after removing the rejected MinuteSet experiment."""
from pathlib import Path

root = Path(__file__).resolve().parent
template = (root / "freeze-final-candidate.py").read_text(encoding="utf-8")
replacements = {
    'OUT = ROOT / "candidate-final-20260930"': 'OUT = ROOT / "candidate-retained-20260930"',
    'baseline_prior = json.loads((BUILD / "baseline-source-after.json").read_text())': 'baseline_prior = json.loads((ROOT / "candidate-final-20260930/baseline-source-after.json").read_text())',
    'destination = Path(STATE["executables"]) / "candidate-final-20260930"': 'destination = Path(STATE["executables"]) / "candidate-retained-20260930"',
    '"baseline-monitor-bench.exe": Path(STATE["baseline_target"]) / TARGET / "release/monitor-bench.exe"': '"baseline-monitor-bench.exe": Path(json.loads((ROOT / "candidate-final-20260930/executable-identity.json").read_text())["baseline-monitor-bench.exe"]["path"])',
}
for before, after in replacements.items():
    assert template.count(before) == 1, before
    template = template.replace(before, after, 1)
start = template.index('old = (BASE / relative).read_bytes()')
end = template.index('save(OUT / "baseline-source-before.json", baseline_before)')
template = template[:start] + '''new = (REPO / relative).read_bytes()
assert (BASE / relative).read_bytes() == new
baseline_before = baseline_prior
''' + template[end:]
lines = template.splitlines(True)
baseline_build = [line for line in lines if line.startswith('call("final-baseline-build", ')]
assert len(baseline_build) == 1
template = template.replace(baseline_build[0], '')
generated = root / "freeze-retained-driver.py"
assert not generated.exists()
generated.write_text(template, encoding="utf-8")
exec(compile(template, str(generated), "exec"), {"__file__": str(generated), "__name__": "__main__"})

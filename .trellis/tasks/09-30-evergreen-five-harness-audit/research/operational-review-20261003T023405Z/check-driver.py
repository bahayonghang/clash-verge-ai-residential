"""独立检查收据；只在主会话 CHECK_GO 后执行。"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from datetime import datetime, timezone

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[5]
OUT = Path(__file__).resolve().parent
ENTRY = Path('C:/Users/lyh/AppData/Local/Temp/trellis-t04-missing-overrides-20261002-q7twazn_/tool/node_modules/@mindfoldhq/trellis/bin/trellis.js')
T06 = '.trellis/tasks/09-30-evergreen-runtime-validation/research/'
FILES = [
    '.codex/config.toml', '.kimi-code/skills/trellis-implement/SKILL.md',
    '.kimi-code/skills/trellis-check/SKILL.md', '.kimi-code/skills/trellis-research/SKILL.md',
    '.trellis/.template-hashes.json', 'scripts/bootstrap-harnesses.js',
    'scripts/check-agent-contract.js', 'scripts/check-harness-environment.js',
    'tests/bootstrap-harnesses.test.js', 'package.json', 'justfile',
    'docs/agents/harnesses.md', '.trellis/spec/frontend/quality-guidelines.md',
] + [T06 + name for name in (
    'runtime-capture.py', 'runtime-parent-prompt.txt', 'runtime-candidate.json',
    'runtime-capture-operational.py', 'operational-kimi-acp.py',
    'operational-parent-prompt.txt', 'operational-verification.py', 'operational-kimi-source.json',
    'operational-bindings.json', 'runtime-bindings-20261001.json',
)] + ['scripts/harness-templates/' + name for name in (
    'codex-config.toml', 'kimi-trellis-implement.md', 'kimi-trellis-check.md', 'kimi-trellis-research.md',
)]

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None

def write(name, value):
    (OUT / name).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

def git(*args):
    return subprocess.run(['git', *args], cwd=ROOT, capture_output=True, check=True).stdout.decode('utf-8').strip()

def state():
    index = Path(git('rev-parse', '--git-path', 'index'))
    if not index.is_absolute():
        index = ROOT / index
    return {'files': {name: digest(ROOT / name) for name in FILES},
            'index_sha256': digest(index), 'head': git('rev-parse', 'HEAD'),
            'branch': git('branch', '--show-current'),
            'entry': str(ENTRY), 'entry_sha256': digest(ENTRY),
            'boundary': '四本机覆盖仅计算hash；未读取local配置、数据库、凭据或历史内容'}

def run(name, argv, timeout=1800):
    started = datetime.now(timezone.utc).isoformat()
    with (OUT / (name + '.stdout.raw.log')).open('xb') as stdout, (OUT / (name + '.stderr.raw.log')).open('xb') as stderr:
        process = subprocess.Popen(argv, cwd=ROOT, stdin=subprocess.DEVNULL, stdout=stdout, stderr=stderr,
                                   shell=False, creationflags=subprocess.CREATE_NO_WINDOW)
        try:
            code = process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            process.terminate()
            code = process.wait(timeout=10)
            write(name + '.timeout.json', {'timeout_seconds': timeout, 'native_exit_code': code})
    value = {'argv': argv, 'shell': False, 'started_utc': started,
             'ended_utc': datetime.now(timezone.utc).isoformat(), 'native_exit_code': code,
             'stdout_sha256': digest(OUT / (name + '.stdout.raw.log')),
             'stderr_sha256': digest(OUT / (name + '.stderr.raw.log'))}
    write(name + '.receipt.json', value)
    print(json.dumps({'check': name, 'native_exit_code': code}), flush=True)
    return code

def main():
    if sys.argv[1:] not in (['CHECK_GO_T04'], ['CHECK_GO']):
        raise RuntimeError('缺少主会话CHECK_GO；未启动检查')
    t04_only = sys.argv[1:] == ['CHECK_GO_T04']
    before = state()
    write('protected-before.json' if t04_only else 'full-protected-before.json', before)
    steps = [
        ('focused-fixtures', ['node', '--test', 'tests/bootstrap-harnesses.test.js', 'tests/check-harness-environment.test.js', 'tests/check-agent-contract.test.js']),
        ('t06-synthetic', [sys.executable, '-B', '-X', 'utf8', T06 + 'operational-verification.py']),
        ('just-ci', ['just', 'ci']), ('docs-build', ['just', 'docs-build']),
        ('secret-scan', ['node', 'scripts/check-template-safety.js']),
        ('skills-check', ['node', 'scripts/install-agent-skills.js', '--check']),
    ]
    for task in ('09-30-evergreen-five-harness-audit', '09-30-evergreen-bootstrap-check', '09-30-evergreen-runtime-validation'):
        steps.append(('validate-' + task, [sys.executable, '-B', '-X', 'utf8', '.trellis/scripts/task.py', 'validate', task]))
    if t04_only:
        steps = steps[:1]
    else:
        steps = steps[1:]
    codes = {name: run(name, argv) for name, argv in steps}
    write('t04-checks.json' if t04_only else 'full-checks.json', codes)
    if any(codes.values()):
        write('protected-after.json', state())
        return 1
    if not t04_only:
        write('full-protected-after.json', state())
        return 0
    if not ENTRY.is_file():
        raise RuntimeError('已核验固定入口缺失；不安装或回落')
    candidate = Path(tempfile.mkdtemp(prefix='trellis-t04-operational-review-')) / 'candidate'
    if not candidate.resolve().is_relative_to(Path(tempfile.gettempdir()).resolve()) or candidate.exists():
        raise RuntimeError('候选路径未满足隔离条件')
    write('native-paths.json', {'candidate_absolute': str(candidate.resolve()), 'candidate_existed': False,
                              'entry_absolute': str(ENTRY.resolve()), 'entry_sha256': digest(ENTRY),
                              'scope': '67项公开最低合同；非完整checkout；不读取或复制本机覆盖'})
    code = run('native-bootstrap', ['node', 'scripts/bootstrap-harnesses.js', '--root', str(candidate), '--entry', str(ENTRY)], 180)
    result = json.loads((OUT / 'native-bootstrap.stdout.raw.log').read_text(encoding='utf-8'))
    write('native-bootstrap-result.json', result)
    codes['native-bootstrap'] = code
    after = state()
    write('protected-after.json', after)
    write('preservation.json', {'same': before == after,
                              'changed_files': [name for name in FILES if before['files'][name] != after['files'][name]],
                              'index_head_branch_same': all(before[key] == after[key] for key in ('index_sha256', 'head', 'branch')),
                              'global_config_trust': '仅未执行写入命令；未读私有内容，不声明全盘hash不变'})
    write('t04-checks.json', codes)
    return 0 if code == 0 and result.get('ok') is True and before == after else 1

if __name__ == '__main__':
    sys.stdout.reconfigure(encoding='utf-8')
    raise SystemExit(main())

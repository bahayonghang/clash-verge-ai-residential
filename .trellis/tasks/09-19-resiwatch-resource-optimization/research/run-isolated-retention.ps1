param(
    [Parameter(Mandatory)][string]$TestExe,
    [Parameter(Mandatory)][string]$Fixture,
    [Parameter(Mandatory)][string]$EvidenceRoot,
    [Parameter(Mandatory)][string]$Label,
    [Parameter(Mandatory)][ValidateSet('first-day', 'all-days', 'aged-dimensions')][string]$Expiry
)

# 仅生成器标记的隔离库；测试程序内部再次验证marker，产品删除常量不变。
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$manifestPath = Join-Path $Fixture 'production-corpus.json'
$manifest = Get-Content -Raw -LiteralPath $manifestPath | ConvertFrom-Json
if ($manifest.kind -ne 'production-corpus') { throw '不是本任务生成的容量库' }
$resultPath = Join-Path $Fixture 'retention-test-result.json'
$logPath = Join-Path $EvidenceRoot "$Label.log"
if ((Test-Path -LiteralPath $resultPath) -or (Test-Path -LiteralPath $logPath)) {
    throw '保留已有实验结果；使用独立fixture与label'
}
$nowUtc = switch ($Expiry) {
    'first-day' { [long]$manifest.start_utc + 31 * 86400 }
    'all-days' { [long]$manifest.end_utc + 31 * 86400 }
    'aged-dimensions' { [long]$manifest.end_utc + 397 * 86400 }
}
$priorDir = $env:RESIWATCH_BENCH_CORPUS_DIR
$priorNow = $env:RESIWATCH_BENCH_CORPUS_NOW_UTC
$priorChunks = $env:RESIWATCH_BENCH_CORPUS_CHUNKS
try {
    $env:RESIWATCH_BENCH_CORPUS_DIR = (Resolve-Path -LiteralPath $Fixture).Path
    $env:RESIWATCH_BENCH_CORPUS_NOW_UTC = "$nowUtc"
    $env:RESIWATCH_BENCH_CORPUS_CHUNKS = '100000'
    $timer = [System.Diagnostics.Stopwatch]::StartNew()
    Write-Output "$([DateTime]::UtcNow.ToString('o')) START $Label now_utc=$nowUtc"
    & $TestExe --ignored --exact bench::corpus::tests::isolated_corpus_retention_capacity_gate --nocapture --test-threads=1 *> $logPath
    $testExit = $LASTEXITCODE
    $timer.Stop()
    if (Test-Path -LiteralPath $resultPath) {
        Copy-Item -LiteralPath $resultPath -Destination (Join-Path $EvidenceRoot "$Label.json")
    }
    [pscustomobject]@{label=$Label; fixture=$Fixture; expiry=$Expiry; now_utc=$nowUtc; exit_code=$testExit; wall_seconds=$timer.Elapsed.TotalSeconds} |
        ConvertTo-Json -Compress | Tee-Object -FilePath (Join-Path $EvidenceRoot 'retention-index.jsonl') -Append
    if ($testExit -ne 0) { throw "保留容量测试失败：$Label；exit=$testExit；证据已保留" }
} finally {
    $env:RESIWATCH_BENCH_CORPUS_DIR = $priorDir
    $env:RESIWATCH_BENCH_CORPUS_NOW_UTC = $priorNow
    $env:RESIWATCH_BENCH_CORPUS_CHUNKS = $priorChunks
}

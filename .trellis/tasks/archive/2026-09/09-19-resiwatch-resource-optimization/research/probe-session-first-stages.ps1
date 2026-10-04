#requires -Version 5.1
param(
    [Parameter(Mandatory)][string]$Fixture,
    [Parameter(Mandatory)][string]$Output
)

$ErrorActionPreference = 'Stop'
$manifestPath = Join-Path $Fixture 'production-corpus.json'
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
if ($manifest.kind -ne 'production-corpus') { throw '只允许显式标记的生成库' }
if (Test-Path -LiteralPath $Output) { throw "证据已存在：$Output" }
$outputParent = Split-Path -Parent $Output
New-Item -ItemType Directory -Path $outputParent -Force | Out-Null
$env:RESIWATCH_SESSION_FIRST_STAGE_DB = Join-Path $Fixture 'monitor.sqlite3'
$env:RESIWATCH_SESSION_FIRST_STAGE_OUT = [System.IO.Path]::GetFullPath($Output)
$hashes = @('residential-monitor/src-tauri/src/c3/service.rs', 'residential-monitor/src-tauri/src/c3/sql.rs') | ForEach-Object {
    [pscustomobject]@{ path = $_; sha256 = (Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash }
}
$env:RESIWATCH_SESSION_FIRST_STAGE_SOURCE = $hashes | ConvertTo-Json -Compress
& rtk proxy cargo test --release --manifest-path residential-monitor/src-tauri/Cargo.toml --lib isolated_session_first_stage_proof -- --ignored --nocapture
if ($LASTEXITCODE -ne 0) { throw "session-first SQL阶段证明失败：exit=$LASTEXITCODE" }

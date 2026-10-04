param(
  [Parameter(Mandatory = $true)][string]$Name,
  [Parameter(Mandatory = $true)][string]$Command
)

$ErrorActionPreference = 'Stop'
$logPath = Join-Path $PSScriptRoot ($Name + '.log')
$resultPath = Join-Path $PSScriptRoot ($Name + '.result.json')
if ((Test-Path -LiteralPath $logPath) -or (Test-Path -LiteralPath $resultPath)) {
  throw "Evidence already exists: $Name"
}
$started = [DateTimeOffset]::Now
$output = & pwsh -NoProfile -Command "$Command; exit `$LASTEXITCODE" 2>&1
$code = $LASTEXITCODE
$output | Set-Content -LiteralPath $logPath -Encoding utf8
@{
  command = $Command
  startedAt = $started.ToString('o')
  finishedAt = [DateTimeOffset]::Now.ToString('o')
  exitCode = $code
  log = [IO.Path]::GetFileName($logPath)
} | ConvertTo-Json | Set-Content -LiteralPath $resultPath -Encoding utf8
Write-Output "$Name exit=$code log=$logPath"
exit $code

# Terminal Orchestrator — Smoke Suite (v0.6)
#
# Demonstrates the three things v0.6 was built to make easy:
#
#   1. Ctrl-C interruption  — start a long-running command, interrupt it cleanly.
#   2. Vim exit              — drive a TUI app to quit without the user knowing how.
#   3. Multi-spawn + reap    — boot several terminals, disconnect, verify they died.
#
# Usage:
#   .\playbook_tests.ps1                   # visible windows
#   .\playbook_tests.ps1 -Headless         # no windows ever appear
#   .\playbook_tests.ps1 -Headless -KeepLogs

[CmdletBinding()]
param(
    [switch]$Headless,
    [switch]$KeepLogs
)

$ErrorActionPreference = 'Stop'

# ─── Pipe + JSON plumbing ─────────────────────────────────────────────────────

$pipeName    = 'vterm-rs-skill'
$visibleFlag = -not $Headless.IsPresent
$nextReqId   = 0
$results     = @()

$pipe = [System.IO.Pipes.NamedPipeClientStream]::new('.', $pipeName, [System.IO.Pipes.PipeDirection]::InOut)
try {
    $pipe.Connect(2000)
} catch {
    Write-Host "FATAL: orchestrator not running on \\.\pipe\$pipeName." -ForegroundColor Red
    Write-Host "       start it in another shell with:  cargo run --release -- $(if ($Headless) {'--headless'} else {'--visible'})" -ForegroundColor Yellow
    exit 1
}
$writer = [System.IO.StreamWriter]::new($pipe);     $writer.AutoFlush = $true
$reader = [System.IO.StreamReader]::new($pipe)

# Send one command, return the response object. -Depth 12 is mandatory; PowerShell's
# default of 2 silently truncates anything deeper, which used to hang us forever.
function Invoke-Cmd($obj) {
    $script:nextReqId += 1
    $obj.req_id = $script:nextReqId
    $json = $obj | ConvertTo-Json -Compress -Depth 12
    $writer.WriteLine($json)

    $line = $reader.ReadLine()
    if (-not $line) { throw "pipe closed waiting for req_id $($obj.req_id)" }
    $resp = $line | ConvertFrom-Json
    if ($resp.req_id -ne $obj.req_id) {
        throw "req_id mismatch: sent $($obj.req_id), got $($resp.req_id)"
    }
    $tag = if ($resp.status -eq 'success') { 'OK ' } else { 'ERR' }
    $color = if ($resp.status -eq 'success') { 'DarkGray' } else { 'Yellow' }
    Write-Host ("  <- {0} req={1} dur={2}ms" -f $tag, $resp.req_id, $resp.duration_ms) -ForegroundColor $color
    return $resp
}

function Assert($name, $condition, $duration_ms) {
    $status = if ($condition) { 'PASS' } else { 'FAIL' }
    $color  = if ($condition) { 'Green' } else { 'Red' }
    Write-Host ("  {0,-4} {1}" -f $status, $name) -ForegroundColor $color
    $script:results += [PSCustomObject]@{ Name = $name; Status = $status; Duration = $duration_ms }
}

# ─── TEST 1: Ctrl-C interruption ──────────────────────────────────────────────
# Goal: prove the AI can interrupt a hung command and recover the prompt.

Write-Host "`n[TEST 1] Ctrl-C interruption" -ForegroundColor Cyan

$spawn = Invoke-Cmd @{ type = 'Spawn'; payload = @{ title = 'ctrl-c demo'; visible = $visibleFlag } }
$id = $spawn.id

# Start an indefinite ping. -t makes it run forever — Ctrl-C is the only way out.
$null = Invoke-Cmd @{ type = 'ScreenWrite'; payload = @{ id = $id; text = 'ping -t 127.0.0.1<Enter>' } }
$null = Invoke-Cmd @{ type = 'WaitUntil'; payload = @{ id = $id; pattern = 'Reply from'; timeout_ms = 5000 } }

# Now interrupt it.
$null = Invoke-Cmd @{ type = 'ScreenWrite'; payload = @{ id = $id; text = '<C-c>' } }

# Confirm the prompt returned.
$ctrlC = Invoke-Cmd @{ type = 'WaitUntil'; payload = @{ id = $id; pattern = 'PS '; timeout_ms = 5000 } }
$read  = Invoke-Cmd @{ type = 'ScreenRead'; payload = @{ id = $id } }

Assert 'Ctrl-C returned to prompt' ($ctrlC.status -eq 'success' -and $read.content -match 'PS ') $ctrlC.duration_ms

$null = Invoke-Cmd @{ type = 'ScreenClose'; payload = @{ id = $id; target = 'single' } }

# ─── TEST 2: Vim exit ─────────────────────────────────────────────────────────
# Goal: prove the AI can drive a TUI app it knows nothing about.
# We test only if vim is on PATH; otherwise we mark SKIPPED.

Write-Host "`n[TEST 2] Vim exit" -ForegroundColor Cyan

$hasVim = $null -ne (Get-Command vim -ErrorAction SilentlyContinue)
if (-not $hasVim) {
    Write-Host "  SKIP vim not on PATH" -ForegroundColor DarkYellow
    $results += [PSCustomObject]@{ Name = 'Vim exit'; Status = 'SKIP'; Duration = 0 }
} else {
    $vimSpawn = Invoke-Cmd @{ type = 'Spawn'; payload = @{ title = 'vim demo'; visible = $visibleFlag } }
    $vid = $vimSpawn.id
    $null = Invoke-Cmd @{ type = 'ScreenWrite'; payload = @{ id = $vid; text = 'vim<Enter>' } }
    $null = Invoke-Cmd @{ type = 'Wait'; payload = @{ ms = 500 } }
    $null = Invoke-Cmd @{ type = 'ScreenWrite'; payload = @{ id = $vid; text = '<Esc>:q!<Enter>' } }
    $vexit = Invoke-Cmd @{ type = 'WaitUntil'; payload = @{ id = $vid; pattern = 'PS '; timeout_ms = 3000 } }
    Assert 'Vim exited cleanly' ($vexit.status -eq 'success') $vexit.duration_ms
    $null = Invoke-Cmd @{ type = 'ScreenClose'; payload = @{ id = $vid; target = 'single' } }
}

# ─── TEST 3: Multi-spawn + reap on disconnect ────────────────────────────────

Write-Host "`n[TEST 3] Multi-spawn + reap" -ForegroundColor Cyan

$ids = @()
foreach ($name in @('svc-a','svc-b','svc-c')) {
    $r = Invoke-Cmd @{ type = 'Spawn'; payload = @{ title = $name; visible = $visibleFlag } }
    $ids += $r.id
}
$listed = Invoke-Cmd @{ type = 'List'; payload = @{} }
Assert 'List returned 3 terminals' (($listed.content | ConvertFrom-Json).Count -eq 3) $listed.duration_ms

# Disconnect and reconnect — the orchestrator should have reaped everything ours owned.
$writer.Dispose(); $reader.Dispose(); $pipe.Dispose()
Start-Sleep -Milliseconds 500

$pipe2 = [System.IO.Pipes.NamedPipeClientStream]::new('.', $pipeName, [System.IO.Pipes.PipeDirection]::InOut)
$pipe2.Connect(2000)
$writer = [System.IO.StreamWriter]::new($pipe2);   $writer.AutoFlush = $true
$reader = [System.IO.StreamReader]::new($pipe2)
$nextReqId = 0

$listed2 = Invoke-Cmd @{ type = 'List'; payload = @{} }
Assert 'After disconnect, new connection sees 0 terminals' (($listed2.content | ConvertFrom-Json).Count -eq 0) $listed2.duration_ms

$writer.Dispose(); $reader.Dispose(); $pipe2.Dispose()

# ─── Report ───────────────────────────────────────────────────────────────────

$reportPath = Join-Path $PSScriptRoot 'test_results.md'
@"
# Smoke results — $(Get-Date -Format o)
Mode: $(if ($Headless) {'headless'} else {'visible'})

| # | Test | Status | Duration (ms) |
| - | ---- | ------ | ------------- |
$( ($results | ForEach-Object -Begin {$i=0} -Process { $i++; "| $i | $($_.Name) | $($_.Status) | $($_.Duration) |" }) -join "`n" )
"@ | Out-File -FilePath $reportPath -Encoding utf8

$pass = ($results | Where-Object Status -eq 'PASS').Count
$fail = ($results | Where-Object Status -eq 'FAIL').Count
$skip = ($results | Where-Object Status -eq 'SKIP').Count
Write-Host "`nReport: $reportPath"
Write-Host ("Total: {0} | Pass: {1} | Fail: {2} | Skip: {3}" -f ($pass+$fail+$skip), $pass, $fail, $skip) `
    -ForegroundColor $(if ($fail -eq 0) {'Green'} else {'Red'})

if (-not $KeepLogs) {
    Get-ChildItem -Path (Split-Path $PSScriptRoot -Parent) -Filter 'vterm-rs_*.log' -ErrorAction SilentlyContinue |
        Remove-Item -Force -ErrorAction SilentlyContinue
}

if ($fail -gt 0) { exit 1 }

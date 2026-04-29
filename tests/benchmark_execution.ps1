$ErrorActionPreference = "Stop"

# Configuration
$COMMAND = "ping -l 600 -n 1 www.google.com"
$VTERM_CTRL = ".\target\debug\vterm-ctrl.exe"

Write-Host "--- VTerm Performance Benchmark (Direct vs. Safe) ---" -ForegroundColor Cyan
Write-Host "Target: $COMMAND`n"

# 1. Direct Execution
Write-Host "[1/2] Measuring Direct Execution (Baseline)..."
$direct = Measure-Command {
    powershell.exe -Command "$COMMAND" | Out-Null
}
$direct_ms = $direct.TotalMilliseconds

# 2. VTerm Orchestration
Write-Host "[2/2] Measuring VTerm Safe Run..."
# Ensure orchestrator is running
$vterm_proc = Get-Process vterm -ErrorAction SilentlyContinue
if (!$vterm_proc) {
    Write-Host "Starting VTerm Orchestrator (Headless)..."
    $vterm_proc = Start-Process ".\target\debug\vterm.exe" -ArgumentList "--headless" -NoNewWindow -PassThru
    sleep 5 # Wait for warm pool
}

# Run once to ensure CLI and Pipe are warm
& $VTERM_CTRL list | Out-Null

$vterm_output = ""
$vterm = Measure-Command {
    $vterm_output = & $VTERM_CTRL run "$COMMAND; exit"
}
$vterm_ms = $vterm.TotalMilliseconds

# Extract timings from output
$spawn_line = $vterm_output | Select-String "Timings: spawn=(\d+)ms, ready=(\d+)ms"
if ($spawn_line) {
    $spawn_ms = $spawn_line.Matches.Groups[1].Value
    $ready_ms = $spawn_line.Matches.Groups[2].Value
}

# Cleanup
if ($vterm_proc) { Stop-Process $vterm_proc.Id -Force }

# Results
Write-Host "`n--- Results ---" -ForegroundColor Cyan
$results = @(
    [PSCustomObject]@{ Method = "Direct Shell"; Time_ms = "{0:N2}" -f $direct_ms; Overhead = "0ms (Baseline)" }
    [PSCustomObject]@{ Method = "VTerm (Safe)"; Time_ms = "{0:N2}" -f $vterm_ms;  Overhead = "+{0:N2}ms" -f ($vterm_ms - $direct_ms) }
)

$results | Format-Table -AutoSize

Write-Host "VTerm Timing Breakdown:" -ForegroundColor Yellow
Write-Host "- Orchestrator Handover: ${spawn_ms}ms"
Write-Host "- Shell Ready Wait:     ${ready_ms}ms"
Write-Host "- Total Safe Overhead:   $(($vterm_ms - $direct_ms).ToString('N2'))ms"

Write-Host "`nAnalysis:" -ForegroundColor Yellow
if (($vterm_ms - $direct_ms) -lt 300) {
    Write-Host "SUCCESS: Overhead is within acceptable range for secure agentic execution."
} else {
    Write-Host "NOTE: High overhead detected. Check system load or shell startup time."
}

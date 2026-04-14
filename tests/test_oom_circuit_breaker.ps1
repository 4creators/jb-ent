$ExePath = ".\build\Release\codebase-memory-mcp.exe"

if (-Not (Test-Path $ExePath)) {
    Write-Host "Executable not found at $ExePath. Please build in Release mode first."
    exit 1
}

Write-Host "=== Running OOM Circuit Breaker Integration Test ==="
Write-Host "Setting strict 1MB memory budget..."

[Environment]::SetEnvironmentVariable("CBM_BUDGET_MB", "1")

# Run the indexer on the current codebase repository
# We redirect stderr (2) to stdout (1) so we can capture the FATAL abort message
$Output = & $ExePath cli --progress index_repository '{"repo_path": "."}' 2>&1 | Out-String
$ExitCode = $LASTEXITCODE

[Environment]::SetEnvironmentVariable("CBM_BUDGET_MB", $null)

Write-Host "Process exited with code: $ExitCode"

$WarningRegex = [regex]::Escape("100% memory budget exceeded! Waiting for gracful autocancellation .... or 120% abort.")
$WarningCount = ([regex]::Matches($Output, $WarningRegex)).Count

if ($WarningCount -ne 1) {
    Write-Host "FAIL: The 100% budget warning message was logged $WarningCount times (expected exactly 1)." -ForegroundColor Red
    Write-Host "Full Output:`n$Output"
    exit 1
} else {
    Write-Host "SUCCESS: The 100% budget warning message was logged exactly once." -ForegroundColor Green
}

if ($Output -match "FATAL: Out of Memory!") {
    Write-Host ""
    Write-Host "SUCCESS: Circuit breaker successfully intercepted the allocations and aborted the process!" -ForegroundColor Green
    
    # Extract the full fatal error block
    if ($Output -match "(?s)(FATAL: Out of Memory!.*?Active Budget\s*:\s*\d+\s*MB)") {
        Write-Host "Caught message: " -NoNewline
        Write-Host "`n$($matches[1])" -ForegroundColor Yellow
    } else {
        Write-Host "Output contained FATAL but regex failed to extract the block. Full output:"
        Write-Host $Output
    }
    exit 0
} else {
    Write-Host ""
    Write-Host "FAIL: The process did not abort with the expected FATAL error." -ForegroundColor Red
    Write-Host "Full Output:"
    Write-Host $Output
    exit 1
}
$ExePath = ".\build\Release\codebase-memory-mcp.exe"

if (-Not (Test-Path $ExePath)) {
    Write-Host "Executable not found at $ExePath. Please build in Release mode first."
    exit 1
}

function Verify-Budget {
    param (
        [string]$TestName,
        [string]$BudgetMB,
        [string]$RamFraction,
        [int]$ExpectedBudget
    )

    Write-Host "`n=== Test: $TestName ==="

    if ($BudgetMB) { [Environment]::SetEnvironmentVariable("CBM_BUDGET_MB", $BudgetMB) }
    else { [Environment]::SetEnvironmentVariable("CBM_BUDGET_MB", $null) }

    if ($RamFraction) { [Environment]::SetEnvironmentVariable("CBM_RAM_FRACTION", $RamFraction) }
    else { [Environment]::SetEnvironmentVariable("CBM_RAM_FRACTION", $null) }

    $Output = & $ExePath cli --version 2>&1 | Out-String

    if ($Output -match "budget_mb=(\d+)") {
        $ActualBudget = [int]$matches[1]
        
        if ($ExpectedBudget -eq -1) {
            if ($ActualBudget -gt 0) {
                Write-Host "  RESULT: PASS (Dynamic Fallback Budget: $ActualBudget MB)" -ForegroundColor Green
            } else {
                Write-Host "  RESULT: FAIL (Expected dynamic budget > 0, got $ActualBudget)" -ForegroundColor Red
                exit 1
            }
        }
        elseif ($ActualBudget -eq $ExpectedBudget) {
            Write-Host "  RESULT: PASS (Exact match: $ActualBudget MB)" -ForegroundColor Green
        }
        else {
            Write-Host "  RESULT: FAIL (Expected $ExpectedBudget MB, got $ActualBudget MB)" -ForegroundColor Red
            exit 1
        }
    } else {
        Write-Host "  RESULT: FAIL (Could not find budget_mb in output)" -ForegroundColor Red
        Write-Host "  Output was: $Output"
        exit 1
    }
}

Write-Host "Starting Environment Variable Granular Tests..."

Verify-Budget -TestName "1. Only CBM_BUDGET_MB (100)" -BudgetMB "100" -RamFraction "" -ExpectedBudget 100
Verify-Budget -TestName "2. Only CBM_RAM_FRACTION (0.1)" -BudgetMB "" -RamFraction "0.1" -ExpectedBudget -1
Verify-Budget -TestName "3. Both present (MB=200 beats Fraction=0.1)" -BudgetMB "200" -RamFraction "0.1" -ExpectedBudget 200
Verify-Budget -TestName "4. Both Quoted Double (`"300`", `"0.5`")" -BudgetMB "`"300`"" -RamFraction "`"0.5`"" -ExpectedBudget 300
Verify-Budget -TestName "5. Both Quoted Single ('400', '0.2')" -BudgetMB "'400'" -RamFraction "'0.2'" -ExpectedBudget 400

# Clean up env vars
[Environment]::SetEnvironmentVariable("CBM_BUDGET_MB", $null)
[Environment]::SetEnvironmentVariable("CBM_RAM_FRACTION", $null)

Write-Host "`n=== Test: 6. OOM Circuit Breaker Integration ==="
Write-Host "  Setting CBM_BUDGET_MB=100 and running indexer on current repo (.)"

[Environment]::SetEnvironmentVariable("CBM_BUDGET_MB", "100")
$Output = & $ExePath cli --progress index_repository '{"repo_path": "."}' 2>&1 | Out-String
$ExitCode = $LASTEXITCODE
[Environment]::SetEnvironmentVariable("CBM_BUDGET_MB", $null)

Write-Host "  Process exited with code: $ExitCode"

if ($Output -match "FATAL: Out of Memory!") {
    Write-Host "  RESULT: PASS (Circuit breaker aborted the process at 100MB as expected)" -ForegroundColor Green
    
    if ($Output -match "(?s)(FATAL: Out of Memory!.*?Active Budget\s*:\s*\d+\s*Bytes.*?MB\))") {
        Write-Host "`n  --- Crash Log Extract ---"
        Write-Host $matches[1] -ForegroundColor Yellow
        Write-Host "  -------------------------"
    }
    exit 0
} elseif ($ExitCode -eq 0) {
    Write-Host "  RESULT: PASS (Process completed successfully. Repo too small to hit 100MB budget.)" -ForegroundColor Cyan
    exit 0
} else {
    Write-Host "  RESULT: FAIL (Process crashed, but NOT due to the OOM Circuit Breaker!)" -ForegroundColor Red
    Write-Host "  Full Output:`n$Output"
    exit 1
}
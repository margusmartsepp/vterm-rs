$ErrorActionPreference = "Stop"

Write-Host "--- VTerm MCP Verification & Documentation ---" -ForegroundColor Cyan

# 1. Build release binaries
Write-Host "Building release binaries..."
cargo build --release
if ($LASTEXITCODE -ne 0) { throw "Build failed" }

# 2. Run doc generator
Write-Host "Running documentation generator..."
python scripts/generate_mcp_docs.py
if ($LASTEXITCODE -ne 0) { throw "Documentation generation/verification failed" }

Write-Host "--- SUCCESS: Documentation updated and verified ---" -ForegroundColor Green

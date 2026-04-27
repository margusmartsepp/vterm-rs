# scripts/generate_graph.ps1
# Generates a Graphify architectural map of the vterm-rs codebase.

$ErrorActionPreference = "Stop"

Write-Host "--- Generating Codebase Graph ---" -ForegroundColor Cyan

# Ensure graphifyy is installed via uv
if (!(uv tool list | Select-String "graphifyy")) {
    Write-Host "Installing graphifyy..."
    uv tool install graphifyy
}

# Run the update (AST pass)
Write-Host "Running Graphify AST pass..."
graphify update .

Write-Host "--- Graph Generation Complete ---" -ForegroundColor Green
Write-Host "Report: graphify-out/GRAPH_REPORT.md"
Write-Host "Visual: graphify-out/graph.html"

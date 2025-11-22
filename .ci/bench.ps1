# bench.ps1
$PKG = "fluxion-stream"
$GROUP = "stream_benches"
$BASELINE = "benches\baseline\$GROUP"
$LATEST = "target\criterion\$GROUP"

Write-Host "Running benchmarks..." -ForegroundColor Cyan
cargo bench --package $PKG

# First run → create baseline
if (-not (Test-Path $BASELINE)) {
    Write-Host "First run → creating baseline" -ForegroundColor Green
    New-Item -ItemType Directory -Force -Path benches\baseline | Out-Null
    Copy-Item -Recurse -Force $LATEST benches\baseline\
    Write-Host "Baseline created. Run again to see comparison." -ForegroundColor Yellow
    exit 0
}

# Install critcmp if missing
if (-not (Get-Command critcmp -ErrorAction SilentlyContinue)) {
    Write-Host "Installing critcmp..." -ForegroundColor Cyan
    cargo install critcmp --locked
}

Write-Host "`nComparison (5% threshold):" -ForegroundColor Cyan
critcmp $BASELINE $LATEST --threshold 5

# Optional: auto-update README.md
if (Get-Command critcmp-markdown -ErrorAction SilentlyContinue) {
    critcmp $BASELINE $LATEST --export markdown > benches\latest-comparison.md

    $start = "<!-- BENCH:START -->"
    $end   = "<!-- BENCH:END -->"
    (Get-Content README.md) -replace "(?s)$start.*?($end)", "$start`n$(Get-Content benches\latest-comparison.md -Raw)`n$end" | Set-Content README.md
    Write-Host "README.md updated with latest numbers" -ForegroundColor Green
}

# Open the beautiful HTML report automatically
Start-Process "target\criterion\$GROUP\report\index.html"
Write-Host "`nDone! Report opened in your browser." -ForegroundColor Magenta

$repo = "C:\Projects\fluxion"
Set-Location $repo
$files = Get-ChildItem -Path $repo -Recurse -Filter *.rs -File -ErrorAction SilentlyContinue | Where-Object { $_.FullName -notmatch '\\target\\' -and $_.FullName -notmatch '\\.git\\' }
if (-not $files -or $files.Count -eq 0) {
    Write-Output "No .rs files found"
    exit 0
}
$total = 0
$prod = 0
$detail = @()
foreach ($f in $files) {
    $lines = (Get-Content -LiteralPath $f.FullName -ErrorAction SilentlyContinue | Measure-Object -Line).Lines
    $total += $lines
    if ($f.FullName -match '\\src\\') { $kind='productive'; $prod += $lines } else { $kind='other' }
    $detail += [pscustomobject]@{Path=$f.FullName; Lines=$lines; Kind=$kind}
}
$other = $total - $prod
$ratio = if ($total -gt 0) { [math]::Round(($prod / $total) * 100, 2) } else { 0 }
Write-Output "Total .rs files: $($files.Count)"
Write-Output "Total Rust lines: $total"
Write-Output "Productive (in src/): $prod"
Write-Output "Other Rust lines: $other"
Write-Output "Productive ratio: $ratio%"

Write-Output "`nTop 5 productive files by lines:"
$detail | Where-Object { $_.Kind -eq 'productive' } | Sort-Object Lines -Descending | Select-Object -First 5 | ForEach-Object { Write-Output ("$($_.Lines) lines - $($_.Path)") }

Write-Output "`nTop 5 other files by lines:"
$detail | Where-Object { $_.Kind -eq 'other' } | Sort-Object Lines -Descending | Select-Object -First 5 | ForEach-Object { Write-Output ("$($_.Lines) lines - $($_.Path)") }

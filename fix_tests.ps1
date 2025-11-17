$file = "c:\Projects\fluxion\fluxion-stream\tests\take_while_with_tests.rs"
# Restore original file first
git checkout $file

$lines = Get-Content $file
$output = @()

for ($i = 0; $i -lt $lines.Count; $i++) {
    $currentLine = $lines[$i]
    
    # Check if current line is .unwrap(); and next line starts with assert_eq!(item,
    if ($currentLine -match '^\s+\.unwrap\(\);$' -and 
        $i -lt $lines.Count - 1 -and 
        $lines[$i+1] -match '^\s+assert_eq!\(item,') {
        # Replace .unwrap(); with .unwrap().get().clone();
        $output += $currentLine -replace '\.unwrap\(\);$', '.unwrap().get().clone();'
    } else {
        $output += $currentLine
    }
}

$output | Set-Content $file
Write-Host "Fixed $file"

# PowerShell script to run the WASM dashboard
# Usage: .\run-dashboard.ps1 [-Port 8080]

param(
    [int]$Port = 8080
)

# Get the dashboard directory (parent of scripts folder)
$DashboardDir = Join-Path $PSScriptRoot ".."

# Check if we're in the right directory
if (-not (Test-Path (Join-Path $DashboardDir "Cargo.toml"))) {
    Write-Error "Could not find Cargo.toml in $DashboardDir"
    exit 1
}

Write-Host "Dashboard directory: $DashboardDir" -ForegroundColor Cyan
Write-Host "Server port: $Port" -ForegroundColor Cyan
Write-Host ""

# Function to stop process on port
function Stop-ProcessOnPort {
    param([int]$PortNum)

    $connections = Get-NetTCPConnection -LocalPort $PortNum -ErrorAction SilentlyContinue
    if ($connections) {
        Write-Host "Port $PortNum is in use. Stopping processes..." -ForegroundColor Yellow
        foreach ($conn in $connections) {
            # Skip system processes (PID 0, 4, etc.)
            if ($conn.OwningProcess -le 4) {
                continue
            }

            $process = Get-Process -Id $conn.OwningProcess -ErrorAction SilentlyContinue
            if ($process) {
                Write-Host "  Stopping process: $($process.Name) (PID: $($process.Id))" -ForegroundColor Yellow
                Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
                Start-Sleep -Milliseconds 500
            }
        }
    }
}

# Check and clear the port if needed
Stop-ProcessOnPort -PortNum $Port

# Build the WASM application with trunk
Write-Host "Building WASM dashboard with trunk..." -ForegroundColor Green

# Change to dashboard directory for trunk build
Push-Location $DashboardDir
try {
    # Build with trunk (index.html is in root)
    # Suppress error records from stderr while checking exit code
    $buildResult = & { trunk build --release 2>&1 } -ErrorAction SilentlyContinue
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Trunk build failed!"
        Write-Host $buildResult
        Pop-Location
        exit 1
    }
    Write-Host "Build successful!" -ForegroundColor Green
} finally {
    Pop-Location
}

# Start Python HTTP server in the dist directory
$DistDir = Join-Path $DashboardDir "dist"
if (-not (Test-Path $DistDir)) {
    Write-Error "Build output directory not found: $DistDir"
    exit 1
}

Write-Host ""
Write-Host "Starting HTTP server on port $Port..." -ForegroundColor Green
Write-Host "Server directory: $DistDir" -ForegroundColor Cyan

# Start the server as a background job
# Refresh PATH to include Python and set working directory
Write-Host "Server will serve files from: $DistDir" -ForegroundColor Cyan
$ServerJob = Start-Job -ScriptBlock {
    param($Dir, $P)
    # Use full Python path to avoid PATH issues
    $pythonExe = "C:\Users\$env:USERNAME\AppData\Local\Programs\Python\Python312\python.exe"
    if (-not (Test-Path $pythonExe)) {
        # Try alternative locations
        $pythonExe = Get-ChildItem -Path "$env:LOCALAPPDATA\Programs\Python" -Recurse -Filter python.exe -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty FullName
    }
    # Change to dist directory
    Set-Location $Dir
    # Verify we're in the right place
    Write-Host "Server starting from: $(Get-Location)"
    Write-Host "Files available:"
    Get-ChildItem | ForEach-Object { Write-Host "  - $($_.Name)" }
    # Start HTTP server
    & $pythonExe -m http.server $P
} -ArgumentList $DistDir, $Port

# Wait a moment for server to start
Start-Sleep -Seconds 2

# Check if server is running
$serverProcess = Get-NetTCPConnection -LocalPort $Port -ErrorAction SilentlyContinue
if (-not $serverProcess) {
    Write-Error "Failed to start HTTP server!"
    Stop-Job $ServerJob
    Remove-Job $ServerJob
    exit 1
}

Write-Host "Server started successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "Dashboard available at: http://localhost:$Port" -ForegroundColor Cyan
Write-Host ""

# Open browser
Write-Host "Opening browser..." -ForegroundColor Green
Start-Process "http://localhost:$Port"

Write-Host ""
Write-Host "Press Ctrl+C to stop the server and exit..." -ForegroundColor Yellow
Write-Host ""

# Monitor the job and keep script running
try {
    while ($true) {
        $jobState = Get-Job -Id $ServerJob.Id | Select-Object -ExpandProperty State
        if ($jobState -ne 'Running') {
            Write-Host "Server job stopped unexpectedly!" -ForegroundColor Red
            break
        }

        # Show any output from the server
        $output = Receive-Job -Id $ServerJob.Id -ErrorAction SilentlyContinue
        if ($output) {
            Write-Host $output
        }

        Start-Sleep -Seconds 1
    }
} finally {
    # Cleanup
    Write-Host ""
    Write-Host "Stopping server..." -ForegroundColor Yellow
    Stop-Job $ServerJob -ErrorAction SilentlyContinue
    Remove-Job $ServerJob -ErrorAction SilentlyContinue

    # Make sure the port is freed
    Stop-ProcessOnPort -PortNum $Port

    Write-Host "Server stopped." -ForegroundColor Green
}

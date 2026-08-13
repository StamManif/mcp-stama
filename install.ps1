# mcp-stama One-Liner PowerShell Installer for Windows
$ErrorActionPreference = "Stop"

$Repo = "StamManif/mcp-stama"
$InstallDir = Join-Path $env:USERPROFILE ".mcp-stama\bin"
$ZipName = "mcp-stama-x86_64-pc-windows-msvc.zip"
$DownloadUrl = "https://github.com/$Repo/releases/latest/download/$ZipName"

Write-Host "⚡ Installing mcp-stama..." -ForegroundColor Cyan

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$TempZip = Join-Path $env:TEMP $ZipName

Write-Host "Downloading release binary from $DownloadUrl..." -ForegroundColor Gray
Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempZip

Write-Host "Extracting executable..." -ForegroundColor Gray
Expand-Archive -Path $TempZip -DestinationPath $InstallDir -Force
Remove-Item -Path $TempZip -Force -ErrorAction SilentlyContinue

$ExePath = Join-Path $InstallDir "mcp-stama.exe"

# Update User PATH environment variable if needed
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$UserPath;$InstallDir", "User")
    Write-Host "Added $InstallDir to User PATH." -ForegroundColor Green
}

Write-Host "✅ mcp-stama installed to $ExePath" -ForegroundColor Green

# Run automatic MCP client configurator
try {
    & $ExePath --install-cursor --install-claude
    Write-Host "🚀 Cursor & Claude Desktop auto-configured!" -ForegroundColor Cyan
} catch {
    Write-Host "Could not auto-configure clients automatically. You can run 'mcp-stama --install-cursor' manually." -ForegroundColor Yellow
}

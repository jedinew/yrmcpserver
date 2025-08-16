Param(
  [string]$LogFile = "$env:TEMP\yr-weather-mcp.log"
)

"$([DateTime]::Now): YR Weather MCP Server Starting" | Out-File -FilePath $LogFile -Append -Encoding utf8
"PWD: $(Get-Location)" | Out-File -FilePath $LogFile -Append -Encoding utf8

if (-not $env:RUST_LOG) { $env:RUST_LOG = "yr_weather_mcp=info" }

$bin = Join-Path -Path (Resolve-Path ".") -ChildPath "target\release\yr-weather-mcp.exe"
if (-not (Test-Path $bin)) {
  "Building release binary..." | Out-File -FilePath $LogFile -Append -Encoding utf8
  cargo build --release
}

& $bin 2>> $LogFile

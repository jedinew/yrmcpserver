# YR Weather MCP Server

Rust-based MCP (Model Context Protocol) server that uses the YR.no (Norwegian Meteorological Institute) API to provide weather forecasts to LLMs.

## Features

- Current, tomorrow, and 7-day forecasts via the YR.no API
- Cross-platform run scripts (macOS/Linux and Windows)
- Portable MCP configuration for easy integration

## Installation

1) Install Rust (if you don’t have it):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2) Build the project:
```bash
cargo build --release
```

## Install from GitHub Releases (no build required)

1) Download the binary for your OS
- Go to the Releases page of this repository and pick the latest version.
- Download the matching file:
  - macOS: `yr-weather-mcp-macOS`
  - Linux: `yr-weather-mcp-Linux`
  - Windows: `yr-weather-mcp-Windows.exe`

2) Install to a folder on your PATH
- macOS:
  ```bash
  chmod +x ~/Downloads/yr-weather-mcp-macOS
  sudo mv ~/Downloads/yr-weather-mcp-macOS /usr/local/bin/yr-weather-mcp
  # If Gatekeeper blocks the binary, allow it:
  sudo xattr -d com.apple.quarantine /usr/local/bin/yr-weather-mcp || true
  ```
- Linux:
  ```bash
  chmod +x ~/Downloads/yr-weather-mcp-Linux
  mkdir -p ~/.local/bin
  mv ~/Downloads/yr-weather-mcp-Linux ~/.local/bin/yr-weather-mcp
  echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc  # or your shell rc
  ```
- Windows (PowerShell):
  ```powershell
  # Option A: Put it in a folder on PATH
  Move-Item "$env:USERPROFILE\Downloads\yr-weather-mcp-Windows.exe" "$env:USERPROFILE\AppData\Local\Microsoft\WindowsApps\yr-weather-mcp.exe"

  # Option B: Keep it in a chosen folder and add that folder to PATH,
  # or use the absolute path in your MCP config.
  ```

3) Verify it runs
```bash
# The server speaks JSON-RPC via stdin/stdout.
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | yr-weather-mcp
```

4) Configure your MCP client
- If the binary is on PATH (recommended):
  ```json
  {
    "mcpServers": {
      "yr-weather": {
        "command": "yr-weather-mcp",
        "env": { "RUST_LOG": "yr_weather_mcp=info" }
      }
    }
  }
  ```
- If not on PATH, use the absolute path:
  - macOS: `/usr/local/bin/yr-weather-mcp`
  - Linux: `/home/you/.local/bin/yr-weather-mcp`
  - Windows: `C:\\path\\to\\yr-weather-mcp.exe`

Tip: adjust logging with `RUST_LOG`, e.g. `RUST_LOG=yr_weather_mcp=debug`.

## Usage

### Run the MCP server

```bash
# macOS / Linux
./run_server.sh

# Windows (PowerShell)
powershell -ExecutionPolicy Bypass -File .\run_server.ps1

# Or run the binary directly if it’s on PATH
yr-weather-mcp
```

### Claude Desktop configuration

Add this to your `claude_desktop_config.json` (cross-platform):

```json
{
  "mcpServers": {
    "yr-weather": {
      "command": "yr-weather-mcp",
      "env": {
        "RUST_LOG": "yr_weather_mcp=info"
      }
    }
  }
}
```

### Example prompts

- "What’s the current weather in Seoul?"
- "How’s the weather in Tokyo tomorrow?"
- "Give me New York’s weekly forecast"

## API Information

This server uses YR.no’s free weather API:
- Docs: https://api.met.no/weatherapi/locationforecast/2.0/documentation
- Note: A valid User-Agent header is required by the API
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
#!/usr/bin/env bash
set -euo pipefail

# MCP Server Wrapper Script with Logging
LOG_FILE="${TMPDIR:-/tmp}/yr-weather-mcp.log"

echo "=== YR Weather MCP Server Starting ===" >> "$LOG_FILE"
echo "Time: $(date)" >> "$LOG_FILE"
echo "PWD: $(pwd)" >> "$LOG_FILE"
echo "PATH: $PATH" >> "$LOG_FILE"

# Set environment
export RUST_LOG="${RUST_LOG:-yr_weather_mcp=info}"

# Build if binary does not exist, then run
BIN="./target/release/yr-weather-mcp"
if [ ! -x "$BIN" ]; then
  echo "Building release binary..." >> "$LOG_FILE"
  cargo build --release
fi

exec "$BIN" 2>> "$LOG_FILE"
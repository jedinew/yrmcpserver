# LLM Integration Guide

## 1. Claude Desktop Integration

### macOS

1. Locate Claude Desktop configuration file:
```bash
~/Library/Application Support/Claude/claude_desktop_config.json
```

2. Edit the configuration file (cross-platform friendly):
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

Or to run in development mode (relative paths recommended):
```json
{
  "mcpServers": {
    "yr-weather": {
      "command": "cargo",
      "args": ["run"],
      "env": {
        "RUST_LOG": "yr_weather_mcp=debug"
      }
    }
  }
}
```

3. Restart Claude Desktop

4. Verify:
   - Ensure the MCP tools icon appears in Claude Desktop
   - Test with a prompt such as "What’s the weather in Seoul?"

### Windows

Configuration file location:
```
%APPDATA%\Claude\claude_desktop_config.json
```

## 2. VS Code + Continue Integration

1. Install the Continue extension

2. Configure `.continuerc.json`:
```json
{
  "models": [
    {
      "title": "Claude with YR Weather",
      "provider": "anthropic",
      "model": "claude-3-sonnet",
      "apiKey": "YOUR_API_KEY",
      "mcpServers": {
        "yr-weather": { "command": "yr-weather-mcp" }
      }
    }
  ]
}
```

## 3. Direct CLI Integration

### Claude CLI (claude-cli)

```bash
# Link Claude CLI with the MCP server
claude --mcp-server "yr-weather-mcp"
```

### Custom Integration

Python example:
```python
import subprocess
import json
import sys

class MCPClient:
    def __init__(self, command):
        self.process = subprocess.Popen(
            command,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            shell=True
        )
    
    def send_request(self, method, params=None):
        request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params or {}
        }
        
        self.process.stdin.write(json.dumps(request) + '\n')
        self.process.stdin.flush()
        
        response = self.process.stdout.readline()
        return json.loads(response)
    
    def get_weather(self, city):
        # Initialize
        self.send_request("initialize", {
            "protocolVersion": "0.1.0",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        })
        
        # Call weather tool
        result = self.send_request("tools/call", {
            "name": "get_weather",
            "arguments": {
                "city": city
            }
        })
        
        return result

# Example usage
client = MCPClient("yr-weather-mcp")
weather = client.get_weather("Seoul")
print(weather)
```

## 4. Run with Docker

### Dockerfile
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/yr-weather-mcp /usr/local/bin/
CMD ["yr-weather-mcp"]
```

### Docker Compose
```yaml
version: '3.8'
services:
  yr-weather-mcp:
    build: .
    ports:
      - "3000:3000"  # MCP default port (if applicable)
    environment:
      - RUST_LOG=yr_weather_mcp=info
```

## 5. Testing

### Manual test
```bash
# Start the server (auto-builds if missing)
./run_server.sh

# From another terminal, send a JSON-RPC request
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | yr-weather-mcp
```

### Automated test script
```bash
#!/bin/bash
# test_mcp.sh

# Start server
cargo run &
SERVER_PID=$!
sleep 2

# Send test request
cat << EOF | nc localhost 3000
{"jsonrpc":"2.0","id":1,"method":"tools/list"}
EOF

# Stop server
kill $SERVER_PID
```

## 6. Troubleshooting

### If the server does not start
1. Ensure Rust is installed: `rustc --version`
2. Build dependencies: `cargo build`
3. Check logs: `RUST_LOG=yr_weather_mcp=debug cargo run`

### If Claude Desktop does not detect the server
1. Verify configuration file path
2. Ensure the executable is accessible on PATH or specify a valid command
3. Fully quit and restart Claude Desktop
4. Inspect console logs via Claude Desktop developer tools

### If weather data cannot be fetched
1. Check internet connectivity
2. Verify YR API status
3. Confirm coordinates are correct or the city is supported

## 7. Additional Settings

### Proxy settings
```bash
export HTTP_PROXY=http://proxy.example.com:8080
export HTTPS_PROXY=http://proxy.example.com:8080
cargo run
```

### Custom city support
Add custom logic to map city names to coordinates in your client before calling `get_weather`.

## Support

- GitHub Issues: [repository]/issues
- YR API Docs: https://api.met.no/weatherapi/locationforecast/2.0/documentation
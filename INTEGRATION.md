# LLM 연동 가이드

## 1. Claude Desktop 연동

### macOS 설정

1. Claude Desktop 설정 파일 위치 확인:
```bash
~/Library/Application Support/Claude/claude_desktop_config.json
```

2. 설정 파일 편집 (크로스 플랫폼 친화적):
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

또는 개발 모드로 실행하려면 (상대 경로 사용 권장):
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

3. Claude Desktop 재시작

4. 사용 확인:
   - Claude Desktop에서 MCP 도구 아이콘이 나타나는지 확인
   - "서울 날씨 알려줘" 같은 명령어로 테스트

### Windows 설정

설정 파일 위치:
```
%APPDATA%\Claude\claude_desktop_config.json
```

## 2. VS Code + Continue 연동

1. Continue 확장 설치

2. `.continuerc.json` 설정:
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

## 3. CLI를 통한 직접 연동

### Claude CLI (claude-cli)

```bash
# Claude CLI와 MCP 서버 연동
claude --mcp-server "yr-weather-mcp"
```

### 커스텀 통합

Python 예제:
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

# 사용 예시
client = MCPClient("cargo run --manifest-path /Users/jedi/code/yrmcpserver/Cargo.toml")
weather = client.get_weather("Seoul")
print(weather)
```

## 4. Docker 컨테이너로 실행

### Dockerfile 생성
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

### Docker Compose 설정
```yaml
version: '3.8'
services:
  yr-weather-mcp:
    build: .
    ports:
      - "3000:3000"  # MCP 기본 포트
    environment:
      - RUST_LOG=yr_weather_mcp=info
```

## 5. 테스트 방법

### 직접 테스트
```bash
# 서버 실행 (빌드 없으면 자동 빌드)
./run_server.sh

# 다른 터미널에서 테스트 (예: echo로 JSON-RPC 요청 보내기)
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | yr-weather-mcp
```

### 자동화 테스트 스크립트
```bash
#!/bin/bash
# test_mcp.sh

# 서버 시작
cargo run &
SERVER_PID=$!
sleep 2

# 테스트 요청 보내기
cat << EOF | nc localhost 3000
{"jsonrpc":"2.0","id":1,"method":"tools/list"}
EOF

# 서버 종료
kill $SERVER_PID
```

## 6. 문제 해결

### 서버가 시작되지 않을 때
1. Rust가 설치되어 있는지 확인: `rustc --version`
2. 의존성 설치: `cargo build`
3. 로그 레벨 확인: `RUST_LOG=debug cargo run`

### Claude Desktop에서 인식하지 못할 때
1. 설정 파일 경로 확인
2. 실행 파일 경로가 절대 경로인지 확인
3. Claude Desktop 완전히 종료 후 재시작
4. 콘솔 로그 확인: Claude Desktop 개발자 도구 열기

### 날씨 정보를 가져오지 못할 때
1. 인터넷 연결 확인
2. YR API 상태 확인
3. 도시 이름이 지원 목록에 있는지 확인

## 7. 추가 설정

### 프록시 설정
```bash
export HTTP_PROXY=http://proxy.example.com:8080
export HTTPS_PROXY=http://proxy.example.com:8080
cargo run
```

### 커스텀 도시 추가
`src/weather.rs`의 `CITY_COORDINATES`에 새 도시 추가:
```rust
m.insert("jeju".to_string(), Coordinates { lat: 33.4996, lon: 126.5312 });
```

## 지원 및 문의

- GitHub Issues: [프로젝트 저장소]/issues
- YR API 문서: https://api.met.no/weatherapi/locationforecast/2.0/documentation
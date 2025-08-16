# YR Weather MCP Server

YR.no API를 사용하는 Rust 기반 MCP (Model Context Protocol) 서버입니다.

## 기능

- YR.no (Norwegian Meteorological Institute) API를 통한 날씨 정보 제공
- 한국어 도시명 지원 (서울, 부산, 인천 등)
- 주요 세계 도시 지원

## 설치

1. Rust 설치 (설치되어 있지 않은 경우):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. 프로젝트 빌드:
```bash
cargo build --release
```

## 사용법

### MCP 서버 실행

```bash
# macOS / Linux
./run_server.sh

# Windows (PowerShell)
powershell -ExecutionPolicy Bypass -File .\run_server.ps1

# 또는 PATH에 추가되어 있다면 직접 실행
yr-weather-mcp
```

### Claude Desktop 설정

`claude_desktop_config.json`에 다음 설정 추가 (크로스 플랫폼):

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

### 사용 예시

LLM에서 다음과 같이 질문:
- "YR 날씨 알려줘 나는 현재 서울에 있어"
- "도쿄 날씨 어때?"
- "뉴욕 날씨 정보 알려줘"

## 지원 도시

### 한국 도시
- 서울 (Seoul)
- 부산 (Busan)  
- 인천 (Incheon)
- 대구 (Daegu)
- 대전 (Daejeon)
- 광주 (Gwangju)

### 세계 주요 도시
- Tokyo, New York, London, Paris, Berlin
- Moscow, Beijing, Shanghai, Singapore
- Sydney, Los Angeles, San Francisco

## API 정보

이 서버는 YR.no의 무료 날씨 API를 사용합니다:
- API 문서: https://api.met.no/weatherapi/locationforecast/2.0/documentation
- 라이선스: 무료 (User-Agent 헤더 필요)
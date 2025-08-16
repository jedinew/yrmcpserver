use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use tracing::{debug, error, info};

mod weather;
use weather::WeatherClient;

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    protocol_version: String,
    capabilities: Value,
    #[serde(rename = "clientInfo")]
    client_info: ClientInfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClientInfo {
    name: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToolCall {
    name: String,
    arguments: Option<Value>,
}

#[tokio::main]
async fn main() -> Result<()> {
    use tracing_subscriber::EnvFilter;
    
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("yr_weather_mcp=debug".parse().unwrap()))
        .init();

    info!("YR Weather MCP Server starting...");
    
    let weather_client = WeatherClient::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stdin_lock = stdin.lock();
    
    loop {
        let mut line = String::new();
        match stdin_lock.read_line(&mut line) {
            Ok(0) => {
                // EOF reached
                info!("EOF reached, shutting down server");
                break;
            }
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                
                debug!("Received: {}", line);
                
                let request: JsonRpcRequest = match serde_json::from_str(line) {
                    Ok(req) => req,
                    Err(e) => {
                        error!("Failed to parse request: {}", e);
                        continue;
                    }
                };
                
                let response = handle_request(request, &weather_client).await;
                
                // Don't send response for notifications
                if response.id.is_some() || response.error.is_some() {
                    let response_str = serde_json::to_string(&response)?;
                    debug!("Sending: {}", response_str);
                    writeln!(stdout, "{}", response_str)?;
                    stdout.flush()?;
                } else {
                    debug!("Notification received, no response sent");
                }
            }
            Err(e) => {
                error!("Failed to read from stdin: {}", e);
                break;
            }
        }
    }
    
    info!("Server shutting down gracefully");
    Ok(())
}

async fn handle_request(request: JsonRpcRequest, weather_client: &WeatherClient) -> JsonRpcResponse {
    // Handle notifications (no response needed)
    if request.method.starts_with("notifications/") {
        // For notifications, return empty response that won't be sent
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: None,
            result: Some(json!(null)),
            error: None,
        };
    }
    
    match request.method.as_str() {
        "initialize" => {
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(json!({
                    "protocolVersion": "2025-06-18",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "yr-weather-mcp",
                        "version": "0.1.0"
                    }
                })),
                error: None,
            }
        }
        "initialized" => {
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(json!({})),
                error: None,
            }
        }
        "tools/list" => {
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(json!({
                    "tools": [
                        {
                            "name": "get_weather",
                            "description": "Get weather forecast for GPS coordinates using YR.no API",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "latitude": {
                                        "type": "number",
                                        "description": "Latitude coordinate (e.g., 37.5665 for Seoul)"
                                    },
                                    "longitude": {
                                        "type": "number",
                                        "description": "Longitude coordinate (e.g., 126.9780 for Seoul)"
                                    },
                                    "location_name": {
                                        "type": "string",
                                        "description": "Optional location name for display purposes"
                                    },
                                    "forecast_type": {
                                        "type": "string",
                                        "enum": ["current", "tomorrow", "weekly"],
                                        "description": "Type of forecast: 'current' for now, 'tomorrow' for next day, 'weekly' for 7-day forecast",
                                        "default": "current"
                                    }
                                },
                                "required": ["latitude", "longitude"]
                            }
                        }
                    ]
                })),
                error: None,
            }
        }
        "resources/list" => {
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(json!({
                    "resources": []
                })),
                error: None,
            }
        }
        "prompts/list" => {
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(json!({
                    "prompts": []
                })),
                error: None,
            }
        }
        "tools/call" => {
            let params = request.params.unwrap_or(json!({}));
            
            if let Some(name) = params.get("name").and_then(|n| n.as_str()) {
                if name == "get_weather" {
                    let default_args = json!({});
                    let arguments = params.get("arguments").unwrap_or(&default_args);
                    
                    let latitude = arguments.get("latitude")
                        .and_then(|l| l.as_f64())
                        .unwrap_or(37.5665); // Default to Seoul
                    let longitude = arguments.get("longitude")
                        .and_then(|l| l.as_f64())
                        .unwrap_or(126.9780);
                    let location_name = arguments.get("location_name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("Unknown Location");
                    let forecast_type = arguments.get("forecast_type")
                        .and_then(|f| f.as_str())
                        .unwrap_or("current");
                    
                    match weather_client.get_weather_by_coords(latitude, longitude, location_name, forecast_type).await {
                        Ok(weather_info) => {
                            JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id: request.id,
                                result: Some(json!({
                                    "content": [
                                        {
                                            "type": "text",
                                            "text": weather_info
                                        }
                                    ]
                                })),
                                error: None,
                            }
                        }
                        Err(e) => {
                            JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id: request.id,
                                result: None,
                                error: Some(JsonRpcError {
                                    code: -32603,
                                    message: format!("Failed to get weather: {}", e),
                                    data: None,
                                }),
                            }
                        }
                    }
                } else {
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32601,
                            message: format!("Unknown tool: {}", name),
                            data: None,
                        }),
                    }
                }
            } else {
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Invalid parameters".to_string(),
                        data: None,
                    }),
                }
            }
        }
        _ => {
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                    data: None,
                }),
            }
        }
    }
}
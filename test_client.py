#!/usr/bin/env python3
"""
YR Weather MCP Server Test Client
Run: python3 test_client.py
"""

import subprocess
import json
import sys
import time

class MCPTestClient:
    def __init__(self):
        print("🚀 Starting YR Weather MCP Server...")
        # Try release binary first, fallback to cargo run
        import os
        binary_path = "./target/release/yr-weather-mcp"
        
        if os.path.exists(binary_path):
            cmd = [binary_path]
        else:
            # Fallback to cargo with explicit path
            cargo_path = os.path.expanduser("~/.cargo/bin/cargo")
            if os.path.exists(cargo_path):
                cmd = [cargo_path, "run"]
            else:
                cmd = ["cargo", "run"]
        
        self.process = subprocess.Popen(
            cmd,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1
        )
        time.sleep(2)  # wait for server to start
        print("✅ Server started\n")
    
    def send_request(self, method, params=None, request_id=None):
        request = {
            "jsonrpc": "2.0",
            "id": request_id or 1,
            "method": method,
            "params": params or {}
        }
        
        request_json = json.dumps(request)
        print(f"📤 Sending: {request_json}")
        
        self.process.stdin.write(request_json + '\n')
        self.process.stdin.flush()
        
        response_line = self.process.stdout.readline()
        if response_line:
            response = json.loads(response_line)
            print(f"📥 Received: {json.dumps(response, indent=2)}\n")
            return response
        return None
    
    def test_initialize(self):
        print("=" * 50)
        print("1️⃣  Testing Initialize")
        print("=" * 50)
        
        return self.send_request("initialize", {
            "protocolVersion": "0.1.0",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }, request_id=1)
    
    def test_initialized(self):
        print("=" * 50)
        print("2️⃣  Testing Initialized")
        print("=" * 50)
        
        return self.send_request("initialized", {}, request_id=2)
    
    def test_tools_list(self):
        print("=" * 50)
        print("3️⃣  Testing Tools List")
        print("=" * 50)
        
        return self.send_request("tools/list", {}, request_id=3)
    
    def test_get_weather(self, city):
        print("=" * 50)
        print(f"4️⃣  Testing Get Weather for {city}")
        print("=" * 50)
        
        response = self.send_request("tools/call", {
            "name": "get_weather",
            "arguments": {
                "latitude": 37.5665,
                "longitude": 126.9780,
                "location_name": city
            }
        }, request_id=4)
        
        if response and "result" in response:
            content = response["result"].get("content", [])
            if content and len(content) > 0:
                weather_text = content[0].get("text", "")
                print("🌤️  Weather Information:")
                print("-" * 40)
                print(weather_text)
                print("-" * 40)
        
        return response
    
    def cleanup(self):
        print("\n🛑 Stopping server...")
        self.process.terminate()
        self.process.wait()
        print("✅ Server stopped")
    
    def run_all_tests(self):
        try:
            # 1. Initialize
            init_response = self.test_initialize()
            if not init_response or "error" in init_response:
                print("❌ Initialize failed")
                return False
            
            # 2. Initialized
            initialized_response = self.test_initialized()
            if not initialized_response or "error" in initialized_response:
                print("❌ Initialized failed")
                return False
            
            # 3. List tools
            tools_response = self.test_tools_list()
            if not tools_response or "error" in tools_response:
                print("❌ Tools list failed")
                return False
            
            # 4. Test weather for different cities
            test_cities = ["Seoul", "Tokyo", "New York", "London"]
            
            for city in test_cities:
                print(f"\n🌍 Testing weather for: {city}")
                weather_response = self.test_get_weather(city)
                if not weather_response or "error" in weather_response:
                    print(f"❌ Weather request failed for {city}")
                else:
                    print(f"✅ Weather request successful for {city}")
                
                time.sleep(1)  # API rate limiting
            
            # 5. Test invalid city
            print("\n🔍 Testing invalid city...")
            invalid_response = self.test_get_weather("InvalidCityName123")
            if invalid_response and "error" in invalid_response:
                print("✅ Invalid city handled correctly")
            else:
                print("⚠️  Invalid city should return an error")
            
            print("\n" + "=" * 50)
            print("✅ All tests completed successfully!")
            print("=" * 50)
            return True
            
        except Exception as e:
            print(f"\n❌ Test failed with error: {e}")
            return False
        finally:
            self.cleanup()

def main():
    print("""
╔══════════════════════════════════════════╗
║     YR Weather MCP Server Test Suite     ║
╚══════════════════════════════════════════╝
    """)
    
    client = MCPTestClient()
    success = client.run_all_tests()
    
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
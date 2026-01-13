#!/bin/bash

# Game Server Connection Test Script

echo "🧪 Testing Game Server Connection"
echo "=================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
GAME_SERVER_URL="${GAME_SERVER_URL:-ws://127.0.0.1:8080}"
TEST_TOKEN="${TEST_TOKEN:-}"

echo "📍 Server URL: $GAME_SERVER_URL"
echo ""

# Test 1: Check if server is reachable
echo "Test 1: Server Reachability"
echo "----------------------------"
if command -v nc &> /dev/null; then
    HOST=$(echo $GAME_SERVER_URL | sed 's/.*:\/\///' | cut -d':' -f1)
    PORT=$(echo $GAME_SERVER_URL | sed 's/.*://')
    
    if nc -z $HOST $PORT 2>/dev/null; then
        echo -e "${GREEN}✓${NC} Server is listening on $HOST:$PORT"
    else
        echo -e "${RED}✗${NC} Cannot connect to $HOST:$PORT"
        echo "Make sure the game server is running:"
        echo "  cd game-server && cargo run"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠${NC} 'nc' command not found, skipping port check"
fi
echo ""

# Test 2: WebSocket upgrade (basic)
echo "Test 2: WebSocket Upgrade"
echo "--------------------------"
HTTP_URL=$(echo $GAME_SERVER_URL | sed 's/ws:/http:/' | sed 's/wss:/https:/')

if command -v curl &> /dev/null; then
    RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" \
        -H "Connection: Upgrade" \
        -H "Upgrade: websocket" \
        -H "Sec-WebSocket-Version: 13" \
        -H "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==" \
        $HTTP_URL 2>&1)
    
    if [ "$RESPONSE" = "101" ] || [ "$RESPONSE" = "400" ]; then
        echo -e "${GREEN}✓${NC} WebSocket endpoint responding (HTTP $RESPONSE)"
    else
        echo -e "${RED}✗${NC} Unexpected response: HTTP $RESPONSE"
    fi
else
    echo -e "${YELLOW}⚠${NC} 'curl' not found, skipping HTTP test"
fi
echo ""

# Test 3: WebSocket connection with websocat
echo "Test 3: WebSocket Connection"
echo "-----------------------------"
if command -v websocat &> /dev/null; then
    echo "Using websocat to test connection..."
    echo "This will attempt to connect (will fail auth without token, which is expected)"
    
    timeout 2 websocat $GAME_SERVER_URL 2>&1 | head -n 5 &
    sleep 1
    echo -e "${GREEN}✓${NC} websocat available for manual testing"
    echo ""
    echo "Manual test command:"
    echo "  websocat $GAME_SERVER_URL"
else
    echo -e "${YELLOW}⚠${NC} 'websocat' not installed"
    echo ""
    echo "Install websocat for interactive testing:"
    echo "  cargo install websocat"
    echo "  brew install websocat  # macOS"
fi
echo ""

# Test 4: Generate test JWT token (requires Laravel)
echo "Test 4: JWT Token Generation"
echo "-----------------------------"
if [ -f "../artisan" ]; then
    echo "Generating test JWT token..."
    TEST_TOKEN=$(php ../artisan tinker --execute="
        \$service = app(App\Services\GameServerService::class);
        echo \$service->generateToken('test-user-123', 'TestUser');
    " 2>/dev/null)
    
    if [ ! -z "$TEST_TOKEN" ]; then
        echo -e "${GREEN}✓${NC} Generated test token"
        echo ""
        echo "Token: $TEST_TOKEN"
        echo ""
        echo "Test with websocat:"
        echo "  websocat $GAME_SERVER_URL"
        echo ""
        echo "Then send:"
        echo '  {"type":"Auth","token":"'$TEST_TOKEN'","game_id":"test-game-123","game_code":"TEST01","difficulty":"medium","text":"Test content","host_id":"test-host"}'
    else
        echo -e "${YELLOW}⚠${NC} Could not generate token (check Laravel setup)"
    fi
else
    echo -e "${YELLOW}⚠${NC} Laravel artisan not found in parent directory"
    echo "Run this script from game-server/ directory"
fi
echo ""

echo "=================================="
echo "📝 Summary"
echo "=================================="
echo ""
echo "Quick Test Commands:"
echo ""
echo "1. Test server is running:"
echo "   curl -I http://127.0.0.1:8080"
echo ""
echo "2. Interactive WebSocket connection:"
echo "   websocat ws://127.0.0.1:8080"
echo ""
echo "3. View server logs:"
echo "   # In development"
echo "   cargo run"
echo "   # In production (systemd)"
echo "   sudo journalctl -u qcxis-game-server -f"
echo ""
echo "4. Check active connections:"
echo "   netstat -an | grep 8080"
echo "   # or"
echo "   ss -tan | grep 8080"
echo ""

#!/bin/bash
# GoatCounter Analytics Checker for nofuzz.app
#
# Usage:
#   ./scripts/check-analytics.sh                    # View public stats page
#   ./scripts/check-analytics.sh --api <token>      # Fetch API stats (requires token)
#
# To get an API token:
#   1. Log in to https://jphollanti.goatcounter.com
#   2. Go to Settings > API
#   3. Create a new API token
#   4. Store it securely (e.g., in ~/.goatcounter_token)

GOATCOUNTER_SITE="jphollanti"
GOATCOUNTER_URL="https://${GOATCOUNTER_SITE}.goatcounter.com"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}  GoatCounter Analytics - nofuzz.app   ${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
}

print_section() {
    echo -e "${GREEN}► $1${NC}"
}

# Check if API token is provided
if [[ "$1" == "--api" && -n "$2" ]]; then
    API_TOKEN="$2"
elif [[ "$1" == "--api" && -f "$HOME/.goatcounter_token" ]]; then
    API_TOKEN=$(cat "$HOME/.goatcounter_token")
elif [[ -n "$GOATCOUNTER_API_TOKEN" ]]; then
    API_TOKEN="$GOATCOUNTER_API_TOKEN"
fi

print_header

if [[ -n "$API_TOKEN" ]]; then
    print_section "Fetching stats via API..."
    echo ""

    # Get total stats
    echo -e "${YELLOW}Total Page Views:${NC}"
    curl -s -H "Authorization: Bearer $API_TOKEN" \
        "${GOATCOUNTER_URL}/api/v0/stats/total" | jq '.' 2>/dev/null || \
        echo "  (Install jq for formatted output, or check token validity)"
    echo ""

    # Get top pages (last 30 days)
    echo -e "${YELLOW}Top Pages (last 30 days):${NC}"
    curl -s -H "Authorization: Bearer $API_TOKEN" \
        "${GOATCOUNTER_URL}/api/v0/stats/hits?period=30d&limit=10" | jq '.' 2>/dev/null || \
        echo "  (Install jq for formatted output)"
    echo ""

    # Get browser stats
    echo -e "${YELLOW}Browser Stats:${NC}"
    curl -s -H "Authorization: Bearer $API_TOKEN" \
        "${GOATCOUNTER_URL}/api/v0/stats/browsers?period=30d" | jq '.' 2>/dev/null || \
        echo "  (Install jq for formatted output)"
    echo ""

    # Get referrer stats
    echo -e "${YELLOW}Top Referrers:${NC}"
    curl -s -H "Authorization: Bearer $API_TOKEN" \
        "${GOATCOUNTER_URL}/api/v0/stats/refs?period=30d&limit=10" | jq '.' 2>/dev/null || \
        echo "  (Install jq for formatted output)"

else
    print_section "Quick Stats Access"
    echo ""
    echo "  Dashboard URL: ${GOATCOUNTER_URL}"
    echo ""
    echo -e "${YELLOW}Note:${NC} For API access, provide a token:"
    echo ""
    echo "  Option 1: Pass as argument"
    echo "    ./scripts/check-analytics.sh --api YOUR_TOKEN"
    echo ""
    echo "  Option 2: Save to file"
    echo "    echo 'YOUR_TOKEN' > ~/.goatcounter_token"
    echo "    ./scripts/check-analytics.sh --api"
    echo ""
    echo "  Option 3: Environment variable"
    echo "    export GOATCOUNTER_API_TOKEN='YOUR_TOKEN'"
    echo "    ./scripts/check-analytics.sh"
    echo ""
    echo -e "${YELLOW}To get an API token:${NC}"
    echo "  1. Log in to ${GOATCOUNTER_URL}"
    echo "  2. Go to Settings > API"
    echo "  3. Create a new API token"
    echo ""

    # Try to open the dashboard in the browser
    if command -v xdg-open &> /dev/null; then
        echo -e "${GREEN}Opening dashboard in browser...${NC}"
        xdg-open "$GOATCOUNTER_URL" 2>/dev/null &
    elif command -v open &> /dev/null; then
        echo -e "${GREEN}Opening dashboard in browser...${NC}"
        open "$GOATCOUNTER_URL" 2>/dev/null &
    fi
fi

echo ""
echo -e "${BLUE}========================================${NC}"

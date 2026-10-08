#!/usr/bin/env bash
# Glance Statistics Viewer
# Fetches live download and installation statistics for Glance across all releases and platforms.

set -euo pipefail

REPO="maycondouglascc/glance"
API_URL="https://api.github.com/repos/${REPO}/releases"
TELEMETRY_URL="https://www.maycondouglas.work/api/glance-telemetry"

BOLD='\033[1m'
BLUE='\033[0;34m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
DIM='\033[2m'
NC='\033[0m'

echo -e "${BOLD}${BLUE}======================================================${NC}"
echo -e "${BOLD}       Glance — Download & Installation Metrics       ${NC}"
echo -e "${BOLD}${BLUE}======================================================${NC}\n"

# 1. Fetch GitHub Releases Data
echo -e "${DIM}Fetching release assets from GitHub...${NC}"
if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
    RELEASES_JSON=$(gh api "repos/${REPO}/releases" 2>/dev/null || echo "[]")
else
    RELEASES_JSON=$(curl -sSL -H "Accept: application/vnd.github.v3+json" "$API_URL" 2>/dev/null || echo "[]")
fi

TOTAL_GITHUB_DOWNLOADS=0
DEB_DOWNLOADS=0
TAR_DOWNLOADS=0

echo -e "${BOLD}📦 GitHub Releases Download Stats:${NC}"
echo -e "------------------------------------------------------"

if command -v jq >/dev/null 2>&1; then
    while IFS= read -r release; do
        TAG=$(echo "$release" | jq -r '.tag_name // "unknown"')
        ASSETS_COUNT=$(echo "$release" | jq '.assets | length')
        if [ "$ASSETS_COUNT" -gt 0 ]; then
            echo -e "${CYAN}${BOLD}Release ${TAG}:${NC}"
            while IFS= read -r asset; do
                NAME=$(echo "$asset" | jq -r '.name')
                DOWNLOADS=$(echo "$asset" | jq -r '.download_count')
                
                # Exclude checksums from total count
                if [[ "$NAME" != *.sha256 ]]; then
                    TOTAL_GITHUB_DOWNLOADS=$((TOTAL_GITHUB_DOWNLOADS + DOWNLOADS))
                    if [[ "$NAME" == *.deb ]]; then
                        DEB_DOWNLOADS=$((DEB_DOWNLOADS + DOWNLOADS))
                    elif [[ "$NAME" == *.tar.gz ]]; then
                        TAR_DOWNLOADS=$((TAR_DOWNLOADS + DOWNLOADS))
                    fi
                fi
                printf "  • %-36s ${GREEN}%5d${NC} downloads\n" "$NAME" "$DOWNLOADS"
            done < <(echo "$release" | jq -c '.assets[]')
            echo ""
        fi
    done < <(echo "$RELEASES_JSON" | jq -c '.[]')
else
    echo "  (jq not found; install jq for per-asset breakdowns)"
fi

echo -e "${BOLD}📊 GitHub Summary:${NC}"
echo -e "  • Total Package Downloads:  ${GREEN}${BOLD}${TOTAL_GITHUB_DOWNLOADS}${NC}"
echo -e "    - Debian Packages (.deb): ${DEB_DOWNLOADS}"
echo -e "    - Tarballs (.tar.gz):     ${TAR_DOWNLOADS}"
echo ""

# 2. Fetch Terminal Script Telemetry
echo -e "${BOLD}⚡ Script Installations (install.sh):${NC}"
echo -e "------------------------------------------------------"
TELEMETRY_RESP=$(curl -sSL --max-time 3 "$TELEMETRY_URL" 2>/dev/null || echo "{}")

if command -v jq >/dev/null 2>&1 && [ "$TELEMETRY_RESP" != "{}" ]; then
    SCRIPT_TOTAL=$(echo "$TELEMETRY_RESP" | jq -r '.scriptInstalls.totalScriptInstalls // .totalScriptInstalls // 0' 2>/dev/null || echo 0)
    echo -e "  • Total Script Runs:        ${GREEN}${BOLD}${SCRIPT_TOTAL}${NC}"
    
    TOP_COUNTRIES=$(echo "$TELEMETRY_RESP" | jq -r '.scriptInstalls.byCountry // .byCountry // {} | to_entries | sort_by(-.value) | .[:5] | map("\(.key): \(.value)") | join(", ")' 2>/dev/null || echo "")
    if [ -n "$TOP_COUNTRIES" ] && [ "$TOP_COUNTRIES" != "null" ]; then
        echo -e "  • Top Locations:            ${TOP_COUNTRIES}"
    fi
else
    echo -e "  • Telemetry endpoint:       ${TELEMETRY_URL}"
fi
echo ""

# 3. Web Analytics Reference
echo -e "${BOLD}🌐 Landing Page Analytics (Visits & Real-Time):${NC}"
echo -e "------------------------------------------------------"
echo -e "  • URL:                      https://www.maycondouglas.work/glance"
echo -e "  • Google Analytics 4 (GA4): Property ID ${BOLD}G-010LYS365T${NC}"
echo -e "    - Page views, real-time visitors, referrers, and command copy events"
echo -e "  • Microsoft Clarity:        Project ID ${BOLD}vgvl92p4uo${NC}"
echo -e "    - Session recordings and click heatmaps"
echo -e "${BOLD}${BLUE}======================================================${NC}\n"

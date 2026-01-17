#!/usr/bin/env bash
set -euo pipefail

# Extract coverage percentage from reports
RUST_COV=$(cat coverage/rust/coverage.json | jq -r '.data[0].totals.lines.percent' | cut -d'.' -f1)
TS_COV=$(cat coverage/typescript/coverage-summary.json | jq -r '.total.lines.pct' | cut -d'.' -f1)

# Average coverage
TOTAL_COV=$(( (RUST_COV + TS_COV) / 2 ))

# Determine color
if [ "$TOTAL_COV" -ge 80 ]; then
    COLOR="brightgreen"
elif [ "$TOTAL_COV" -ge 60 ]; then
    COLOR="yellow"
else
    COLOR="red"
fi

# Generate badge URL
BADGE_URL="https://img.shields.io/badge/coverage-${TOTAL_COV}%25-${COLOR}"

echo "Coverage: ${TOTAL_COV}%"
echo "Badge URL: $BADGE_URL"

# Update README badge (if needed)
# sed -i "s|coverage-[0-9]*%25-[a-z]*|coverage-${TOTAL_COV}%25-${COLOR}|g" README.md
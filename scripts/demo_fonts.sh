#!/usr/bin/env bash
# Demo: Same gradient across different font families
# Usage: ./demo_fonts.sh [text] [width] [height]
set -euo pipefail

text="${1:-DEMO}"
width="${2:-60}"
height="${3:-10}"

families=(
    "banner"
    "cyber"
    "slant"
    "script"
    "shadow"
    "poison"
)

echo "=== Font Families with Rainbow Gradient ==="
echo ""

for family in "${families[@]}"; do
    echo "--- $family ---"
    cargo run --quiet --example gradient -- "$text" "$width" "$height" \
        --gradient horizontal \
        --from "255,100,200" --to "100,200,255" \
        --family "$family"
    echo ""
done

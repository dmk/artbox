#!/usr/bin/env bash
# Demo: Shows different gradient types side-by-side
set -euo pipefail

text="${1:-ARTBOX}"
width="${2:-50}"
height="${3:-8}"

echo "=== Gradient Types Demo ==="
echo ""

echo "--- Horizontal Gradient (Red -> Blue) ---"
cargo run --quiet --example gradient -- "$text" "$width" "$height" \
    --gradient horizontal --from "255,0,0" --to "0,0,255"
echo ""

echo "--- Vertical Gradient (Yellow -> Purple) ---"
cargo run --quiet --example gradient -- "$text" "$width" "$height" \
    --gradient vertical --from "255,255,0" --to "128,0,255"
echo ""

echo "--- Diagonal Gradient (Cyan -> Orange) ---"
cargo run --quiet --example gradient -- "$text" "$width" "$height" \
    --gradient diagonal --from "0,255,255" --to "255,128,0"
echo ""

echo "--- Radial Gradient (White -> Blue) ---"
cargo run --quiet --example gradient -- "$text" "$width" "$height" \
    --gradient radial --from "255,255,255" --to "80,120,255"
echo ""

echo "--- Solid Color (Bright Green) ---"
cargo run --quiet --example gradient -- "$text" "$width" "$height" \
    --color "0,255,128"
echo ""

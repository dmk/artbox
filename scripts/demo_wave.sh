#!/usr/bin/env bash
# Demo: Animated moving gradient wave
# Usage: ./demo_wave.sh [text] [width] [height] [delay]
set -euo pipefail

text="${1:-WAVE}"
width="${2:-50}"
height="${3:-10}"
delay="${4:-0.05}"

# Get terminal size
term_lines=$(tput lines 2>/dev/null || echo 24)
term_cols=$(tput cols 2>/dev/null || echo 80)

# Clamp height to terminal
max_height=$((term_lines - 2))
if [ "$max_height" -lt 1 ]; then max_height=1; fi
if [ "$height" -gt "$max_height" ]; then height="$max_height"; fi

# Calculate centering
pad_top=$(( (term_lines - height) / 2 ))
pad_left=$(( (term_cols - width) / 2 ))
if [ "$pad_top" -lt 0 ]; then pad_top=0; fi
if [ "$pad_left" -lt 0 ]; then pad_left=0; fi

# Build gradient binary
profile="${ARTBOX_PROFILE:-debug}"
target_dir="${CARGO_TARGET_DIR:-target}"
if [[ "$profile" == "release" ]]; then
    cargo build --quiet --release --example gradient
    gradient_bin="$target_dir/release/examples/gradient"
else
    cargo build --quiet --example gradient
    gradient_bin="$target_dir/debug/examples/gradient"
fi

cleanup() {
    tput sgr0 2>/dev/null || true
    tput cnorm 2>/dev/null || true
    clear
}

tput civis 2>/dev/null || true
trap cleanup INT TERM EXIT

# Clear once at start
clear

angle=0
while true; do
    output=$("$gradient_bin" "$text" "$width" "$height" \
        --gradient horizontal \
        --from "0,210,255" --to "255,20,160" \
        --angle "$angle" \
        --no-border)

    # Position cursor and draw each line
    row=$pad_top
    while IFS= read -r line; do
        tput cup "$row" "$pad_left" 2>/dev/null || true
        printf '%s' "$line"
        tput el 2>/dev/null || true
        row=$((row + 1))
    done <<< "$output"

    tput sgr0 2>/dev/null || true
    angle=$(( (angle + 15) % 360 ))
    sleep "$delay"
done

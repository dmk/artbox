#!/usr/bin/env bash
# Master showcase script - runs all demos in sequence
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "========================================"
echo "       ARTBOX Feature Showcase"
echo "========================================"
echo ""
echo "This will demonstrate various artbox features."
echo "Press Enter to continue, Ctrl+C to quit."
read -r

# Static gradient demo
echo ""
echo ">>> Static Gradient Types"
echo ""
"$SCRIPT_DIR/demo_gradient.sh" "ARTBOX" 50 8

echo ""
echo "Press Enter to see font families with gradients..."
read -r

# Font families demo
echo ""
"$SCRIPT_DIR/demo_fonts.sh" "HELLO" 55 9

echo ""
echo ">>> Animated Demos"
echo ""
echo "The following demos are animated and run until you press Ctrl+C."
echo ""
echo "Available animations:"
echo "  1) Rainbow - Cycling hue animation"
echo "  2) Pulse   - Breathing brightness effect"
echo "  3) Wave    - Moving gradient wave"
echo "  4) Skip    - End showcase"
echo ""
read -rp "Select animation (1-4): " choice

case "$choice" in
    1)
        echo "Starting rainbow animation (Ctrl+C to stop)..."
        sleep 1
        "$SCRIPT_DIR/demo_rainbow.sh" "RAINBOW" 55 9 0.1
        ;;
    2)
        echo "Starting pulse animation (Ctrl+C to stop)..."
        sleep 1
        "$SCRIPT_DIR/demo_pulse.sh" "PULSE" 45 9 0.05
        ;;
    3)
        echo "Starting wave animation (Ctrl+C to stop)..."
        sleep 1
        "$SCRIPT_DIR/demo_wave.sh" "WAVE" 45 9 0.08
        ;;
    *)
        echo "Showcase complete!"
        ;;
esac

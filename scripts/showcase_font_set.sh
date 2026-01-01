#!/usr/bin/env bash
set -euo pipefail

families=(
  "banner|banner|banner3|banner4"
  "cyber|cybersmall|cybermedium|cyberlarge"
  "isometric1|small_isometric1|isometric1"
  "keyboard|small_keyboard|keyboard"
  "poison|smpoison|small_poison|poison"
  "script|smscript|small_script|script"
  "shadow|small_shadow|shadow"
  "slant|small_slant|slant"
  "tengwar|smtengwar|small_tengwar|tengwar"
)

family_names_csv() {
  local entry name out
  out=""
  for entry in "${families[@]}"; do
    IFS='|' read -r name _ <<< "$entry"
    if [ -z "$out" ]; then
      out="$name"
    else
      out="${out},${name}"
    fi
  done
  printf '%s' "$out"
}

find_family_entry() {
  local target entry name
  target="$1"
  for entry in "${families[@]}"; do
    IFS='|' read -r name _ <<< "$entry"
    if [ "$name" = "$target" ]; then
      printf '%s' "$entry"
      return 0
    fi
  done
  return 1
}

render_font() {
  local label font
  label="$1"
  font="$2"
  echo "---- ${label} ----"
  cargo run --quiet --example print -- "$text" "$width" "$height" --alignment "$alignment" --spacing "$spacing" --font "$font"
  echo ""
}

usage() {
  local available_families
  available_families="$(family_names_csv)"
  echo "Usage: showcase_font_set.sh <text> <width> <height> [family1,family2,...] [alignment] [spacing]" >&2
  echo "Families: ${available_families}" >&2
  echo "Example: showcase_font_set.sh \"HELLO\" 60 12 slant,script c 0" >&2
}

if [ $# -lt 3 ]; then
  usage
  exit 2
fi

text="$1"
width="$2"
height="$3"
set_list="${4:-}"
alignment="${5:-c}"
spacing="${6:-0}"

if [ -z "$set_list" ]; then
  set_list="$(family_names_csv)"
fi

IFS=',' read -r -a selected_sets <<< "$set_list"

for family_name in "${selected_sets[@]}"; do
  family_name="$(printf '%s' "$family_name" | tr -d '[:space:]' | tr '[:upper:]' '[:lower:]')"
  if [ -z "$family_name" ]; then
    continue
  fi

  if ! entry="$(find_family_entry "$family_name")"; then
    echo "Unknown family: ${family_name}" >&2
    usage
    exit 2
  fi

  IFS='|' read -r -a parts <<< "$entry"
  fonts=("${parts[@]:1}")
  labels=("small" "medium" "large")
  if [ "${#fonts[@]}" -eq 2 ]; then
    labels=("small" "large")
  elif [ "${#fonts[@]}" -eq 1 ]; then
    labels=("font")
  fi

  echo "====== ${family_name} ======"
  for i in "${!fonts[@]}"; do
    label="${labels[$i]:-font$((i + 1))}"
    render_font "$label" "${fonts[$i]}"
  done
done

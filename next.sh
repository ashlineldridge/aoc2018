#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat << EOF
Usage: $(basename "${0}")

Sets up the next Advent of Code day directory.

EOF
  exit 1
}

create_day() {
  day_dir="${1}"
  cargo new --bin "${day_dir}"
  cat << EOF > "${day_dir}/rustfmt.toml"
edition = "2018"
EOF
}

for day_num in $(seq -w 1 25); do
  day_dir="day${day_num}"
  if [[ ! -e "${day_dir}" ]]; then
    create_day "${day_dir}"
    exit 0
  fi
done

echo >&2 "You've already created the last day!"

#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

find "$repo_root/reports" -mindepth 2 -maxdepth 3 -type f -name '*.tex' \
  ! -path "$repo_root/reports/templates/*" -print0 |
  while IFS= read -r -d '' report; do
    bash "$repo_root/scripts/build_report.sh" "$report"
  done

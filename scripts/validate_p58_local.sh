#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

parent_root="$(cd "$repo_root/.." && pwd)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$parent_root/.cargo-target}"

work_dir="$(mktemp -d "${TMPDIR:-/tmp}/astra-p58.XXXXXX")"
trap 'rm -rf "$work_dir"' EXIT

section() {
  printf '\n== %s ==\n' "$1"
}

run() {
  printf '+'
  printf ' %q' "$@"
  printf '\n'
  "$@"
}

section "Rust workspace"
run cargo fmt --all -- --check
run cargo build --workspace
run cargo test --workspace

section "P53 strict check/export"
run cargo run -p atlas-cli -- check examples/p53_strict.atlas
cargo run -p atlas-cli -- export examples/p53_strict.atlas --format json \
  >"$work_dir/p53_strict.json"
run diff -u tests/golden/p53_strict.json "$work_dir/p53_strict.json"

section "P58 bench and runtime modes"
run cargo run -p atlas-cli -- bench --mode smoke
run cargo run -p atlas-cli -- bench --mode standard
run cargo run -p atlas-cli -- run examples/p53_strict.atlas --mode smoke
run cargo run -p atlas-cli -- run examples/p53_strict.atlas --mode standard

section "P58 metrics"
cargo run -p atlas-cli -- metrics examples/p53_strict.atlas --mode smoke --format json \
  >"$work_dir/p58_metrics_smoke.json"
cargo run -p atlas-cli -- metrics examples/p53_strict.atlas --mode standard --format json \
  >"$work_dir/p58_metrics_standard.json"
grep -q '"astra_iteration": "ASTRA-SYS-P58"' "$work_dir/p58_metrics_smoke.json"
grep -q '"decision": "RECALIBRATE"' "$work_dir/p58_metrics_smoke.json"
grep -q '"decision": "VALIDATE"' "$work_dir/p58_metrics_standard.json"

section "P58 reports"
cargo run -p atlas-cli -- report examples/p53_strict.atlas --mode smoke --format json \
  >"$work_dir/p58_report_smoke.json"
cargo run -p atlas-cli -- report examples/p53_strict.atlas --mode standard --format json \
  >"$work_dir/p58_report_standard.json"
cargo run -p atlas-cli -- report examples/p53_strict.atlas --mode standard --format markdown \
  >"$work_dir/p58_report_standard.md"
run diff -u tests/golden/p58_report_smoke.json "$work_dir/p58_report_smoke.json"
run diff -u tests/golden/p58_report_standard.json "$work_dir/p58_report_standard.json"
grep -q 'ASTRA-SYS-P58 runtime report' "$work_dir/p58_report_standard.md"
grep -q 'P58_G7_report_generated' "$work_dir/p58_report_standard.md"

section "P57 compatibility"
cargo run -p atlas-cli -- report examples/p53_strict.atlas --format json \
  >"$work_dir/p57_report.json"
run diff -u tests/golden/p57_report.json "$work_dir/p57_report.json"

section "Invalid corpus"
invalid_checked=0
for invalid_file in examples/invalid/*.atlas; do
  invalid_checked=$((invalid_checked + 1))
  log_file="$work_dir/invalid-$(basename "$invalid_file").out"
  if cargo run -p atlas-cli -- check "$invalid_file" >"$log_file" 2>&1; then
    cat "$log_file"
    printf 'Expected %s to be refused.\n' "$invalid_file" >&2
    exit 1
  fi
  grep -Eq 'E_[A-Z0-9_]+' "$log_file"
  printf 'PASS invalid refused: %s\n' "$invalid_file"
done

if [ "${P58_WRITE_ARTIFACTS:-0}" = "1" ]; then
  mkdir -p artifacts/p58
  cat >artifacts/p58/astra-p58-local-validation-summary.json <<EOF
{
  "astra_iteration": "ASTRA-SYS-P58",
  "validation_trace_status": "LOCAL_PASS",
  "invalid_examples_checked": $invalid_checked,
  "smoke_decision": "RECALIBRATE",
  "standard_decision": "VALIDATE",
  "ambitious_required_by_ci": false
}
EOF
fi

section "Summary"
printf 'ASTRA-SYS-P58 local validation passed. Invalid examples checked: %s\n' "$invalid_checked"

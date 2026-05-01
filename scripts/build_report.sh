#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "usage: bash scripts/build_report.sh path/to/report.tex" >&2
  exit 2
fi

tex_path="$1"

if [ ! -f "$tex_path" ]; then
  echo "error: report source not found: $tex_path" >&2
  exit 1
fi

case "$tex_path" in
  *.tex) ;;
  *)
    echo "error: report source must end with .tex: $tex_path" >&2
    exit 1
    ;;
esac

tex_dir="$(cd "$(dirname "$tex_path")" && pwd)"
tex_file="$(basename "$tex_path")"
tex_base="${tex_file%.tex}"
pdf_path="$tex_dir/$tex_base.pdf"

find_tectonic() {
  if command -v tectonic >/dev/null 2>&1; then
    command -v tectonic
    return 0
  fi
  if [ -d "$HOME/.codex/plugins/cache" ]; then
    find "$HOME/.codex/plugins/cache" -maxdepth 6 -type f \
      -path '*/latex-tectonic/*/bin/tectonic' 2>/dev/null | head -n 1
  fi
  return 0
}

cleanup_aux() {
  find "$tex_dir" -maxdepth 1 -type f \( \
    -name "$tex_base.aux" -o \
    -name "$tex_base.log" -o \
    -name "$tex_base.out" -o \
    -name "$tex_base.toc" -o \
    -name "$tex_base.fls" -o \
    -name "$tex_base.fdb_latexmk" -o \
    -name "$tex_base.synctex.gz" -o \
    -name "$tex_base.bbl" -o \
    -name "$tex_base.blg" -o \
    -name "$tex_base.bcf" -o \
    -name "$tex_base.run.xml" \
  \) -delete
}

tectonic_bin="$(find_tectonic)"

if [ -n "$tectonic_bin" ]; then
  echo "compiler: tectonic"
  "$tectonic_bin" "$tex_path"
  cwd_pdf="$(pwd)/$tex_base.pdf"
  if [ ! -f "$pdf_path" ] && [ -f "$cwd_pdf" ]; then
    mv "$cwd_pdf" "$pdf_path"
  fi
elif command -v latexmk >/dev/null 2>&1; then
  echo "compiler: latexmk"
  (cd "$tex_dir" && latexmk -pdf -interaction=nonstopmode -halt-on-error "$tex_file")
elif command -v pdflatex >/dev/null 2>&1; then
  echo "compiler: pdflatex"
  (cd "$tex_dir" && pdflatex -interaction=nonstopmode -halt-on-error "$tex_file")
  (cd "$tex_dir" && pdflatex -interaction=nonstopmode -halt-on-error "$tex_file")
else
  echo "error: no LaTeX compiler found; expected tectonic, latexmk, or pdflatex" >&2
  exit 1
fi

if [ ! -s "$pdf_path" ]; then
  echo "error: PDF was not generated or is empty: $pdf_path" >&2
  exit 1
fi

cleanup_aux
echo "pdf: $pdf_path"

#!/usr/bin/env bash
# Compiles the load test figures to PDFs in ../out/figures, to \includegraphics them in the paper.
# Run `cargo run -p bencmark_results` first. Needs a TeX installation with pgfplots.
set -euo pipefail
cd "$(dirname "$0")"

# TinyTeX has no pgfplots, prefer the full TeX Live if it is installed
texlive=(/usr/local/texlive/*/bin/*)
if [[ -d ${texlive[0]} ]]; then
  PATH="${texlive[0]}:$PATH"
fi

build=$(mktemp -d)
trap 'rm -rf "$build"' EXIT
for run in axum express; do
  for figure in requests response; do
    name="lt_${run}_${figure}"
    pdflatex -interaction=nonstopmode -halt-on-error -output-directory="$build" -jobname="$name" \
      "\def\loadtestrun{$run}\def\figurefile{loadtest_$figure}\input{figures/standalone.tex}" \
      > "$build/$name.out" || { tail -n 30 "$build/$name.out" >&2; exit 1; }
    cp "$build/$name.pdf" "../out/figures/$name.pdf"
    echo "wrote out/figures/$name.pdf"
  done
done

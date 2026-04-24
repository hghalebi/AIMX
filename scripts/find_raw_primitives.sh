#!/usr/bin/env bash
set -euo pipefail

if (($# == 0)); then
  set -- src tests examples build.rs
fi

echo "Scanning public Rust API surfaces for meaning-bearing raw primitives..."

patterns=(
  'pub[[:space:]]+struct[^{;]*\{[^}]*pub[[:space:]]+[A-Za-z_][A-Za-z0-9_]*[[:space:]]*:[[:space:]]*(String|&str|bool|usize|u[0-9]+|i[0-9]+|f32|f64)'
  'pub[[:space:]]+(async[[:space:]]+)?fn[^(]+\([^)]*(String|&str|bool|usize|u[0-9]+|i[0-9]+|f32|f64)'
  'pub[[:space:]]+type[[:space:]]+[A-Za-z_][A-Za-z0-9_]*[[:space:]]*=[[:space:]]*(String|&str|bool|usize|u[0-9]+|i[0-9]+|f32|f64)'
)

found=0
for pattern in "${patterns[@]}"; do
  if rg --line-number --glob '*.rs' --regexp "$pattern" "$@"; then
    found=1
  fi
done

if ((found == 0)); then
  echo "No obvious public raw primitive API leaks found."
else
  echo
  echo "Review each hit. Boundary constructors and accessors are allowed only when they immediately parse into or expose from a domain type."
fi

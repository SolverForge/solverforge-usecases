#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

copy_app() {
  local source_dir="$1"
  local target_dir="$2"

  if [[ ! -d "$source_dir" ]]; then
    printf 'missing source directory: %s\n' "$source_dir" >&2
    exit 1
  fi

  mkdir -p "$target_dir"
  rsync -a --delete \
    --exclude='.git' \
    --exclude='target' \
    --exclude='test-results' \
    --exclude='playwright-report' \
    --exclude='.osm_cache' \
    "$source_dir"/ "$target_dir"/
}

copy_app "$ROOT/../use-cases/solverforge-deliveries" "$ROOT/uc-deliveries"
copy_app "$ROOT/../use-cases/solverforge-fsr" "$ROOT/uc-fsr"
copy_app "$ROOT/../use-cases/solverforge-hospital" "$ROOT/uc-hospital"
copy_app "$ROOT/../use-cases/solverforge-lessons" "$ROOT/uc-lessons"

printf 'Imported source-backed SolverForge use cases.\n'

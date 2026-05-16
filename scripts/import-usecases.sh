#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE_ROOT="${USECASE_SOURCE_ROOT:-$ROOT/../use-cases}"

if [[ ! -d "$SOURCE_ROOT" ]]; then
  printf 'missing source root: %s\n' "$SOURCE_ROOT" >&2
  printf 'set USECASE_SOURCE_ROOT=/path/to/use-cases to refresh imported apps\n' >&2
  exit 1
fi

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

copy_app "$SOURCE_ROOT/solverforge-deliveries" "$ROOT/uc-deliveries"
copy_app "$SOURCE_ROOT/solverforge-fsr" "$ROOT/uc-fsr"
copy_app "$SOURCE_ROOT/solverforge-hospital" "$ROOT/uc-hospital"
copy_app "$SOURCE_ROOT/solverforge-lessons" "$ROOT/uc-lessons"

printf 'Imported source-backed SolverForge use cases.\n'

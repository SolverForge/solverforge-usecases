#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE_ROOT="${USECASE_SOURCE_ROOT:-$ROOT/../use-cases}"
STRICT="${VERIFY_IMPORTS_STRICT:-0}"

EXCLUDES=(
  --exclude='.git'
  --exclude='target'
  --exclude='test-results'
  --exclude='playwright-report'
  --exclude='.osm_cache'
)

verify_pair() {
  local source_dir="$1"
  local target_dir="$2"

  if [[ ! -d "$source_dir" ]]; then
    printf 'missing source directory: %s\n' "$source_dir" >&2
    exit 1
  fi
  if [[ ! -d "$target_dir" ]]; then
    printf 'missing imported directory: %s\n' "$target_dir" >&2
    exit 1
  fi

  local diff_output
  diff_output="$(rsync -a --delete --dry-run --itemize-changes --checksum --no-times "${EXCLUDES[@]}" "$source_dir"/ "$target_dir"/)"
  if [[ -n "$diff_output" ]]; then
    printf 'import drift: %s -> %s\n' "$source_dir" "$target_dir" >&2
    printf '%s\n' "$diff_output" >&2
    exit 1
  fi
}

if [[ ! -d "$SOURCE_ROOT" ]]; then
  if [[ "$STRICT" == "1" || "$STRICT" == "true" ]]; then
    printf 'import source root not found: %s\n' "$SOURCE_ROOT" >&2
    printf 'set USECASE_SOURCE_ROOT=/path/to/use-cases to run source-backed drift verification\n' >&2
    exit 1
  fi
  printf 'import source root not found: %s\n' "$SOURCE_ROOT"
  printf 'set USECASE_SOURCE_ROOT=/path/to/use-cases to run source-backed drift verification\n'
  printf 'Skipping import drift verification because this checkout has no source repo root.\n'
  exit 0
fi

verify_pair "$SOURCE_ROOT/solverforge-deliveries" "$ROOT/uc-deliveries"
verify_pair "$SOURCE_ROOT/solverforge-fsr" "$ROOT/uc-fsr"
verify_pair "$SOURCE_ROOT/solverforge-hospital" "$ROOT/uc-hospital"
verify_pair "$SOURCE_ROOT/solverforge-lessons" "$ROOT/uc-lessons"

printf 'Source-backed SolverForge imports verified.\n'

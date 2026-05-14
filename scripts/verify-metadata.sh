#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

open_source_dirs=(
  uc-deliveries
  uc-fsr
  uc-hospital
  uc-lessons
)

expected_spaces=(
  solverforge-deliveries
  solverforge-fsr
  solverforge-hospital
  solverforge-lessons
)

required_files=(
  README.md
  AGENTS.md
  WIREFRAME.md
  Cargo.toml
  Dockerfile
  solverforge.app.toml
  static/sf-config.json
  docs/screenshot.png
)

is_open_source_dir() {
  local candidate="$1"
  local open_source
  for open_source in "${open_source_dirs[@]}"; do
    [[ "$candidate" == "$open_source" ]] && return 0
  done
  return 1
}

for dir in "${open_source_dirs[@]}"; do
  [[ -d "$dir" ]] || { printf 'missing open-source directory: %s\n' "$dir" >&2; exit 1; }
  for file in "${required_files[@]}"; do
    [[ -f "$dir/$file" ]] || { printf 'missing required file: %s/%s\n' "$dir" "$file" >&2; exit 1; }
  done
done

while IFS= read -r dir; do
  [[ -n "$dir" ]] || continue
  if ! is_open_source_dir "$dir"; then
    printf 'non-allowlisted use-case directory: %s\n' "$dir" >&2
    exit 1
  fi
done < <(find . -maxdepth 1 -type d -name 'uc-*' -printf '%f\n' | sort)

for index in "${!open_source_dirs[@]}"; do
  dir="${open_source_dirs[$index]}"
  expected="${expected_spaces[$index]}"
  actual="${dir/uc-/solverforge-}"
  [[ "$actual" == "$expected" ]] || {
    printf 'unexpected Space mapping for %s: got %s expected %s\n' "$dir" "$actual" "$expected" >&2
    exit 1
  }
done

upper_agent="$(printf '\x43\x4c\x41\x55\x44\x45')"
title_agent="$(printf '\x43\x6c\x61\x75\x64\x65')"
lower_agent="$(printf '\x63\x6c\x61\x75\x64\x65')"
title_index="$(printf '\x47\x69\x74\x4e\x65\x78\x75\x73')"
lower_index="$(printf '\x67\x69\x74\x6e\x65\x78\x75\x73')"
residue_pattern="$(printf '%s|%s|%s|%s|%s' "$upper_agent" "$title_agent" "$lower_agent" "$title_index" "$lower_index")"

repo_files=()
while IFS= read -r -d '' file; do
  [[ -e "$file" ]] || continue
  repo_files+=("$file")
done < <(git ls-files -z --cached --others --exclude-standard)

if ((${#repo_files[@]})) && rg -n "$residue_pattern" -- "${repo_files[@]}" >/tmp/solverforge-usecases-residue.txt; then
  cat /tmp/solverforge-usecases-residue.txt >&2
  rm -f /tmp/solverforge-usecases-residue.txt
  exit 1
fi
rm -f /tmp/solverforge-usecases-residue.txt

name_residue=()
for file in "${repo_files[@]}"; do
  lower_file="${file,,}"
  if [[ "$lower_file" == *"$lower_agent"* || "$lower_file" == *"$lower_index"* ]]; then
    name_residue+=("$file")
  fi
done

if ((${#name_residue[@]})); then
  printf '%s\n' "${name_residue[@]}" >&2
  exit 1
fi

printf 'SolverForge use-case metadata verified.\n'

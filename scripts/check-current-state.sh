#!/bin/sh
# Gate for auto-pr: a branch ahead of main must carry a new roadmap snapshot.
#
# The files already use this shape:
#   primness_roadmap.md metadata line `- **Last Modified**: <stamp>`
#   primness_roadmap.md section `## Current State` begins with `As of <stamp>:`
#   context_convo_notes/current_state_history.md keeps older snapshots, newest
#   first, under `### <stamp>` headings. The previous heading stays in the file.
# README.md is updated in the same change (README, Documentation Maintenance).
#
# Usage:
#   scripts/check-current-state.sh [base-ref] [head-ref]
#   scripts/check-current-state.sh --print-section [head-ref]
set -eu

print_section() {
  ref=$1
  git show "$ref:primness_roadmap.md" | awk '
    /^## Current State$/ { print; p=1; next }
    /^## / { if (p) exit }
    /^---$/ { if (p) exit }
    p { print }
  '
}

if [ "${1:-}" = "--print-section" ]; then
  print_section "${2:-HEAD}"
  exit 0
fi

base=${1:-origin/main}
head=${2:-HEAD}

fail() {
  echo "current-state check: $*" >&2
  exit 1
}

show() {
  ref=$1
  path=$2
  if ! git cat-file -e "$ref:$path" 2>/dev/null; then
    fail "$ref is missing $path"
  fi
  git show "$ref:$path"
}

section_of() {
  printf '%s\n' "$1" | awk '
    /^## Current State$/ { p=1; next }
    /^## / { if (p) exit }
    /^---$/ { if (p) exit }
    p { print }
  '
}

as_of_stamp() {
  printf '%s\n' "$1" | sed -n 's/^As of \(.*\):$/\1/p' | head -n 1
}

meta_value() {
  label=$1
  printf '%s\n' "$2" | sed -n "s/^- \\*\\*${label}\\*\\*: //p" | head -n 1
}

base_road=$(show "$base" primness_roadmap.md)
head_road=$(show "$head" primness_roadmap.md)
base_state=$(section_of "$base_road")
head_state=$(section_of "$head_road")

[ -n "$base_state" ] || fail "$base has no ## Current State section in primness_roadmap.md"
[ -n "$head_state" ] || fail "$head has no ## Current State section in primness_roadmap.md"
[ "$base_state" != "$head_state" ] || fail "primness_roadmap.md ## Current State is unchanged from $base. Update that section and record the snapshot in context_convo_notes/current_state_history.md before Auto-PR will open or refresh the draft."

base_as=$(as_of_stamp "$base_state")
head_as=$(as_of_stamp "$head_state")
[ -n "$base_as" ] || fail "$base Current State has no 'As of <stamp>:' line"
[ -n "$head_as" ] || fail "$head Current State has no 'As of <stamp>:' line"
[ "$base_as" != "$head_as" ] || fail "the Current State 'As of' stamp is unchanged from $base"

base_lm=$(meta_value "Last Modified" "$base_road")
head_lm=$(meta_value "Last Modified" "$head_road")
[ -n "$base_lm" ] || fail "$base roadmap metadata has no Last Modified line"
[ -n "$head_lm" ] || fail "$head roadmap metadata has no Last Modified line"
[ "$base_lm" != "$head_lm" ] || fail "roadmap metadata Last Modified is unchanged from $base"

base_hist=$(show "$base" context_convo_notes/current_state_history.md)
head_hist=$(show "$head" context_convo_notes/current_state_history.md)
printf '%s\n' "$head_hist" | grep -Fxq "### $head_as" || fail "current_state_history.md is missing the new snapshot heading: ### $head_as"
printf '%s\n' "$head_hist" | grep -Fxq "### $base_as" || fail "current_state_history.md no longer contains the previous snapshot heading: ### $base_as"

base_hlm=$(meta_value "Last Modified" "$base_hist")
head_hlm=$(meta_value "Last Modified" "$head_hist")
[ -n "$base_hlm" ] || fail "$base history metadata has no Last Modified line"
[ -n "$head_hlm" ] || fail "$head history metadata has no Last Modified line"
[ "$base_hlm" != "$head_hlm" ] || fail "current_state_history.md metadata Last Modified is unchanged from $base"

if git diff --quiet "$base" "$head" -- README.md; then
  fail "README.md is unchanged from $base. The README says it is updated in the same pull request as the roadmap Current State."
fi

echo "current-state check: $base_as -> $head_as"

#!/bin/sh
set -eu

bin=${1:?usage: onboarding-smoke.sh BINARY EXPECTED_SKILL}
expected=${2:?usage: onboarding-smoke.sh BINARY EXPECTED_SKILL}
case "$bin" in
  /*) ;;
  *) bin="$(cd "$(dirname "$bin")" && pwd)/$(basename "$bin")" ;;
esac
test -x "$bin"
test -f "$expected"

root=$(mktemp -d)
cleanup() { find "$root" -depth -delete; }
trap cleanup EXIT HUP INT TERM
mkdir -p "$root/home" "$root/project" "$root/bindings"
config="$root/project/soulmate.json"
activation="$root/activation.json"

invoke() {
  env -i HOME="$root/home" SOULMATE_BINDINGS_DIR="$root/bindings" \
    PATH=/nonexistent "$bin" "$@"
}

next=$(invoke init --mode portable --root "$root/project")
case "$next" in
  *"soulmate brief"*"soulmate work begin"*"soulmate check"*) ;;
  *) echo "onboarding smoke: init did not print the core path" >&2; exit 1 ;;
esac

test -f "$root/project/.agents/skills/soulmate/SKILL.md"
test -f "$root/project/.claude/skills/soulmate/SKILL.md"
cmp "$expected" "$root/project/.agents/skills/soulmate/SKILL.md"
cmp "$expected" "$root/project/.claude/skills/soulmate/SKILL.md"
cmp "$root/project/.agents/skills/soulmate/SKILL.md" "$root/project/.claude/skills/soulmate/SKILL.md"
invoke brief worker --task "First bounded handoff" --config "$config" >/dev/null
invoke work begin change --goal "First bounded handoff" \
  --check-command true --config "$config" >"$activation"
grep -q '"work": "smw_' "$activation"
grep -q '"action":' "$activation"
test "$(find "$root/project/.soulmate/runs" -type f -name 'work-*.jsonl' | wc -l | tr -d ' ')" = 1
invoke check --config "$config" >/dev/null
invoke benchmark >/dev/null

printf '%s\n' "onboarding smoke passed"

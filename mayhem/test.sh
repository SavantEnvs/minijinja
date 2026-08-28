#!/usr/bin/env bash
#
# mayhem/test.sh — BEHAVIORAL oracle (SPEC §6.3). Runs the KAT probe that
# mayhem/build.sh built (/mayhem/kat-probe) and asserts its EXACT rendered output
# against known answers computed by the unmodified minijinja engine.
#
# Why a KAT probe rather than `cargo test`: the gate proves the oracle is behavioral
# by LD_PRELOADing a shim that _exit(0)s every non-system binary and requiring test.sh
# to then FAIL. The probe lives under /mayhem, so the shim neuters it (empty output),
# every known-answer assertion misses, and the suite fails — exactly the behavior the
# anti-reward-hacking check demands. bash/coreutils (used for the comparisons) are
# spared by the shim, so the assertions run where sabotage cannot hide.
#
# Emits a CTRF summary. Exits 0 iff failed==0. Does NOT compile (build.sh did).
set -uo pipefail
[ -n "${SOURCE_DATE_EPOCH:-}" ] || unset SOURCE_DATE_EPOCH
cd "$SRC"

# emit_ctrf <tool> <passed> <failed> [skipped] [pending] [other]
emit_ctrf() {
  local tool="$1" passed="$2" failed="$3" skipped="${4:-0}" pending="${5:-0}" other="${6:-0}"
  local tests=$(( passed + failed + skipped + pending + other ))
  cat > "${CTRF_REPORT:-$SRC/ctrf-report.json}" <<JSON
{
  "results": {
    "tool": { "name": "$tool" },
    "summary": {
      "tests": $tests,
      "passed": $passed,
      "failed": $failed,
      "pending": $pending,
      "skipped": $skipped,
      "other": $other
    }
  }
}
JSON
  printf 'CTRF {"results":{"tool":{"name":"%s"},"summary":{"tests":%d,"passed":%d,"failed":%d,"pending":%d,"skipped":%d,"other":%d}}}\n' \
    "$tool" "$tests" "$passed" "$failed" "$pending" "$skipped" "$other"
  [ "$failed" -eq 0 ]
}

PROBE=/mayhem/kat-probe
PASSED=0
FAILED=0

check() {
  # check <description> <expected> <actual>
  local desc="$1" expected="$2" actual="$3"
  if [ "$actual" = "$expected" ]; then
    PASSED=$((PASSED + 1)); echo "PASS $desc: '$actual'"
  else
    FAILED=$((FAILED + 1)); echo "FAIL $desc: expected '$expected' got '$actual'"
  fi
}

# The probe binary MUST exist (build.sh built it). A missing probe is a hard FAILURE,
# never a skip (an unconditional oracle, §4).
if [ ! -x "$PROBE" ]; then
  echo "ERROR: KAT probe $PROBE missing or not executable — build.sh did not produce it" >&2
  emit_ctrf "minijinja-kat" 0 1 0
  exit 1
fi

# Capture the probe's three known-answer lines.
OUT="$("$PROBE" 2>/dev/null || true)"
L1="$(printf '%s\n' "$OUT" | sed -n 1p)"
L2="$(printf '%s\n' "$OUT" | sed -n 2p)"
L3="$(printf '%s\n' "$OUT" | sed -n 3p)"

check "arithmetic precedence (1 + 3*2)"      "7"          "$L1"
check "upper filter (minijinja -> upper)"    "MINIJINJA"  "$L2"
check "range(3) for-loop"                    "012"        "$L3"

emit_ctrf "minijinja-kat" "$PASSED" "$FAILED" 0

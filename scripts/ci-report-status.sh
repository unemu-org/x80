#!/usr/bin/env bash
# Report every required step, including skipped steps, as a commit status.
# OUTCOMES: space-separated <step>=<outcome> pairs from the workflow.
# TOKEN / API: job token and commit-status endpoint; RUN_URL: status target.
# JOB_STATUS: optional overall job outcome, to catch failures outside the gates.
# CI_REPORT_DRY_RUN=1 prints JSON without requiring TOKEN / API or making a POST.
set -euo pipefail

verdict=0
failures=()
details=""
failed_test='^[[:space:]]*(FAIL( \+ LEAK)?|LEAK-FAIL|TIMEOUT|SIG[A-Z0-9]+|ABORT( SIG [0-9]+)?) \['

for step in nix devshell fmt clippy tests doctests deny msrv; do
  outcome=missing
  for pair in ${OUTCOMES:-}; do
    if [[ ${pair%%=*} == "$step" ]]; then
      outcome=${pair#*=}
      break
    fi
  done
  [[ "$outcome" != success ]] || continue
  verdict=1
  failures+=("$step=$outcome")
  log="ci-logs/$step.log"
  if [[ -f "$log" ]]; then
    # First failing nextest names, otherwise the first compiler/test errors.
    # sed reads the full stream so pipefail cannot turn truncation into SIGPIPE.
    detail=$(grep -E "$failed_test" "$log" | sed -n '1,2p' || true)
    if [[ -z "$detail" ]]; then
      detail=$(grep -Ei 'error|panicked|failed' "$log" | sed -n '1,2p' || true)
    fi
    if [[ -n "$detail" ]]; then
      details+=$'\n'"$step: ${detail:0:240}"
    fi
  fi
done

if [[ ${JOB_STATUS:-success} != success ]]; then
  verdict=1
  failures+=("job=$JOB_STATUS")
fi

state=success
desc="all gate steps passed"
if (( verdict != 0 )); then
  state=failure
  # List all failing steps first so log excerpts cannot hide later failures.
  desc="failed: ${failures[*]}$details"
fi

body=$(STATE="$state" DESC="$desc" node -e '
  // Keep the description within a conservative 255-byte API budget, preserving
  // UTF-8 and leaving room for a truncation marker.
  const limit = 255;
  let description = process.env.DESC.replace(/\r/g, "");
  if (Buffer.byteLength(description) > limit) {
    let prefix = "";
    for (const char of description) {
      if (Buffer.byteLength(prefix + char) > limit - 3) break;
      prefix += char;
    }
    description = prefix + "...";
  }
  process.stdout.write(JSON.stringify({
    state: process.env.STATE,
    context: "ci / x86_64",
    description,
    target_url: process.env.RUN_URL || "",
  }));
')

if [[ ${CI_REPORT_DRY_RUN:-0} == 1 ]]; then
  printf '%s\n' "$body"
else
  # Do not print the token or the response body (which may contain server data).
  # A failed POST never overrides the verdict established by the gates.
  if [[ -z ${TOKEN:-} || -z ${API:-} ]]; then
    printf '%s\n' 'Status report not posted: TOKEN or API is missing.' >&2
  elif ! curl -sS --fail-with-body --connect-timeout 10 --max-time 30 \
    -X POST -H "Authorization: token $TOKEN" -H 'Content-Type: application/json' \
    --data "$body" --output /dev/null "$API" 2>/dev/null; then
    printf '%s\n' 'Status report POST failed; the gate verdict stands.' >&2
  fi
  printf '%s\n' "$body"
fi
exit "$verdict"

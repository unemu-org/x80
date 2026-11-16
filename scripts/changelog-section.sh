#!/usr/bin/env bash
# changelog-section — print the CHANGELOG.md section of one x80-core version.
# The release workflow (.github/workflows/release.yml) uses it twice: to check
# the section before publication and as the GitHub Release notes
# (decision-21 item 7).
#
# Usage: scripts/changelog-section.sh [--dated] VERSION [FILE]
#
# The section starts at the one heading "## [VERSION] - DATE" and ends before
# the next line that starts with "## ", or at the end of the file. DATE is
# YYYY-MM-DD or, until the release commit dates it, "Unreleased"; with --dated
# it must be a date. The section body is printed without its heading and
# without leading or trailing blank lines, and must hold at least one non-blank
# line. FILE defaults to the repository's CHANGELOG.md. On any other shape the
# script prints nothing on stdout, explains on stderr and exits 1.

set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"

die() {
  echo "changelog-section: $*" >&2
  exit 1
}

dated=0
if [[ ${1-} == --dated ]]; then
  dated=1
  shift
fi
(($# == 1 || $# == 2)) || die "usage: changelog-section.sh [--dated] VERSION [FILE]"
version="$1"
file="${2:-$root/CHANGELOG.md}"

[[ $version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "not a version X.Y.Z: $version"
[[ -f $file ]] || die "no such file: $file"

# The heading is matched as a literal prefix, so the dots in VERSION are not
# regular-expression wildcards. The body is printed only after every check
# has passed.
awk -v head="## [$version]" -v dated="$dated" -v file="$file" '
  function fail(msg) {
    printf "changelog-section: %s in %s\n", msg, file > "/dev/stderr"
    exit 1
  }
  /^## / { inside = 0 }
  index($0, head) == 1 {
    count++
    suffix = substr($0, length(head) + 1)
    inside = 1
    next
  }
  inside { lines[++n] = $0 }
  END {
    date_re = "^ - [0-9][0-9][0-9][0-9]-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])$"
    if (count == 0) fail("no \"" head " - DATE\" heading")
    if (count > 1) fail("more than one \"" head "\" heading")
    if (suffix !~ date_re && suffix != " - Unreleased")
      fail("heading \"" head suffix "\" is not \"" head " - YYYY-MM-DD\" or \"" head " - Unreleased\"")
    if (dated && suffix !~ date_re)
      fail("heading \"" head suffix "\" has no release date (YYYY-MM-DD)")
    first = 1
    while (first <= n && lines[first] ~ /^[[:space:]]*$/) first++
    last = n
    while (last >= first && lines[last] ~ /^[[:space:]]*$/) last--
    if (first > last) fail("the \"" head "\" section is empty")
    for (i = first; i <= last; i++) print lines[i]
  }
' "$file"

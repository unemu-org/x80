#!/usr/bin/env bash
# fetch-cpu-tests — download only the CPU test programs that the release-mode
# campaigns run (decision-21 item 3), verify each one against the pinned
# oracles/SHA256SUMS.fetched and place it where the campaign commands in
# backlog/docs/commands.md expect it.
#
# The upstream URLs keep one source: the fetch-only table of
# oracles/fetch-oracles.sh. That script is parsed here, never sourced or run:
#   - the table is the body of its `fetchonly_files(){ cat <<EOF` here-document,
#     up to the next line that is exactly `EOF`;
#   - each needed row is `<path> <url> <min-bytes>` and occurs exactly once;
#   - each `${NAME}` in a URL is replaced by the value of the one column-0
#     `NAME="value"` assignment in that script, repeatedly.
# A change to that format stops this script with an error: a renamed function
# or delimiter, a missing, duplicated or reshaped row, an assignment that is not
# one plain double-quoted value, or any other shell syntax in a URL.
#
# Usage: scripts/fetch-cpu-tests.sh [DEST]
# DEST defaults to the repository's oracles/ directory. Nothing is placed unless
# every file downloads and verifies. Needs bash, curl, awk, grep and sha256sum
# or shasum. The GPL-2.0-or-later programs must never be committed, cached or
# uploaded.

set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
oracles_script="$root/oracles/fetch-oracles.sh"
sums="$root/oracles/SHA256SUMS.fetched"
dest="${1:-$root/oracles}"

# What the campaigns read: x80-oracle opens only the program path it is given
# and the z80test ZIP.
files=(
  8080-exercisers/8080PRE.COM
  8080-exercisers/8080EXM.COM
  8080-exercisers/8085EXM.COM
  z80-exercisers/prelim.com
  z80-exercisers/zexdoc.com
  z80-exercisers/zexall.com
  z80-exercisers/z80test-1.2a.zip
)

die() {
  echo "fetch-cpu-tests: $*" >&2
  exit 1
}

for f in "$oracles_script" "$sums"; do
  [[ -f $f ]] || die "missing $f; check out the oracles submodule"
done

if command -v sha256sum >/dev/null 2>&1; then
  check=(sha256sum -c)
elif command -v shasum >/dev/null 2>&1; then
  check=(shasum -a 256 -c)
else
  die "need sha256sum or shasum"
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir "$work/files"
: >"$work/SHA256SUMS"

# The body of the fetch-only table.
awk '
  $0 == "fetchonly_files(){ cat <<EOF" { inside = 1; found = 1; next }
  inside && $0 == "EOF" { closed = 1; exit }
  inside { print }
  END { if (!found || !closed) exit 1 }
' "$oracles_script" >"$work/table" ||
  die "no fetchonly_files(){ cat <<EOF ... EOF table in $oracles_script"

ref_re='[$][{]([A-Za-z0-9_]+)[}]'
value_re='^"(([A-Za-z0-9._~:/%-]|[$][{][A-Za-z0-9_]+[}])*)"([[:space:]]+#.*)?$'
url_re='^https://[A-Za-z0-9._~:/%-]+$'

# Sets value to the one NAME="..." assignment at column 0 of the oracles script.
assignment() {
  local name="$1" line
  line="$(grep -E "^${name}=" "$oracles_script" || true)"
  [[ -n $line && $line != *$'\n'* ]] ||
    die "expected exactly one ${name}= assignment in $oracles_script"
  [[ ${line#"${name}="} =~ $value_re ]] ||
    die "${name} is not one plain double-quoted value: $line"
  value="${BASH_REMATCH[1]}"
}

# Sets url to the expanded URL of the one table row for PATH.
url_for() {
  local path="$1" row _ template min extra name pattern i
  row="$(awk -v p="$path" '$1 == p' "$work/table")"
  [[ -n $row && $row != *$'\n'* ]] ||
    die "expected exactly one row for $path in the fetch-only table"
  read -r _ template min extra <<<"$row"
  [[ $min =~ ^[0-9]+$ && -z $extra ]] ||
    die "row is not '<path> <url> <min-bytes>': $row"
  url="$template"
  for ((i = 0; ; i++)); do
    [[ $url =~ $ref_re ]] || break
    ((i < 10)) || die "too many nested references in $template"
    name="${BASH_REMATCH[1]}"
    assignment "$name"
    pattern="\${$name}"
    url="${url//"$pattern"/$value}"
  done
  [[ $url =~ $url_re ]] ||
    die "row for $path does not give a plain https URL: $template -> $url"
}

for f in "${files[@]}"; do
  # The pinned hash: exactly one "<sha256>  <path>" line.
  line="$(awk -v p="$f" 'NF == 2 && $2 == p && $1 ~ /^[0-9a-f]+$/ && length($1) == 64' "$sums")"
  [[ -n $line && $line != *$'\n'* ]] ||
    die "expected exactly one SHA-256 line for $f in $sums"
  printf '%s\n' "$line" >>"$work/SHA256SUMS"

  url_for "$f"
  echo "fetch $f <- $url"
  mkdir -p "$work/files/$(dirname "$f")"
  # -q ignores any curlrc: no custom User-Agent, headers or credentials.
  # --retry covers transient upstream failures (timeouts, 408, 429, 5xx).
  curl -q -sSfL --retry 3 --retry-delay 5 --max-time 120 -o "$work/files/$f" "$url" ||
    die "download failed: $f <- $url"
done

(cd "$work/files" && "${check[@]}" ../SHA256SUMS) ||
  die "SHA-256 verification failed; nothing was placed"

for f in "${files[@]}"; do
  mkdir -p "$dest/$(dirname "$f")"
  mv -f "$work/files/$f" "$dest/$f"
done
echo "fetch-cpu-tests: ${#files[@]} files verified and placed under $dest"

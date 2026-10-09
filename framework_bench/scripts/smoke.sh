#!/usr/bin/env bash
# Checks that the Axum and Express framework servers answer every endpoint identically
# (status, media type, body) before any measurement starts. Exits non-zero on a mismatch.
#
#   framework_bench/scripts/smoke.sh http://localhost:8081 http://localhost:8082
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <axum-base-url> <express-base-url>" >&2
  exit 2
fi
AXUM=${1%/}
EXPRESS=${2%/}
FILE_PATH=${FILE_PATH:-"$(dirname "$0")/../../loadtest/src/loadtest.glb"}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# ~2 KB echo payload: 40 items with fractional values.
node -e '
const items = Array.from({ length: 40 }, (_, i) => ({ name: `item-${String(i).padStart(2, "0")}-north-facade`, value: i * 1.5 + 0.25 }))
process.stdout.write(JSON.stringify({ id: 42, title: "Window frame review", items }))
' > "$tmp/echo.json"
echo '{"id":"not-a-number","title":"x","items":[]}' > "$tmp/echo_invalid.json"
head -c 1048576 /dev/urandom > "$tmp/upload.bin"

failures=0

# fetch <name> <url> [curl args...] -> writes $tmp/<name>.{status,type,body}
fetch() {
  local name=$1 url=$2
  shift 2
  curl -sS -o "$tmp/$name.body" -w '%{http_code}\n%{content_type}' "$@" "$url" > "$tmp/$name.meta"
  sed -n 1p "$tmp/$name.meta" > "$tmp/$name.status"
  sed -n 2p "$tmp/$name.meta" | cut -d';' -f1 | tr -d ' ' > "$tmp/$name.type"
}

# compare <label> <mode: json|bytes|status> <path> [curl args...]
compare() {
  local label=$1 mode=$2 path=$3
  shift 3
  fetch a "$AXUM$path" "$@"
  fetch e "$EXPRESS$path" "$@"
  local sa se ta te ok=1 note=""
  sa=$(cat "$tmp/a.status"); se=$(cat "$tmp/e.status")
  ta=$(cat "$tmp/a.type"); te=$(cat "$tmp/e.type")
  [[ $sa == "$se" ]] || { ok=0; note+=" status $sa != $se;"; }
  if [[ $mode != status ]]; then
    [[ $ta == "$te" ]] || { ok=0; note+=" type '$ta' != '$te';"; }
    case $mode in
      json)
        node -e '
          const fs = require("fs"), util = require("util")
          const [a, e] = process.argv.slice(1).map((f) => JSON.parse(fs.readFileSync(f, "utf8")))
          process.exit(util.isDeepStrictEqual(a, e) ? 0 : 1)
        ' "$tmp/a.body" "$tmp/e.body" || { ok=0; note+=" JSON bodies differ;"; }
        ;;
      bytes)
        cmp -s "$tmp/a.body" "$tmp/e.body" || { ok=0; note+=" bodies differ;"; }
        ;;
    esac
  fi
  local size
  size=$(stat -c %s "$tmp/a.body")
  if [[ $ok == 1 ]]; then
    printf 'OK    %-28s %s %-26s %8s B\n' "$label" "$sa" "$ta" "$size"
  else
    printf 'FAIL  %-28s%s\n' "$label" "$note"
    failures=$((failures + 1))
  fi
}

echo "Axum:    $AXUM"
echo "Express: $EXPRESS"
echo

compare "GET /plaintext" bytes /plaintext
compare "GET /json" json /json
compare "POST /echo-json" json /echo-json -X POST -H 'content-type: application/json' --data-binary "@$tmp/echo.json"
compare "GET /json-large" json /json-large
compare "GET /cpu" bytes /cpu
compare "GET /delay?ms=20" bytes "/delay?ms=20"
compare "GET /delay (default)" bytes /delay
compare "GET /file" bytes /file
compare "POST /upload (1 MiB)" json /upload -X POST -H 'content-type: application/octet-stream' --data-binary "@$tmp/upload.bin"
compare "POST /echo-json invalid" status /echo-json -X POST -H 'content-type: application/json' --data-binary "@$tmp/echo_invalid.json"
compare "GET /delay?ms=abc" status "/delay?ms=abc"
compare "GET /missing" status /missing

# The served file must be the source file, not just identical between the servers.
fetch a "$AXUM/file"
if cmp -s "$tmp/a.body" "$FILE_PATH"; then
  echo "OK    /file matches $(basename "$FILE_PATH")"
else
  echo "FAIL  /file does not match $FILE_PATH"
  failures=$((failures + 1))
fi

# The upload size must be echoed back exactly.
fetch a "$AXUM/upload" -X POST -H 'content-type: application/octet-stream' --data-binary "@$tmp/upload.bin"
if [[ $(node -p 'JSON.parse(require("fs").readFileSync(process.argv[1], "utf8")).bytes' "$tmp/a.body") == 1048576 ]]; then
  echo "OK    /upload reports 1048576 bytes"
else
  echo "FAIL  /upload reports $(cat "$tmp/a.body")"
  failures=$((failures + 1))
fi

# Keep-alive: three requests through one curl call must reuse a single connection.
for target in "axum $AXUM" "express $EXPRESS"; do
  set -- $target
  connects=$(curl -sS -o /dev/null -o /dev/null -o /dev/null -w '%{num_connects} ' "$2/json" "$2/json" "$2/json")
  if [[ $connects == "1 0 0 " ]]; then
    echo "OK    keep-alive $1 (connects: $connects)"
  else
    echo "FAIL  keep-alive $1 (connects: $connects)"
    failures=$((failures + 1))
  fi
done

echo
if [[ $failures -gt 0 ]]; then
  echo "$failures check(s) failed"
  exit 1
fi
echo "all checks passed"

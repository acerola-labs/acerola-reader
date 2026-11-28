#!/bin/sh
# Runs an app's Komodo Procedure through the API and waits for it to finish: the deploy step of
# every <local>-deploy.yml, started only by a version tag (acerola-git skill). Same file in every
# project.
#
#   sh scripts/release/komodo-deploy.sh <procedure>
#
# Environment: KOMODO_URL (vars) and KOMODO_API_KEY, KOMODO_API_SECRET (secrets). Needs curl and
# jq (both on the GitHub runner).
set -eu

procedure="${1:?Procedure name, e.g. hetzner_acerola}"
: "${KOMODO_URL:?}" "${KOMODO_API_KEY:?}" "${KOMODO_API_SECRET:?}"

api() {
  curl -fsS -X POST "${KOMODO_URL%/}/$1" \
    -H "X-Api-Key: $KOMODO_API_KEY" \
    -H "X-Api-Secret: $KOMODO_API_SECRET" \
    -H 'Content-Type: application/json' \
    -d "$2"
}

update=$(api execute "{\"type\":\"RunProcedure\",\"params\":{\"procedure\":\"$procedure\"}}")
id=$(printf '%s' "$update" | jq -r '._id."$oid" // .id')
echo "Procedure $procedure started (update $id)."

# Pull, Build and Deploy take a few minutes; give up after 20.
i=0
while [ "$i" -lt 80 ]; do
  sleep 15
  i=$((i + 1))
  state=$(api read "{\"type\":\"GetUpdate\",\"params\":{\"id\":\"$id\"}}")
  [ "$(printf '%s' "$state" | jq -r .status)" = "Complete" ] || continue
  if [ "$(printf '%s' "$state" | jq -r .success)" = "true" ]; then
    echo "Procedure $procedure finished: deployed."
    exit 0
  fi
  printf '%s' "$state" | jq -r '.logs[] | "== \(.stage)\n\(.stdout)\(.stderr)"' >&2
  echo "Procedure $procedure failed." >&2
  exit 1
done

echo "Procedure $procedure did not finish in 20 minutes." >&2
exit 1

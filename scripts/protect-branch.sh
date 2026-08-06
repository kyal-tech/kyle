#!/bin/bash
# Protect a branch on any GitHub repo using the REST API (classic protection).
#
# Reusable across any of your repositories — same effect as Settings → Branches
# → Add branch protection rule, but scriptable.
#
# Usage:
#   ./protect-branch.sh OWNER REPO BRANCH
#   ./protect-branch.sh kyal-tech kyle main
#
# Auth (pick one):
#   1. Export a token:   GITHUB_TOKEN=ghp_xxx ./protect-branch.sh ...
#   2. macOS keychain:   script reads the stored credential for github.com.
#   3. gh CLI fallback:  GITHUB_TOKEN= gh auth token
#
# To REMOVE protection:  ./protect-branch.sh OWNER REPO BRANCH --remove
#
# Configuration:
#   REQUIRED_CHECKS   CI checks to require, pipe-separated.
#   APPROVALS         Required approvals (0 = solo dev; >=1 = team). Default 0.
set -eu

if [ "$#" -lt 3 ]; then
    echo "Usage: $0 OWNER REPO BRANCH [--remove]" >&2
    echo "  ./protect-branch.sh kyal-tech kyle main" >&2
    echo "  ./protect-branch.sh kyal-tech kyle main --remove" >&2
    exit 1
fi

OWNER="$1"
REPO="$2"
BRANCH="$3"
MODE="${4:-protect}"

API="https://api.github.com/repos/$OWNER/$REPO/branches/$BRANCH/protection"

# --- Resolve token -----------------------------------------------------------
TOKEN="${GITHUB_TOKEN:-}"
if [ -z "$TOKEN" ] && command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
    TOKEN="$(gh auth token 2>/dev/null || true)"
fi
if [ -z "$TOKEN" ]; then
    # Try the macOS keychain credential (https://github.com)
    TOKEN="$(printf 'protocol=https\nhost=github.com\n' | git credential fill 2>/dev/null | sed -n 's/^password=//p')"
fi
if [ -z "$TOKEN" ]; then
    echo "ERROR: no GitHub token found. Set GITHUB_TOKEN or run 'gh auth login'." >&2
    exit 1
fi

# --- Dispatch ----------------------------------------------------------------
if [ "$MODE" = "--remove" ]; then
    code=$(curl -s -o /dev/null -w "%{http_code}" -X DELETE \
        -H "Authorization: Bearer $TOKEN" \
        -H "Accept: application/vnd.github+json" \
        "$API")
    case "$code" in
        204) echo "OK: protection removed from $OWNER/$REPO:$BRANCH" ;;
        404) echo "INFO: no protection was set on $OWNER/$REPO:$BRANCH" ;;
        *)   echo "ERROR: failed to remove protection (HTTP $code)" >&2; exit 1 ;;
    esac
    exit 0
fi

# --- Build payload -----------------------------------------------------------
# CI status checks to require. Edit this list for your workflows, or find the
# exact names with:
#   gh api repos/OWNER/REPO/commits/BRANCH/check-runs --jq '.check_runs[].name'
# Override at call time:  REQUIRED_CHECKS="CI / build|CI / test" ./protect-branch.sh ...
CHECKS="${REQUIRED_CHECKS:-test (ubuntu-24.04)|test (macos-15)|test-arm}"
APPROVALS="${APPROVALS:-0}"

contexts_json=$(CHECKS="$CHECKS" python3 -c 'import json, os; print(json.dumps([x for x in os.environ["CHECKS"].split("|") if x]))')

if command -v jq >/dev/null 2>&1; then
    payload=$(jq -n \
        --argjson ctx "$contexts_json" \
        --argjson reviews "$APPROVALS" \
        '{
            required_status_checks: {
                strict: true,
                contexts: $ctx
            },
            enforce_admins: true,
            required_pull_request_reviews: {
                required_approving_review_count: $reviews,
                dismiss_stale_reviews: true,
                require_code_owner_reviews: false
            },
            restrictions: null,
            allow_force_pushes: false,
            allow_deletions: false
        }')
else
    payload=$(python3 - "$contexts_json" "$APPROVALS" <<'PY'
import json, sys
contexts = json.loads(sys.argv[1])
reviews = int(sys.argv[2])
print(json.dumps({
    "required_status_checks": {"strict": True, "contexts": contexts},
    "enforce_admins": True,
    "required_pull_request_reviews": {
        "required_approving_review_count": reviews,
        "dismiss_stale_reviews": True,
        "require_code_owner_reviews": False
    },
    "restrictions": None,
    "allow_force_pushes": False,
    "allow_deletions": False
}))
PY
)
fi

code=$(curl -s -o /tmp/protect_branch_resp.json -w "%{http_code}" -X PUT \
    -H "Authorization: Bearer $TOKEN" \
    -H "Accept: application/vnd.github+json" \
    -H "Content-Type: application/json" \
    -d "$payload" \
    "$API")

case "$code" in
    200)
        echo "OK: $OWNER/$REPO:$BRANCH protected"
        echo "  - PR required, $APPROVALS approval(s)"
        echo "  - CI checks (strict): $CHECKS"
        echo "  - Applies to admins, force-push and deletions blocked"
        ;;
    404) echo "ERROR: repo/branch not found, or token lacks access (HTTP 404)" >&2; exit 1 ;;
    *)   echo "ERROR: failed to protect branch (HTTP $code)" >&2
         rg -o '"message":\s*"[^"]*"' /tmp/protect_branch_resp.json 2>/dev/null | head -1 >&2
         exit 1 ;;
esac

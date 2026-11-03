#!/usr/bin/env bash
# PR 본문(pull_request_template ## Changelog)에서 CHANGELOG 한 줄 추출
set -euo pipefail

PR_TITLE="${PR_TITLE:?}"
PR_NUMBER="${PR_NUMBER:?}"
PR_URL="${PR_URL:?}"
PR_BODY="${PR_BODY:-}"

if ! printf '%s' "$PR_BODY" | grep -q '^## Changelog'; then
  echo "::error::PR 본문에 ## Changelog 섹션이 없습니다. .github/pull_request_template.md 형식을 유지해 주세요."
  exit 1
fi

NOTE=$(
  printf '%s' "$PR_BODY" |
    sed -n '/^## Changelog/,/^## /p' |
    tail -n +2 |
    sed '/^## /d' |
    sed '/^[[:space:]]*$/d' |
    sed '/^<!--/d' |
    head -n 1
)

if [ -z "$NOTE" ]; then
  NOTE="$PR_TITLE"
  echo "::notice::Changelog 칸이 비어 있어 PR 제목을 CHANGELOG에 씁니다." >&2
fi

NOTE="${NOTE//$'\r'/}"
echo "- ${NOTE} ([#${PR_NUMBER}](${PR_URL}))"

#!/usr/bin/env bash
set -euo pipefail
release=false
case "${1:-}" in
  --help|-h)
    echo 'Usage: ./ops ci:run [--release]'
    echo 'Dispatch checks on GitHub main and watch to completion. Requires authenticated gh and jq.'
    echo 'Run ./ops check locally and push first. --release warms the optimized build cache.'
    exit 0 ;;
  --release) release=true; shift ;;
esac
(( $# == 0 )) || { echo 'Usage: ./ops ci:run [--release]' >&2; exit 2; }
for tool in gh jq; do
  command -v "$tool" >/dev/null || { echo "$tool is required" >&2; exit 1; }
done
repo=$(gh repo view --json nameWithOwner --jq .nameWithOwner)
previous=$(gh run list --repo "$repo" --workflow ci.yml --branch main --event workflow_dispatch --limit 1 --json databaseId --jq '.[0].databaseId // 0')
result=$(gh workflow run ci.yml --repo "$repo" --ref main -f "release=$release")
printf '%s\n' "$result"
# Recent gh versions return the dispatched run URL directly.
run_id=$(sed -nE 's@.*github.com/[^ /]+/[^ /]+/actions/runs/([0-9]+).*@\1@p' <<< "$result" | head -1)
if [[ -z "$run_id" ]]; then
  for (( attempt=0; attempt<20; attempt++ )); do
    run_id=$(gh run list --repo "$repo" --workflow ci.yml --branch main --event workflow_dispatch --limit 10 --json databaseId --jq ".[] | select(.databaseId > $previous) | .databaseId" | sort -n | head -1)
    [[ -z "$run_id" ]] || break
    sleep 3
  done
fi
[[ "$run_id" =~ ^[0-9]+$ ]] || { echo 'Dispatch accepted, but no run ID appeared. Inspect gh run list; do not blindly dispatch again.' >&2; exit 1; }
echo "Watching https://github.com/$repo/actions/runs/$run_id (main, release=$release)"
exec gh run watch "$run_id" --repo "$repo" --exit-status --interval 15

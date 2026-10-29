#!/usr/bin/env bash
# Download the newest successful CI run's artifacts for a branch into ./ci-artifacts/<run-id>/<artifact>/, and point
# ./ci-artifacts/latest/<workflow> at it. Needs the GitHub CLI, signed in.
#   scripts/fetch-artifacts.sh                 # this branch; linux and ios-sim
#   scripts/fetch-artifacts.sh main ios        # another branch, other workflows
set -euo pipefail
repo=${REPO:-heyskylo/horizon-ios}
branch=${1:-$(git rev-parse --abbrev-ref HEAD)}
shift || true
workflows=${*:-linux ios-sim}
cd "$(git rev-parse --show-toplevel)"
mkdir -p ci-artifacts/latest
for wf in $workflows; do
  id=$(gh run list -R "$repo" --workflow "$wf.yml" --branch "$branch" --status success --limit 1 \
    --json databaseId --jq '.[0].databaseId // empty')
  if [ -z "$id" ]; then echo "$wf: no successful run on $branch"; continue; fi
  dir="ci-artifacts/$id"
  if [ ! -d "$dir" ]; then
    gh run download "$id" -R "$repo" -D "$dir"
  fi
  # The Linux build arrives as a tarball: unpack it beside itself.
  for tarball in "$dir"/*/*.tar.gz; do
    [ -e "$tarball" ] && tar -xzf "$tarball" -C "$(dirname "$tarball")"
  done
  ln -sfn "../$id" "ci-artifacts/latest/$wf"
  echo "$wf: run $id -> ci-artifacts/latest/$wf"
done

#!/usr/bin/env bash
# Cut a release mechanically — one command, no memory required (owner ruling
# 2026-08-14, after three stale-version incidents in one day: install.md's
# hardcoded pin, plugin.json's manifest version twice). Everything that
# states a version moves in lockstep with the tag; smoke re-checks the
# lockstep; the tag push triggers release.yml for binaries + skill tarball.
# The marketplace serves the LATEST TAG, so this IS the skill publish step.
set -euo pipefail
cd "$(dirname "$0")/.."

TAG=${1:?usage: scripts/cut-release.sh vX.Y.Z}
[[ $TAG =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "cut-release: tag must be vX.Y.Z"; exit 1; }
VER=${TAG#v}

[[ -z $(git status --porcelain) ]] || { echo "cut-release: working tree must be clean"; exit 1; }
[[ -z $(git tag -l "$TAG") ]] || { echo "cut-release: $TAG already exists"; exit 1; }

# The release notes are a precondition of the tag, same standing as the
# version stamps: release.yml publishes this file as the release body, so a
# tag without it ships boilerplate (v0.4.0 did). Owner-voiced, user impact
# first — docs/release-notes/TEMPLATE.md is the shape.
NOTES="docs/release-notes/RELEASE-NOTES-$TAG.md"
[[ -f $NOTES ]] || {
  echo "cut-release: $NOTES is missing — draft the release notes first (docs/release-notes/TEMPLATE.md), commit them, then cut"
  exit 1
}

# Stamp every version the repo states, in lockstep with the tag.
sed -i.bak -e "s/^version = \".*\"/version = \"$VER\"/" Cargo.toml
sed -i.bak -e "s/\"version\": \".*\"/\"version\": \"$VER\"/" .claude-plugin/plugin.json
rm -f Cargo.toml.bak .claude-plugin/plugin.json.bak

cargo build --quiet                                # refresh Cargo.lock
MESHWORK_BLESS=1 cargo test --quiet >/dev/null 2>&1 # golden version stamps
./scripts/smoke.sh                                 # includes the lockstep guards

git add Cargo.toml Cargo.lock .claude-plugin/plugin.json fixtures/golden
git commit -m "chore(release): $TAG — version stamps in lockstep (cut-release.sh)"
git tag "$TAG"
echo "cut-release: $TAG ready — next: git push origin main && git push origin $TAG"

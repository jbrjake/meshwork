#!/usr/bin/env bash
# The full demo: three agent sessions across three public repos trace a lost
# edit to a library's conflict resolution, ask the library for the change, and
# catch what that change breaks in the app. Run from a clone:
# ./scripts/demo-full.sh [--pause]
#
# Needs the network: it clones github.com/jbrjake/meshwork-demo-notes-portfolio
# over https and runs its story/replay.sh, which clones the other two repos and
# builds them with cargo. The replay runs the meshwork release the demo repos
# pin; MESHWORK_BIN runs a candidate build instead. Arguments pass through to
# the replay.
set -euo pipefail

DEMO=$(mktemp -d)
trap 'rm -rf "$DEMO"' EXIT
git clone -q https://github.com/jbrjake/meshwork-demo-notes-portfolio.git "$DEMO/meshwork-demo-notes-portfolio"
"$DEMO/meshwork-demo-notes-portfolio/story/replay.sh" "$@"

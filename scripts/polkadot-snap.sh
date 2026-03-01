#!/usr/bin/env bash
set -euo pipefail

# Wrapper to ensure a writable base path when using the snap polkadot binary.
# If a base path is already provided, keep it; otherwise use a temp path.
if printf '%s\n' "$@" | rg -q "--base-path"; then
  exec snap run polkadot "$@"
else
  exec snap run polkadot --base-path /tmp/zombienet-relay "$@"
fi

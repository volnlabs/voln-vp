#!/usr/bin/env bash
set -euo pipefail

if ! command -v renode >/dev/null; then
  echo "MISSING: renode (install from renode.io or your distro)" >&2
  exit 1
fi

renode --version
echo "renode backend ok"

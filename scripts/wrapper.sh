#!/usr/bin/env bash
set -euo pipefail

GITBASH="C:\Program Files\Git\bin\bash.exe"
BINARY="$1"

if [[ "$RUNNER_OS" == "Windows" ]]; then
    "$GITBASH" "$(dirname "$0")"/install.sh "$BINARY"
else
    "$(dirname "$0")"/install.sh "$BINARY"
fi

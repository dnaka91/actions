#!/usr/bin/env bash
set -euo pipefail

BINARY=$1

cd /tmp

curl -LsSfO https://github.com/dnaka91/actions/releases/latest/download/"$BINARY"-x86_64-unknown-linux-gnu.tar.gz
curl -LsSfO https://github.com/dnaka91/actions/releases/latest/download/checksums.b2
curl -LsSfO https://github.com/dnaka91/actions/releases/latest/download/checksums.b2.asc

gpg --recv-keys 24B536EEDC7D1FBCFC678E786C63F836857F5C34
gpg --verify /tmp/checksums.b2.asc

b2sum -c --ignore-missing /tmp/checksums.b2

mkdir -p "$HOME"/.local/"$BINARY"/bin
tar -xzf /tmp/"$BINARY"-x86_64-unknown-linux-gnu.tar.gz -C "$HOME"/.local/"$BINARY"/bin
rm -rf /tmp/*.{tar.gz,b2,b2.asc}

echo "$HOME/.local/$BINARY/bin" >> "$GITHUB_PATH"

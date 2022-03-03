#!/usr/bin/env bash
set -euo pipefail

BINARY=$1
VERSION=$2

case $RUNNER_OS in
    Linux)
        TARGET=x86_64-unknown-linux-musl
        ;;
    macOS)
        TARGET=x86_64-apple-darwin
        brew install coreutils
        ;;
    Windows)
        TARGET=x86_64-pc-windows-msvc
        ;;
    *)
        bail "unsupported GitHub Runner OS: ${RUNNER_OS}"
        ;;
esac

ARCHIVE="$BINARY"-"$VERSION"-"$TARGET".tar.gz

cd /tmp

curl -LsSfO https://github.com/EmbarkStudios/cargo-deny/releases/download/"$VERSION"/"$ARCHIVE"
curl -LsSfO https://github.com/EmbarkStudios/cargo-deny/releases/download/"$VERSION"/"$ARCHIVE".sha256

echo " *$ARCHIVE" >> "$ARCHIVE".sha256
sha256sum -c --ignore-missing /tmp/"$ARCHIVE".sha256

mkdir -p "$HOME"/.local/"$BINARY"/bin

tar -xzf /tmp/"$ARCHIVE" -C "$HOME"/.local/"$BINARY"/bin --strip-components 1
rm -rf /tmp/*.{tar.gz,sha256}

echo "$HOME/.local/$BINARY/bin" >> "$GITHUB_PATH"

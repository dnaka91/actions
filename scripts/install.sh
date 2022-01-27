#!/usr/bin/env bash
set -euo pipefail

BINARY=$1
EXT=tar.gz

case $RUNNER_OS in
    Linux)
        TARGET=x86_64-unknown-linux-gnu
        ;;
    macOS)
        TARGET=x86_64-apple-darwin
        brew install coreutils
        ;;
    Windows)
        TARGET=x86_64-pc-windows-msvc
        EXT=zip
        ;;
    *)
        bail "unsupported GitHub Runner OS: ${RUNNER_OS}"
        ;;
esac

ARCHIVE="$BINARY"-"$TARGET"."$EXT"

cd /tmp

curl -LsSfO https://github.com/dnaka91/actions/releases/latest/download/"$ARCHIVE"
curl -LsSfO https://github.com/dnaka91/actions/releases/latest/download/checksums.b2
curl -LsSfO https://github.com/dnaka91/actions/releases/latest/download/checksums.b2.asc

gpg --recv-keys --keyserver hkps://keys.openpgp.org 24B536EEDC7D1FBCFC678E786C63F836857F5C34
gpg --verify /tmp/checksums.b2.asc

b2sum -c --ignore-missing /tmp/checksums.b2

mkdir -p "$HOME"/.local/"$BINARY"/bin

if [[ "${RUNNER_OS}" == "Windows" ]]; then
    unzip /tmp/"$ARCHIVE" -d "$HOME"/.local/"$BINARY"/bin
    rm -rf /tmp/*.{zip,b2,b2.asc}
else
    tar -xzf /tmp/"$ARCHIVE" -C "$HOME"/.local/"$BINARY"/bin
    rm -rf /tmp/*.{tar.gz,b2,b2.asc}
fi

echo "$HOME/.local/$BINARY/bin" >> "$GITHUB_PATH"
